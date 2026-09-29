use crate::error::{AppError, AppResult};
use crate::vehicle::DiagnosticProfile;
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, StopBits};
use std::io::{ErrorKind, Read, Write};
use std::time::{Duration, Instant};

const READ_SLICE: Duration = Duration::from_millis(20);
const MAX_REPLY_BYTES: usize = 64 * 1024;

/// Half-duplex command transport used by the STN command client.
///
/// Both real serial communication and offline demo sessions pass through the
/// same STN parser and higher protocol layers. Implementations return the raw
/// adapter response including the terminating `>` prompt.
pub trait StnTransport: Send {
    fn clear_input(&mut self) -> AppResult<()>;
    fn exchange(&mut self, command: &str, timeout: Duration) -> AppResult<String>;
    fn description(&self) -> String;
    fn is_simulated(&self) -> bool {
        false
    }
}

pub struct SerialStnTransport {
    port: Box<dyn serialport::SerialPort>,
    path: String,
    carry: Vec<u8>,
}

impl SerialStnTransport {
    pub fn open(path: &str) -> AppResult<Self> {
        if cfg!(target_os = "macos") && path.starts_with("/dev/tty.") {
            return Err(AppError::serial(format!(
                "Port {path} je dial-in zařízení; na macOS použijte odpovídající /dev/cu.* port."
            )));
        }

        let port = serialport::new(path, 115_200)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(READ_SLICE)
            .open()
            .map_err(|error| AppError::serial(format!("Port {path} nelze otevřít: {error}")))?;

        Ok(Self {
            port,
            path: path.to_owned(),
            carry: Vec::new(),
        })
    }

    fn completed_reply(&mut self, received: &mut Vec<u8>) -> Option<String> {
        let prompt = received.iter().position(|byte| *byte == b'>')?;
        let trailing = received.split_off(prompt + 1);
        self.carry = trailing;
        Some(String::from_utf8_lossy(received).into_owned())
    }
}

impl StnTransport for SerialStnTransport {
    fn clear_input(&mut self) -> AppResult<()> {
        self.carry.clear();
        self.port
            .clear(ClearBuffer::Input)
            .map_err(|error| AppError::serial(format!("Vyčištění vstupu portu selhalo: {error}")))
    }

    fn exchange(&mut self, command: &str, timeout: Duration) -> AppResult<String> {
        let mut wire_command = command.as_bytes().to_vec();
        wire_command.push(b'\r');
        self.port
            .write_all(&wire_command)
            .map_err(|error| AppError::serial(format!("Zápis na port selhal: {error}")))?;
        self.port
            .flush()
            .map_err(|error| AppError::serial(format!("Flush portu selhal: {error}")))?;

        let deadline = Instant::now() + timeout;
        let mut received = std::mem::take(&mut self.carry);
        if let Some(reply) = self.completed_reply(&mut received) {
            return Ok(reply);
        }

        let mut buffer = [0_u8; 512];
        while Instant::now() < deadline {
            match self.port.read(&mut buffer) {
                Ok(0) => continue,
                Ok(count) => {
                    received.extend_from_slice(&buffer[..count]);
                    if received.len() > MAX_REPLY_BYTES {
                        return Err(AppError::serial(
                            "Odpověď adaptéru překročila bezpečný limit 64 KiB.",
                        ));
                    }
                    if let Some(reply) = self.completed_reply(&mut received) {
                        return Ok(reply);
                    }
                }
                Err(error) if error.kind() == ErrorKind::TimedOut => continue,
                Err(error) => {
                    return Err(AppError::serial(format!("Čtení z portu selhalo: {error}")))
                }
            }
        }

        self.carry = received;
        Err(AppError::serial(format!(
            "Timeout po {} ms při čekání na prompt adaptéru.",
            timeout.as_millis()
        )))
    }

    fn description(&self) -> String {
        format!("USB sériový port {}", self.path)
    }
}

/// Deterministic offline STN adapter used for UI development and parser tests.
/// It generates raw adapter replies; no decoded diagnostic model bypasses the
/// production parsing path.
pub struct DemoStnTransport {
    profile: DiagnosticProfile,
    response_id: u32,
}

