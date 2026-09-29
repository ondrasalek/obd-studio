use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticProfile {
    GenericObd2,
    VwMqb,
}

impl DiagnosticProfile {
    pub fn slug(self) -> &'static str {
        match self {
            Self::GenericObd2 => "generic-obd2",
            Self::VwMqb => "vw-mqb",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::GenericObd2 => "Standardní OBD-II · automatická detekce",
            Self::VwMqb => "Volkswagen Group MQB · UDS 500 kbps",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticProfile;

    #[test]
    fn profile_uses_frontend_camel_case_names() {
        assert_eq!(
            serde_json::to_string(&DiagnosticProfile::GenericObd2).unwrap(),
            "\"genericObd2\""
        );
        assert_eq!(
            serde_json::to_string(&DiagnosticProfile::VwMqb).unwrap(),
            "\"vwMqb\""
        );
    }

    #[test]
    fn profile_has_stable_transport_slug() {
        assert_eq!(DiagnosticProfile::GenericObd2.slug(), "generic-obd2");
        assert_eq!(DiagnosticProfile::VwMqb.slug(), "vw-mqb");
    }
}
