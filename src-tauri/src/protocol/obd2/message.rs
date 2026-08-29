use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StandardObdQuery {
    SupportedPids,
    VehicleIdentificationNumber,
    StoredDiagnosticTroubleCodes,
}

#[derive(Debug, Clone)]
pub struct ValidatedObdRequest {
    pub query: StandardObdQuery,
    pub payload_hex: &'static str,
    pub expected_response_service: u8,
    pub timeout_ms: u64,
}

impl StandardObdQuery {
    pub fn into_request(self) -> ValidatedObdRequest {
        let (payload_hex, expected_response_service) = match self {
            Self::SupportedPids => ("0100", 0x41),
            Self::VehicleIdentificationNumber => ("0902", 0x49),
            Self::StoredDiagnosticTroubleCodes => ("03", 0x43),
        };

        ValidatedObdRequest {
            query: self,
            payload_hex,
            expected_response_service,
            // První request může spustit hledání přes všechny legislativní protokoly.
            timeout_ms: 12_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StandardObdQuery;

    #[test]
    fn maps_queries_to_read_only_obd_modes() {
        let pids = StandardObdQuery::SupportedPids.into_request();
        let vin = StandardObdQuery::VehicleIdentificationNumber.into_request();
        let dtcs = StandardObdQuery::StoredDiagnosticTroubleCodes.into_request();

        assert_eq!(pids.payload_hex, "0100");
        assert_eq!(vin.payload_hex, "0902");
        assert_eq!(dtcs.payload_hex, "03");
    }
}
