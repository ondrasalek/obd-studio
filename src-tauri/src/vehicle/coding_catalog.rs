//! Candidate features, not a compatibility database or executable write plans.
//! No payloads, offsets, access keys or inferred cross-brand mappings belong here.
use super::capabilities::{CapabilityKind, CapabilityRisk, CapabilityState, VehicleCapability};
use CapabilityRisk::{High, Medium};

type CodingEntry = (
    &'static str,
    &'static str,
    &'static str,
    CapabilityRisk,
    &'static str,
);
type CodingGroup = (&'static str, &'static [CodingEntry]);

const GROUPS: &[CodingGroup] = &[
    ("Přístroje a infotainment", &[
        ("needle-sweep", "Needle Sweep / Staging", "Protočení ručiček při zapnutí zapalování, pokud ho přístrojový štít podporuje.", Medium, "Kompatibilní přístrojový štít a jeho firmware"),
        ("digital-speedometer", "Digitální rychloměr", "Číselné zobrazení rychlosti na displeji přístrojového štítu.", Medium, "Přístrojový štít s podporovaným displejem a položkou menu"),
        ("sport-displays", "Sportovní zobrazení", "Zobrazení výkonu a momentu; aktivace grafiky nezaručuje dostupnost skutečných dat.", Medium, "Podporovaný infotainment a datové signály motorové ECU"),
        ("startup-animation", "Startovací logo a animace", "Výběr existující uvítací grafiky podporované firmwarem; bez nahrávání cizího firmware.", Medium, "Konkrétní verze infotainmentu nebo přístrojového štítu"),
        ("infotainment-warnings", "Úvodní hlášky infotainmentu", "Kandidát pro nastavení úvodních upozornění; bezpečnostní hlášky nelze obecně vypínat.", High, "Posouzení konkrétního upozornění a místních pravidel"),
        ("cluster-layout", "Grafika přístrojového štítu", "Pouze motivy explicitně kompatibilní s hardwarem, firmwarem a výbavou štítu.", High, "Ověřená kompatibilita grafiky a indikátorů vozidla"),
        ("hidden-menus", "Rozšířené menu přístrojů", "Teplota oleje, stopky a další položky, pro které vůz skutečně poskytuje data.", Medium, "Přístrojový štít a potřebné zdrojové senzory"),
        ("video-in-motion", "Video in Motion", "Riziko rozptýlení řidiče; tato položka je pouze v katalogu a nemá dostupný zápis.", High, "Samostatné bezpečnostní a právní posouzení; ne pro sledování řidičem za jízdy"),
    ]),
    ("Světla a viditelnost", &[
        ("scandinavian-lights", "Skandinávské denní svícení", "Zadní světla společně s předním denním svícením.", Medium, "Centrální elektrika a odpovídající konfigurace zadních světel"),
        ("drl-signature", "Světelný podpis / Angel Eyes", "Nastavení existujících světelných prvků; název funkce nezaručuje podporu jiné značky.", Medium, "Příslušné světlomety, světelné moduly a schválený režim svícení"),
        ("corner-lights", "Přisvěcování do zatáček", "Nastavení podporované corner funkce a jejích spouštěcích podmínek.", Medium, "Kompatibilní mlhovky nebo světlomety, řídicí modul a vstup úhlu řízení"),
        ("drl-brightness", "Jas světel podle režimu", "Intenzita DRL a dalších podporovaných výstupů; bez univerzálních procent či rozsahů.", High, "Přesný typ světelného výstupu, povolené meze a homologace"),
        ("coming-home", "Coming / Leaving Home", "Doba svícení, spouštěcí podmínky a podporované světelné výstupy.", Medium, "Centrální elektrika, konfigurace světel a případný světelný senzor"),
        ("ambient-colors", "Ambientní osvětlení", "Paleta barev a světelné zóny podle skutečně instalovaného hardware.", Medium, "RGB moduly, podporované zóny a kompatibilní infotainment"),
    ]),
    ("Komfort, okna a zamykání", &[
        ("comfort-windows", "Komfortní ovládání oken", "Dálkové nebo dešťové zavírání, otevírání a podporované zpoždění.", High, "Dveřní jednotky, ochrana proti přiskřípnutí a pro dešťový režim odpovídající senzor"),
        ("comfort-sunroof", "Komfortní ovládání střechy", "Zapojení střešního okna do podporovaného komfortního ovládání.", High, "Střešní jednotka, ochrana proti přiskřípnutí a kompatibilní dálkové ovládání"),
        ("mirror-fold", "Sklápění zrcátek", "Načasování sklopení při zamknutí na voze s elektricky sklopnými zrcátky.", Medium, "Elektrické sklápění a podporované dveřní jednotky"),
        ("mirror-reverse-tilt", "Naklopení zrcátka při couvání", "Aktivace a případně úhel sklopení; přesná paměť polohy závisí na výbavě.", Medium, "Zrcátko s potřebnou polohovou zpětnou vazbou a kompatibilní paměťovou funkcí"),
        ("lock-confirmation", "Potvrzení zamknutí", "Zvukové potvrzení zamčení a odemčení podporovaným výstupem.", Medium, "Alarmová siréna nebo podporovaný klaksonový výstup; místní hluková pravidla"),
        ("seat-heating-memory", "Paměť vyhřívání", "Obnovení podporovaného nastavení sedadel či volantu po novém startu.", Medium, "Příslušné topné prvky a klimatizační, sedadlová nebo volantová jednotka"),
    ]),
    ("Jízdní režimy a upozornění", &[
        ("start-stop", "Start-Stop a paměť tlačítka", "Pouze změny doložené pro konkrétní variantu. Univerzální změna napěťového prahu se nepoužívá.", Medium, "Ověřená varianta motorové ECU / energetického managementu a posouzení provozních pravidel"),
        ("comfort-indicators", "Komfortní blikání", "Počet bliknutí při krátkém dotyku páčky; povolené hodnoty určuje profil ECU.", Medium, "Centrální elektrika s podporovaným adaptačním kanálem"),
        ("drive-mode-memory", "Výchozí jízdní režim", "Paměť nebo výchozí režim po startu; nepřidává neinstalované jízdní funkce.", Medium, "Instalovaná volba režimů a kompatibilita všech dotčených jednotek"),
        ("seatbelt-warning", "Výstraha bezpečnostních pásů", "Bezpečnostní funkce. Změna akustického varování není v aplikaci zpřístupněna.", High, "Samostatné bezpečnostní a právní posouzení; zapnutý pás zůstává nutný"),
    ]),
    ("Specifické varianty a elektrifikace", &[
        ("rex-hold-charge", "REx · udržování nabití", "Kandidát pouze pro příslušné varianty s prodlužovačem dojezdu, například BMW i3 REx; ne obecná funkce elektromobilů.", High, "Přesná varianta REx, trh, software a povolená strategie energetického managementu"),
        ("remote-preconditioning", "Předklimatizace z klíče", "Přiřazení podporované předklimatizace ovladači, pokud to konkrétní vozidlo umožňuje.", High, "Podporovaná HVAC a přístupová jednotka; nesmí zrušit nouzovou funkci tlačítka"),
        ("chime-profile", "Profil akustických upozornění", "Pouze kompatibilní zvuky již přítomné ve firmware; bez kopírování dat jiné značky.", Medium, "Příslušný zvukový modul a zachování rozlišitelnosti bezpečnostních upozornění"),
    ]),
];

