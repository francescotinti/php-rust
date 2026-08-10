# REPORT_GAP_125 — SOLO S-125 (2026-08-10). Coppia WP sul pin s124 c5ba2573 (phpr; server non esercitato), pair109 INVARIATA ×4 gambe INTERCALATE stessa notte (off1→on1→off2→on2), rc=0 ×4, gate_void=0 ×4.

## Cifre (raw: wp125-harness/pair-out/leg*-*/; verdetto: wp125-harness/s125-pair-verdetto.out)
| voce | OFF (gamba1 / gamba2) | ON (gamba1 / gamba2) |
|---|---|---|
| full master CPU ratio (coppie proprie) | **1,896 / 1,850** | **1,818 / 1,818** |
| full oracle → phpr (s) | 436,83→828,20 · 442,11→817,94 | 441,39→802,25 · 441,36→802,21 |
| media user CPU ratio | 2,518 / 2,485 | 2,492 / 2,505 |
| full peak phpr (MiB) | 1959,4 / 1901,0 | 1906,7 / 1838,1 |
| parità | full == solo wp_is_stream #2; media 0 fail | idem ×2 |

## Lettura (N=2 per modo, stessa notte, oracle N=4 spread 1,2%)
- **Matrice 16 celle: full = 1,815–1,896** — intervallo SOVRAPPOSTO al riferimento
  S-120 (1,810–1,889): **l'effetto full di PhpStr è ≤~2%, NON RISOLVIBILE a N=2** (attesa dalle micro ~1–2% del full; phpr spread 3,2% gonfiato dalla gamba off1, inquinata: 249.003 involuntary ctx-switch vs 78–81k delle altre, rev. S-125). Il gap full vive
  altrove: da profilare sul workload WP, non sulle micro.
- ON di nuovo CONCORDE e più stretto di OFF (802,2/802,2 vs 817,9/828,2).
- media user-only 2,485–2,518 vs 2,511–2,534 (S-120): lieve discesa, direzione
  coerente con le micro str/arr, magnitudine dentro il rumore N=2.
- peak 1838–1959 MiB: dentro la banda run-to-run storica (~120 MiB), nessun
  effetto della single-alloc sul picco (attesa: neutra, confermata).
- **NUOVO riferimento full = 1,815–1,896 @ pin s124** (sostituisce 1,810–1,889).
