use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

pub const ENGINE_TX_ID: u32 = 0x7E0;
pub const ENGINE_RX_ID: u32 = 0x7E8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveDataGroup {
    Engine,
    Injectors,
    Dpf,
    Turbo,
}

impl LiveDataGroup {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Engine => "Motor",
            Self::Injectors => "Vstřikování",
            Self::Dpf => "Filtr pevných částic",
            Self::Turbo => "Přeplňování",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDataSnapshot {
    pub group: LiveDataGroup,
    pub ecu_name: String,
    pub tx_id: u32,
    pub rx_id: u32,
    pub captured_at_ms: u64,
    pub values: Vec<LiveDataValue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDataValue {
    pub key: String,
    pub label: String,
    pub did: String,
    pub value: Option<f64>,
    pub unit: String,
    pub precision: u8,
    pub status: LiveDataStatus,
    pub assessment: LiveDataAssessment,
    pub raw_response: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveDataStatus {
    Available,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveDataAssessmentLevel {
    Normal,
    Information,
    Attention,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDataAssessment {
    pub level: LiveDataAssessmentLevel,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SignalDefinition {
    pub key: &'static str,
    pub label: &'static str,
    pub did: u16,
    pub unit: &'static str,
    pub precision: u8,
    decoder: Decoder,
}

#[derive(Debug, Clone, Copy)]
enum Decoder {
    U8 { scale: f64, offset: f64 },
    U16 { scale: f64, offset: f64 },
    I16 { scale: f64, offset: f64 },
    U32 { scale: f64, offset: f64 },
}

const ENGINE_SIGNALS: &[SignalDefinition] = &[
    signal(
        "engineRpm",
        "Otáčky motoru",
        0xF40C,
        "rpm",
        0,
        Decoder::U16 {
            scale: 0.25,
            offset: 0.0,
        },
    ),
    signal(
        "coolantTemperature",
        "Chladicí kapalina",
        0xF405,
        "°C",
        0,
        Decoder::U8 {
            scale: 1.0,
            offset: -40.0,
        },
    ),
    signal(
        "oilTemperature",
        "Teplota motorového oleje",
        0x11BE,
        "°C",
        1,
        Decoder::U16 {
            scale: 0.1,
            offset: -273.1,
        },
    ),
    signal(
        "intakeTemperature",
        "Teplota nasávaného vzduchu",
        0xF40F,
        "°C",
        0,
        Decoder::U8 {
            scale: 1.0,
            offset: -40.0,
        },
    ),
    signal(
        "massAirFlow",
        "Hmotnostní průtok vzduchu",
        0xF410,
        "g/s",
        2,
        Decoder::U16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
    signal(
        "controlModuleVoltage",
        "Napětí řídicí jednotky",
        0x105E,
        "V",
        2,
        Decoder::U16 {
            scale: 0.02,
            offset: 0.0,
        },
    ),
    signal(
        "engineTorque",
        "Aktuální točivý moment",
        0x1047,
        "Nm",
        1,
        Decoder::U16 {
            scale: 0.1,
            offset: 0.0,
        },
    ),
];

const INJECTOR_SIGNALS: &[SignalDefinition] = &[
    signal(
        "injectorDeviation1",
        "Korekce vstřiku · válec 1",
        0x10FF,
        "mg/zdvih",
        2,
        Decoder::I16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
    signal(
        "injectorDeviation2",
        "Korekce vstřiku · válec 2",
        0x1105,
        "mg/zdvih",
        2,
        Decoder::I16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
    signal(
        "injectorDeviation3",
        "Korekce vstřiku · válec 3",
        0x1100,
        "mg/zdvih",
        2,
        Decoder::I16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
    signal(
        "injectorDeviation4",
        "Korekce vstřiku · válec 4",
        0x1104,
        "mg/zdvih",
        2,
        Decoder::I16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
];

const DPF_SIGNALS: &[SignalDefinition] = &[
    signal(
        "dpfSootCalculated",
        "Vypočtená hmotnost sazí",
        0x2609,
        "g",
        2,
        Decoder::I16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
    signal(
        "dpfSootMeasured",
        "Měřená hmotnost sazí",
        0x1ABE,
        "g",
        2,
        Decoder::I16 {
            scale: 0.01,
            offset: 0.0,
        },
    ),
    signal(
        "dpfAshMass",
        "Hmotnost popela",
        0x1ABD,
        "g",
        2,
        Decoder::U32 {
            scale: 0.000_000_119_21,
            offset: 0.0,
        },
    ),
    signal(
        "dpfDifferentialPressure",
        "Diferenční tlak",
        0x14F5,
        "hPa",
        0,
        Decoder::U16 {
            scale: 1.0,
            offset: 0.0,
        },
    ),
    signal(
        "dpfInletTemperature",
        "Teplota před filtrem",
        0x11B2,
        "°C",
        1,
        Decoder::U16 {
            scale: 0.1,
            offset: -273.1,
        },
    ),
    signal(
        "dpfOutletTemperature",
        "Teplota za filtrem",
        0x10F9,
        "°C",
        1,
        Decoder::U16 {
            scale: 0.1,
            offset: -273.1,
        },
    ),
    signal(
        "distanceSinceRegeneration",
        "Vzdálenost od regenerace",
        0x1ABA,
        "km",
        1,
        Decoder::U16 {
            scale: 0.1,
            offset: 0.0,
        },
    ),
    signal(
        "interruptedRegenerations",
        "Přerušené regenerace",
        0x1AC9,
        "×",
        0,
        Decoder::U8 {
            scale: 1.0,
            offset: 0.0,
        },
    ),
];

const TURBO_SIGNALS: &[SignalDefinition] = &[
    signal(
        "boostPressureActual",
        "Skutečný plnicí tlak",
        0x1B0E,
        "hPa",
        0,
        Decoder::U16 {
            scale: 0.078_125,
            offset: 0.0,
        },
    ),
    signal(
        "boostPressureSpecified",
        "Požadovaný plnicí tlak",
        0x1149,
        "hPa",
        0,
        Decoder::U16 {
            scale: 1.0,
            offset: 0.0,
        },
    ),
    signal(
        "boostPressure",
        "Plnicí tlak",
        0x1057,
        "bar",
        2,
        Decoder::U16 {
            scale: 0.001,
            offset: 0.0,
        },
    ),
    signal(
        "turboTemperature",
        "Teplota za turbodmychadlem",
        0x10FB,
        "°C",
        1,
        Decoder::U16 {
            scale: 0.1,
            offset: -273.1,
        },
    ),
    signal(
        "ambientPressure",
        "Atmosférický tlak",
        0xF433,
        "kPa",
        0,
        Decoder::U8 {
            scale: 1.0,
            offset: 0.0,
        },
    ),
];

const fn signal(
    key: &'static str,
    label: &'static str,
    did: u16,
    unit: &'static str,
    precision: u8,
    decoder: Decoder,
) -> SignalDefinition {
    SignalDefinition {
        key,
        label,
        did,
        unit,
        precision,
        decoder,
    }
}

pub(crate) fn signal_definitions(group: LiveDataGroup) -> &'static [SignalDefinition] {
    match group {
        LiveDataGroup::Engine => ENGINE_SIGNALS,
        LiveDataGroup::Injectors => INJECTOR_SIGNALS,
        LiveDataGroup::Dpf => DPF_SIGNALS,
        LiveDataGroup::Turbo => TURBO_SIGNALS,
    }
}

impl SignalDefinition {
    pub fn request_hex(self) -> String {
        format!("22{:04X}", self.did)
    }

    pub fn available(self, payload_hex: String) -> AppResult<LiveDataValue> {
        let expected_prefix = format!("62{:04X}", self.did);
        let data_hex = payload_hex.strip_prefix(&expected_prefix).ok_or_else(|| {
            AppError::adapter(format!(
                "DID 0x{:04X} vrátil neočekávaný payload {payload_hex}.",
                self.did
            ))
        })?;
        let bytes = decode_hex(data_hex)?;
        let value = match self.decoder {
            Decoder::U8 { scale, offset } => {
                f64::from(*bytes.first().ok_or_else(|| missing_bytes(self.did, 1))?) * scale
                    + offset
            }
            Decoder::U16 { scale, offset } => {
                let raw = u16::from_be_bytes(read_array::<2>(&bytes, self.did)?);
                f64::from(raw) * scale + offset
            }
            Decoder::I16 { scale, offset } => {
                let raw = i16::from_be_bytes(read_array::<2>(&bytes, self.did)?);
                f64::from(raw) * scale + offset
            }
            Decoder::U32 { scale, offset } => {
                let raw = u32::from_be_bytes(read_array::<4>(&bytes, self.did)?);
                f64::from(raw) * scale + offset
            }
        };

        Ok(LiveDataValue {
            key: self.key.to_owned(),
            label: self.label.to_owned(),
            did: format!("{:04X}", self.did),
            value: Some(value),
            unit: self.unit.to_owned(),
            precision: self.precision,
            status: LiveDataStatus::Available,
            assessment: assess_signal(self.key, value),
            raw_response: Some(payload_hex),
            detail: None,
        })
    }

    pub fn unavailable(
        self,
        status: LiveDataStatus,
        detail: impl Into<String>,
        raw_response: Option<String>,
    ) -> LiveDataValue {
        let detail = detail.into();
        let assessment_label = match status {
            LiveDataStatus::Unsupported => "Nepodporováno",
            LiveDataStatus::Error => "Nelze vyhodnotit",
            LiveDataStatus::Available => "Bez hodnocení",
        };

        LiveDataValue {
            key: self.key.to_owned(),
            label: self.label.to_owned(),
            did: format!("{:04X}", self.did),
            value: None,
            unit: self.unit.to_owned(),
            precision: self.precision,
            status,
            assessment: assessment(
                LiveDataAssessmentLevel::Unknown,
                assessment_label,
                "Řídicí jednotka neposkytla hodnotu, proto nelze určit její stav.",
            ),
            raw_response,
            detail: Some(detail),
        }
    }
}

fn assess_signal(key: &str, value: f64) -> LiveDataAssessment {
    match key {
        "engineRpm" => assess_engine_rpm(value),
        "coolantTemperature" => assess_coolant_temperature(value),
        "oilTemperature" => assess_oil_temperature(value),
        "intakeTemperature" => assess_intake_temperature(value),
        "controlModuleVoltage" => assess_control_module_voltage(value),
        "injectorDeviation1" | "injectorDeviation2" | "injectorDeviation3"
        | "injectorDeviation4" => assess_injector_deviation(value),
        "dpfSootCalculated" => assess_dpf_soot(value, false),
        "dpfSootMeasured" => assess_dpf_soot(value, true),
        "dpfDifferentialPressure" => assess_dpf_pressure(value),
        "dpfInletTemperature" | "dpfOutletTemperature" | "turboTemperature" => {
            assess_exhaust_temperature(value)
        }
        "distanceSinceRegeneration" => assess_regeneration_distance(value),
        "interruptedRegenerations" => assess_interrupted_regenerations(value),
        "ambientPressure" => assess_ambient_pressure(value),
        "dpfAshMass" => assessment(
            LiveDataAssessmentLevel::Information,
            "Sledujte trend",
            "Limit popela závisí na konkrétní variantě DPF a nelze jej bezpečně určit pouze z této hodnoty.",
        ),
        "massAirFlow" | "engineTorque" => contextual(
            "Vyžaduje provozní kontext",
            "Hodnota se mění podle zatížení a otáček; samostatně neurčuje technický stav motoru.",
        ),
        "boostPressureActual" | "boostPressureSpecified" | "boostPressure" => contextual(
            "Vyžaduje porovnání",
            "Plnicí tlak je potřeba hodnotit při zatížení společně s požadovanou hodnotou.",
        ),
        _ => contextual(
            "Informativní hodnota",
            "Pro tuto hodnotu zatím není v aktivním profilu definován bezpečný limit.",
        ),
    }
}

fn assess_engine_rpm(value: f64) -> LiveDataAssessment {
    if value < 300.0 {
        contextual(
            "Motor stojí",
            "Hodnota odpovídá zapnutému zapalování bez běžícího motoru.",
        )
    } else if value < 550.0 {
        contextual(
            "Startovací otáčky",
            "Nízká hodnota může krátce odpovídat startování motoru.",
        )
    } else if value <= 1_100.0 {
        assessment(
            LiveDataAssessmentLevel::Normal,
            "Volnoběžný rozsah",
            "Otáčky jsou v obvyklém rozsahu volnoběhu.",
        )
    } else if value <= 4_500.0 {
        contextual(
            "Provozní otáčky",
            "Vyšší otáčky jsou běžné při jízdě; stav závisí na zatížení a teplotě motoru.",
        )
    } else {
        assessment(
            LiveDataAssessmentLevel::Attention,
            "Vysoké otáčky",
            "Hodnota je vysoká; posuďte ji podle typu motoru a aktuální jízdní situace.",
        )
    }
}

fn assess_coolant_temperature(value: f64) -> LiveDataAssessment {
    if value < 70.0 {
        contextual(
            "Motor se zahřívá",
            "Teplota je pod běžným rozsahem zahřátého motoru.",
        )
    } else if value <= 105.0 {
        normal("Běžná provozní teplota")
    } else if value <= 115.0 {
        attention("Zvýšená teplota", "Sledujte, zda teplota dále neroste.")
    } else {
        critical(
            "Přehřátí motoru",
            "Bezpečně zastavte a řiďte se pokyny výrobce vozidla.",
        )
    }
}

fn assess_oil_temperature(value: f64) -> LiveDataAssessment {
    if value < 70.0 {
        contextual(
            "Olej se zahřívá",
            "Před vysokým zatížením nechte motorový olej dosáhnout provozní teploty.",
        )
    } else if value <= 120.0 {
        normal("Běžná provozní teplota")
    } else if value <= 135.0 {
        attention(
            "Zvýšená teplota oleje",
            "Omezte zatížení a sledujte další vývoj teploty.",
        )
    } else {
        critical(
            "Velmi vysoká teplota oleje",
            "Omezte zatížení a postupujte podle pokynů výrobce vozidla.",
        )
    }
}

fn assess_intake_temperature(value: f64) -> LiveDataAssessment {
    if value <= 70.0 {
        normal("Přijatelná teplota sání")
    } else if value <= 90.0 {
        attention(
            "Teplý nasávaný vzduch",
            "Hodnotu ovlivňuje okolní teplota, stání vozidla a zatížení motoru.",
        )
    } else {
        attention(
            "Velmi teplý nasávaný vzduch",
            "Zkontrolujte hodnotu za jízdy a funkci chlazení plnicího vzduchu.",
        )
    }
}

fn assess_control_module_voltage(value: f64) -> LiveDataAssessment {
    if value < 11.5 {
        critical(
            "Velmi nízké napětí",
            "Diagnostické nebo servisní procedury neprovádějte; zkontrolujte baterii a napájení.",
        )
    } else if value < 12.0 {
        attention(
            "Nízké napětí",
            "Zkontrolujte stav baterie, zejména před delší diagnostikou.",
        )
    } else if value <= 15.2 {
        normal("Napětí je v obvyklém rozsahu")
    } else if value <= 15.8 {
        attention(
            "Zvýšené napětí",
            "Ověřte nabíjecí soustavu a podmínky měření.",
        )
    } else {
        critical(
            "Velmi vysoké napětí",
            "Ukončete diagnostické procedury a nechte zkontrolovat nabíjecí soustavu.",
        )
    }
}

fn assess_injector_deviation(value: f64) -> LiveDataAssessment {
    let deviation = value.abs();
    if deviation <= 1.5 {
        normal("Korekce je vyrovnaná")
    } else if deviation <= 3.0 {
        attention(
            "Zvýšená korekce",
            "Porovnejte všechny válce na zahřátém motoru při stabilním volnoběhu.",
        )
    } else {
        critical(
            "Výrazná korekce",
            "Hodnota vyžaduje odborné ověření vstřikování a mechanického stavu válce.",
        )
    }
}

fn assess_dpf_soot(value: f64, measured: bool) -> LiveDataAssessment {
    if measured && value < 0.0 {
        contextual(
            "Korekce modelu",
            "Mírně záporná měřená hodnota může vzniknout korekcí modelu diferenčního tlaku.",
        )
    } else if value < 20.0 {
        normal("Nízké až běžné zanesení")
    } else if value <= 40.0 {
        attention(
            "Zvýšené množství sazí",
            "Může se blížit nebo probíhat regenerace; sledujte teploty a další vývoj.",
        )
    } else {
        critical(
            "Vysoké množství sazí",
            "Další postup ověřte servisní diagnostikou; nevynucujte regeneraci bez kontrolních podmínek.",
        )
    }
}

fn assess_dpf_pressure(value: f64) -> LiveDataAssessment {
    if value <= 15.0 {
        normal("Nízký diferenční tlak")
    } else if value <= 40.0 {
        attention(
            "Zvýšený diferenční tlak",
            "Výsledek porovnejte na zahřátém motoru při volnoběhu; při zatížení je vyšší tlak běžný.",
        )
    } else {
        attention(
            "Vysoký diferenční tlak",
            "Hodnota bez znalosti otáček není definitivní; ověřte ji při stabilním volnoběhu.",
        )
    }
}

fn assess_exhaust_temperature(value: f64) -> LiveDataAssessment {
    if value <= 750.0 {
        normal("Teplota v měřitelném provozním rozsahu")
    } else {
        attention(
            "Velmi vysoká teplota",
            "Vysoká teplota může souviset s regenerací DPF; sledujte provozní stav vozidla.",
        )
    }
}

fn assess_regeneration_distance(value: f64) -> LiveDataAssessment {
    if value <= 600.0 {
        normal("Běžný interval od regenerace")
    } else {
        attention(
            "Dlouhý interval od regenerace",
            "Posuďte společně s množstvím sazí, teplotami a jízdním profilem.",
        )
    }
}

fn assess_interrupted_regenerations(value: f64) -> LiveDataAssessment {
    if value < 0.5 {
        normal("Bez přerušené regenerace")
    } else {
        attention(
            "Regenerace byla přerušena",
            "Opakované přerušování může zvyšovat zanesení; sledujte četnost a množství sazí.",
        )
    }
}

fn assess_ambient_pressure(value: f64) -> LiveDataAssessment {
    if (70.0..=110.0).contains(&value) {
        normal("Plausibilní atmosférický tlak")
    } else {
        contextual(
            "Ověřte podle nadmořské výšky",
            "Atmosférický tlak výrazně závisí na počasí a nadmořské výšce.",
        )
    }
}

fn normal(detail: &str) -> LiveDataAssessment {
    assessment(LiveDataAssessmentLevel::Normal, "V pořádku", detail)
}

fn contextual(label: &str, detail: &str) -> LiveDataAssessment {
    assessment(LiveDataAssessmentLevel::Information, label, detail)
}

fn attention(label: &str, detail: &str) -> LiveDataAssessment {
    assessment(LiveDataAssessmentLevel::Attention, label, detail)
}

fn critical(label: &str, detail: &str) -> LiveDataAssessment {
    assessment(LiveDataAssessmentLevel::Critical, label, detail)
}

fn assessment(level: LiveDataAssessmentLevel, label: &str, detail: &str) -> LiveDataAssessment {
    LiveDataAssessment {
        level,
        label: label.to_owned(),
        detail: detail.to_owned(),
    }
}

fn decode_hex(value: &str) -> AppResult<Vec<u8>> {
    if !value.len().is_multiple_of(2) || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::adapter(
            "Živá data obsahují neplatný hex payload.",
        ));
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AppError::adapter("Živá data nelze dekódovat."))
}

fn read_array<const N: usize>(bytes: &[u8], did: u16) -> AppResult<[u8; N]> {
    bytes
        .get(..N)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| missing_bytes(did, N))
}

fn missing_bytes(did: u16, expected: usize) -> AppError {
    AppError::adapter(format!(
        "DID 0x{did:04X} nevrátil očekávaných {expected} B dat."
    ))
}

#[cfg(test)]
mod tests {
    use super::{signal_definitions, LiveDataAssessmentLevel, LiveDataGroup};

    fn signal(group: LiveDataGroup, key: &str) -> super::SignalDefinition {
        *signal_definitions(group)
            .iter()
            .find(|signal| signal.key == key)
            .unwrap()
    }

    #[test]
    fn decodes_signed_measured_soot_mass() {
        let value = signal(LiveDataGroup::Dpf, "dpfSootMeasured")
            .available("621ABEFF74".into())
            .unwrap();
        assert!((value.value.unwrap() - -1.4).abs() < 0.000_001);
    }

    #[test]
    fn decodes_four_byte_ash_mass() {
        let value = signal(LiveDataGroup::Dpf, "dpfAshMass")
            .available("621ABD1C89E2C6".into())
            .unwrap();
        let decoded = value.value.unwrap();
        assert!((decoded - 57.077_573).abs() < 0.000_001);
    }

    #[test]
    fn decodes_deci_kelvin_temperature() {
        let value = signal(LiveDataGroup::Dpf, "dpfInletTemperature")
            .available("6211B20EE0".into())
            .unwrap();
        assert!((value.value.unwrap() - 107.7).abs() < 0.000_001);
    }

    #[test]
    fn rejects_payload_for_different_did() {
        let error = signal(LiveDataGroup::Engine, "engineRpm")
            .available("62F4056A".into())
            .unwrap_err();
        assert_eq!(error.code, "ADAPTER_ERROR");
    }

    #[test]
    fn classifies_confirmed_dpf_values() {
        let soot = signal(LiveDataGroup::Dpf, "dpfSootCalculated")
            .available("62260905AA".into())
            .unwrap();
        let pressure = signal(LiveDataGroup::Dpf, "dpfDifferentialPressure")
            .available("6214F50007".into())
            .unwrap();

        assert_eq!(soot.assessment.level, LiveDataAssessmentLevel::Normal);
        assert_eq!(pressure.assessment.level, LiveDataAssessmentLevel::Normal);
    }

    #[test]
    fn distinguishes_warming_oil_from_overheated_oil() {
        let warming = signal(LiveDataGroup::Engine, "oilTemperature")
            .available("6211BE0CC4".into())
            .unwrap();
        let overheated = signal(LiveDataGroup::Engine, "oilTemperature")
            .available("6211BE100A".into())
            .unwrap();

        assert_eq!(
            warming.assessment.level,
            LiveDataAssessmentLevel::Information
        );
        assert_eq!(
            overheated.assessment.level,
            LiveDataAssessmentLevel::Critical
        );
    }

    #[test]
    fn marks_large_injector_correction_as_critical() {
        let value = signal(LiveDataGroup::Injectors, "injectorDeviation1")
            .available("6210FF0141".into())
            .unwrap();

        assert_eq!(value.value, Some(3.21));
        assert_eq!(value.assessment.level, LiveDataAssessmentLevel::Critical);
    }
}
