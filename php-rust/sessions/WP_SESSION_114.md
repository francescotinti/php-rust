# WP_SESSION_114 — S-114: banda-layout MISURATA (globale 6,67) · leva L-A tentata: direzione firmata (+30,33 5/5) ma NON promossa dal criterio · delta batteria 1741/1742 CHIUSO

**In una frase**: abbiamo finalmente misurato quanto il solo «rimescolamento
di impaginazione» del binario sposta i cronometri (fino a 6,7 ns: tanto — e
spiega il morso di ieri su calls); la fusione tentata sulle proprietà velocizza
davvero il suo giudice in tutte e 5 le prove, ma due letture anomale del
sistema hanno inquinato il metro e una guardia è scattata: per regola si torna
indietro e si rimisura con metro emendato; il giallo del test «sparito» di ieri
non era colpa del codice.

**SCOREBOARD** (pin s112 f71abd2a INVARIATO; micro = valori S-112, frecce =):
**arith 5,5 = · prop 7,6 = · calls 5,2 = · str 5,3 = · arr 4,2 = · re 3,4 =** ·
held-out baseline 6,4·2,5·5,6 (non rimisurati: nessuna promozione). WP non
rimisurato (rif full ON 1,867 / OFF 1,869 · media 2,632/2,603). **Leve perf
spedite: 0 — dichiarato; ritmo rispettato: 1 leva TENTATA con A/B pieno R=5 e
verdetto + 1 leva-nulla di misura.** 2026-08-08 · Fable 5 · 8cf0b61→e89353c.

## Esiti secchi
1·LEVA-NULLA (criterio PRE 8cf0b61): zavorra +2.360 B nel run_loop (braccio
Throw, mai preso), admission 12/12, bl 5864→5865; **banda(cat)=|Δ mediano|:
arith 0,40 · prop 4,33 · calls 5,50 · str 5,00 · arr 6,67 · re 0,00 →
GLOBALE 6,67** (N=1 perturbazione, limite dichiarato). calls −5,50 = replica
ESATTA del morso S-113 (era layout); una leva NULLA fa 5/5 segni concordi su
3 categorie; il +3,33 di H-P1 sta DENTRO banda. Revert al byte (f71abd2a) →
2·LEVA L-A (istruttoria+criterio PRE f9e3fe1; codice 2c18b2e): peephole runtime
PropGetSlotRecv+BinaryTCPropSetPop, probe bipartito solo-borrow, miss verbatim,
fuori dalle build census; admission: batteria rc=0 **1742/0/2 CON NOMI**, dump
12/12, run_loop +3.176 B (bl+25); smoke 2/2 (+29,33/+5,33); full R=5 a
macchina: **prop +30,33, 5/5 positivi** MA spread_A=47,00 (2 run del PIN a
~150 vs ~107: ambiente, ipotesi P/E-core NON attribuita) → soglia 47 FAIL;
**guardia calls −6,50 < −5,50 SFONDATA** (famiglia layout, N=2 su calls) →
**NON PROMOSSA: «direzione firmata, magnitudine non stabilita»**; revert al
byte VERIFICATO (f71abd2a ×2, diff crates/ vuoto; candidato conservato
phpr-s114-la 052ea417) → 3·DELTA BATTERIA CHIUSO: tree H-P1 ricostruito
(8bb395c) → 1742/0/2, inventario 1744 nomi IDENTICO al tree S-114; il 1741
S-113 = one-off ambientale senza log; nessuna lettera-gate; da S-114 i log
batteria si conservano nei raw. Incidenti processo: 3 (batteria contaminata da
edit concorrente: uccisa e rifatta · redirect prima del mkdir · staging
implicito di `git checkout <commit> -- path` ha committato run.rs: corretto
index-only 3cf7c39).

## ⭐ Lezioni (max 3)
- ⭐⭐ **La banda-layout misurata declassa i verdetti mono-punto**: una leva
  nulla produce 5/5 segni concordi su tre categorie — la concordanza di segno
  senza margine sopra banda non firma una causa.
- ⭐⭐ **Il metro può essere il colpevole**: 2 run del PIN a +40% hanno gonfiato
  spread_A a 47 e bocciato un mediano +30 5/5 — il criterio emendato deve
  rendere lo scheduling osservabile o escludere per NOME (pre-registrato) le
  run fuori famiglia.
- ⭐ **`git checkout <commit> -- path` STAGIA**: dopo un checkout parziale, ogni
  commit va ispezionato con `show --stat` (il dente anti-.rs non copre questo).
