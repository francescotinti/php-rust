# REPORT_GAP_173 — gap perf oracle↔phpr al pin s173 (SOLO sessione S-173, 2026-09-10 notte)

Pin: phpr da4921a52eba0187 + server 2d2adc4549a820e0 (promozione braccio C, wp172-harness/s173-promo-verdetto.out).

## Micro (run-micro.sh R=5, pavimenti sottratti per binario) — rapporto phpr/oracle
| categoria | s172 | **s173** | Δ |
|---|---|---|---|
| arith | 2,7 | **2,8** | +0,1 (tick) (guardia: BinarySCSCDst intatto) |
| prop | 3,8 | **3,0** | −0,8 (leva fetta 3: P3 peephole PropGetSlot+BinarySTDst + P4 borrow unico) |
| calls | 4,7 | 4,8 | +0,1 (tick) |
| str | 4,1 | 4,1 | = |
| arr | 3,0 | 3,1 | +0,1 (tick) (guardia vera di P3: `$s += $a[k]` NON è coperta dal peephole) |
| re | 2,5 | 2,5 | = |

## Giudice proprio della leva (A/B R=5, ns/iter, s173-f3-verdetto.out, ordine dei tre bracci ruotato)
- prop-dq (N=150M): pin s172 52,53 → P3 44,93 → P3+P4 41,13 (oracle 13,93): 3,77× → 3,22× → **2,95×**.
- Contrasto C−B (P4, alternato): +3,80 firmato, NON nominato (soglia 4,00) ⇒ solo direzione.
- Guardia arith-dq: 23,80 → 23,68/23,68 (piatta).
- Az.rev. S-172 (b): A/B alternato B↔C sui bracci S-172: B−C=+4,87 nominata ⇒ cifra a P2 (s173-abbc-verdetto.out).
- Conferma post-pin (pin s173 vs stash s172, R=5): +11,67 rumore 0,93 segni 5/5.

## Suite (coppia t19 al pin s173, DOVUTA)
- WP full: t19 LETTA (S-173, 2026-09-10 06:12): WP mediana 1,772 in banda [1,738;1,799] = COMPATIBILE, 6/6 gambe pulite, banda_ON 0,052 (s173-pair-verdetto-t19.out rc=0); ORM net [6,988;7,012] vs registrato [7,023;7,053] ⇒ RIENTRA sotto 7,05, regola 4 NON scatta (la voce t18 [7,090;7,092] si chiude); Δ_norm [+0,02;+0,21] nel rumore ±0,293, sentinella oracle 4,94/4,93 in banda (a filo dell estremo 4,94), parità ORM 16 · dbal 10 stabili (s173-orm-coppia-verdetto.out rc=0)
- ORM: t19 LETTA (S-173, 2026-09-10 06:12): WP mediana 1,772 in banda [1,738;1,799] = COMPATIBILE, 6/6 gambe pulite, banda_ON 0,052 (s173-pair-verdetto-t19.out rc=0); ORM net [6,988;7,012] vs registrato [7,023;7,053] ⇒ RIENTRA sotto 7,05, regola 4 NON scatta (la voce t18 [7,090;7,092] si chiude); Δ_norm [+0,02;+0,21] nel rumore ±0,293, sentinella oracle 4,94/4,93 in banda (a filo dell estremo 4,94), parità ORM 16 · dbal 10 stabili (s173-orm-coppia-verdetto.out rc=0)

## Residuo nominato (direzione, non magnitudine — IPOTESI: le cifre per-op vengono da altri binari, S-169; revisione rilievo 6)
prop-dq 41,13 vs oracle 13,93: residuo 27,2 ns/iter = dispatch 7×1,75=12,3 + Sweep×2 (~5,8) +
CmpJmpSC/IncDec (~7) + corpi prop ≈2: il CORPO prop è quasi esaurito sul giudice; il residuo è
dispatch + Sweep + controllo di loop, comune a tutte le categorie ⇒ prossime candidate trasversali:
Sweep elidibile senza temporanei vivi, fusione del bigramma di loop CmpJmpSC+IncDecSlotJmp (criterio
proprio, giudice arith-dq). Perimetro: il probe sigillato P1/P4 copre SOLO classi plain (IC set non
riempita su typed/private): census WP/ORM delle forme typed prima di una fetta 4.
