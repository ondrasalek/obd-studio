use super::ValidatedUdsRequest;
use crate::adapter::stn::StnReply;
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UdsResponse {
    pub request_id: u64,
    pub tx_id: u32,
    pub rx_id: u32,
    pub request_hex: String,
    pub payload_hex: String,
    pub status: UdsResponseStatus,
    pub negative_response: Option<NegativeResponse>,
    pub raw_response: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UdsResponseStatus {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NegativeResponse {
    pub requested_service: String,
    pub code: String,
    pub description: String,
}

pub fn parse_uds_response(
    request_id: u64,
    request: &ValidatedUdsRequest,
    reply: StnReply,
) -> AppResult<UdsResponse> {
    let rx_prefix = format!("{:03X}", request.rx_id);
    let payloads: Vec<String> = reply
        .lines
        .iter()
        .filter_map(|line| {
            let compact: String = line
                .chars()
                .filter(|character| character.is_ascii_hexdigit())
                .collect::<String>()
                .to_ascii_uppercase();
            compact
                .strip_prefix(&rx_prefix)
                .filter(|payload| payload.len() >= 2 && payload.len() % 2 == 0)
                .map(ToOwned::to_owned)
        })
        .collect();
    let expected_service = request.payload[0].wrapping_add(0x40);
    let expected_prefix = format!("{expected_service:02X}");
    let payload_hex = payloads
        .iter()
        .find(|payload| payload.starts_with(&expected_prefix))
        .or_else(|| {
            payloads.iter().rev().find(|payload| {
                !(payload.starts_with("7F")
                    && payload.len() >= 6
                    && payload[4..6].eq_ignore_ascii_case("78"))
            })
        })
        .cloned()
        .ok_or_else(|| {
            AppError::adapter(format!(
                "Odpověď neobsahuje očekávané RX CAN ID 0x{:03X}.",
                request.rx_id
            ))
        })?;

    let payload = decode_hex(&payload_hex)?;
    let negative_response = if payload.first() == Some(&0x7F) && payload.len() >= 3 {
        Some(NegativeResponse {
            requested_service: format!("{:02X}", payload[1]),
            code: format!("{:02X}", payload[2]),
            description: nrc_description(payload[2]).to_owned(),
        })
    } else {
        if payload.first() != Some(&expected_service) {
            return Err(AppError::adapter(format!(
                "Neočekávaná UDS služba v odpovědi: očekávána 0x{expected_service:02X}."
            )));
        }
        None
    };

    let status = if negative_response.is_some() {
        UdsResponseStatus::Negative
    } else {
        UdsResponseStatus::Positive
    };

    Ok(UdsResponse {
        request_id,
        tx_id: request.tx_id,
        rx_id: request.rx_id,
        request_hex: request.payload_hex.clone(),
        payload_hex,
        status,
        negative_response,
        raw_response: reply.raw,
    })
}

fn decode_hex(value: &str) -> AppResult<Vec<u8>> {
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AppError::adapter("UDS odpověď obsahuje neplatná hex data."))
}

fn nrc_description(code: u8) -> &'static str {
    match code {
        0x10 => "General reject",
        0x11 => "Service not supported",
        0x12 => "Sub-function not supported",
        0x13 => "Incorrect message length or invalid format",
        0x22 => "Conditions not correct",
        0x31 => "Request out of range",
        0x33 => "Security access denied",
        0x35 => "Invalid key",
        0x36 => "Exceeded number of attempts",
        0x37 => "Required time delay not expired",
        0x78 => "Response pending",
        _ => "Unknown negative response code",
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_uds_response, UdsResponseStatus};
    use crate::adapter::stn::StnReply;
    use crate::protocol::uds::UdsRequest;

    fn request() -> crate::protocol::uds::ValidatedUdsRequest {
        UdsRequest {
            tx_id: 0x70E,
            rx_id: 0x778,
            payload_hex: "22F187".into(),
            timeout_ms: Some(1_000),
        }
        .validate()
        .unwrap()
    }

    #[test]
    fn parses_positive_read_did_response() {
        let reply = StnReply::parse("77862F187354130393337303834\r>".into(), "STPX").unwrap();
        let response = parse_uds_response(1, &request(), reply).unwrap();
        assert!(matches!(response.status, UdsResponseStatus::Positive));
        assert_eq!(response.payload_hex, "62F187354130393337303834");
    }

    #[test]
    fn parses_negative_response_code() {
        let reply = StnReply::parse("7787F2231\r>".into(), "STPX").unwrap();
        let response = parse_uds_response(2, &request(), reply).unwrap();
        assert!(matches!(response.status, UdsResponseStatus::Negative));
        assert_eq!(response.negative_response.unwrap().code, "31");
    }

    #[test]
    fn prefers_positive_response_after_response_pending() {
        let reply =
            StnReply::parse("7787F2278\r77862F18730344C39303630323750\r>".into(), "STPX").unwrap();
        let response = parse_uds_response(3, &request(), reply).unwrap();
        assert!(matches!(response.status, UdsResponseStatus::Positive));
        assert_eq!(response.payload_hex, "62F18730344C39303630323750");
    }
}
