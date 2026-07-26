# REPORT_GAP_62 — misure della SOLA sessione WP-62 (2026-07-26 sera)

Binario: tree WP-62 `c20a42d` (phpr-wp62 = 2f5220c7…). Cambi engine
della sessione: contatori unit-cache M1 (path include, freddo) +
refactor reloc marker-espliciti — attesi neutri; gate62 verde completo.

## Media group (gap62, orchestrate62 — 1 coppia oracle/phpr)

| | user CPU | peak phys | vs oracle |
|---|---|---|---|
| oracle 8.5.7 | 21,00s | 367,8MB | — |
| phpr wp62 | 55,05s | 1663,5MB | **CPU 2,62× · footprint 4,52×** |

⚠️ Caveat di lettura: nello STESSO A/B serale (4 run media identici nel
carico) il peak phpr ha oscillato 1611,6-1663,2MB (±1,6%) e lo user
54,17-56,03s (±1,7%); l'oracle stanotte è sceso a 367,8MB da 394,9MB
di WP-60 (−6,9%, singola run). Il 4,52× è quindi rumore di coppia
singola sul numeratore E sul denominatore: il riferimento strutturale
resta ~4,1× finché una media multi-run non dice altro. Nessuna leva
footprint/CPU è stata spedita in WP-62 (leva sospesa al decision
point): non c'è meccanismo che possa aver mosso il footprint.

## Media A/B interno (K-M1, stessa sera, interleaved a1/b1/b2/a2)

A (tree WP-62) 54,74/56,03s user · B (A+relbase-probe) 54,17/54,37s
⇒ B −2,0% = l'indirezione negli arm caldi NON emerge sul media
(micro advisory: +1,0-1,3% sui canali op-densi).

## Full-suite (coppia stessa-sera run49: new c20a42d vs old phpr-wp61)

<!-- SLOT-RUN49 -->
