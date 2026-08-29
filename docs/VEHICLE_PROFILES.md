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
- aktuálně obsahuje experimentální předvolbu ECU 09/J519 pro Superb III facelift

## Přidání dalšího výrobce nebo platformy

Nový rozšířený profil by měl obsahovat:

1. stabilní identifikátor a uživatelský název profilu,
2. přesně definovanou inicializaci fyzického protokolu,
3. identifikaci ECU varianty podle part number a verze software,
4. typované případy použití místo obecného raw-hex vstupu,
5. allowlist služeb a testovací nahrávky odpovědí,
6. dokumentovaný seznam ověřených značek, modelů, roků a ECU.

Zápis nebo kódování musí být samostatná transakční vrstva s kontrolou napětí, zálohou původní hodnoty, explicitním potvrzením a recovery postupem. Nestačí rozšířit read-only allowlist.