impl DemoStnTransport {
    pub fn new(profile: DiagnosticProfile) -> Self {
        Self {
            profile,
            response_id: 0x7E8,
        }
    }

    fn response(&mut self, command: &str) -> String {
        let upper = command.trim().to_ascii_uppercase();
        if let Some(value) = upper.strip_prefix("ATCRA ") {
            if let Ok(response_id) = u32::from_str_radix(value.trim(), 16) {
                self.response_id = response_id;
            }
            return ok();
        }

        match upper.as_str() {
            "ATZ" => "OBDLink EX demo\r>".to_owned(),
            "STDI" => "OBDLink EX · offline demo\r>".to_owned(),
            "STI" => "STN2232 v5.13.1 DEMO\r>".to_owned(),
            "STVR" => "12.70V\r>".to_owned(),
            "STPBRR" => "500000\r>".to_owned(),
            "STPRS" => "ISO 15765-4 CAN 11/500\r>".to_owned(),
            "0100" => "4100BE3EA813\r>".to_owned(),
            "0902" => concat!(
                "014\r",
                "0: 49 02 01 54 4D 42\r",
                "1: 44 45 4D 4F 30 30 30\r",
                "2: 30 30 30 30 30 30 31\r>"
            )
            .to_owned(),
            "03" => "4301330000\r>".to_owned(),
            "04" => "44\r>".to_owned(),
            _ if upper == "010C" && self.profile == DiagnosticProfile::GenericObd2 => {
                "410C0000\r>".to_owned()
            }
            _ if upper.starts_with("01") && upper.len() == 4 => standard_pid_response(&upper[2..4]),
            _ if upper.starts_with("STPX ") => self.stpx_response(&upper),
            _ => ok(),
        }
    }

    fn stpx_response(&self, command: &str) -> String {
        let Some(payload) = command
            .split(',')
            .find_map(|part| part.trim().strip_prefix("D:"))
        else {
            return "?\r>".to_owned();
        };

        let response = if payload == "22F187" {
            let identity = format!("DEMO-{:03X}", self.response_id);
            format!("62F187{}", ascii_hex(&identity))
        } else if payload == "22F189" {
            format!("62F189{}", ascii_hex("1.0-DEMO"))
        } else if payload == "190208" {
            if matches!(self.response_id, 0x77D | 0x778) {
                "5902FF11223308".to_owned()
            } else {
                "5902FF".to_owned()
            }
        } else if let Some(did) = payload.strip_prefix("22") {
            match demo_did_data(did) {
                Some(data) => format!("62{did}{data}"),
                None => "7F2231".to_owned(),
            }
        } else if payload.starts_with("01") && payload.len() == 4 {
            return with_header(self.response_id, &standard_pid_response(&payload[2..4]));
        } else if payload == "3E00" {
            "7E00".to_owned()
        } else {
            "7F1111".to_owned()
        };

        format!("{:03X}{response}\r>", self.response_id)
    }
}

impl StnTransport for DemoStnTransport {
    fn clear_input(&mut self) -> AppResult<()> {
        Ok(())
    }

    fn exchange(&mut self, command: &str, _timeout: Duration) -> AppResult<String> {
        Ok(self.response(command))
    }

    fn description(&self) -> String {
        format!("offline demo · {}", self.profile.display_name())
    }

    fn is_simulated(&self) -> bool {
        true
    }
}

fn standard_pid_response(pid: &str) -> String {
    let data = match pid {
        "04" => "4D",
        "05" | "5C" => "82",
        "06" => "82",
        "07" => "7D",
        "0B" | "33" => "64",
        "0C" => "0C80",
        "0D" => "00",
        "0F" => "3C",
        "10" => "04E2",
        "11" => "26",
        "13" => "01",
        "21" => "0000",
        "23" => "09C4",
        "2F" => "A6",
        "42" => "3138",
        "5E" => "0018",
        "78" | "79" | "7C" => "01170C",
        "7A" => "001E",
        "83" => "010064",
        _ => return "NO DATA\r>".to_owned(),
    };
    format!("41{pid}{data}\r>")
}

