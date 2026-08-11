# REPORT_GAP_128 — SOLO S-128 (2026-08-11). Coppia WP sul pin s127b ccb63dca (due leve accumulate dal rif: cbargs2 + stampo), pair109 INVARIATA ×4 gambe INTERCALATE stessa notte (off1→on1→off2→on2), rc=0 ×4, gate_void=0 ×4.

## Cifre (raw: wp128-harness/pair-out/leg*-*/; verdetto: wp128-harness/pair-out/cross-ratios.out)
| voce | OFF (gamba1 / gamba2) | ON (gamba1 / gamba2) |
|---|---|---|
| full master CPU ratio (coppie proprie) | **1,909 / 1,805** | **1,767 / 1,765** |
| full oracle → phpr (s) | 423,57→808,73 · 440,90→795,62 | 444,99→786,31 · 443,33→782,36 |
| media user CPU ratio (user-only) | 2,539 / 2,447 | 2,463 / 2,455 |
| full peak phpr (MiB) | 1894,4 / 1880,2 | 1837,3 / 1828,2 |
| peak ratio | 2,374 / 2,363 | 2,308 / 2,299 |
| parità | full == solo wp_is_stream #2; media 0 fail | idem ×2 |

## Lettura (N=2 per modo, stessa notte; oracle N=4 spread 5,1%, phpr 3,4%)
- **Matrice 16 celle: full = 1,758–1,909** (nuovo riferimento @ s127b; sostituisce
  1,815–1,896 @ s124). Il numeratore SCENDE su OGNI gamba (782–809 vs 802–828 S-125,
  −2,3% mediano: direzione firmata cbargs2+stampo, magnitudine non ripartita); il
  bordo alto 1,909 viene dal DENOMINATORE (gamba off1 oracle veloce 423,6 s, spread
  oracle 5,1% vs 1,2% S-125) — le coppie proprie ON dicono 1,765–1,767.
- media user-only **2,447–2,539** (S-125: 2,485–2,518): bordo basso migliora, bordo
  alto = gamba off1 (stessa gamba col denominatore anomalo).
- peak 1828–1894 MiB (2,299–2,374×): sotto la banda S-125 (1838–1959), coerente
  con lo stampo (default condivisi COW).
- ON di nuovo più stretto e più basso di OFF (782,4/786,3 vs 795,6/808,7).
