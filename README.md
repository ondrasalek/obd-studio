# OBD Studio

Open-source desktopová diagnostika přes OBDLink EX. Základní profil používá legislativní OBD-II/EOBD/JOBD a automatickou detekci protokolu, takže není svázaný s jednou značkou. Samostatný rozšířený profil je zaměřený na vozy Volkswagen Group MQB, zejména Škodu Superb III po faceliftu (2020). Cílovou platformou MVP je macOS na Apple Silicon.

> Projekt je ve fázi vývoje a aktuální verze je záměrně **read-only**. Univerzální profil nabízí pouze standardní čtecí OBD-II režimy. Profil VW MQB povoluje jen UDS služby `0x19`, `0x22` a `0x3E`. `SecurityAccess (0x27)`, `WriteDataByIdentifier (0x2E)`, mazání závad a ostatní měnící operace backend odmítne ještě před odesláním do vozidla.

## Rozsah kompatibility

| Profil | Vozidla | Aktuální možnosti |
| --- | --- | --- |
| Univerzální OBD-II | Značky a modely kompatibilní s OBD-II, EOBD nebo JOBD | Automatická volba protokolu, podporované PID, VIN a uložené emisní DTC |
| Volkswagen Group MQB | Ověřované varianty platformy MQB | Fyzická UDS komunikace, živá data motorové ECU a read-only scan očekávaných ECU podle oblastí |

„Univerzální“ zde znamená standardizovanou emisní diagnostiku. Neznamená automatickou podporu komfortního kódování, airbagů, ABS ani všech výrobcem specifických jednotek u každé značky. Tyto funkce vyžadují samostatné, verzované profily vozidel.

## Co už funguje

- Tauri 2 backend v Rustu a React 19/Vite/TypeScript frontend
- moderní tmavý dashboard postavený na Tailwind CSS a shadcn/ui
- výčet a volba sériových portů včetně preferování macOS `/dev/cu.*`
- asynchronní Tauri IPC nad dedikovaným serial workerem
- volba diagnostického profilu před připojením
- univerzální inicializace přes `ATSP0` s automatickou detekcí OBD-II protokolu
- standardní read-only dotazy `01 00`, `09 02` a `03` včetně dekódování VIN a DTC
- rozšířená inicializace MQB na HS-CAN ISO 15765, 11-bit, 500 kbps
- AT/ST, OBD-II i UDS komunikace zobrazovaná v živé konzoli
- kontrola identity, firmware, napětí, zvoleného a detekovaného protokolu
- diagnostická navigace: motor, vstřikování, DPF, turbo, převodovka, jízdní režimy, podvozek, interiér, karoserie, okna a dveře, světla, exteriér, klimatizace a infotainment
- dávkové read-only čtení živých hodnot ECU motoru přes UDS `ReadDataByIdentifier (0x22)`
- přehled dostupnosti jednotlivých hodnot včetně bezpečného označení nepodporovaných DID
- orientační stav každé hodnoty (`v pořádku`, `informativní`, `pozornost`, `kritické`) s vysvětlením a profilovými prahy
- read-only scan 41 očekávaných MQB ECU seskupených do deseti diagnostických pohledů
- volitelné automatické načtení při prvním otevření oblasti v relaci; nejde o nepřetržité dotazování sběrnice
- identifikace nalezených jednotek přes DID `F187` a `F189`, zobrazení CAN trasy a read-only počet potvrzených DTC přes `19 01 08`
- samostatný stav každé ECU a jejích závad; neodpovídající volitelná jednotka se nepovažuje automaticky za poruchu
- omezený průzkum nejvýše 16 výrobce-specifických DID na jednu nalezenou ECU; backend povolí pouze katalogový CAN cíl a službu `ReadDataByIdentifier (0x22)`
- raw výsledek průzkumu je výslovně označený jako neinterpretovaný, dokud přesný part number a SW nemají ověřený dekodér, jednotku a diagnostické meze
- testovací čtení ECU 09: `0x70E → 0x778`, UDS `22 F1 87`
- parsování pozitivních odpovědí, UDS Negative Response Code i následné odpovědi po `0x78 ResponsePending`

## Bezpečnost

Automotive diagnostika není běžná práce se sériovým portem. Nesprávný rámec, přerušený zápis nebo nízké napětí může způsobit ztrátu konfigurace ECU nebo nepojízdné vozidlo.

- Vyvíjejte nejprve proti ECU simulátoru nebo bench setupu.
- Na vozidle pracujte zaparkovaní, s dobře nabitou baterií a podle servisních postupů výrobce.
- Aplikaci nepoužívejte za jízdy.
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

## Kontroly a build

```bash
npm run check
npm run build
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets -- -D warnings
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
├── transport/serial/      # výčet portů a single-owner worker
├── adapter/stn/           # AT/ST klient a parser promptu
├── protocol/
│   ├── obd2/              # standardní read-only dotazy a parsování
│   └── uds/               # validace UDS requestů a parser odpovědí
├── vehicle/               # profily vozidel, MQB adresy a předvolby
├── app_state.rs           # sdílený handle workeru
├── error.rs               # strukturované IPC chyby
└── events.rs              # log a connection-state events
```

Podrobnosti jsou v [architektonickém popisu](docs/ARCHITECTURE.md), [poznámkách k OBDLink EX](docs/OBDLINK_EX.md) a [pravidlech pro profily vozidel](docs/VEHICLE_PROFILES.md).

## Referenční dokumentace

- [OBDLink Family Reference and Programming Manual](https://www.scantool.net/scantool/downloads/98/stn11xx21xx_frpm-c.pdf)
- [VAG UDS CAN ID catalog](https://github.com/ConnorHowell/vag-uds-ids) — komunitní katalog adres extrahovaných z diagnostických dat ODIS
- [MQB-sniffer](https://github.com/mrfixpl/MQB-sniffer) a [VAG-CP-Docs](https://github.com/dspl1236/VAG-CP-Docs/blob/main/technical/cp-offline-resolution.md) — doplňkové křížové ověření vybraných tras
- ISO 14229 (UDS) a ISO 15765-2 jsou normy chráněné autorským právem; pro produkční implementaci použijte licencované znění.

## Licence a ochranné známky

Kód je dostupný pod licencí [MIT](LICENSE). Škoda, Volkswagen, MQB a OBDLink jsou ochranné známky příslušných vlastníků. Projekt s nimi není spojen ani jimi schválen.
