# OBDLink EX: sériová a protokolová inicializace

Implementace vychází z OBDLink Family Reference and Programming Manual, revize F (2025). OBDLink EX používá USB UART `115200 8-N-1`; software musí po každém příkazu čekat na prompt `>`. Produkt existuje v revizích STN2230, STN2231 a STN2232, proto aplikace neodvozuje schopnosti jen z jednoho pevného čísla čipu a vždy čte skutečné `STDI`/`STI`.

## Připojení

```text
115200 baud
8 data bits
no parity
1 stop bit
no host flow control
command terminator: CR (0x0D)
```

## Společná inicializační sekvence

```text
ATZ          reset, čekat až 2500 ms na prompt
ATE0         vypnout echo
ATL0         vypnout LF
ATS0         vypnout mezery
ATD0         nezobrazovat DLC

STDI         hardware ID
STI          firmware ID
STVR         napětí adaptéru/vozidla
```

Konkrétní nastavení hlaviček a protokolu následuje podle vybraného profilu.

## Univerzální OBD-II profil

```text
ATH0         skrýt protokolové hlavičky
ATAL         povolit dlouhé odpovědi
ATAT1        adaptive timing
ATCAF1       automatické ISO 15765 formátování
ATCFC1       automatický ISO 15765 flow control
STPTO 1500   výchozí timeout adaptéru
ATSP0        automaticky najít legislativní OBD-II protokol
```

První datový požadavek po `ATSP0` může trvat déle, protože adaptér zkouší podporované legislativní protokoly. Patří mezi ně ISO 15765-4 CAN, ISO 14230-4, ISO 9141-2 a SAE J1850. Po odpovědi aplikace načte detekovaný protokol příkazem `STPRS`.

## Volkswagen Group MQB profil

```text
ATH1         zobrazovat CAN hlavičky

STP 33       ISO 15765, 11-bit, 500 kbps, DLC 8
STPBRR       ověřit skutečných 500000 bps
STCAF 0      normal CAN addressing
ATCAF1       automatické ISO 15765 formátování
ATCFC1       automatický flow control
ATAL         povolit dlouhé odpovědi
ATAT1        adaptive timing
STCSEGR 1    složit přijímané ISO-TP multi-frame zprávy
STCSEGT 1    segmentovat odesílané multi-frame zprávy
STPTO 1000   výchozí timeout requestu v ms
STPO         otevřít protokol
```

`STP 33` je v oficiálním manuálu předvolba HS-CAN na OBD pinech 6/14 pro ISO 15765, 11bit CAN ID a 500 kbps. `STPBRR` je kontrola výsledné rychlosti, ne další konfigurace.

## Konfigurace fyzického cíle

Před requestem se nastaví cílová dvojice CAN ID a flow control:

```text
STPC
ATSH 70E
ATCRA 778
STFFCC
STFFCA 778,7FF
STCFCPC
STCFCPA 70E,778
STPO
```

Potom se request odešle atomicky například:

```text
STPX h:70E,d:22F187,t:1000,r:1
```

Hodnota `22F187` je UDS ReadDataByIdentifier. STPX předá zprávu internímu ISO-TP subsystému adaptéru, parametr `t` přepíše response timeout a `r:1` očekává jednu fyzickou odpověď. OBDLink provádí CAN segmentaci, reassembly a flow control; Rust vrstva pracuje s úplným UDS payloadem.

## Poznámky k reálnému vozidlu

- Testovací profil předpokládá ECU 09/J519 na `0x70E → 0x778`. Adresy je nutné potvrdit pro konkrétní vůz a gateway.
- Automatický profil potvrzuje jen standardní OBD-II kompatibilitu. Neobjevuje výrobce-specifické řídicí jednotky a neposkytuje komfortní kódování jiné značky.
- OBDLink EX existuje v několika hardwarových revizích; aplikace proto čte `STDI` a `STI` a neváže komunikaci na jediný číselný identifikátor čipu.
- Přepnutí UART na 2 Mbit/s není součástí bezpečné inicializace. Vyžaduje koordinovanou změnu rychlosti hostitele, ověření konkrétní revize adaptéru a recovery postup; pro diagnostické čtení zůstává výchozí rychlost 115200 stabilnější volbou.
- Napětí se v MVP pouze zobrazuje. Jakákoli budoucí zapisovací funkce musí mít tvrdý voltage interlock a stabilizované napájení.
- Při vývoji používejte ECU simulátor nebo bench setup; živé vozidlo není vhodné jako první test prostředí.

Referenční dokument: [OBDLink Family Reference and Programming Manual, revize F](https://www.scantool.net/scantool/downloads/682/obdlink_frpm_f.pdf).
