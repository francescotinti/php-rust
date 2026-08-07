# WP_SESSION_111 — S-111: hot-cluster dispatch REFUTATO con verdetto pieno; held-out al debutto
**In una frase**: congelati tre giudici «fuori concorso» PRIMA di progettare
la leva; il dispatch raggruppato dà all'aritmetica appena +3% mentre il resto
peggiora (due fuori concorso su tre inclusi) — nella forma provata la
dispersione degli handler caldi non regge come motore (direzione); ritirata
al byte.
**SCOREBOARD** (micro NON rimisurati: pin 92909544 invariato, valori S-109):
**arith 9,3 = · prop 7,9 = · calls 5,1 = · str 5,3 = · arr 3,9 = · re 3,5 =**
WordPress: non rimisurato (coppia non dovuta: leva non spedita) — rif resta
full 1,867/1,869, media 2,673/2,612 (S-110). **Leve perf spedite: 0
(dichiarato)** — leva TENTATA con A/B pieno e verdetto (ritmo rispettato).
**Data**: 2026-08-08 · **Modello**: Fable 5 · **Commit**: fdbe5c8→e68fa1d pushati.

## Esiti secchi
1·held-out CONGELATI pre-progettazione (fdbe5c8; parità 9/9, N solo-oracle,
tare az.4 nel README) → 2·criterio
PRE (451d747: soglia max(4 ns/iter; rumore ABAB; banda-layout 0,67), tetto
×1,48, verdetto a 3 esiti) → 3·leva: 8 corpi caldi in metodi inline(always) +
pre-match compatto in testa a run_loop, match grande intatto (admission:
batteria 1742/0 rc=0, parità 9/9; churn relink batteria dichiarato) → 4·A/B
ABAB R=2 con early-stop ESERCITATO: **FAMIGLIA-REFUTATA** (arith −2,70 ns/iter
sotto soglia; prop +0,67 segno opposto = banda-layout; guardie SFONDATE:
calls +11,4%, arr +8,4% = tassa del pre-filtro sui sentieri freddi) →
5·guardie §9: held-out PRIMA LETTURA **pin 6,7·2,6·5,6 / leva 7,2·2,6·6,0**
(poly/wploop regrediti, err ENTRO spread — az.5 rev.: NON generalizza);
contro-lettura delivery arith 0,325→0,295 (quote per-motore = DIREZIONE) con
tempo −3,4% coerente, residuo ~0,30 coi 4 handler ADIACENTI → 6·revert
al byte (e68fa1d: run.rs diff vuoto vs pre-leva; release dallo stash =
92909544). Disasm (protocollo): run_loop 287.944→226.092 B, bl 5849→4770,
br 22→36 — inliner flippato, dichiarato.
## ⭐ Lezioni (max 3)
- ⭐⭐ **Il held-out pre-congelato morde alla prima uscita**: leva che vince
  (poco) sui giudici di progetto e perde su due fuori-concorso (terzo fermo)
  = overfitting fotografato, non opinione.
- ⭐⭐ **La contro-lettura a contatori trasforma la refutazione in meccanismo**,
  in DIREZIONE: residuo delivery ~0,30 con handler adiacenti ⇒ nella forma
  provata lo scatter non regge come motore; il colpevole (salto indiretto/
  fetch helper) è ipotesi da MISURARE, non premessa.
- ⭐ **Le guardie esplicite sui non-bersagli decidono i verdetti** (+11% calls
  per −3% arith); la «tassa del pre-filtro» resta NON ripartita dal flip
  dell'inliner (br 22→36): dente = A/B con filtro che non intercetta nulla.
