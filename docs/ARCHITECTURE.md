# Architektura OBD Studio

## Zásady

1. Aktivní STN transport má právě jednoho vlastníka.
2. Frontend nikdy neposílá AT/ST příkazy přímo.
3. Profil se volí před otevřením portu a určuje inicializaci adaptéru i povolené příkazy.
4. Každý OBD-II nebo UDS request prochází Rust allowlist policy.
5. IPC vrací strukturované výsledky; surový provoz teče samostatnými eventy do konzole.
6. Připojení, transport, protokol a definice vozidla jsou oddělené vrstvy.
7. Offline demo používá stejné parsery jako sériový transport a musí být v UI nezaměnitelně označené.

## Tok požadavku

```text
React action + selected profile
    │ invoke("send_standard_obd_query" | "read_live_data" | "scan_control_units" | "probe_dids" | "clear_emission_dtcs")
    ▼
Tauri command
    │ typed query
    ▼
DiagnosticHandle ── mpsc ──► SerialWorker thread
                                │
                                ├─ serial or deterministic demo transport
                                ├─ enforce active profile
                                ├─ validate profile and typed operation policy
                                ├─ OBD-II payload or targeted STPX request
                                └─ wait for complete `>` prompt
                                         │
                                         ▼
                                  parse typed response
    ◄────────────── tokio oneshot ───────┘
```

Worker používá blokující transport na vlastním vlákně. Produkční `SerialStnTransport` obaluje crate `serialport`; `DemoStnTransport` generuje deterministické surové STN odpovědi pro vývoj bez auta. Tauri async command proto neblokuje UI ani async runtime, ale současně nevznikají souběžné transakce. MPSC fronta serializuje connect, disconnect, OBD-II a UDS operace; oneshot kanál doručí odpověď konkrétnímu IPC volání.

## Vrstvy backendu

### `commands`

Úzká Tauri hranice. Převádí frontendový požadavek na typovaný backendový request a používá sdílený `DiagnosticHandle`. Bez transportní logiky.

### `transport::serial`

Vyhledává porty, sjednocuje macOS dial-in/callout názvy a řídí životní cyklus relace. Worker vlastní jedinou instanci transportu a zajišťuje sekvenční zpracování.

### `transport::stn`

Definuje half-duplex command transport zakončený promptem `>`. Sériová implementace řeší CR terminátor, krátké read timeouty, maximální velikost odpovědi, macOS `/dev/cu.*` a zachování bytů za promptem. Demo implementace vrací raw AT/ST, OBD-II a UDS odpovědi, které dále zpracovává produkční `StnClient`; frontend nedostává předem dekódované zkratky.

### `adapter::stn`

Implementuje AT/ST request/response klienta OBDLinku nad zvoleným transportem a převádí známé STN chyby na `AppError`. Inicializace je profilová: `ATSP0` pro automatický standardní OBD-II režim, `STP 33` pro pevný MQB HS-CAN profil.

ISO-TP segmentaci, sestavení více rámců a flow control zajišťuje v této konfiguraci firmware OBDLinku (`STCSEGT`, `STCSEGR`, `ATCFC` a flow-control páry). Aplikace proto neimplementuje druhý konkurenční ISO-TP stavový automat nad textovým STN rozhraním. Vlastní ISO-TP stack by dával smysl až pro jiný typ CAN transportu, který předává surové rámce a neposkytuje tuto službu.

### `protocol::obd2`

Mapuje pojmenované akce na pevnou allowlist standardních payloadů. Parser podporuje více odpovídajících ECU, segmentovaný VIN, dekódování Mode 03 DTC a zachovává surovou odpověď pro audit. Standardní live data používají samostatné deklarativní definice PID a dekodéry. Frontend neposílá libovolný hex payload.

### `protocol::uds`

Normalizuje hex, kontroluje 11bit CAN ID, timeout a povolenou service. Parser rozlišuje pozitivní odpověď od `0x7F <service> <NRC>` a vrací jak strukturovaný výsledek, tak surovou odpověď pro audit.

### `vehicle`

Obsahuje identitu diagnostického profilu, verzovaný manifest, předvolby ECU, katalog očekávaných MQB jednotek a definice signálů. Manifest schema 2 kontroluje ID, read/write SID, původ dat a variant matchery. `verified_live_data` před každou výrobce-specifickou skupinou znovu načte identitu motorové ECU; `verification` vyžaduje přesné číslo dílu, verzi software a skupinu signálů. Bez shody se signálové DID vůbec neodesílají. Prázdný seznam verzí znamená nedoloženou kompatibilitu, ne wildcard. Dávkové čtení vybere fyzický cíl pouze jednou; nepodporovaný DID po úspěšné kontrole označí samostatně. Backend vrací původ dat a orientační hodnocení. UI potlačí hodnocení pro simulaci, neověřenou ECU a vzorek starší 30 sekund. Podrobnosti a aktuální omezení jsou v `DIAGNOSTIC_DATA.md`.

