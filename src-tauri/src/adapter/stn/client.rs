use super::StnReply;
use crate::error::{AppError, AppResult};
use crate::events::{emit_log, LogDirection, LogKind};
use crate::protocol::obd2::ValidatedObdRequest;
use crate::protocol::uds::ValidatedUdsRequest;
use crate::transport::stn::StnTransport;
use crate::vehicle::DiagnosticProfile;
use std::time::Duration;
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct AdapterIdentity {
    pub hardware: String,
    pub firmware: String,
    pub voltage: Option<f32>,
    pub protocol: String,
    pub can_bitrate: Option<u32>,
}

pub struct StnClient<'a> {
    transport: &'a mut dyn StnTransport,
    app: &'a AppHandle,
}

impl<'a> StnClient<'a> {
    pub fn new(transport: &'a mut dyn StnTransport, app: &'a AppHandle) -> Self {
        Self { transport, app }
    }

    pub fn initialize(&mut self, profile: DiagnosticProfile) -> AppResult<AdapterIdentity> {
        self.transport.clear_input()?;

        let reset = self.transact("ATZ", Duration::from_millis(2_500), LogKind::At)?;
        if reset.lines.is_empty() {
            return Err(AppError::adapter("ATZ nevrátil identifikaci adaptéru."));
        }

        for command in ["ATE0", "ATL0", "ATS0", "ATD0"] {
            self.command_ok(command)?;
        }

        let hardware = self.query_value("STDI")?;
        let firmware = self.query_value("STI")?;
        let voltage = parse_voltage(&self.query_value("STVR")?);

        let (protocol, can_bitrate) = match profile {
            DiagnosticProfile::GenericObd2 => {
                for command in [
                    "ATH0",
                    "ATAL",
                    "ATAT1",
                    "ATCAF1",
                    "ATCFC1",
                    "STPTO 1500",
                    "ATSP0",
                ] {
                    self.command_ok(command)?;
                }
                ("AUTO · OBD-II/EOBD/JOBD".to_owned(), None)
            }
            DiagnosticProfile::VwMqb => {
                self.command_ok("ATH1")?;
                self.command_ok("STP 33")?;
                let bitrate = self
                    .query_value("STPBRR")?
                    .parse::<u32>()
                    .map_err(|_| AppError::adapter("STPBRR nevrátil platnou CAN rychlost."))?;

                if bitrate != 500_000 {
                    return Err(AppError::adapter(format!(
                        "Očekávána CAN rychlost 500000 bps, adaptér hlásí {bitrate} bps."
                    )));
                }

                for command in [
                    "STCAF 0",
                    "ATCAF1",
                    "ATCFC1",
                    "ATAL",
                    "ATAT1",
                    "STCSEGR 1",
                    "STCSEGT 1",
                    "STPTO 1000",
                    "STPO",
                ] {
                    self.command_ok(command)?;
                }
                ("ISO 15765 · 11-bit HS-CAN".to_owned(), Some(bitrate))
            }
        };

        Ok(AdapterIdentity {
            hardware,
            firmware,
            voltage,
            protocol,
            can_bitrate,
        })
    }

    pub fn send_standard_obd(&mut self, request: &ValidatedObdRequest) -> AppResult<StnReply> {
        self.transact(
            request.payload_hex,
            Duration::from_millis(request.timeout_ms),
            LogKind::Obd,
        )
    }

    /// Typovaná interní cesta pro legislativní Mode 01.
    ///
    /// Volající dodává pouze PID, takže touto metodou nelze obejít read-only
    /// hranici a poslat Mode 04 nebo výrobce-specifický zápis.
    pub fn send_standard_current_data_pid(
        &mut self,
        pid: u8,
        timeout_ms: u64,
    ) -> AppResult<StnReply> {
        self.transact(
            &format!("01{pid:02X}"),
            Duration::from_millis(timeout_ms.clamp(100, 12_000)),
            LogKind::Obd,
        )
    }

