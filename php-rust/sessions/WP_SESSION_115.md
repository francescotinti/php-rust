# WP_SESSION_115 — L-A: magnitudine STABILITA (+26,33 5/5) ma NON promossa · banda N=2 + PRIMA banda held-out (il gate bocciato da una leva NULLA)

**In una frase**: la fusione sulle proprietà è confermata e quantificata (~26 ns
a operazione, sei volte la soglia), ma la promozione è rinviata: il controllo
su un giudice esterno è così sensibile all'«impaginazione» del binario che
anche una modifica VUOTA lo farebbe scattare — ora è misurato, S-116 lo corregge.

**SCOREBOARD** (pin s112 f71abd2a INVARIATO; micro = S-112, frecce =): **arith
5,5 = · prop 7,6 = · calls 5,2 = · str 5,3 = · arr 4,2 = · re 3,4 =** · held-out
6,4·2,5·5,6 e WP (1,867/1,869 · 2,632/2,603) non rimisurati. **Leve spedite: 0 —
dichiarato; ritmo: 1 leva RIMISURATA (A/B pieno+held-out) + 1 nulla.**
2026-08-08 · Fable 5 · 1a29438→d3070ba.

## Esiti secchi
1·L-A RIMISURATA (criterio PRE emendato 1a29438, lettere a-f): admission
hit+4miss output E dump identici al pin, bigramma in TUTTI (i miss esercitano
il probe); full R=5: **prop +26,33 5/5, spread_A DEPURATO 2,00** (famiglie
1,3×min: escluse per NOME coppia2/6 con A a 155/166 — in S-114 lo stesso
inquinamento dava spread 47 e invalidava) → PASS 6× la soglia 4,33; guardie
tengono (calls −7,00 SUL filo −7,00, dichiarato); parità output 6/6;
**held-out: poly 9,86>9,71 → NON PROMOSSA** (p.7, zero deroghe). Meccanismo
semantico REFUTATO: poly ha 0 PropGetSlotRecv → la leva non gira mai lì.
2·NULLA-2 (criterio PRE b8c2cfe + emendamento held-out PRE-run 36b286e):
32 round in Op::Clone (+1.676 B, bl+1), admission rc=0, parità 6/6; **banda
micro N=2: max = 0,40/4,33/5,50/5,00/6,67/10,00** (re 0→10: una banda N=1 può
mentire; calls −5,50 IDENTICO = 4° campione famiglia; arr +3,33 con 4/5
positivi da leva NULLA); **held-out PRIMO campione: poly 0,20s · err 0,01 ·
wploop 0,06 — il NULLO farebbe 9,80>9,71: il gate p.7 non distingue L-A dal
nulla** ⇒ blocco ATTRIBUITO a layout CON campione. Revert al byte VERIFICATO
(f71abd2a, cargo_rc dal comando); patch versionata s115-zavorra2.
3·Pre-flight: disco Data ✗ (3,5G: bundle VM Claude Desktop 7,5G — decisione
utente); mitigato con A/B su binari CONSERVATI (zero rebuild per L-A).
Incidente processo: 1 (BUILD_RC dopo pipe, dichiarato, MAI usato come gate).
Candidati: s114-la 052ea417 · s114-nulla 846d0df4 · s115-nulla2 d9093a6b.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Un giudice senza banda misurata non può fare da gate a soglia fissa**: la leva nulla sfonda il suo limite — prima la banda, poi il gate.
- ⭐⭐ **Le famiglie pre-registrate (1,3×min, esclusione per NOME) recuperano la misura senza toccare la leva**: spread_A 47→2 sugli stessi binari.
- ⭐ **Una banda a N=1 mente per categoria** (re 0→10): il valore d'uso è il max sui campioni, mai l'ultimo.