pub(super) fn coding_capabilities() -> Vec<VehicleCapability> {
    GROUPS
        .iter()
        .flat_map(|(category, entries)| {
            entries
                .iter()
                .map(
                    move |&(id, title, description, risk, equipment)| VehicleCapability {
                        id,
                        title,
                        description,
                        kind: CapabilityKind::Coding,
                        state: CapabilityState::ProfileRequired,
                        risk,
                        target_section: Some("coding"),
                        category: Some(category),
                        requirements: vec![
                            equipment,
                            "Shoda značky, modelu, roku, výbavy, čísla dílu a software ECU",
                            "Ověřený zápisový profil, záloha a kontrolní čtení",
                            "Stabilní napájení a podmínky podle servisního postupu",
                        ],
                    },
                )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn candidates_have_unique_ids_equipment_requirements_and_no_enabled_writes() {
        let items = coding_capabilities();
        assert_eq!(items.len(), 27);
        assert_eq!(
            items
                .iter()
                .map(|item| item.category)
                .collect::<HashSet<_>>()
                .len(),
            5
        );
        let mut ids = HashSet::new();
        for item in items {
            assert!(ids.insert(item.id));
            assert!(matches!(item.state, CapabilityState::ProfileRequired));
            assert!(matches!(item.kind, CapabilityKind::Coding));
            assert_eq!(item.requirements.len(), 4);
            assert!(!item.requirements[0].is_empty());
        }
    }
}
