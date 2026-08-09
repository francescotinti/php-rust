# WP_SESSION_122 — banda-LAYOUT emessa · L-ST1 refutazione CONFERMATA · L-RE2 fermata dallo smoke

**In una frase**: misurato quanto due build identiche differiscono per solo
caso (banda layout); con quella riga il verdetto sulla leva stringhe è
DEFINITIVO (era la leva, non il caso), e una seconda leva regex — pur togliendo
l'allocazione promessa — ha peggiorato il tempo ed è stata fermata dalla
regola scritta prima.

**SCOREBOARD** (pin s120 885d2c64 INVARIATO — micro non rimisurate; vs S-121):
**arith 5,5 = · prop 5,5 = · calls 4,8 = · str 5,3 = · arr 3,7 = · re 2,8 =**
· rif WP **full = 1,810–1,889** (non rimisurato) · **leve perf spedite: 0**
(2 TENTATE con A/B e verdetto: ritmo rispettato; sessioni-senza-Δ = 2).
2026-08-09 · Fable 5 · f09e9a1→(chiusura).

## Esiti secchi
1·**BANDA_LAYOUT** (criterio 71411c1 PRIMA; K=4 layout, probe 17/173/1731,
admission 6/6+6/6, R=5 interleaved): **arith 1,00 · prop 1,00 · calls 0,00 ·
str 5,00 · arr 3,33 · re 5,00** ns/iter. 1° probe DEAD-STRIPPATO (__TEXT
identico ⇒ NULLA, admission ha morso) → emendazione DICHIARATA (keep-alive).
2·**L-ST1 full** (2e1eda8d): str D_med **−5,00**, +0/5 ⇒ **REFUTAZIONE
CONFERMATA** a **margine ZERO** (soglia −5,00 esatta, a verbale); guardie 5/5;
superstite il bookkeeping, non il layout (BL_str 5,00 < smoke 6,25).
3·**L-RE2 TENTATA e FERMATA** (criterio a893912 PRIMA; SmallVec inline
Caps.groups): census **re 10→9,00 ESATTO** + predizioni secondarie 5/5;
admission 6/6+6/6; smoke re **−20/−10 concorde, |D_med| 15,00 > banda 10,00**
(≥2×quanto: asimmetria S-121 sanata) ⇒ early-stop, niente full. Anti-tesi
NOMINATA nel criterio: mosse Caps 24→176 B > malloc. Reperto phpr-s122-re2.
4·**Contorno**: gate preg §3.18 cablato (8 gate); istruttoria prop: il census
SPEGNE il peephole fuso (run.rs:4282) ⇒ classifica sovrastima il release;
istruttorie arr/re a verbale; PGO stadio-2 RINVIATO dichiarato.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il delta-alloc non predice il tempo**: 2ª e 3ª leva alloc-removal
  refutate con meccanismo nominato — la classifica per delta-alloc ha esaurito
  i guadagni facili, i residui sono STRUTTURALI.
- ⭐⭐ **Una firma census può descrivere un sentiero che il release non
  percorre**: la prossima classifica vuole contatori NEL ramo fuso.
- ⭐ **Un no-op senza riferimento non esiste**: ld64 lo dead-strippa; ogni
  probe di layout pretende simbolo in nm e hash ≠ pin.
