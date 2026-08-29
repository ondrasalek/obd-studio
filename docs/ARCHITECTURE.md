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
    │ invoke("send_standard_obd_query" | "read_central_electronics")
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

Obsahuje identitu diagnostického profilu a ověřitelné předvolby pro konkrétní řídicí jednotku. Do této vrstvy patří pozdější katalog podporovaných ECU, DID a kódovacích funkcí — ne do UI.

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

Budoucí write workflow nesmí pouze rozšířit tento seznam. Potřebuje samostatný typ commandu, deklarativní profil operace, kontrolu napětí a kompatibility ECU, explicitní potvrzení uživatele, zálohu původní hodnoty a recovery strategii.

## Rozšiřování

- Přidat testovatelnou abstrakci `SerialIo` a nahrávky reálných STN dialogů.
- Modelovat diagnostickou session a `0x78 ResponsePending` jako stavový request.
- Přidat ECU discovery přes gateway a katalog variant podle part number/software version.
- Přidávat další značky jako explicitní, testované profily; automatická OBD-II detekce se nesmí zaměňovat za univerzální výrobce-specifickou diagnostiku.
- Uchovávat audit log relace bez citlivých identifikátorů vozidla ve výchozím stavu.
- Teprve poté přidávat write/coding use cases jako samostatné transakce.
