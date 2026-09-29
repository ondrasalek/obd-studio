use crate::error::{AppError, AppResult};
use serde::Deserialize;

const READ_ONLY_SERVICES: &[u8] = &[0x19, 0x22, 0x3E];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdsRequest {
    pub tx_id: u32,
    pub rx_id: u32,
    pub payload_hex: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ValidatedUdsRequest {
    pub tx_id: u32,
    pub rx_id: u32,
    pub payload_hex: String,
    pub payload: Vec<u8>,
    pub timeout_ms: u64,
}

impl UdsRequest {
    pub fn validate(self) -> AppResult<ValidatedUdsRequest> {
        if self.tx_id > 0x7FF || self.rx_id > 0x7FF {
            return Err(AppError::validation(
                "MVP podporuje pouze 11bit CAN identifikátory 0x000–0x7FF.",
            ));
        }
        if self.tx_id == self.rx_id {
            return Err(AppError::validation("TX a RX CAN ID musí být odlišné."));
        }

        let payload_hex: String = self
            .payload_hex
            .chars()
            .filter(|character| !character.is_ascii_whitespace())
            .collect::<String>()
            .to_ascii_uppercase();

        if payload_hex.is_empty() || !payload_hex.len().is_multiple_of(2) {
            return Err(AppError::validation(
                "UDS payload musí obsahovat sudý počet hexadecimálních znaků.",
            ));
        }
        if payload_hex.len() > 8_192 || !payload_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::validation(
                "UDS payload není platný hexadecimální řetězec nebo je příliš dlouhý.",
            ));
        }

        let payload = (0..payload_hex.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&payload_hex[index..index + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| AppError::validation("UDS payload nelze dekódovat."))?;

        let service = payload[0];
        if !READ_ONLY_SERVICES.contains(&service) {
            return Err(AppError::new(
                "SERVICE_BLOCKED",
                format!(
                    "UDS služba 0x{service:02X} je v read-only UDS režimu zablokovaná. Povoleny jsou 0x19, 0x22 a 0x3E."
                ),
            ));
        }

        Ok(ValidatedUdsRequest {
            tx_id: self.tx_id,
            rx_id: self.rx_id,
            payload_hex,
            payload,
            timeout_ms: self.timeout_ms.unwrap_or(1_000).clamp(100, 10_000),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::UdsRequest;

    fn request(payload: &str) -> UdsRequest {
        UdsRequest {
            tx_id: 0x70E,
            rx_id: 0x778,
            payload_hex: payload.into(),
            timeout_ms: None,
        }
    }

    #[test]
    fn accepts_and_normalizes_read_did() {
        let validated = request("22 f1 87").validate().unwrap();
        assert_eq!(validated.payload_hex, "22F187");
        assert_eq!(validated.payload, [0x22, 0xF1, 0x87]);
    }

    #[test]
    fn blocks_write_data_by_identifier() {
        let error = request("2EF18700").validate().unwrap_err();
        assert_eq!(error.code, "SERVICE_BLOCKED");
    }
}
