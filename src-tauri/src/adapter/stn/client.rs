use super::StnReply;
use crate::error::{AppError, AppResult};
use crate::events::{emit_log, LogDirection, LogKind};
use crate::protocol::obd2::ValidatedObdRequest;
use crate::protocol::uds::ValidatedUdsRequest;
use crate::vehicle::DiagnosticProfile;
use serialport::{ClearBuffer, SerialPort};
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use tauri::AppHandle;

const MAX_REPLY_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct AdapterIdentity {
    pub hardware: String,
    pub firmware: String,
    pub voltage: Option<f32>,
    pub protocol: String,
    pub can_bitrate: Option<u32>,
}

pub struct StnClient<'a> {
    port: &'a mut dyn SerialPort,
    app: &'a AppHandle,
}

impl<'a> StnClient<'a> {
    pub fn new(port: &'a mut dyn SerialPort, app: &'a AppHandle) -> Self {
        Self { port, app }
    }

    pub fn initialize(&mut self, profile: DiagnosticProfile) -> AppResult<AdapterIdentity> {
        let _ = self.port.clear(ClearBuffer::Input);

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

    pub fn detected_protocol(&mut self) -> Option<String> {
        self.query_value("STPRS").ok()
    }

    pub fn send_uds(&mut self, request: &ValidatedUdsRequest) -> AppResult<StnReply> {
        self.configure_target(request.tx_id, request.rx_id)?;

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

        let mut wire_command = command.as_bytes().to_vec();
        wire_command.push(b'\r');
        self.port
            .write_all(&wire_command)
            .map_err(|error| AppError::serial(format!("Zápis na port selhal: {error}")))?;
        self.port
            .flush()
            .map_err(|error| AppError::serial(format!("Flush portu selhal: {error}")))?;

        let deadline = Instant::now() + timeout;
        let mut received = Vec::with_capacity(512);
        let mut buffer = [0_u8; 256];

        while Instant::now() < deadline {
            match self.port.read(&mut buffer) {
                Ok(0) => continue,
                Ok(count) => {
                    received.extend_from_slice(&buffer[..count]);
                    if received.contains(&b'>') {
                        break;
                    }
                    if received.len() > MAX_REPLY_BYTES {
                        return Err(AppError::adapter(
                            "Odpověď adaptéru překročila bezpečný limit 64 KiB.",
                        ));
                    }
                }
                Err(error) if error.kind() == ErrorKind::TimedOut => continue,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => {
                    return Err(AppError::serial(format!(
                        "Čtení ze sériového portu selhalo: {error}"
                    )))
                }
            }
        }

        if !received.contains(&b'>') {
            return Err(AppError::adapter(format!(
                "Timeout při čekání na prompt po příkazu {command}."
            )));
        }

        let raw = String::from_utf8_lossy(&received).into_owned();
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
