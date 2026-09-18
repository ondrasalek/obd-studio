use crate::adapter::stn::StnReply;
use crate::vehicle::live_data::{
    assess_signal, LiveDataAssessment, LiveDataAssessmentLevel, LiveDataGroup, LiveDataStatus,
    LiveDataValue,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ObdSignalDefinition {
    pub key: &'static str,
    pub label: &'static str,
    pub pid: u8,
    pub unit: &'static str,
    pub precision: u8,
    decoder: ObdDecoder,
}

#[derive(Debug, Clone, Copy)]
enum ObdDecoder {
    Byte,
    ByteMinus40,
    BytePercent,
    FuelTrim,
    Word,
    WordQuarter,
    WordHundredth,
    WordMilli,
    WordTimesTen,
    WordTwentieth,
    SensorArrayTemperature,
    RawOnly,
}

const ENGINE: &[ObdSignalDefinition] = &[
    signal(
        "engineRpm",
        "Otáčky motoru",
        0x0C,
        "rpm",
        0,
        ObdDecoder::WordQuarter,
    ),
    signal(
        "coolantTemperature",
        "Teplota chladicí kapaliny",
        0x05,
        "°C",
        0,
        ObdDecoder::ByteMinus40,
    ),
    signal(
        "oilTemperature",
        "Teplota motorového oleje",
        0x5C,
        "°C",
        0,
        ObdDecoder::ByteMinus40,
    ),
    signal(
        "intakeTemperature",
        "Teplota nasávaného vzduchu",
        0x0F,
        "°C",
        0,
        ObdDecoder::ByteMinus40,
    ),
    signal(
        "engineLoad",
        "Vypočtené zatížení motoru",
        0x04,
        "%",
        1,
        ObdDecoder::BytePercent,
    ),
    signal(
        "vehicleSpeed",
        "Rychlost vozidla",
        0x0D,
        "km/h",
        0,
        ObdDecoder::Byte,
    ),
];

const INJECTION: &[ObdSignalDefinition] = &[
    signal(
        "fuelRailPressure",
        "Tlak paliva v liště",
        0x23,
        "kPa",
        0,
        ObdDecoder::WordTimesTen,
    ),
    signal(
        "shortFuelTrim",
        "Krátkodobá korekce směsi",
        0x06,
        "%",
        1,
        ObdDecoder::FuelTrim,
    ),
    signal(
        "longFuelTrim",
        "Dlouhodobá korekce směsi",
        0x07,
        "%",
        1,
        ObdDecoder::FuelTrim,
    ),
    signal(
        "fuelRate",
        "Průtok paliva",
        0x5E,
        "l/h",
        2,
        ObdDecoder::WordTwentieth,
    ),
];

const DPF: &[ObdSignalDefinition] = &[
    signal(
        "exhaustTemperatureBank1",
        "Teplota výfuku · banka 1",
        0x78,
        "°C",
        1,
        ObdDecoder::SensorArrayTemperature,
    ),
    signal(
        "exhaustTemperatureBank2",
        "Teplota výfuku · banka 2",
        0x79,
        "°C",
        1,
        ObdDecoder::SensorArrayTemperature,
    ),
    signal(
        "dpfTemperature",
        "Teplota filtru pevných částic",
        0x7C,
        "°C",
        1,
        ObdDecoder::SensorArrayTemperature,
    ),
    signal(
        "dpfPressureRaw",
        "Tlak DPF · syrová data",
        0x7A,
        "",
        0,
        ObdDecoder::RawOnly,
    ),
];

const TURBO: &[ObdSignalDefinition] = &[
    signal(
        "intakeManifoldPressure",
        "Absolutní tlak v sání",
        0x0B,
        "kPa",
        0,
        ObdDecoder::Byte,
    ),
    signal(
        "ambientPressure",
        "Barometrický tlak",
        0x33,
        "kPa",
        0,
        ObdDecoder::Byte,
    ),
    signal(
        "massAirFlow",
        "Hmotnostní průtok vzduchu",
        0x10,
        "g/s",
        2,
        ObdDecoder::WordHundredth,
    ),
    signal(
        "throttlePosition",
        "Poloha škrticí klapky",
        0x11,
        "%",
        1,
        ObdDecoder::BytePercent,
    ),
];

const ELECTRICAL: &[ObdSignalDefinition] = &[signal(
    "batteryVoltage",
    "Napětí řídicí jednotky / palubní sítě",
    0x42,
    "V",
    3,
    ObdDecoder::WordMilli,
)];

const EMISSIONS: &[ObdSignalDefinition] = &[
    signal(
        "shortFuelTrim",
        "Krátkodobá korekce směsi",
        0x06,
        "%",
        1,
        ObdDecoder::FuelTrim,
    ),
    signal(
        "longFuelTrim",
        "Dlouhodobá korekce směsi",
        0x07,
        "%",
        1,
        ObdDecoder::FuelTrim,
    ),
    signal(
        "oxygenSensorsRaw",
        "Lambda sondy · konfigurace",
        0x13,
        "",
        0,
        ObdDecoder::RawOnly,
    ),
    signal(
        "noxSensorRaw",
        "NOx senzor · syrová data",
        0x83,
        "",
        0,
        ObdDecoder::RawOnly,
    ),
];

const FUEL: &[ObdSignalDefinition] = &[
    signal(
        "fuelLevel",
        "Hladina paliva",
        0x2F,
        "%",
        1,
        ObdDecoder::BytePercent,
    ),
    signal(
        "fuelRailPressure",
        "Tlak paliva v liště",
        0x23,
        "kPa",
        0,
        ObdDecoder::WordTimesTen,
    ),
    signal(
        "fuelRate",
        "Okamžitý průtok paliva",
        0x5E,
        "l/h",
        2,
        ObdDecoder::WordTwentieth,
    ),
    signal(
        "distanceWithMil",
        "Vzdálenost s aktivní MIL",
        0x21,
        "km",
        0,
        ObdDecoder::Word,
    ),
];

const fn signal(
    key: &'static str,
    label: &'static str,
    pid: u8,
    unit: &'static str,
    precision: u8,
    decoder: ObdDecoder,
) -> ObdSignalDefinition {
    ObdSignalDefinition {
        key,
        label,
        pid,
        unit,
        precision,
        decoder,
    }
}

pub(crate) fn obd_signal_definitions(group: LiveDataGroup) -> &'static [ObdSignalDefinition] {
    match group {
        LiveDataGroup::Engine => ENGINE,
        LiveDataGroup::Injectors => INJECTION,
        LiveDataGroup::Dpf => DPF,
        LiveDataGroup::Turbo => TURBO,
        LiveDataGroup::Electrical => ELECTRICAL,
        LiveDataGroup::Emissions => EMISSIONS,
        LiveDataGroup::Fuel => FUEL,
    }
}

