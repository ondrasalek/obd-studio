# OBD Studio

Open-source desktopová diagnostika přes OBDLink EX. Základní profil používá legislativní OBD-II/EOBD/JOBD a automatickou detekci protokolu, takže není svázaný s jednou značkou. Samostatný rozšířený profil je zaměřený na vozy Volkswagen Group MQB, zejména Škodu Superb III po faceliftu (2020). Cílovou platformou MVP je macOS na Apple Silicon.

> Projekt je ve fázi vývoje. Diagnostika a celý profil VW MQB zůstávají záměrně **read-only**; profil povoluje jen UDS služby `0x19`, `0x22` a `0x3E`. Jedinou změnovou operací je typované smazání standardních emisních OBD-II závad přes Mode `04` v univerzálním profilu. Backend je povolí pouze po přesné potvrzovací frázi, kontrole napětí a ověření, že motor neběží. `SecurityAccess (0x27)`, `WriteDataByIdentifier (0x2E)` a výrobce-specifické mazání závad jsou nadále blokované.

## Rozsah kompatibility

| Profil | Vozidla | Aktuální možnosti |
| --- | --- | --- |
| Univerzální OBD-II | Značky a modely kompatibilní s OBD-II, EOBD nebo JOBD | Automatická volba protokolu, standardní live PID, VIN, emisní DTC a bezpečnostně omezený Mode 04 |
| Volkswagen Group MQB | Ověřované varianty platformy MQB | Fyzická UDS komunikace, profilová živá data motorové ECU a read-only scan očekávaných ECU podle oblastí |

„Univerzální“ zde znamená standardizovanou emisní diagnostiku. Neznamená automatickou podporu komfortního kódování, airbagů, ABS ani všech výrobcem specifických jednotek u každé značky. Tyto funkce vyžadují samostatné, verzované profily vozidel.

## Co už funguje

- Tauri 2 backend v Rustu a React 19/Vite/TypeScript frontend
- moderní tmavý dashboard postavený na Tailwind CSS a shadcn/ui
- společná STN transportní abstrakce pro reálný USB port a deterministickou offline demo relaci
- offline demo pro vývoj UI bez adaptéru a vozidla; testovací odpovědi procházejí stejnými OBD-II/UDS parsery jako reálný transport
- verzované JSON manifesty profilů se schema kontrolou, deklarovanými službami, původem dat a explicitními ECU variant matchery
- výčet a volba sériových portů včetně preferování macOS `/dev/cu.*`
- asynchronní Tauri IPC nad dedikovaným serial workerem
- volba diagnostického profilu před připojením
- univerzální inicializace přes `ATSP0` s automatickou detekcí OBD-II protokolu
- standardní dotazy `01 00`, `09 02` a `03` včetně dekódování VIN a DTC
- typované standardní live PID pro motor, vstřikování, turbo, elektrickou síť, emise a palivo; nepodporovaný PID se zobrazí jako nepodporovaný a neodhaduje se
- rozšířená inicializace MQB na HS-CAN ISO 15765, 11-bit, 500 kbps
- AT/ST, OBD-II i UDS komunikace zobrazovaná v živé konzoli
- kontrola identity, firmware, napětí, zvoleného a detekovaného protokolu
- diagnostická navigace: motor, vstřikování, DPF, turbo, elektrická síť, emise, palivo, převodovka, jízdní režimy, podvozek, interiér, karoserie, okna a dveře, světla, exteriér, klimatizace a infotainment
- dávkové read-only čtení ECU motoru přes UDS `ReadDataByIdentifier (0x22)` až po přesné shodě čísla dílu, software a skupiny signálů; bez shody se čtou pouze identifikační DID
- řízené živé obnovování vybrané skupiny bez překrývání sériových požadavků
- přehled dostupnosti jednotlivých hodnot včetně bezpečného označení nepodporovaných DID
- orientační stav hodnot (`v orientačním rozsahu`, `informativní`, `pozornost`, `kritické`); nejde o potvrzení bezvadnosti komponenty
- původ a verze definic, identita ECU a explicitní rozlišení standardních, profilově spárovaných, neověřených a simulovaných vzorků
- po 30 sekundách se vzorek označí jako zastaralý a UI skryje jeho hodnocení; simulace ani neověřená ECU nedostanou zelené hodnocení
- read-only scan 41 očekávaných MQB ECU seskupených do deseti diagnostických pohledů
- volitelný automatický úplný scan všech diagnostických oblastí po připojení profilu MQB s viditelným průběhem
- identifikace nalezených jednotek přes DID `F187` a `F189`, zobrazení CAN trasy a seznam potvrzených DTC přes `19 02 08`
- samostatný stav každé ECU a strukturovaný seznam jejích závad; neodpovídající volitelná jednotka se nepovažuje automaticky za poruchu
- omezený průzkum nejvýše 16 výrobce-specifických DID na jednu nalezenou ECU; backend povolí pouze katalogový CAN cíl a službu `ReadDataByIdentifier (0x22)`
- raw výsledek průzkumu je výslovně označený jako neinterpretovaný, dokud přesný part number a SW nemají ověřený dekodér, jednotku a diagnostické meze
- testovací čtení ECU 09: `0x70E → 0x778`, UDS `22 F1 87`
- parsování pozitivních odpovědí, UDS Negative Response Code i následné odpovědi po `0x78 ResponsePending`
- centrum schopností oddělující dostupné čtení, plánované výkonové testy, kódování a servisní procedury
- katalog 27 kandidátních úprav v pěti oblastech s vyhledáváním a požadavky na ECU/výbavu; nejde o potvrzenou podporu značek ani implementované zápisy
- trvale viditelná horní navigace Diagnostika / Kódování / Servis; v demu není nutné hledat Kódování na konci seznamu ECU
- standardní Mode `04` pouze v univerzálním profilu: přesná potvrzovací fráze, napětí nejméně 12,0 V, ověřené nulové otáčky a pevný payload bez raw-hex vstupu

