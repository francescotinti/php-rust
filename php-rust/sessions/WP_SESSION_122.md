# WP_SESSION_122 — banda-LAYOUT (PROVVISORIA) · L-ST1 refut. NON ACQUISITA (effetto-ordine, revisore) · L-RE2 fermata

**In una frase**: misurata la «banda layout» e con essa giudicate due leve —
ma il revisore ha scoperto che l'ordine FISSO di misura (pin sempre primo)
può spiegare da solo sia la banda sia il verdetto al filo: i verdetti si
ridimensionano e S-123 rimisura con ordine permutato.

**SCOREBOARD** (pin s120 885d2c64 INVARIATO — micro non rimisurate; vs S-121):
**arith 5,5 = · prop 5,5 = · calls 4,8 = · str 5,3 = · arr 3,7 = · re 2,8 =**
· rif WP **full = 1,810–1,889** (non rimisurato) · **leve perf spedite: 0**
(2 TENTATE con A/B e verdetto: ritmo rispettato; sessioni-senza-Δ = 2).
2026-08-09 · Fable 5 · f09e9a1→(chiusura).

## Esiti secchi
1·**BANDA_LAYOUT PROVVISORIA** (criterio 71411c1 PRIMA; K=4 layout, probe
17/173/1731, admission 6/6+6/6, R=5): **arith 1,00 · prop 1,00 · calls 0,00 ·
str 5,00 · arr 3,33 · re 5,00** ns/iter — MA ordine di misura FISSO (P0 primo
in ogni round; P0 min in 4/6, re al contrario): banda ⊄ effetto-ordine, si
rimisura permutata + pin-vs-pin (az. rev. #1). 1° probe DEAD-STRIPPATO
(admission ha morso) → emendazione dichiarata (keep-alive).
2·**L-ST1 full** (2e1eda8d): str D_med **−5,00** +0/5 = soglia ⇒ macchina
dice «confermata», revisore RIDIMENSIONA a **NON ACQUISITA**: TA sempre prima
di TB (stesso artefatto), margine zero; soglia_ref senza 2×spread_A.
3·**L-RE2 TENTATA e FERMATA** (criterio a893912 PRIMA; SmallVec inline
Caps.groups): census **re 10→9,00 ESATTO** + predizioni secondarie 5/5;
admission 6/6+6/6; smoke re **−20/−10, |D_med| 15,00 > banda 10,00** ⇒
early-stop. Revisore: «fermata PRUDENTE, non un dato» (R=2, un tick dal
bordo, stesso ordine fisso) — retry R≥5 ordine alternato PRIMA di archiviare
(az. rev. #5). Anti-tesi mosse Caps 176 B resta NOMINATA. Reperto s122-re2.
4·**Contorno**: gate preg §3.18 cablato (8 gate); il census SPEGNE il peephole
fuso (run.rs:4282) ⇒ classifica sovrastima il release; istruttorie
prop/arr/re a verbale; PGO stadio-2 RINVIATO dichiarato.

## ⭐ Lezioni (max 3)
- ⭐⭐ **L'ordine fisso di misura è un confondente sistematico**: banda e A/B
  misuravano entrambi «il primo del giro vince» e si auto-confermavano a
  margine zero — ordine PERMUTATO per round e pin-vs-pin come pavimento.
- ⭐⭐ **Una firma census può descrivere un sentiero che il release non
  percorre**: la prossima classifica vuole contatori NEL ramo fuso.
- ⭐ **Un no-op senza riferimento non esiste**: ld64 lo dead-strippa (nm + hash ≠ pin obbligatori).