pub(crate) fn engine_rpm_from_reply(reply: StnReply) -> Option<f64> {
    ENGINE
        .iter()
        .find(|signal| signal.pid == 0x0C)
        .copied()
        .and_then(|definition| definition.decode(reply).value)
}

impl ObdSignalDefinition {
    pub fn decode(self, reply: StnReply) -> LiveDataValue {
        let expected = format!("41{:02X}", self.pid);
        let payload = reply
            .lines
            .iter()
            .filter_map(|line| compact_hex(line))
            .find_map(|line| {
                line.find(&expected)
                    .map(|position| line[position..].to_owned())
            });
        let Some(payload) = payload else {
            return self.unavailable(
                LiveDataStatus::Unsupported,
                "Vozidlo nevrátilo očekávanou odpověď pro tento standardní PID.",
                Some(reply.raw),
            );
        };
        let raw = payload.strip_prefix(&expected).unwrap_or_default();
        let Some(bytes) = decode_hex(raw) else {
            return self.unavailable(
                LiveDataStatus::Error,
                "OBD-II odpověď obsahuje neplatná data.",
                Some(payload),
            );
        };

        if matches!(self.decoder, ObdDecoder::RawOnly) {
            return LiveDataValue {
                key: self.key.to_owned(),
                label: self.label.to_owned(),
                did: format!("01{:02X}", self.pid),
                value: None,
                unit: self.unit.to_owned(),
                precision: self.precision,
                status: LiveDataStatus::Available,
                assessment: unknown_assessment(
                    "Syrová standardní data",
                    "PID je podporovaný, ale veřejně ověřený dekodér není k dispozici. Raw data zůstávají viditelná pro audit.",
                ),
                raw_response: Some(payload),
                detail: Some(format_raw(&bytes)),
            };
        }

        let value = match self.decoder {
            ObdDecoder::Byte => bytes.first().map(|a| f64::from(*a)),
            ObdDecoder::ByteMinus40 => bytes.first().map(|a| f64::from(*a) - 40.0),
            ObdDecoder::BytePercent => bytes.first().map(|a| f64::from(*a) * 100.0 / 255.0),
            ObdDecoder::FuelTrim => bytes
                .first()
                .map(|a| (f64::from(*a) - 128.0) * 100.0 / 128.0),
            ObdDecoder::Word => word(&bytes).map(f64::from),
            ObdDecoder::WordQuarter => word(&bytes).map(|v| f64::from(v) / 4.0),
            ObdDecoder::WordHundredth => word(&bytes).map(|v| f64::from(v) / 100.0),
            ObdDecoder::WordMilli => word(&bytes).map(|v| f64::from(v) / 1000.0),
            ObdDecoder::WordTimesTen => word(&bytes).map(|v| f64::from(v) * 10.0),
            ObdDecoder::WordTwentieth => word(&bytes).map(|v| f64::from(v) / 20.0),
            ObdDecoder::SensorArrayTemperature => first_sensor_temperature(&bytes),
            ObdDecoder::RawOnly => None,
        };

        match value {
            Some(value) => LiveDataValue {
                key: self.key.to_owned(),
                label: self.label.to_owned(),
                did: format!("01{:02X}", self.pid),
                value: Some(value),
                unit: self.unit.to_owned(),
                precision: self.precision,
                status: LiveDataStatus::Available,
                assessment: assess_signal(self.key, value),
                raw_response: Some(payload),
                detail: None,
            },
            None => self.unavailable(
                LiveDataStatus::Error,
                "Odpověď je kratší, než vyžaduje standardní dekodér tohoto PID.",
                Some(payload),
            ),
        }
    }

