# Profily vozidel

OBD Studio odděluje standardní kompatibilitu OBD-II od výrobce-specifické diagnostiky. Výběr profilu probíhá před připojením a backend jej ukládá spolu se sériovou relací. Příkaz určený pro jiný profil skončí chybou `PROFILE_MISMATCH` ještě před zápisem na port.

## `genericObd2`

- nechá OBDLink EX automaticky detekovat legislativní protokol přes `ATSP0`
- používá pouze pojmenované read-only dotazy z Rust allowlistu
- neobsahuje CAN ID, DID ani kódovací funkce konkrétní značky
- je vhodný pro základní motorovou a emisní diagnostiku vozidel kompatibilních s OBD-II/EOBD/JOBD

Podpora konkrétního PID nebo VIN vždy závisí na implementaci vozidla. Standardní konektor ani úspěšné připojení samy o sobě neznamenají, že automobil poskytuje všechny položky.

## `vwMqb`

- nastavuje ISO 15765 s 11bit CAN ID a rychlostí 500 kbps
- používá explicitní fyzické adresování a UDS
- má vlastní read-only allowlist služeb
- obsahuje experimentální předvolbu ECU 09/J519 pro Superb III facelift
- zpřístupňuje read-only skupiny motor, vstřikování, DPF a turbo na fyzické adrese ECU motoru `0x7E0 → 0x7E8`
- obsahuje read-only katalog očekávaných ECU pro převodovku a pohon, jízdní režimy, podvozek a asistenty, interiér, karoserii a komfort, okna a dveře, světla, exteriér, klimatizaci a infotainment
- oblast načte jednou po jejím otevření, pokud je automatika zapnutá, nebo ručně; u odpovídající jednotky čte part number `F187`, verzi software `F189` a počet potvrzených DTC přes `19 01 08`
- výsledek „nezjištěna“ není hodnocen jako porucha, protože volitelná ECU nemusí být ve výbavě, může být uspána nebo nedostupná přes danou trasu
- definice DPF byly ověřeny proti ECU `04L 906 027 P`; ostatní DID zůstávají experimentální a UI je při odmítnutí označí jako nepodporované
- každá dostupná hodnota nese orientační hodnocení a vysvětlení; veličiny závislé na otáčkách nebo zatížení se bez potřebného kontextu neoznačují automaticky jako bezvadné
- po nalezení ECU lze ručně ověřit nejvýše 16 konkrétních DID; výsledek zůstává raw a informační, dokud není doplněný variantní dekodér

Katalog fyzických adres je experimentální. Je založený na komunitních datech odvozených z ODIS a křížově kontrolovaných proti veřejným MQB trasám. Úspěšná identifikace ECU je podmínkou pro pozdější přiřazení přesného datového a kódovacího profilu; samotná shoda CAN adresy nestačí k povolení zápisu.

Stavové DID pro polohu oken, světelné výstupy, klimatizaci a volbu jízdního profilu nejsou mezi všemi MQB variantami jednotné. Aplikace je proto nesmí odhadnout pouze podle diagnostické adresy. Doplní se jako verzované deklarativní profily navázané nejméně na part number, software identifikaci a ověřenou datovou délku odpovědi.

Pro jejich bezpečné získání slouží omezený průzkum DID: vždy pracuje s již katalogizovanou ECU, používá pouze službu `0x22`, deduplikuje vstup a odmítne dávku delší než 16 položek. Je určený pro ověřené kandidátní DID z ODX/diagnostických dat, nikoli pro brute-force hledání celého prostoru.

## Přidání dalšího výrobce nebo platformy

Nový rozšířený profil by měl obsahovat:

1. stabilní identifikátor a uživatelský název profilu,
2. přesně definovanou inicializaci fyzického protokolu,
3. identifikaci ECU varianty podle part number a verze software,
4. typované případy použití místo obecného raw-hex vstupu,
5. allowlist služeb a testovací nahrávky odpovědí,
6. dokumentovaný seznam ověřených značek, modelů, roků a ECU.

Zápis nebo kódování musí být samostatná transakční vrstva s kontrolou napětí, zálohou původní hodnoty, explicitním potvrzením a recovery postupem. Nestačí rozšířit read-only allowlist.
