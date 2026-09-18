use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::error::{AppError, AppResult};

/// Seznam potvrzených DTC. Subfunkce 0x02 je na MQB spolehlivější než
/// 0x01 (pouhý počet), kterou některé jednotky odmítají nebo zkracují.
pub(crate) const CONFIRMED_DTC_LIST_REQUEST: &str = "190208";
pub(crate) const MAX_DID_PROBES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlUnitArea {
    Transmission,
    Chassis,
    Interior,
    Lighting,
    Exterior,
    Climate,
    Infotainment,
    Body,
    Windows,
    DrivingModes,
}

impl ControlUnitArea {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Transmission => "Převodovka a pohon",
            Self::Chassis => "Brzdy, podvozek a asistenti",
            Self::Interior => "Interiér a bezpečnost",
            Self::Lighting => "Světla",
            Self::Exterior => "Exteriér",
            Self::Climate => "Klimatizace",
            Self::Infotainment => "Infotainment",
            Self::Body => "Karoserie a komfort",
            Self::Windows => "Okna a dveře",
            Self::DrivingModes => "Jízdní režimy",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlUnitAreaSnapshot {
    pub area: ControlUnitArea,
    pub captured_at_ms: u64,
    pub units: Vec<ControlUnitInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidProbeSnapshot {
    pub area: ControlUnitArea,
    pub unit_key: String,
    pub unit_name: String,
    pub address: String,
    pub tx_id: u32,
    pub rx_id: u32,
    pub captured_at_ms: u64,
    pub values: Vec<DidProbeValue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidProbeValue {
    pub did: String,
    pub status: DidProbeStatus,
    pub data_hex: Option<String>,
    pub payload_hex: Option<String>,
    pub raw_response: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DidProbeStatus {
    Readable,
    Unsupported,
    Rejected,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlUnitInfo {
    pub key: String,
    pub address: String,
    pub name: String,
    pub tx_id: u32,
    pub rx_id: u32,
    pub status: ControlUnitScanStatus,
    pub part_number: Option<String>,
    pub software_version: Option<String>,
    pub dtc_count: Option<u16>,
    pub dtcs: Vec<DiagnosticTroubleCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticTroubleCode {
    pub code: String,
    pub raw: String,
    pub status: u8,
    pub failure_type: u8,
    pub confirmed: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlUnitScanStatus {
    Detected,
    NotDetected,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ControlUnitDefinition {
    pub key: &'static str,
    pub address: &'static str,
    pub name: &'static str,
    pub tx_id: u32,
    pub rx_id: u32,
}

const TRANSMISSION_UNITS: &[ControlUnitDefinition] = &[
    unit(
        "transmission",
        "02",
        "Řídicí jednotka převodovky",
        0x7E1,
        0x7E9,
    ),
    unit("allWheelDrive", "22", "Pohon všech kol", 0x71D, 0x787),
    unit("gearSelector", "81", "Volicí páka převodovky", 0x753, 0x7BD),
];

const CHASSIS_UNITS: &[ControlUnitDefinition] = &[
    unit("absEsc", "03", "ABS / ESC", 0x713, 0x77D),
    unit(
        "parkingBrake",
        "53",
        "Elektronická parkovací brzda",
        0x752,
        0x7BC,
    ),
    unit("steeringAssist", "44", "Posilovač řízení", 0x712, 0x77C),
    unit(
        "adaptiveDamping",
        "14",
        "Adaptivní podvozek DCC",
        0x772,
        0x7DC,
    ),
    unit("adaptiveCruise", "13", "Adaptivní tempomat", 0x757, 0x7C1),
    unit(
        "frontCamera",
        "A5",
        "Kamera asistenčních systémů",
        0x74F,
        0x7B9,
    ),
];

const INTERIOR_UNITS: &[ControlUnitDefinition] = &[
    unit("dashboard", "17", "Přístrojový panel", 0x714, 0x77E),
    unit("airbag", "15", "Airbag", 0x715, 0x77F),
    unit(
        "steeringColumn",
        "16",
        "Elektronika sloupku řízení",
        0x70C,
        0x776,
    ),
    unit("driverSeat", "36", "Sedadlo řidiče", 0x74C, 0x7B6),
    unit("passengerSeat", "06", "Sedadlo spolujezdce", 0x74D, 0x7B7),
];

const LIGHTING_UNITS: &[ControlUnitDefinition] = &[
    unit(
        "centralElectrics",
        "09",
        "Centrální elektrika J519",
        0x70E,
        0x778,
    ),
    unit(
        "headlightRange",
        "55",
        "Regulace sklonu světlometů",
        0x754,
        0x7BE,
    ),
    unit(
        "highBeamAssist",
        "20",
        "Asistent dálkových světel",
        0x748,
        0x7B2,
    ),
];

const EXTERIOR_UNITS: &[ControlUnitDefinition] = &[
    unit("parkingAssist", "10", "Parkovací asistence", 0x70A, 0x774),
    unit("rearCamera", "6C", "Zadní kamera", 0x769, 0x7D3),
    unit(
        "deckLid",
        "6D",
        "Elektrické víko zavazadlového prostoru",
        0x723,
        0x78D,
    ),
    unit("trailer", "69", "Tažné zařízení", 0x747, 0x7B1),
    unit(
        "laneChangeAssist",
        "3C",
        "Asistent změny jízdního pruhu",
        0x74E,
        0x7B8,
    ),
];

const CLIMATE_UNITS: &[ControlUnitDefinition] = &[
    unit("climatronic", "08", "Climatronic J255", 0x746, 0x7B0),
    unit("auxiliaryHeater", "18", "Přídavné topení", 0x76A, 0x7D4),
    unit(
        "rearClimate",
        "28",
        "Zadní ovládání klimatizace",
        0x71A,
        0x784,
    ),
    unit(
        "airConditioningCompressor",
        "C5",
        "Kompresor klimatizace",
        0x719,
        0x783,
    ),
];

const INFOTAINMENT_UNITS: &[ControlUnitDefinition] = &[
    unit(
        "informationElectronics",
        "5F",
        "Informační elektronika",
        0x773,
        0x7DD,
    ),
    unit("radio", "56", "Rádio", 0x718, 0x782),
    unit("navigation", "37", "Navigace", 0x76C, 0x7D6),
    unit("soundSystem", "47", "Zvukový systém", 0x76F, 0x7D9),
    unit("telematics", "75", "Telematika", 0x767, 0x7D1),
    unit("telephone", "77", "Telefonní rozhraní", 0x76B, 0x7D5),
];

const BODY_UNITS: &[ControlUnitDefinition] = &[
    unit(
        "centralElectrics",
        "09",
        "Centrální elektrika J519",
        0x70E,
        0x778,
    ),
    unit("comfort", "46", "Centrální komfortní systém", 0x70D, 0x777),
    unit("accessStart", "05", "Přístup a start KESSY", 0x732, 0x79C),
    unit("gateway", "19", "Diagnostická brána J533", 0x710, 0x77A),
    unit(
        "deckLid",
        "6D",
        "Elektrické víko zavazadlového prostoru",
        0x723,
        0x78D,
    ),
    unit("roof", "CA", "Střešní systém", 0x72D, 0x797),
];

const WINDOW_UNITS: &[ControlUnitDefinition] = &[
    unit("driverDoor", "42", "Dveře a okno řidiče", 0x74A, 0x7B4),
    unit(
        "passengerDoor",
        "52",
        "Dveře a okno spolujezdce",
        0x74B,
        0x7B5,
    ),
    unit(
        "rearLeftDoor",
        "BB",
        "Levé zadní dveře a okno",
        0x73E,
        0x7A8,
    ),
    unit(
        "rearRightDoor",
        "BC",
        "Pravé zadní dveře a okno",
        0x73F,
        0x7A9,
    ),
];

const DRIVING_MODE_UNITS: &[ControlUnitDefinition] = &[
    unit("engine", "01", "Řídicí jednotka motoru", 0x7E0, 0x7E8),
    unit(
        "transmission",
        "02",
        "Řídicí jednotka převodovky",
        0x7E1,
        0x7E9,
    ),
    unit("gateway", "19", "Diagnostická brána J533", 0x710, 0x77A),
    unit("steeringAssist", "44", "Posilovač řízení", 0x712, 0x77C),
    unit(
        "adaptiveDamping",
        "14",
        "Adaptivní podvozek DCC",
        0x772,
        0x7DC,
    ),
    unit(
        "informationElectronics",
        "5F",
        "Informační elektronika",
        0x773,
        0x7DD,
    ),
    unit("allWheelDrive", "22", "Pohon všech kol", 0x71D, 0x787),
];

const fn unit(
    key: &'static str,
    address: &'static str,
    name: &'static str,
    tx_id: u32,
    rx_id: u32,
) -> ControlUnitDefinition {
    ControlUnitDefinition {
        key,
        address,
        name,
        tx_id,
        rx_id,
    }
}

pub(crate) fn control_unit_definitions(area: ControlUnitArea) -> &'static [ControlUnitDefinition] {
    match area {
        ControlUnitArea::Transmission => TRANSMISSION_UNITS,
        ControlUnitArea::Chassis => CHASSIS_UNITS,
        ControlUnitArea::Interior => INTERIOR_UNITS,
        ControlUnitArea::Lighting => LIGHTING_UNITS,
        ControlUnitArea::Exterior => EXTERIOR_UNITS,
        ControlUnitArea::Climate => CLIMATE_UNITS,
        ControlUnitArea::Infotainment => INFOTAINMENT_UNITS,
        ControlUnitArea::Body => BODY_UNITS,
        ControlUnitArea::Windows => WINDOW_UNITS,
        ControlUnitArea::DrivingModes => DRIVING_MODE_UNITS,
    }
}

pub(crate) fn find_control_unit_definition(
    area: ControlUnitArea,
    unit_key: &str,
) -> AppResult<&'static ControlUnitDefinition> {
    control_unit_definitions(area)
        .iter()
        .find(|definition| definition.key == unit_key)
        .ok_or_else(|| {
            AppError::validation(
                "Řídicí jednotka nepatří do vybrané oblasti diagnostického katalogu.",
            )
        })
}

pub(crate) fn validate_probe_dids(dids: Vec<u16>) -> AppResult<Vec<u16>> {
    if dids.is_empty() {
        return Err(AppError::validation(
            "Zadejte alespoň jeden čtyřmístný DID.",
        ));
    }
    if dids.len() > MAX_DID_PROBES {
        return Err(AppError::validation(format!(
            "Jeden průzkum může obsahovat nejvýše {MAX_DID_PROBES} DID.",
        )));
    }

    let mut known = HashSet::with_capacity(dids.len());
    Ok(dids.into_iter().filter(|did| known.insert(*did)).collect())
}

pub(crate) fn extract_did_data(payload_hex: &str, did: u16) -> Option<String> {
    let data = payload_hex.strip_prefix(&format!("62{did:04X}"))?;
    (data.len() % 2 == 0 && data.chars().all(|character| character.is_ascii_hexdigit()))
        .then(|| data.to_owned())
}

pub(crate) fn decode_ascii_identifier(payload_hex: &str, did: u16) -> Option<String> {
    let prefix = format!("62{did:04X}");
    let data = payload_hex.strip_prefix(&prefix)?;
    if data.len() % 2 != 0 || !data.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }

    let bytes = (0..data.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&data[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    let first = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace() && !matches!(byte, 0x00 | 0xFF))?;
    let last = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace() && !matches!(byte, 0x00 | 0xFF))?;
    let value = String::from_utf8_lossy(&bytes[first..=last]).into_owned();

    (!value.is_empty()).then_some(value)
}

pub(crate) fn decode_dtc_list(payload_hex: &str) -> Option<Vec<DiagnosticTroubleCode>> {
    let data = payload_hex.strip_prefix("5902")?;
    // První byte je maska podporovaných stavů, potom následují čtveřice
    // <3 B DTC><1 B status>. Samotná maska znamená platný prázdný seznam.
    if data.len() < 2 || !data.len().saturating_sub(2).is_multiple_of(8) {
        return None;
    }
    if !data.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }

    let bytes = (2..data.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&data[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    const SYSTEMS: [char; 4] = ['P', 'C', 'B', 'U'];

    Some(
        bytes
            .chunks_exact(4)
            .map(|entry| {
                let code = format!(
                    "{}{}{:X}{:02X}",
                    SYSTEMS[(entry[0] >> 6) as usize],
                    (entry[0] >> 4) & 0x03,
                    entry[0] & 0x0F,
                    entry[1]
                );
                DiagnosticTroubleCode {
                    code,
                    raw: format!("{:02X} {:02X} {:02X}", entry[0], entry[1], entry[2]),
                    status: entry[3],
                    failure_type: entry[2],
                    confirmed: entry[3] & 0x08 != 0,
                    active: entry[3] & 0x01 != 0,
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        control_unit_definitions, decode_ascii_identifier, decode_dtc_list, extract_did_data,
        find_control_unit_definition, validate_probe_dids, ControlUnitArea,
        CONFIRMED_DTC_LIST_REQUEST, MAX_DID_PROBES,
    };
    use std::collections::HashSet;

    #[test]
    fn decodes_vag_part_number() {
        assert_eq!(
            decode_ascii_identifier("62F18730344C39303630323750", 0xF187),
            Some("04L906027P".to_owned())
        );
    }

    #[test]
    fn every_area_has_unique_routes() {
        let mut known_routes = HashSet::new();
        for area in [
            ControlUnitArea::Transmission,
            ControlUnitArea::Chassis,
            ControlUnitArea::Interior,
            ControlUnitArea::Lighting,
            ControlUnitArea::Exterior,
            ControlUnitArea::Climate,
            ControlUnitArea::Infotainment,
            ControlUnitArea::Body,
            ControlUnitArea::Windows,
            ControlUnitArea::DrivingModes,
        ] {
            let definitions = control_unit_definitions(area);
            assert!(!definitions.is_empty());
            let keys = definitions
                .iter()
                .map(|definition| definition.key)
                .collect::<HashSet<_>>();
            let routes = definitions
                .iter()
                .map(|definition| (definition.tx_id, definition.rx_id))
                .collect::<HashSet<_>>();
            assert_eq!(keys.len(), definitions.len());
            assert_eq!(routes.len(), definitions.len());
            known_routes.extend(routes);
        }
        assert_eq!(known_routes.len(), 41);
    }

    #[test]
    fn area_uses_frontend_camel_case_name() {
        assert_eq!(
            serde_json::to_string(&ControlUnitArea::Infotainment).unwrap(),
            "\"infotainment\""
        );
        assert_eq!(
            serde_json::to_string(&ControlUnitArea::DrivingModes).unwrap(),
            "\"drivingModes\""
        );
    }

    #[test]
    fn decodes_confirmed_dtc_list() {
        let dtcs = decode_dtc_list("590219FFFF3A08").unwrap();
        assert_eq!(dtcs.len(), 1);
        assert_eq!(dtcs[0].code, "U3FFF");
        assert_eq!(dtcs[0].failure_type, 0x3A);
        assert!(dtcs[0].confirmed);
        assert!(!dtcs[0].active);
        assert_eq!(decode_dtc_list("590299"), Some(Vec::new()));
    }

    #[test]
    fn confirmed_dtc_scan_uses_confirmed_status_bit_only() {
        assert_eq!(CONFIRMED_DTC_LIST_REQUEST, "190208");
    }

    #[test]
    fn validates_bounded_unique_probe_dids() {
        assert_eq!(
            validate_probe_dids(vec![0xF187, 0xF189, 0xF187]).unwrap(),
            vec![0xF187, 0xF189]
        );
        assert!(validate_probe_dids(Vec::new()).is_err());
        assert!(validate_probe_dids(vec![0; MAX_DID_PROBES + 1]).is_err());
    }

    #[test]
    fn resolves_only_units_from_requested_area() {
        assert!(find_control_unit_definition(ControlUnitArea::Body, "gateway").is_ok());
        assert!(find_control_unit_definition(ControlUnitArea::Lighting, "gateway").is_err());
    }

    #[test]
    fn extracts_data_only_for_echoed_did() {
        assert_eq!(
            extract_did_data("62F18730344C", 0xF187),
            Some("30344C".to_owned())
        );
        assert_eq!(extract_did_data("62F18930344C", 0xF187), None);
        assert_eq!(extract_did_data("62F1871", 0xF187), None);
    }
}
