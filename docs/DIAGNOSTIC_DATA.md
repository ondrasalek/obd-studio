# Diagnostická data, profily a ODX/PDX

Komunikační standard sám neurčuje význam většiny výrobce-specifických dat. UDS definuje strukturu služeb, ale konkrétní DID, rutiny, kódování, měřítka a podmínky závisejí na přesné variantě ECU. OBD Studio proto odděluje transport, protokol a diagnostická data.

## Vrstvy zdrojů dat

1. Standardní OBD-II PID mají veřejně definovanou sémantiku a mohou být součástí aplikace.
2. Vestavěný profil obsahuje kandidátní definice s původem dat. Pro použití na skutečné ECU musí mít doloženou přesnou identitu, software a rozsah signálů.
3. Externí podepsané a verzované balíčky definic jsou plánované; jejich import zatím není implementován.
4. ODX/PDX je budoucí importní formát; originální OEM data se nesmí distribuovat bez odpovídající licence.

„Univerzální“ aplikace tedy znamená univerzální diagnostické jádro a rozšiřitelné datové balíčky. Neznamená, že lze z CAN adresy nebo názvu modelu bezpečně odhadnout libovolný DID či kódovací bit.

Aktuální vestavěné profily mají JSON manifest schema verze 2. Manifest deklaruje stabilní ID a verzi, transportní očekávání, read/write SID, stav ověření, původ dat a variant matchery. Backend manifest validuje před vytvořením diagnostické relace a aktivní verzi ukazuje v panelu připojení. Samotné signály se budou do datových balíčků přesouvat postupně; manifest zatím není vydáván za plný ODX D-Server.

## Runtime kontrola varianty

Před každou skupinou výrobce-specifických měření motoru backend čte `F187` a `F189` z fyzického cíle `7E0/7E8`. Dekódování povolí pouze přesná shoda `ecuAddress`, `partNumber`, jedné z `softwareVersions` a `signalGroups`. Shoda pro DPF neautorizuje vstřikovače ani turbo. Prázdný seznam verzí **není wildcard**. Neúplná, odmítnutá nebo neshodná identita znamená stav `unverified`, nulové číselné hodnoty a žádné další dotazy na signálové DID. Ztráta sériového spojení vrací chybu, nikoli výsledek kompatibility. Identita se necachuje přes skupiny ani reconnect.

Vestavěný kandidát `04L906027P` nemá doložené verze software; na reálném autě tedy zatím neopravňuje k výrobce-specifickému měření. Verze z testů (`TEST-0001`) jsou umělé a nejsou obsažené v produkčním manifestu. Standardní OBD-II PID variantní kontrolu nevyžadují. Omezený ruční DID průzkum zůstává raw a není touto kontrolou vydáván za dekódované měření.

Snapshot vrací stav `standard`, `matched`, `unverified` nebo `simulated`, identitu, zdroj a verzi definice. Simulaci určuje backendový transport, nikoli parametr z UI. Simulované hodnoty UI nehodnotí jako zdraví skutečného auta. Po 30 sekundách bez nového snapshotu UI ponechá historická čísla, ale skryje jejich hodnocení; tento interval je pravidlo prezentace, nikoli servisní limit ECU. Stávající mezní hodnoty jsou stále orientační, ne plný model provozního kontextu.

## Plán importu ODX/PDX

První implementace bude záměrně read-only a pokryje omezenou, testovatelnou podmnožinu ASAM MCD-2 D 2.2:

- manifest balíčku a hash všech vstupních souborů,
- ECU/Base/Protocol varianty a deterministické řešení referencí,
- request/positive response/negative response diagnostické služby,
- jednoduché datové objekty a CompuMethods typu identity, linear, rational a text table,
- jednotky, fyzikální rozsahy a vícejazyčné popisky,
- variantní identifikaci podle explicitních odpovědí ECU.

PDX je ZIP kontejner a musí se zpracovávat jako nedůvěryhodný vstup. Importer bude mít limity velikosti a počtu souborů, zakáže path traversal, externí XML entity a síťové reference a nebude provádět žádný kód obsažený v balíčku. Neúplná nebo nejednoznačná definice se zobrazí jako raw data; nesmí se tiše dopočítat odhadem.

## Zápisové a bezpečnostní profily

Definice čtení sama nikdy nepovoluje zápis. Každá měnící operace musí být samostatný profilový objekt s:

- přesnou identitou podporované ECU a software,
- deklarovanou UDS session a povolenými službami,
- pre-flight podmínkami, voltage interlockem a stavem motoru,
- čtením a zálohou původní hodnoty,
- explicitním rozdílem před/po a potvrzením uživatele,
- verifikací po zápisu a recovery postupem,
- auditním záznamem bez výchozího ukládání VIN.

SecurityAccess `0x27`, Authentication `0x29` a Secure Gateway nejsou univerzální „unlock algoritmy“. Budoucí rozhraní bude podporovat pouze legitimní poskytovatele oprávnění a uživatelem dodané licencované materiály. Privátní klíče patří do systémového keychainu nebo hardwarového úložiště; aplikace nebude obsahovat obcházení SGW ani sdílená tajemství získaná reverzním inženýrstvím.

## Offline demo a budoucí replay

Aktuální demo transport generuje deterministické raw odpovědi a používá produkční parsery. Další krok je verzovaný replay formát obsahující dvojice command/response, časování, profil a anonymizovaná metadata. Import replay souboru musí odmítnout neznámé schéma a před uložením nabídnout redakci VIN, sériových čísel a dalších identifikátorů.

Referenční standard: [ASAM MCD-2 D](https://www.asam.net/standards/detail/mcd-2-d/). ODX popisuje diagnostická data a služby; nezajišťuje přístupová oprávnění k ECU ani licenci k OEM datovým balíčkům.