fn with_header(response_id: u32, response: &str) -> String {
    format!("{response_id:03X}{}", response.trim_start())
}

fn demo_did_data(did: &str) -> Option<&'static str> {
    match did {
        "F40C" => Some("0C80"),
        "F405" => Some("82"),
        "11BE" => Some("0E2F"),
        "F40F" => Some("3C"),
        "F410" => Some("04E2"),
        "105E" => Some("0276"),
        "1047" => Some("012C"),
        "10FF" => Some("000A"),
        "1105" => Some("FFF8"),
        "1100" => Some("0005"),
        "1104" => Some("FFFC"),
        "2609" => Some("04E2"),
        "1ABE" => Some("0384"),
        "1ABD" => Some("14000000"),
        "14F5" => Some("0006"),
        "11B2" => Some("240F"),
        "10F9" => Some("2260"),
        "1ABA" => Some("0708"),
        "1AC9" => Some("00"),
        "1B0E" => Some("3200"),
        "1149" => Some("03E8"),
        "1057" => Some("03E8"),
        "10FB" => Some("2260"),
        "F433" => Some("64"),
        _ => None,
    }
}

fn ascii_hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

fn ok() -> String {
    "OK\r>".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::stn::StnReply;
    use crate::protocol::obd2::{parse_standard_obd_response, StandardObdQuery};
    use crate::protocol::uds::{parse_uds_response, UdsRequest};
    use crate::vehicle::decode_ascii_identifier;

    #[test]
    fn demo_tracks_selected_response_id_and_returns_raw_uds() {
        let mut demo = DemoStnTransport::new(DiagnosticProfile::VwMqb);
        assert_eq!(demo.exchange("ATCRA 778", Duration::ZERO).unwrap(), "OK\r>");
        let response = demo
            .exchange("STPX h:70E,d:22F187,t:1000,r:1", Duration::ZERO)
            .unwrap();
        assert!(response.starts_with("77862F187"));
        assert!(response.ends_with('>'));
    }

    #[test]
    fn demo_returns_supported_standard_pid_and_no_data_for_unknown_pid() {
        let mut demo = DemoStnTransport::new(DiagnosticProfile::GenericObd2);
        assert_eq!(
            demo.exchange("010C", Duration::ZERO).unwrap(),
            "410C0000\r>"
        );
        assert_eq!(demo.exchange("0199", Duration::ZERO).unwrap(), "NO DATA\r>");
    }

    #[test]
    fn demo_is_explicitly_marked_as_simulated() {
        let demo = DemoStnTransport::new(DiagnosticProfile::GenericObd2);
        assert!(demo.is_simulated());
        assert!(demo.description().contains("offline demo"));
    }

    #[test]
    fn demo_uds_reply_passes_production_parser() {
        let mut demo = DemoStnTransport::new(DiagnosticProfile::VwMqb);
        demo.exchange("ATCRA 778", Duration::ZERO).unwrap();
        let command = "STPX h:70E,d:22F187,t:1000,r:1";
        let raw = demo.exchange(command, Duration::ZERO).unwrap();
        let reply = StnReply::parse(raw, command).unwrap();
        let request = UdsRequest {
            tx_id: 0x70E,
            rx_id: 0x778,
            payload_hex: "22F187".to_owned(),
            timeout_ms: None,
        }
        .validate()
        .unwrap();
        let response = parse_uds_response(1, &request, reply).unwrap();

        assert_eq!(
            decode_ascii_identifier(&response.payload_hex, 0xF187).as_deref(),
            Some("DEMO-778")
        );
    }

    #[test]
    fn demo_vin_reply_passes_production_parser() {
        let mut demo = DemoStnTransport::new(DiagnosticProfile::GenericObd2);
        let request = StandardObdQuery::VehicleIdentificationNumber.into_request();
        let raw = demo.exchange(request.payload_hex, Duration::ZERO).unwrap();
        let reply = StnReply::parse(raw, request.payload_hex).unwrap();
        let response = parse_standard_obd_response(1, &request, reply, None).unwrap();

        assert_eq!(response.vin.as_deref(), Some("TMBDEMO0000000001"));
    }
}
