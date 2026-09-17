# Architektura OBD Studio

## Zásady

1. Sériový port má právě jednoho vlastníka.
2. Frontend nikdy neposílá AT/ST příkazy přímo.
3. Profil se volí před otevřením portu a určuje inicializaci adaptéru i povolené příkazy.
4. Každý OBD-II nebo UDS request prochází Rust allowlist policy.
5. IPC vrací strukturované výsledky; surový provoz teče samostatnými eventy do konzole.
6. Připojení, transport, protokol a definice vozidla jsou oddělené vrstvy.

## Tok požadavku

```text
React action + selected profile
    │ invoke("send_standard_obd_query" | "read_live_data" | "scan_control_units" | "probe_dids")
    ▼
Tauri command
    │ typed query
    ▼
DiagnosticHandle ── mpsc ──► SerialWorker thread
                                │
                                ├─ enforce active profile
                                ├─ validate read-only allowlist
                                ├─ OBD-II payload or targeted STPX request
                                └─ wait for complete `>` prompt
                                         │
                                         ▼
                                  parse typed response
    ◄────────────── tokio oneshot ───────┘
```

Worker používá blokující API crate `serialport` na vlastním vlákně. Tauri async command proto neblokuje UI ani async runtime, ale současně nevznikají souběžné zápisy do stejného UARTu. MPSC fronta serializuje connect, disconnect, OBD-II a UDS operace; oneshot kanál doručí odpověď konkrétnímu IPC volání.

## Vrstvy backendu

### `commands`

Úzká Tauri hranice. Převádí frontendový požadavek na typovaný backendový request a používá sdílený `DiagnosticHandle`. Bez transportní logiky.

### `transport::serial`

Vyhledává porty, sjednocuje macOS dial-in/callout názvy a vlastní otevřený port. Worker zajišťuje sekvenční zpracování a životní cyklus spojení.

### `adapter::stn`

Implementuje request/response terminál OBDLinku. Každý příkaz končí `CR`, čtení vždy čeká na prompt `>`, omezuje velikost odpovědi a převádí známé STN chyby na `AppError`. Inicializace je profilová: `ATSP0` pro automatický standardní OBD-II režim, `STP 33` pro pevný MQB HS-CAN profil.

### `protocol::obd2`

Mapuje pojmenované čtecí akce na pevnou allowlist standardních payloadů. Parser podporuje více odpovídajících ECU, segmentovaný VIN, dekódování Mode 03 DTC a zachovává surovou odpověď pro audit. Frontend neposílá libovolný hex payload.

### `protocol::uds`

Normalizuje hex, kontroluje 11bit CAN ID, timeout a povolenou service. Parser rozlišuje pozitivní odpověď od `0x7F <service> <NRC>` a vrací jak strukturovaný výsledek, tak surovou odpověď pro audit.

### `vehicle`

Obsahuje identitu diagnostického profilu, předvolby ECU, katalog očekávaných MQB jednotek a deklarativní definice živých signálů včetně DID, jednotky, dekodéru a orientačního hodnocení. Dávkové čtení jedné skupiny vybere fyzický cíl pouze jednou; nepodporovaný DID označí samostatně a nepřeruší kvůli němu ostatní hodnoty. Hodnocení vzniká v profilové vrstvě backendu, aby bylo testovatelné a později verzovatelné podle varianty ECU; UI pouze vykresluje vrácený stav a vysvětlení.

`scan_control_units` přijímá typovanou oblast, nikoli uživatelský CAN rozsah. Worker sekvenčně zkouší pouze deklarované fyzické trasy, čte číslo dílu (`F187`), verzi software (`F189`) a počet potvrzených DTC podle status masky `0x08` (`19 01 08`). Jednotlivé timeouty jsou izolované, takže nepřítomná volitelná ECU nepřeruší celou oblast. Výsledek rozlišuje odpověď ECU, běžnou nepřítomnost/neodpověď a transportní chybu.

`probe_dids` je vývojová read-only cesta pro mapování výrobce-specifických dat. Přijímá oblast, klíč jednotky a maximálně 16 čísel DID. Backend ověří, že jednotka patří do katalogu dané oblasti, odstraní duplicity a každý požadavek sestaví interně jako `0x22 + DID`. Volající nemůže dodat CAN adresu ani payload. Odpověď rozlišuje čitelný DID, běžně nepodporovaný DID (`NRC 0x11/0x12/0x31`), jiné odmítnutí ECU a transportní chybu. Čitelná raw data mají informační stav, nikoli hodnocení bezvadnosti.

Frontend může po prvním otevření diagnostické oblasti v aktuální relaci automaticky zavolat odpovídající read command. Klíč oblasti se drží v session-scoped sadě, aby React Strict Mode ani opakované renderování nevytvořily dvojitý request. Nejde o časovaný polling; uživatel může automatiku vypnout a každý pohled ručně obnovit.

## Bezpečnostní hranice univerzálního profilu

`send_standard_obd_query` přijímá enum, nikoli libovolný payload. Povolené jsou pouze:

- Mode `01`, PID `00` — dostupné current-data PID
- Mode `09`, PID `02` — VIN, pokud jej vozidlo podporuje
- Mode `03` — uložené emisní DTC

Mazání DTC přes Mode `04`, aktivní testy a výrobce-specifické příkazy nejsou součástí profilu.

## Bezpečnostní hranice MQB profilu

Generický IPC command `send_uds_payload` není neomezený raw socket. Všechny požadavky procházejí stejnou allowlist policy. Povolené jsou pouze:

- `0x19` ReadDTCInformation
- `0x22` ReadDataByIdentifier
- `0x3E` TesterPresent

Aktuální interní scany používají konkrétní podfunkci `0x19 01 08`; maska `0x08` počítá pouze potvrzené DTC. Nejde o mazání závad ani akční test.

Průzkum DID není obecný raw UDS command: cíl pochází z profilového katalogu, počet dotazů je omezený a služba je pevně `0x22`. Úplný scan prostoru `0x0000–0xFFFF` aplikace neposkytuje.

Budoucí write workflow nesmí pouze rozšířit tento seznam. Potřebuje samostatný typ commandu, deklarativní profil operace, kontrolu napětí a kompatibility ECU, explicitní potvrzení uživatele, zálohu původní hodnoty a recovery strategii.

## Rozšiřování

- Přidat testovatelnou abstrakci `SerialIo` a nahrávky reálných STN dialogů.
- Modelovat diagnostickou session a delší vícefázové procedury nad již podporovaným `0x78 ResponsePending`.
- Doplnit centrální discovery přes instalační seznam gateway a katalog ověřených variant podle part number/software version.
- Přidávat další značky jako explicitní, testované profily; automatická OBD-II detekce se nesmí zaměňovat za univerzální výrobce-specifickou diagnostiku.
- Uchovávat audit log relace bez citlivých identifikátorů vozidla ve výchozím stavu.
- Teprve poté přidávat write/coding use cases jako samostatné transakce.
