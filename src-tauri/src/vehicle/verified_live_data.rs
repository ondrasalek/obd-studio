use super::live_data::{signal_definitions, LiveDataValue};
use super::manifest::ProfileManifest;
use super::verification::{EcuIdentity, LiveDataVerification};
use super::{decode_ascii_identifier, LiveDataGroup, LiveDataStatus};
use crate::error::AppResult;
use crate::protocol::uds::UdsResponse;

/// Identity is read afresh for every group, including after reconnects. The
/// callback is invoked for signal DIDs only after the exact variant gate passes.
/// `simulated` is supplied exclusively by the backend-owned transport.
pub(crate) fn read_verified_mqb_values(
    group: LiveDataGroup,
    manifest: &ProfileManifest,
    simulated: bool,
    mut read_did: impl FnMut(u16) -> AppResult<UdsResponse>,
) -> AppResult<(LiveDataVerification, Vec<LiveDataValue>)> {
    let mut read_identity = |did| -> AppResult<Option<String>> {
        match read_did(did) {
            Ok(response) if response.negative_response.is_none() => {
                Ok(decode_ascii_identifier(&response.payload_hex, did)
                    .filter(|value| value.bytes().all(|byte| byte.is_ascii_graphic())))
            }
            // A lost serial connection must not turn into a compatibility result.
            Err(error) if error.code == "SERIAL_ERROR" => Err(error),
            _ => Ok(None),
        }
    };
    let identity = EcuIdentity {
        part_number: read_identity(0xF187)?,
        software_version: read_identity(0xF189)?,
    };
    let verification = LiveDataVerification::for_mqb_engine(manifest, group, identity, simulated);
    if !verification.allows_mqb_decoding() {
        let values = signal_definitions(group)
            .iter()
            .map(|definition| {
                definition.unavailable(LiveDataStatus::Unverified, &verification.detail, None)
            })
            .collect();
        return Ok((verification, values));
    }

    let mut values = Vec::new();
    for definition in signal_definitions(group) {
        let value = match read_did(definition.did) {
            Ok(response) => {
                if let Some(negative) = response.negative_response {
                    definition.unavailable(
                        if matches!(negative.code.as_str(), "11" | "12" | "31") {
                            LiveDataStatus::Unsupported
                        } else {
                            LiveDataStatus::Error
                        },
                        negative.description,
                        Some(response.payload_hex),
                    )
                } else {
                    definition
                        .available(response.payload_hex.clone())
                        .unwrap_or_else(|error| {
                            definition.unavailable(
                                LiveDataStatus::Error,
                                error.message,
                                Some(response.payload_hex),
                            )
                        })
                }
            }
            Err(error) if error.code == "SERIAL_ERROR" => return Err(error),
            Err(error) => definition.unavailable(LiveDataStatus::Error, error.message, None),
        };
        values.push(value);
    }
    Ok((verification, values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::stn::StnReply;
    use crate::error::AppError;
    use crate::protocol::uds::{parse_uds_response, UdsRequest};
    use crate::transport::stn::{DemoStnTransport, StnTransport};
    use crate::vehicle::live_data::LiveDataAssessmentLevel;
    use crate::vehicle::verification::VerificationStatus;
    use crate::vehicle::{profile_manifest, DiagnosticProfile};
    use std::time::Duration;

    fn response(did: u16, payload: &str) -> UdsResponse {
        let request = UdsRequest {
            tx_id: 0x7E0,
            rx_id: 0x7E8,
            payload_hex: format!("22{did:04X}"),
            timeout_ms: None,
        }
        .validate()
        .unwrap();
        let raw = format!("7E8{payload}\r>");
        parse_uds_response(1, &request, StnReply::parse(raw, "STPX").unwrap()).unwrap()
    }

    fn identity_reply(did: u16) -> UdsResponse {
        let text = match did {
            0xF187 => "04L906027P",
            0xF189 => "TEST-0001",
            _ => panic!("Unexpected signal request before verification: {did:04X}"),
        };
        let hex: String = text.bytes().map(|byte| format!("{byte:02X}")).collect();
        response(did, &format!("62{did:04X}{hex}"))
    }

    #[test]
    fn real_bundled_profile_reads_identity_only_and_never_guesses_values() {
        let manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        let mut requests = Vec::new();
        let (verification, values) =
            read_verified_mqb_values(LiveDataGroup::Dpf, &manifest, false, |did| {
                requests.push(did);
                Ok(identity_reply(did))
            })
            .unwrap();
        assert_eq!(requests, [0xF187, 0xF189]);
        assert_eq!(verification.status, VerificationStatus::Unverified);
        assert!(!values.is_empty());
        assert!(values
            .iter()
            .all(|value| value.status == LiveDataStatus::Unverified
                && value.value.is_none()
                && value.raw_response.is_none()
                && value.assessment.level == LiveDataAssessmentLevel::Unknown));
    }

    #[test]
    fn rejected_malformed_or_wrong_did_identity_never_authorizes_signals() {
        let mut manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        manifest.variant_matchers[0]
            .software_versions
            .push("TEST-0001".into());
        for payload in [
            "7F2231",
            "62F189FF",
            "62F18830303031",
            "62F18954455354002D30303031",
        ] {
            let (verification, values) =
                read_verified_mqb_values(LiveDataGroup::Dpf, &manifest, false, |did| {
                    Ok(if did == 0xF189 {
                        response(did, payload)
                    } else {
                        identity_reply(did)
                    })
                })
                .unwrap();
            assert_eq!(verification.status, VerificationStatus::Unverified);
            assert!(values.iter().all(|value| value.value.is_none()));
        }
    }

    #[test]
    fn verified_fixture_decodes_but_another_group_or_next_ecu_does_not() {
        let mut manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        manifest.variant_matchers[0]
            .software_versions
            .push("TEST-0001".into());
        let (verification, values) =
            read_verified_mqb_values(LiveDataGroup::Dpf, &manifest, false, |did| {
                Ok(if matches!(did, 0xF187 | 0xF189) {
                    identity_reply(did)
                } else if did == 0x2609 {
                    response(did, "62260904E2")
                } else {
                    response(did, "7F2231")
                })
            })
            .unwrap();
        assert_eq!(verification.status, VerificationStatus::Matched);
        assert_eq!(values[0].value, Some(12.5));
        assert_eq!(values[1].status, LiveDataStatus::Unsupported);
        let (verification, _) =
            read_verified_mqb_values(LiveDataGroup::Engine, &manifest, false, |did| {
                Ok(identity_reply(did))
            })
            .unwrap();
        assert_eq!(verification.status, VerificationStatus::Unverified);
        let (verification, _) =
            read_verified_mqb_values(LiveDataGroup::Dpf, &manifest, false, |did| {
                Ok(if did == 0xF189 {
                    response(did, "7F2231")
                } else {
                    identity_reply(did)
                })
            })
            .unwrap();
        assert_eq!(verification.status, VerificationStatus::Unverified);
    }

    #[test]
    fn disconnected_transport_is_an_error_not_a_compatibility_result() {
        let manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        let error = read_verified_mqb_values(LiveDataGroup::Dpf, &manifest, false, |_| {
            Err(AppError::serial("disconnected"))
        })
        .unwrap_err();
        assert_eq!(error.code, "SERIAL_ERROR");
    }

    #[test]
    fn demo_uses_raw_production_parsers_and_is_never_reported_as_matched() {
        let manifest = profile_manifest(DiagnosticProfile::VwMqb).unwrap();
        let mut transport = DemoStnTransport::new(DiagnosticProfile::VwMqb);
        let (verification, values) = read_verified_mqb_values(
            LiveDataGroup::Dpf,
            &manifest,
            transport.is_simulated(),
            |did| {
                let request = UdsRequest {
                    tx_id: 0x7E0,
                    rx_id: 0x7E8,
                    payload_hex: format!("22{did:04X}"),
                    timeout_ms: None,
                }
                .validate()?;
                let command = format!("STPX h:7E0,d:22{did:04X},t:1000,r:1");
                let raw = transport.exchange(&command, Duration::ZERO)?;
                parse_uds_response(1, &request, StnReply::parse(raw, &command)?)
            },
        )
        .unwrap();
        assert_eq!(verification.status, VerificationStatus::Simulated);
        assert!(values
            .iter()
            .all(|value| value.status == LiveDataStatus::Available));
    }
}