`scan_control_units` přijímá typovanou oblast, nikoli uživatelský CAN rozsah. Worker sekvenčně zkouší pouze deklarované fyzické trasy, čte číslo dílu (`F187`), verzi software (`F189`) a potvrzené DTC podle status masky `0x08` (`19 02 08`). Z odpovědi `59 02` vrací kód, status, failure type i raw záznam. Jednotlivé timeouty jsou izolované, takže nepřítomná volitelná ECU nepřeruší celou oblast. Výsledek rozlišuje odpověď ECU, běžnou nepřítomnost/neodpověď a transportní chybu.

`probe_dids` je vývojová read-only cesta pro mapování výrobce-specifických dat. Přijímá oblast, klíč jednotky a maximálně 16 čísel DID. Backend ověří, že jednotka patří do katalogu dané oblasti, odstraní duplicity a každý požadavek sestaví interně jako `0x22 + DID`. Volající nemůže dodat CAN adresu ani payload. Odpověď rozlišuje čitelný DID, běžně nepodporovaný DID (`NRC 0x11/0x12/0x31`), jiné odmítnutí ECU a transportní chybu. Čitelná raw data mají informační stav, nikoli hodnocení bezvadnosti.

Frontend může po připojení MQB profilu sekvenčně načíst všechny diagnostické oblasti. Worker všechny sériové operace nadále serializuje a UI ukazuje průběh. U živých hodnot je časovač naplánovaný až po dokončení předchozího čtení, takže se požadavky nepřekrývají. Uživatel může automatiku i živé obnovování vypnout a každý pohled ručně obnovit.

## Bezpečnostní hranice univerzálního profilu

`send_standard_obd_query` přijímá enum, nikoli libovolný payload. Povolené jsou pouze:

- Mode `01`, PID `00` — dostupné current-data PID
- Mode `09`, PID `02` — VIN, pokud jej vozidlo podporuje
- Mode `03` — uložené emisní DTC
- Mode `01` — deklarované current-data PID používané jednotlivými live skupinami

Jedinou změnovou operací je samostatný command `clear_emission_dtcs`. Neakceptuje payload a vždy sestaví přesně Mode `04`. Před odesláním vyžaduje přesnou potvrzovací frázi, univerzální profil, napětí alespoň 12,0 V a úspěšně dekódované otáčky pod 100 rpm. Tímto příkazem nelze mazat závady ABS, airbagů ani jiných výrobce-specifických ECU. Aktivní testy a výrobce-specifické příkazy nejsou součástí profilu.

## Bezpečnostní hranice MQB profilu

Generický IPC command `send_uds_payload` není neomezený raw socket. Všechny požadavky procházejí stejnou allowlist policy. Povolené jsou pouze:

- `0x19` ReadDTCInformation
- `0x22` ReadDataByIdentifier
- `0x3E` TesterPresent

Aktuální interní scany používají konkrétní podfunkci `0x19 02 08`; maska `0x08` vrací potvrzené DTC. Nejde o mazání závad ani akční test.

Průzkum DID není obecný raw UDS command: cíl pochází z profilového katalogu, počet dotazů je omezený a služba je pevně `0x22`. Úplný scan prostoru `0x0000–0xFFFF` aplikace neposkytuje.

Budoucí write workflow nesmí pouze rozšířit tento seznam. Potřebuje samostatný typ commandu, deklarativní profil operace, kontrolu napětí a kompatibility ECU, explicitní potvrzení uživatele, zálohu původní hodnoty a recovery strategii.

## Rozšiřování

- Ukládat redigované replay záznamy nad stávající transportní abstrakcí; VIN a další identifikátory nesmějí být součástí fixture bez výslovného souhlasu.
- Modelovat diagnostickou session a delší vícefázové procedury nad již podporovaným `0x78 ResponsePending`.
- Doplnit centrální discovery přes instalační seznam gateway a katalog ověřených variant podle part number/software version.
- Přidávat další značky jako explicitní, testované profily; automatická OBD-II detekce se nesmí zaměňovat za univerzální výrobce-specifickou diagnostiku.
- Uchovávat audit log relace bez citlivých identifikátorů vozidla ve výchozím stavu.
- Teprve poté přidávat write/coding use cases jako samostatné transakce.
