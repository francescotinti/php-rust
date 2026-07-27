# REPORT_GAP_63 — misure della SOLA sessione WP-63 (2026-07-27 notte)

Leva spedita: stub-elision default-ON (contratto v2, stash phpr-wp63
`1666e1b4…`). Tutte le misure = coppie stessa-sera.

## Media group (coppia SINGOLA — per G3 non entra nel trend come mediana)

| | user CPU | peak phys (time -l) |
|---|---|---|
| oracle 8.5.7 | 21,01s | 393,3MB |
| phpr (flip WP-63) | 54,14s | 1.170,3MB |
| **ratio** | **2,58×** | **2,98×** |

Spread: coppia singola ~02:02-02:04, deriva serale nota ~2,6% (CPU);
il footprint 2,98× vs riferimento 4,1× è variazione STRUTTURALE
(−30% assoluto: 1.611-1.663MB → 1.170MB), da confermare come nuovo
riferimento con protocollo ≥3+3 (G3).

## Full suite (coppia run50 stessa-sera, wpdev 30.472 test)

| | user CPU | peak phys |
|---|---|---|
| run50-old (phpr-wp62) | 783,80s | 3,896GB |
| run50-new (flip WP-63) | 781,32s | **2,105GB** |
| **delta** | **−0,32%** | **−46,0%** |

Fail-set: **88 BYTE-ID su entrambe = run33** (baseline invariata).
CPU full riferimento resta 2,06-2,11× (nessuna variazione oltre il
rumore); compile-side counted (census63-full master): net_tot
1.973,3MB → 499,9MB (−74,7%).

## Note di metro

R3 counted/net 0,557 ⇒ banda NET→PHYS counted-only (KG2): i verdetti
qui sopra sono coppie MISURATE, non traduzioni. Drain/net osservato:
1,19× su --list-tests (sopra banda [0,75-1,10], direzione prevista dal
rounding per-bin Leijen a.i).
