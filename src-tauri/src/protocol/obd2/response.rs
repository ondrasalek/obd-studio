use super::{StandardObdQuery, ValidatedObdRequest};
use crate::adapter::stn::StnReply;
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardObdResponse {
    pub request_id: u64,
    pub query: StandardObdQuery,
    pub request_hex: String,
    pub payloads: Vec<String>,
    pub detected_protocol: Option<String>,
    pub vin: Option<String>,
    pub diagnostic_trouble_codes: Vec<String>,
    pub raw_response: String,
}

pub fn parse_standard_obd_response(
    request_id: u64,
    request: &ValidatedObdRequest,
    reply: StnReply,
    detected_protocol: Option<String>,
) -> AppResult<StandardObdResponse> {
    let expected_prefix = format!("{:02X}", request.expected_response_service);
    let mut payloads = Vec::new();
    let mut segmented_payload = String::new();

    for line in &reply.lines {
        if line.eq_ignore_ascii_case("SEARCHING...") || line.eq_ignore_ascii_case("OK") {
            continue;
        }

        if let Some((sequence, data)) = line.split_once(':') {
            if sequence
                .trim()
                .chars()
                .all(|character| character.is_ascii_hexdigit())
            {
                if let Some(compact) = compact_hex_line(data) {
                    segmented_payload.push_str(&compact);
                }
                continue;
            }
        }

        if let Some(compact) = compact_hex_line(line) {
            // OBDLink může před segmentovanou odpovědí vytisknout pouze lichou délku
            // (např. 014); tu compact_hex_line odfiltruje. Krátké "43" je naopak
            // platná odpověď Mode 03 bez uložených závad.
            payloads.push(compact);
        }
    }

    if !segmented_payload.is_empty() {
        payloads.push(segmented_payload);
    }

    payloads.retain(|payload| payload.starts_with(&expected_prefix));
    if payloads.is_empty() {
        return Err(AppError::adapter(format!(
            "OBD-II odpověď neobsahuje očekávanou službu 0x{}.",
            expected_prefix
        )));
    }

    let vin = if request.query == StandardObdQuery::VehicleIdentificationNumber {
        payloads.iter().find_map(|payload| decode_vin(payload))
    } else {
        None
    };
    let diagnostic_trouble_codes =
        if request.query == StandardObdQuery::StoredDiagnosticTroubleCodes {
            payloads
                .iter()
                .flat_map(|payload| decode_dtcs(payload))
                .collect()
        } else {
            Vec::new()
        };

    Ok(StandardObdResponse {
        request_id,
        query: request.query,
        request_hex: request.payload_hex.to_owned(),
        payloads,
        detected_protocol,
        vin,
        diagnostic_trouble_codes,
        raw_response: reply.raw,
    })
}

fn compact_hex_line(line: &str) -> Option<String> {
    if !line
        .chars()
        .all(|character| character.is_ascii_hexdigit() || character.is_ascii_whitespace())
    {
        return None;
    }

    let compact: String = line
        .chars()
        .filter(|character| character.is_ascii_hexdigit())
        .collect::<String>()
        .to_ascii_uppercase();

    (!compact.is_empty() && compact.len().is_multiple_of(2)).then_some(compact)
}

fn decode_vin(payload: &str) -> Option<String> {
    let bytes = (0..payload.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&payload[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;

    if bytes.len() < 4 || bytes[0..3] != [0x49, 0x02, 0x01] {
        return None;
    }

    let vin: String = bytes[3..]
        .iter()
        .copied()
        .filter(|byte| *byte != 0)
        .map(char::from)
        .collect();

    (vin.len() == 17
        && vin
            .chars()
            .all(|character| character.is_ascii_alphanumeric()))
    .then_some(vin)
}

fn decode_dtcs(payload: &str) -> Vec<String> {
    let Some(bytes) = (0..payload.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&payload[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()
    else {
        return Vec::new();
    };

    if bytes.first() != Some(&0x43) {
        return Vec::new();
    }

    const SYSTEMS: [char; 4] = ['P', 'C', 'B', 'U'];
    bytes[1..]
        .chunks_exact(2)
        .filter(|pair| pair != &[0, 0])
        .map(|pair| {
            format!(
                "{}{}{:X}{:X}{:X}",
                SYSTEMS[(pair[0] >> 6) as usize],
                (pair[0] >> 4) & 0x03,
                pair[0] & 0x0F,
                pair[1] >> 4,
                pair[1] & 0x0F
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_standard_obd_response;
    use crate::adapter::stn::StnReply;
    use crate::protocol::obd2::StandardObdQuery;

    #[test]
    fn parses_multiple_pid_responders() {
        let request = StandardObdQuery::SupportedPids.into_request();
        let reply =
            StnReply::parse("SEARCHING...\r4100BE1FA813\r4100983A8011\r>".into(), "0100").unwrap();
        let response =
            parse_standard_obd_response(1, &request, reply, Some("ISO 15765".into())).unwrap();

        assert_eq!(response.payloads.len(), 2);
        assert_eq!(response.payloads[0], "4100BE1FA813");
    }

    #[test]
    fn assembles_and_decodes_segmented_vin() {
        let request = StandardObdQuery::VehicleIdentificationNumber.into_request();
        let reply = StnReply::parse(
            "014\r0: 49 02 01 54 4D 42\r1: 4A 45 37 4E 45 30 4C\r2: 30 31 32 33 34 35 36\r>".into(),
            "0902",
        )
        .unwrap();
        let response = parse_standard_obd_response(2, &request, reply, None).unwrap();

        assert_eq!(response.vin.as_deref(), Some("TMBJE7NE0L0123456"));
    }

    #[test]
    fn decodes_standard_dtc_pairs() {
        let request = StandardObdQuery::StoredDiagnosticTroubleCodes.into_request();
        let reply = StnReply::parse("430133C1230000\r>".into(), "03").unwrap();
        let response = parse_standard_obd_response(3, &request, reply, None).unwrap();

        assert_eq!(response.diagnostic_trouble_codes, ["P0133", "U0123"]);
    }

    #[test]
    fn accepts_empty_mode_three_response() {
        let request = StandardObdQuery::StoredDiagnosticTroubleCodes.into_request();
        let reply = StnReply::parse("43\r>".into(), "03").unwrap();
        let response = parse_standard_obd_response(4, &request, reply, None).unwrap();

        assert!(response.diagnostic_trouble_codes.is_empty());
    }
}
