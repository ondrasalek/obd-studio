use super::{DiagnosticProfile, LiveDataGroup};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CURRENT_SCHEMA_VERSION: u16 = 2;
const GENERIC_MANIFEST: &str = include_str!("profile_manifests/generic-obd2.json");
const MQB_MANIFEST: &str = include_str!("profile_manifests/vw-mqb.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest {
    pub schema_version: u16,
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub diagnostic_profile: DiagnosticProfile,
    pub status: ProfileManifestStatus,
    pub transport: ProfileTransport,
    pub read_services: Vec<String>,
    pub write_services: Vec<String>,
    pub variant_matchers: Vec<EcuVariantMatcher>,
    pub data_source: ProfileDataSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProfileManifestStatus {
    Standard,
    Experimental,
    Verified,
}

impl ProfileManifestStatus {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Experimental => "experimentální",
            Self::Verified => "ověřený",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTransport {
    pub protocol: String,
    pub addressing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcuVariantMatcher {
    pub ecu_address: String,
    pub part_number: String,
    // Exact, evidence-backed versions only. An empty list never authorizes decoding.
    pub software_versions: Vec<String>,
    pub signal_groups: Vec<LiveDataGroup>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDataSource {
    pub kind: ProfileDataSourceKind,
    pub reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProfileDataSourceKind {
    Standard,
    Curated,
    Odx,
}

pub fn profile_manifest(profile: DiagnosticProfile) -> AppResult<ProfileManifest> {
    let raw = match profile {
        DiagnosticProfile::GenericObd2 => GENERIC_MANIFEST,
        DiagnosticProfile::VwMqb => MQB_MANIFEST,
    };
    let manifest: ProfileManifest = serde_json::from_str(raw).map_err(|error| {
        AppError::new(
            "PROFILE_MANIFEST_INVALID",
            format!("Vestavěný diagnostický profil nelze načíst: {error}"),
        )
    })?;
    validate_manifest(&manifest, profile)?;
    Ok(manifest)
}

fn validate_manifest(
    manifest: &ProfileManifest,
    expected_profile: DiagnosticProfile,
) -> AppResult<()> {
    if manifest.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(profile_error(format!(
            "Profil {} používá schema {}, podporováno je schema {}.",
            manifest.id, manifest.schema_version, CURRENT_SCHEMA_VERSION
        )));
    }
    if manifest.id != expected_profile.slug() || manifest.diagnostic_profile != expected_profile {
        return Err(profile_error(
            "Identita manifestu neodpovídá vybranému diagnostickému profilu.",
        ));
    }
    if manifest.version.trim().is_empty() || manifest.display_name.trim().is_empty() {
        return Err(profile_error("Manifest nemá verzi nebo zobrazovaný název."));
    }

    let mut services = HashSet::new();
    for service in manifest
        .read_services
        .iter()
        .chain(manifest.write_services.iter())
    {
        if service.len() != 2
            || !service
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(profile_error(format!(
                "Služba {service} není jednobajtový hex SID."
            )));
        }
        if !services.insert(service.to_ascii_uppercase()) {
            return Err(profile_error(format!(
                "Služba {service} je v manifestu uvedena vícekrát."
            )));
        }
    }

    if expected_profile == DiagnosticProfile::VwMqb && !manifest.write_services.is_empty() {
        return Err(profile_error(
            "Experimentální MQB manifest nesmí povolit zápisové služby.",
        ));
    }
    for matcher in &manifest.variant_matchers {
        if matcher.ecu_address.len() != 2
            || !matcher
                .ecu_address
                .chars()
                .all(|character| character.is_ascii_hexdigit())
            || matcher.part_number.trim().is_empty()
            || matcher.scope.trim().is_empty()
        {
            return Err(profile_error("Manifest obsahuje neplatný ECU matcher."));
        }
        if matcher.signal_groups.is_empty()
            || matcher
                .signal_groups
                .iter()
                .any(|group| super::signal_definitions(*group).is_empty())
            || matcher.software_versions.iter().any(|version| {
                version.trim().is_empty()
                    || version != version.trim()
                    || !version.bytes().all(|byte| byte.is_ascii_graphic())
                    || version.contains(['*', '?'])
            })
        {
            return Err(profile_error(
                "ECU matcher musí určit skupiny signálů a přesné verze software bez wildcardů.",
            ));
        }
    }
    Ok(())
}

fn profile_error(message: impl Into<String>) -> AppError {
    AppError::new("PROFILE_MANIFEST_INVALID", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_manifests_are_valid_and_have_stable_ids() {
        let generic = profile_manifest(DiagnosticProfile::GenericObd2).unwrap();
        let mqb = profile_manifest(DiagnosticProfile::VwMqb).unwrap();

        assert_eq!(generic.id, "generic-obd2");
        assert_eq!(mqb.id, "vw-mqb");
        assert_ne!(generic.id, mqb.id);
    }

    #[test]
    fn mqb_manifest_is_read_only_and_variant_scoped() {
        let manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();

        assert!(manifest.write_services.is_empty());
        assert_eq!(manifest.status, ProfileManifestStatus::Experimental);
        assert!(manifest
            .variant_matchers
            .iter()
            .any(|matcher| matcher.part_number == "04L906027P"));
    }

    #[test]
    fn rejects_duplicate_or_malformed_services() {
        let mut manifest = profile_manifest(DiagnosticProfile::GenericObd2).unwrap();
        manifest.read_services.push("01".to_owned());
        assert!(validate_manifest(&manifest, DiagnosticProfile::GenericObd2).is_err());

        manifest.read_services.pop();
        manifest.read_services.push("XYZ".to_owned());
        assert!(validate_manifest(&manifest, DiagnosticProfile::GenericObd2).is_err());
    }
}
