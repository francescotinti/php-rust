# REPORT_GAP_171 — gap perf oracle↔phpr al pin s171 (SOLO sessione S-171, 2026-09-07 sera)

Pin: phpr b360b2933eddfe18 + server b3ddaede545ba894 (promozione rc=0, wp171-harness/s171-promo-verdetto.out).

## Micro (run-micro.sh R=5, pavimenti sottratti per binario) — rapporto phpr/oracle
| categoria | s166 | **s171** | Δ |
|---|---|---|---|
| arith | 5,4 | **2,7** | −2,7 (leva L-SL1; TAPPA ≤3× raggiunta) |
| prop | 5,5 | 5,2 | −0,3 (guardia, direzione coerente: CmpJmpSC/IncDec nel loop) |
| calls | 4,8 | 4,8 | = |
| str | 4,2 | 4,1 | −0,1 (tick) |
| arr | 3,2 | 3,2 | = |
| re | 2,5 | 2,5 | = |

## Giudici propri della leva (A/B R=5, ns/iter, s171-leva4-verdetto.out)
- arith-dq: pin s166 46,76 → s171 23,44 (oracle 8,64): 5,41× → 2,71×.
- arith-e2 (loop nudo): 14,72 → 10,64 (oracle 3,48): 4,23× → 3,06×; per-op 7,36 → 5,32 vs 1,74.
- Contrasto col tetto mock m13 (S-170): B−C = −1,04 su dq (banda ≤5: tetto riprodotto).

## Suite (coppia t17 al pin s171, DOVUTA)
In corso al momento della rotazione: esiti in wp171-harness/s171-pair-verdetto-t17.out e
s171-orm-coppia-verdetto.out (rif. precedenti: WP 1,746-1,749 mediana t15/t16; ORM [7,023;7,053]).
Attesa dichiarata: direzione ≤0, magnitudine SOTTO-risoluzione (quota Long-loop in WP/ORM non censita).

## Residuo nominato (direzione, non magnitudine)
arith-dq 23,44 = e2 10,64 + statement 12,80 (oracle 5,16): il divario residuo sta ora per ~metà
nel controllo-loop (2 op: CmpJmpSC+IncDecSlotJmp = 5,32/op vs 1,74) e per metà nello statement
fuso (12,80 vs 5,16); dispatch 1,75/op (S-169) resta il pavimento noto.