## Bezpečnost

Vestavěný MQB manifest zatím **neobsahuje doložené verze software**. Na skutečné ECU proto blokuje výrobce-specifické živé hodnoty i při shodě čísla dílu `04L906027P`. Jde o záměrnou ochranu před chybným dekódováním, nikoli závadu spojení. Standardní OBD-II, identifikace jednotek, DTC a explicitní raw průzkum zůstávají dostupné. Demo není důkazem kompatibility s vozidlem.

Automotive diagnostika není běžná práce se sériovým portem. Nesprávný rámec, přerušený zápis nebo nízké napětí může způsobit ztrátu konfigurace ECU nebo nepojízdné vozidlo.

- Vyvíjejte nejprve proti ECU simulátoru nebo bench setupu.
- Na vozidle pracujte zaparkovaní, s dobře nabitou baterií a podle servisních postupů výrobce.
- Aplikaci nepoužívejte za jízdy.
- Mode `04` maže emisní DTC, freeze-frame data a resetuje readiness monitory; použijte jej až po opravě příčiny závady. Nemaže ABS, airbag ani výrobce-specifické UDS závady.
- Před budoucími zápisy bude nutné doplnit zálohy, pre-flight kontroly napětí a stavu vozidla, diagnostickou session, přesně definovaný SecurityAccess a transakční recovery postup.
- CAN adresy a DID vždy ověřte proti konkrétní platformě, ECU a verzi software.
- Neprovádějte plošný brute-force scan rozsahu DID. Průzkum v UI je záměrně omezený a má sloužit ke kontrolovanému ověření konkrétních kandidátů.
- Katalog MQB adres je experimentální a vychází z komunitně zdokumentovaných diagnostických tras; před budoucím zápisem se musí každá trasa potvrdit identifikací konkrétní ECU.
- Barevné hodnocení živých dat je orientační pomůcka, nikoli náhrada servisních limitů výrobce; u hodnot závislých na zatížení aplikace výslovně vyžaduje provozní kontext.

## Spuštění na macOS

Požadavky:

- macOS a Xcode Command Line Tools
- Node.js 20+ a npm
- stabilní Rust toolchain
- OBDLink EX dostupný jako virtuální sériový port

```bash
npm install
npm run tauri dev
```

Samostatný webový náhled bez sériové komunikace:

```bash
npm run dev
```

Po připojení adaptéru se na macOS typicky objeví `/dev/cu.usbserial-*`. Aplikace sloučí dvojici `/dev/tty.*` a `/dev/cu.*` a preferuje callout zařízení `/dev/cu.*`.

Pro práci bez auta zvolte profil a stiskněte **Demo**. Aplikace relaci viditelně označí jako `OFFLINE DEMO`; lze v ní zkoušet živá data, VIN, DTC, MQB scan i bezpečnostní workflow bez fyzického zápisu na sběrnici.

## Kontroly a build

```bash
npm run check
npm test # Node.js >= 22.6 s podporou --experimental-strip-types
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run tauri build
```

## Architektura

```text
src/
├── app/                    # shell dashboardu
├── components/ui/         # shadcn/ui primitives
├── features/
│   ├── connection/        # výběr portu a stav adaptéru
│   ├── console/           # živý TX/RX log
│   └── diagnostics/       # navigace systémů, živá data a bezpečné akce
├── hooks/                  # stav diagnostické relace
├── lib/                    # typované Tauri IPC
└── types/                  # sdílené frontendové kontrakty

src-tauri/src/
├── commands/              # Tauri command boundary
├── transport/             # STN serial/demo transport a single-owner worker
├── adapter/stn/           # AT/ST klient a parser promptu
├── protocol/
│   ├── obd2/              # standardní typované dotazy a parsování
│   └── uds/               # validace UDS requestů a parser odpovědí
├── vehicle/               # profily vozidel, MQB adresy a předvolby
├── app_state.rs           # sdílený handle workeru
├── error.rs               # strukturované IPC chyby
└── events.rs              # log a connection-state events
```

Podrobnosti jsou v [architektonickém popisu](docs/ARCHITECTURE.md), [poznámkách k OBDLink EX](docs/OBDLINK_EX.md), [návrhu diagnostických dat a ODX](docs/DIAGNOSTIC_DATA.md) a [pravidlech pro profily vozidel](docs/VEHICLE_PROFILES.md).

## Referenční dokumentace

- [OBDLink Family Reference and Programming Manual, revize F](https://www.scantool.net/scantool/downloads/682/obdlink_frpm_f.pdf)
- [ASAM MCD-2 D / ODX](https://www.asam.net/standards/detail/mcd-2-d/)
- [VAG UDS CAN ID catalog](https://github.com/ConnorHowell/vag-uds-ids) — komunitní katalog adres extrahovaných z diagnostických dat ODIS
- [MQB-sniffer](https://github.com/mrfixpl/MQB-sniffer) a [VAG-CP-Docs](https://github.com/dspl1236/VAG-CP-Docs/blob/main/technical/cp-offline-resolution.md) — doplňkové křížové ověření vybraných tras
- ISO 14229 (UDS) a ISO 15765-2 jsou normy chráněné autorským právem; pro produkční implementaci použijte licencované znění.

## Licence a ochranné známky

Kód je dostupný pod licencí [MIT](LICENSE). Škoda, Volkswagen, MQB a OBDLink jsou ochranné známky příslušných vlastníků. Projekt s nimi není spojen ani jimi schválen.