    pub fn unavailable(
        self,
        status: LiveDataStatus,
        detail: impl Into<String>,
        raw_response: Option<String>,
    ) -> LiveDataValue {
        let detail = detail.into();
        LiveDataValue {
            key: self.key.to_owned(),
            label: self.label.to_owned(),
            did: format!("01{:02X}", self.pid),
            value: None,
            unit: self.unit.to_owned(),
            precision: self.precision,
            status,
            assessment: unknown_assessment(
                if matches!(status, LiveDataStatus::Unsupported) {
                    "PID není podporovaný"
                } else {
                    "Nelze vyhodnotit"
                },
                "Bez použitelné odpovědi nelze určit stav hodnoty.",
            ),
            raw_response,
            detail: Some(detail),
        }
    }
}

fn compact_hex(line: &str) -> Option<String> {
    if !line
        .chars()
        .all(|c| c.is_ascii_hexdigit() || c.is_ascii_whitespace())
    {
        return None;
    }
    let value = line
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_ascii_uppercase();
    (!value.is_empty() && value.len().is_multiple_of(2)).then_some(value)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    value.len().is_multiple_of(2).then(|| {
        (0..value.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
            .collect::<Option<Vec<_>>>()
    })?
}

fn word(bytes: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]))
}

fn first_sensor_temperature(bytes: &[u8]) -> Option<f64> {
    let mask = *bytes.first()?;
    (0..4)
        .find(|index| mask & (1 << index) != 0)
        .and_then(|index| {
            let offset = 1 + index * 2;
            word(bytes.get(offset..offset + 2)?).map(|raw| f64::from(raw) / 10.0 - 40.0)
        })
}

fn format_raw(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn unknown_assessment(label: &str, detail: &str) -> LiveDataAssessment {
    LiveDataAssessment {
        level: LiveDataAssessmentLevel::Unknown,
        label: label.to_owned(),
        detail: detail.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(raw: &str) -> StnReply {
        StnReply::parse(format!("{raw}\r>"), "010C").unwrap()
    }

    #[test]
    fn decodes_engine_speed() {
        let definition = ENGINE
            .iter()
            .find(|signal| signal.pid == 0x0C)
            .copied()
            .unwrap();
        let value = definition.decode(reply("410C1AF8"));
        assert_eq!(value.value, Some(1726.0));
    }

    #[test]
    fn decodes_fuel_trim() {
        let definition = INJECTION
            .iter()
            .find(|signal| signal.pid == 0x06)
            .copied()
            .unwrap();
        let value = definition.decode(StnReply::parse("410690\r>".into(), "0106").unwrap());
        assert_eq!(value.value, Some(12.5));
    }

    #[test]
    fn keeps_unknown_dpf_pressure_as_raw() {
        let definition = DPF
            .iter()
            .find(|signal| signal.pid == 0x7A)
            .copied()
            .unwrap();
        let value = definition.decode(StnReply::parse("417A010203\r>".into(), "017A").unwrap());
        assert!(value.value.is_none());
        assert!(matches!(value.status, LiveDataStatus::Available));
        assert_eq!(value.detail.as_deref(), Some("01 02 03"));
    }
}