    /// Mode 01 fyzicky adresovaný na motorovou ECU aktivního MQB profilu.
    pub fn send_physical_current_data_pid(
        &mut self,
        tx_id: u32,
        pid: u8,
        timeout_ms: u64,
    ) -> AppResult<StnReply> {
        let timeout_ms = timeout_ms.clamp(100, 10_000);
        let command = format!("STPX h:{tx_id:03X},d:01{pid:02X},t:{timeout_ms},r:1");
        self.transact(
            &command,
            Duration::from_millis(timeout_ms + 500),
            LogKind::Obd,
        )
    }

    pub fn detected_protocol(&mut self) -> Option<String> {
        self.query_value("STPRS").ok()
    }

    pub fn supply_voltage(&mut self) -> AppResult<f32> {
        parse_voltage(&self.query_value("STVR")?)
            .ok_or_else(|| AppError::adapter("Adaptér nevrátil platné napětí vozidla."))
    }

    /// Pevně typovaná standardní operace Mode 04. Volající nemůže dodat
    /// jiný payload; výrobce-specifické mazání UDS touto cestou neprojde.
    pub fn clear_emission_dtcs(&mut self) -> AppResult<StnReply> {
        let reply = self.transact("04", Duration::from_millis(5_000), LogKind::Obd)?;
        let positive = reply.lines.iter().any(|line| {
            line.chars()
                .filter(|character| character.is_ascii_hexdigit())
                .collect::<String>()
                .to_ascii_uppercase()
                .starts_with("44")
        });
        if !positive {
            return Err(AppError::adapter(
                "Vozidlo nepotvrdilo vymazání emisních DTC odpovědí Mode 44.",
            ));
        }
        Ok(reply)
    }

    pub fn send_uds(&mut self, request: &ValidatedUdsRequest) -> AppResult<StnReply> {
        self.select_target(request.tx_id, request.rx_id)?;

        self.send_uds_to_selected_target(request)
    }

    pub fn select_target(&mut self, tx_id: u32, rx_id: u32) -> AppResult<()> {
        self.configure_target(tx_id, rx_id)
    }

    pub fn send_uds_to_selected_target(
        &mut self,
        request: &ValidatedUdsRequest,
    ) -> AppResult<StnReply> {
        let command = format!(
            "STPX h:{:03X},d:{},t:{},r:1",
            request.tx_id, request.payload_hex, request.timeout_ms
        );

        self.transact(
            &command,
            Duration::from_millis(request.timeout_ms + 500),
            LogKind::Uds,
        )
    }

    pub fn close_protocol(&mut self) {
        let _ = self.transact("STPC", Duration::from_millis(500), LogKind::At);
    }

    fn configure_target(&mut self, tx_id: u32, rx_id: u32) -> AppResult<()> {
        let commands = [
            "STPC".to_owned(),
            format!("ATSH {:03X}", tx_id),
            format!("ATCRA {:03X}", rx_id),
            "STFFCC".to_owned(),
            format!("STFFCA {:03X},7FF", rx_id),
            "STCFCPC".to_owned(),
            format!("STCFCPA {:03X},{:03X}", tx_id, rx_id),
            "STPO".to_owned(),
        ];

        for command in commands {
            self.command_ok(&command)?;
        }

        Ok(())
    }

    fn command_ok(&mut self, command: &str) -> AppResult<()> {
        let reply = self.transact(command, Duration::from_millis(1_500), LogKind::At)?;
        reply.require_ok(command)
    }

    fn query_value(&mut self, command: &str) -> AppResult<String> {
        self.transact(command, Duration::from_millis(1_500), LogKind::At)?
            .first_value(command)
    }

    fn transact(&mut self, command: &str, timeout: Duration, kind: LogKind) -> AppResult<StnReply> {
        emit_log(self.app, LogDirection::Tx, kind.clone(), command);
        let raw = self.transport.exchange(command, timeout)?;
        emit_log(
            self.app,
            LogDirection::Rx,
            kind,
            raw.replace('\r', "\n").trim().to_owned(),
        );
        StnReply::parse(raw, command)
    }
}

fn parse_voltage(value: &str) -> Option<f32> {
    value
        .trim()
        .trim_end_matches(['V', 'v'])
        .parse::<f32>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::parse_voltage;

    #[test]
    fn parses_adapter_voltage() {
        assert_eq!(parse_voltage("12.48V"), Some(12.48));
        assert_eq!(parse_voltage("--.--V"), None);
    }
}
