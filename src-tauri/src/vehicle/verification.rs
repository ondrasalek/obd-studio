use super::manifest::ProfileManifest;
use super::LiveDataGroup;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcuIdentity {
    pub part_number: Option<String>,
    pub software_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VerificationStatus {
    Standard,
    Matched,
    Unverified,
    Simulated,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDataVerification {
    pub status: VerificationStatus,
    pub detail: String,
    pub profile_id: String,
    pub profile_version: String,
    pub source: String,
    pub identity: Option<EcuIdentity>,
}

impl LiveDataVerification {
    pub fn standard(manifest: &ProfileManifest, simulated: bool) -> Self {
        Self::new(
            manifest,
            if simulated {
                VerificationStatus::Simulated
            } else {
                VerificationStatus::Standard
            },
            if simulated {
                "Simulované OBD-II odpovědi; nejde o měření skutečného vozidla."
            } else {
                "Standardní OBD-II PID; podporu a použitelnost určuje odpověď ECU."
            },
            None,
        )
    }

    pub fn for_mqb_engine(
        manifest: &ProfileManifest,
        group: LiveDataGroup,
        identity: EcuIdentity,
        simulated: bool,
    ) -> Self {
        let (status, detail) = if simulated {
            (VerificationStatus::Simulated, "Simulovaná data pro vývoj rozhraní. Neověřují kompatibilitu skutečné ECU ani stav vozidla.")
        } else if identity.part_number.is_none() || identity.software_version.is_none() {
            (VerificationStatus::Unverified, "Nepodařilo se získat úplnou identitu motorové ECU (F187/F189). Výrobce-specifické hodnoty nebyly vyžádány ani dekódovány.")
        } else if manifest.variant_matchers.iter().any(|matcher| {
            matcher.ecu_address == "01"
                && matcher.signal_groups.contains(&group)
                && identity.part_number.as_deref() == Some(matcher.part_number.as_str())
                && identity
                    .software_version
                    .as_ref()
                    .is_some_and(|version| matcher.software_versions.contains(version))
        }) {
            (VerificationStatus::Matched, "Číslo dílu, verze software a skupina signálů odpovídají datovému profilu. Shoda definice není potvrzení bezvadnosti vozidla.")
        } else {
            (VerificationStatus::Unverified, "Pro tuto kombinaci čísla dílu, verze software a skupiny signálů není doložený dekodér. Výrobce-specifické hodnoty nebyly vyžádány ani dekódovány. Standardní hodnoty lze číst v univerzálním OBD-II profilu.")
        };
        Self::new(manifest, status, detail, Some(identity))
    }

    fn new(
        manifest: &ProfileManifest,
        status: VerificationStatus,
        detail: &str,
        identity: Option<EcuIdentity>,
    ) -> Self {
        Self {
            status,
            detail: detail.to_owned(),
            profile_id: manifest.id.clone(),
            profile_version: manifest.version.clone(),
            source: manifest.data_source.reference.clone(),
            identity,
        }
    }

    pub fn allows_mqb_decoding(&self) -> bool {
        matches!(
            self.status,
            VerificationStatus::Matched | VerificationStatus::Simulated
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vehicle::{profile_manifest, DiagnosticProfile};

    fn fixture() -> (ProfileManifest, EcuIdentity) {
        let mut manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        // Synthetic test-only version: not a claim of real ECU compatibility.
        manifest.variant_matchers[0].software_versions = vec!["TEST-0001".into()];
        (
            manifest,
            EcuIdentity {
                part_number: Some("04L906027P".into()),
                software_version: Some("TEST-0001".into()),
            },
        )
    }

    #[test]
    fn requires_exact_part_software_and_signal_group() {
        let (manifest, identity) = fixture();
        assert_eq!(
            LiveDataVerification::for_mqb_engine(
                &manifest,
                LiveDataGroup::Dpf,
                identity.clone(),
                false
            )
            .status,
            VerificationStatus::Matched
        );
        for group in [
            LiveDataGroup::Engine,
            LiveDataGroup::Injectors,
            LiveDataGroup::Turbo,
        ] {
            assert!(!LiveDataVerification::for_mqb_engine(
                &manifest,
                group,
                identity.clone(),
                false
            )
            .allows_mqb_decoding());
        }
        for identity in [
            EcuIdentity {
                part_number: Some("04L906027Q".into()),
                ..identity.clone()
            },
            EcuIdentity {
                software_version: Some("TEST-0002".into()),
                ..identity.clone()
            },
            EcuIdentity {
                software_version: None,
                ..identity.clone()
            },
            EcuIdentity {
                part_number: None,
                ..identity
            },
        ] {
            assert!(!LiveDataVerification::for_mqb_engine(
                &manifest,
                LiveDataGroup::Dpf,
                identity,
                false
            )
            .allows_mqb_decoding());
        }
    }

    #[test]
    fn absent_software_evidence_is_never_a_wildcard() {
        let (_, identity) = fixture();
        let manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        assert!(!LiveDataVerification::for_mqb_engine(
            &manifest,
            LiveDataGroup::Dpf,
            identity,
            false
        )
        .allows_mqb_decoding());
    }

    #[test]
    fn demo_and_standard_do_not_claim_a_verified_ecu_match() {
        let (manifest, identity) = fixture();
        assert_eq!(
            LiveDataVerification::for_mqb_engine(&manifest, LiveDataGroup::Dpf, identity, true)
                .status,
            VerificationStatus::Simulated
        );
        assert!(!LiveDataVerification::standard(&manifest, false).allows_mqb_decoding());
    }
}
