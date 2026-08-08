# WP_SESSION_114 — banda-layout MISURATA (6,67) · L-A firmata (+30,33 5/5) NON promossa · delta batteria non-H-P1

**In una frase**: misurata l'«impaginazione» del binario che sposta i
cronometri (fino a 6,7 ns: spiega il morso di ieri su calls); la fusione sulle
proprietà velocizza il giudice 5/5 ma letture anomale del sistema hanno
inquinato il metro: si riverte per regola; il test «sparito» non era il codice.

**SCOREBOARD** (pin s112 f71abd2a INVARIATO; micro = S-112, frecce =):
**arith 5,5 = · prop 7,6 = · calls 5,2 = · str 5,3 = · arr 4,2 = · re 3,4 =** ·
held-out 6,4·2,5·5,6 e WP (1,867/1,869 · 2,632/2,603) non rimisurati.
**Leve spedite: 0 — dichiarato; ritmo: 1 leva TENTATA (A/B pieno) + 1 leva-nulla.** 2026-08-08 · Fable 5 · 8cf0b61→0b04d7b.

## Esiti secchi
1·LEVA-NULLA (criterio PRE 8cf0b61): zavorra +2.360 B nel braccio Throw,
admission 12/12; **banda(cat): arith 0,40 · prop 4,33 · calls 5,50 · str 5,00
· arr 6,67 · re 0,00 → GLOBALE 6,67** (N=1). calls −5,50 = replica ESATTA del
morso S-113 (era layout); una leva NULLA fa 5/5 segni concordi su 3 categorie;
+3,33 H-P1 DENTRO banda. Revert al byte → 2·LEVA L-A (criterio PRE f9e3fe1;
codice 2c18b2e): peephole PropGetSlotRecv+BinaryTCPropSetPop, probe bipartito
solo-borrow, miss verbatim, fuori dai census; admission: batteria **1742/0/2
CON NOMI**, dump 12/12, run_loop +3.176 B; full R=5 a macchina: **prop +30,33
5/5** MA spread_A=47,00 (2 run del PIN a ~150 vs ~107: ambiente, ipotesi
P/E-core NON attribuita) → soglia 47 FAIL; **guardia calls −6,50 < −5,50
SFONDATA** (famiglia layout) → **NON PROMOSSA: «direzione firmata, magnitudine
non stabilita»**; revert al byte VERIFICATO ×2 (candidati conservati
phpr-s114-nulla 846d0df4, phpr-s114-la 052ea417) → 3·DELTA BATTERIA: tree
H-P1 ricostruito (8bb395c) → 1742/0/2, inventario 1744 nomi IDENTICO; 1741
S-113 NON attribuibile a H-P1 con l'evidenza disponibile (az.4 revisione:
flaky non escluso, N≥3 in apertura); i log ora si conservano. Incidenti processo: 3 (batteria contaminata da edit concorrente,
uccisa e rifatta · redirect pre-mkdir · checkout-staging di run.rs: corretto
index-only 3cf7c39).

## ⭐ Lezioni (max 3)
- ⭐⭐ **La banda-layout misurata declassa i verdetti mono-punto**: una leva
  nulla fa 5/5 segni concordi su tre categorie — la concordanza di segno senza
  margine sopra banda non firma una causa.
- ⭐⭐ **Il metro può essere il colpevole**: 2 run del PIN a +40% bocciano un
  mediano +30 5/5 via spread_A — il criterio emendato renda lo scheduling
  osservabile o escluda per NOME (pre-registrato) le run fuori famiglia.
- ⭐ **`git checkout <commit> -- path` STAGIA**: dopo un checkout parziale, `show --stat` su ogni commit.
