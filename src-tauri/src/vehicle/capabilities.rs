use super::DiagnosticProfile;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityKind {
    Read,
    Performance,
    Coding,
    Service,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityState {
    Available,
    ProfileRequired,
    Planned,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityRisk {
    ReadOnly,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleCapability {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub kind: CapabilityKind,
    pub state: CapabilityState,
    pub risk: CapabilityRisk,
    pub target_section: Option<&'static str>,
    pub requirements: Vec<&'static str>,
}

pub fn capabilities(profile: DiagnosticProfile) -> Vec<VehicleCapability> {
    let mut values = vec![
        read("dtc", "Chybové kódy všech jednotek", "Výpis potvrzených DTC včetně raw kódu, stavu a typu závady.", "overview"),
        read("temperatures", "Teploty kapalin a vzduchu", "Chladicí kapalina, motorový olej, sání a podporované výfukové senzory.", "engine"),
        read("pressures", "Tlaky motoru a přeplňování", "Tlak v sání, barometrický tlak, tlak paliva a profilové hodnoty turba.", "turbo"),
        read("air-fuel", "Průtok vzduchu a směs", "MAF, korekce směsi, poloha klapky a podporované lambda/NOx údaje.", "emissions"),
        read("electrical", "Elektrická síť", "Napětí palubní sítě a orientační vyhodnocení baterie a dobíjení.", "electrical"),
        read("emissions", "Emise a DPF/GPF", "Standardní emisní PID a ověřené MQB hodnoty sazí, popela a regenerace.", "dpf"),
        read("fuel", "Palivová data", "Hladina paliva, tlak v liště a okamžitý průtok, pokud je ECU poskytuje.", "fuel"),
        catalog_entry(
            "transmission-temperature",
            "Teplota oleje převodovky",
            "Výrobce-specifická hodnota vyžaduje ověřený profil konkrétní převodovky.",
            CapabilityKind::Read,
            CapabilityState::ProfileRequired,
            CapabilityRisk::ReadOnly,
            vec!["Identita a software ECU převodovky", "Ověřený DID a dekodér"],
        ),
        catalog_entry(
            "brake-pressure",
            "Tlak brzdové soustavy",
            "Tlak a stavy ABS/ESC nejsou součástí legislativního OBD-II a musí je definovat profil ABS jednotky.",
            CapabilityKind::Read,
            CapabilityState::ProfileRequired,
            CapabilityRisk::ReadOnly,
            vec!["Identita ABS/ESC", "Ověřený DID a jednotka"],
        ),
        catalog_entry(
            "fuel-consumption",
            "Okamžitá a dlouhodobá spotřeba",
            "Přesný výpočet potřebuje stabilní záznam průtoku, rychlosti a variantní kalibraci.",
            CapabilityKind::Read,
            CapabilityState::Planned,
            CapabilityRisk::ReadOnly,
            vec!["Podporovaný průtok paliva", "Časová řada", "Kalibrace paliva"],
        ),
        catalog_entry(
            "tank-volume",
            "Objem paliva v litrech",
            "Standardní OBD-II obvykle vrací procenta; litry vyžadují ověřený tvar nádrže a kalibraci modelu.",
            CapabilityKind::Read,
            CapabilityState::ProfileRequired,
            CapabilityRisk::ReadOnly,
            vec!["Varianta nádrže", "Kalibrační křivka", "Ověřený profil vozidla"],
        ),
        catalog_entry(
            "co2-estimate",
            "Výpočet emisí CO₂",
            "CO₂ se nebude vydávat za přímo měřenou hodnotu; výpočet musí uvést palivo, metodiku a nejistotu.",
            CapabilityKind::Read,
            CapabilityState::Planned,
            CapabilityRisk::ReadOnly,
            vec!["Typ paliva", "Ověřený průtok", "Dokumentovaná metodika"],
        ),
        catalog_entry(
            "performance-tests",
            "Výkonové testy a grafy",
            "Záznam rychlosti, otáček a zatížení pro 0–100 km/h, 1/4 míle a grafy. Vyžaduje samostatný bezpečný záznamový režim.",
            CapabilityKind::Performance,
            CapabilityState::Planned,
            CapabilityRisk::Medium,
            vec!["Uzavřený prostor nebo závodní trať", "Obsluha aplikace spolujezdcem", "Stabilní vzorkovací frekvence"],
        ),
    ];

    let profile_state = CapabilityState::ProfileRequired;

    for (id, title, description, risk) in [
        ("start-stop", "Start-Stop", "Výchozí stav, paměť nebo obrácení logiky tlačítka.", CapabilityRisk::Medium),
        ("scandinavian-lights", "Skandinávské denní svícení", "Zadní světla společně s denním svícením.", CapabilityRisk::Medium),
        ("corner-lights", "Přisvěcování do zatáček", "Aktivace a parametry corner funkce.", CapabilityRisk::Medium),
        ("coming-home", "Coming / Leaving Home", "Délka, spouštěcí podmínky a světelné výstupy.", CapabilityRisk::Medium),
        ("comfort-windows", "Komfortní ovládání oken", "Dálkové nebo dešťové zavírání oken.", CapabilityRisk::High),
        ("mirror-fold", "Sklápění zrcátek", "Sklopení ihned po zamknutí.", CapabilityRisk::Medium),
        ("needle-sweep", "Needle Sweep", "Protočení ručiček po zapnutí zapalování.", CapabilityRisk::Medium),
        ("cluster-layout", "Grafika přístrojového štítu", "Pouze varianty explicitně kompatibilní s konkrétním hardwarem a softwarem štítu.", CapabilityRisk::High),
        ("hidden-menus", "Rozšířené menu přístrojů", "Teplota oleje, stopky a další položky podporované danou jednotkou.", CapabilityRisk::Medium),
        ("ambient-colors", "Ambientní osvětlení", "Paleta barev a dostupné světelné zóny.", CapabilityRisk::Medium),
        ("lock-confirmation", "Potvrzení zamknutí", "Akustické potvrzení podporovaným výstupem vozidla.", CapabilityRisk::Medium),
        ("seat-heating-memory", "Paměť vyhřívání", "Obnovení posledního stavu sedadel nebo volantu.", CapabilityRisk::Medium),
        ("video-in-motion", "Video in Motion", "Funkce může být právně omezená a zvyšuje riziko rozptýlení řidiče.", CapabilityRisk::High),
        ("seatbelt-warning", "Výstraha bezpečnostních pásů", "Bezpečnostní funkce; aplikace ji bez jurisdikčního a profilového povolení nezpřístupní.", CapabilityRisk::High),
    ] {
        values.push(catalog_entry(
            id,
            title,
            description,
            CapabilityKind::Coding,
            profile_state,
            risk,
            vec!["Přesná identita ECU", "Záloha původní hodnoty", "Napětí alespoň 12,5 V", "Ověřený coding profil"],
        ));
    }

    for (id, title, description, risk) in [
        (
            "clear-dtc",
            "Smazání závad",
            "Smazání až po opravě příčiny a následný kontrolní scan.",
            CapabilityRisk::Low,
        ),
        (
            "service-reset",
            "Reset servisních intervalů",
            "Olejový a inspekční interval podle konkrétního profilu přístrojové desky.",
            CapabilityRisk::Medium,
        ),
        (
            "parking-brake-service",
            "Servis elektronické parkovací brzdy",
            "Řízené otevření, zavření a základní nastavení po výměně destiček.",
            CapabilityRisk::High,
        ),
        (
            "battery-registration",
            "Registrace baterie",
            "Výrobce, kapacita, technologie AGM/EFB a sériové číslo.",
            CapabilityRisk::High,
        ),
        (
            "throttle-adaptation",
            "Adaptace klapek",
            "Základní nastavení pouze při splněných podmínkách ECU.",
            CapabilityRisk::High,
        ),
    ] {
        let state = if id == "clear-dtc" && profile == DiagnosticProfile::GenericObd2 {
            CapabilityState::Available
        } else {
            CapabilityState::ProfileRequired
        };
        values.push(catalog_entry(
            id,
            title,
            description,
            CapabilityKind::Service,
            state,
            risk,
            vec![
                "Vozidlo stojí",
                "Předepsaná diagnostická session",
                "Kontrola napětí a podmínek",
                "Možnost bezpečného návratu",
            ],
        ));
    }

    values
}

fn read(
    id: &'static str,
    title: &'static str,
    description: &'static str,
    target_section: &'static str,
) -> VehicleCapability {
    VehicleCapability {
        id,
        title,
        description,
        kind: CapabilityKind::Read,
        state: CapabilityState::Available,
        risk: CapabilityRisk::ReadOnly,
        target_section: Some(target_section),
        requirements: Vec::new(),
    }
}

fn catalog_entry(
    id: &'static str,
    title: &'static str,
    description: &'static str,
    kind: CapabilityKind,
    state: CapabilityState,
    risk: CapabilityRisk,
    requirements: Vec<&'static str>,
) -> VehicleCapability {
    let target_section = match kind {
        CapabilityKind::Performance => "performance",
        CapabilityKind::Coding => "coding",
        CapabilityKind::Service => "service",
        CapabilityKind::Read => "diagnostics",
    };
    VehicleCapability {
        id,
        title,
        description,
        kind,
        state,
        risk,
        target_section: Some(target_section),
        requirements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_capabilities_are_never_marked_available() {
        for item in capabilities(DiagnosticProfile::VwMqb) {
            if matches!(item.kind, CapabilityKind::Coding | CapabilityKind::Service) {
                assert!(!matches!(item.state, CapabilityState::Available));
            }
        }
    }
}
