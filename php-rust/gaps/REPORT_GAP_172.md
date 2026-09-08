# REPORT_GAP_172 — gap perf oracle↔phpr al pin s172 (SOLO sessione S-172, 2026-09-07/08 notte)

Pin: phpr 5f2dff7d17ebed79 + server 812f7962952da67f (promozione tentativo 2 + ripresa, wp172-harness/s172-promo-verdetto.out).

## Micro (run-micro.sh R=5, pavimenti sottratti per binario) — rapporto phpr/oracle
| categoria | s171 | **s172** | Δ |
|---|---|---|---|
| arith | 2,7 | **2,7** | = (guardia: BinarySCSCDst intatto) |
| prop | 5,2 | **3,8** | −1,4 (leva L-SL2: P1 bigramma fuso + P2 BinarySTDst) |
| calls | 4,8 | 4,7 | −0,1 (tick) |
| str | 4,1 | 4,1 | = |
| arr | 3,2 | 3,0 | −0,2 (P2: `$s += $a[$i]` è BinarySTDst; direzione, non attribuita) |
| re | 2,5 | 2,5 | = |

## Giudice proprio della leva (A/B R=5, ns/iter, s172-leva-verdetto.out)
- prop-dq (N=150M): pin s171 72,60 → P1 57,53 → P1+P2 52,87 (oracle 14,00): 5,19× → 4,11× → 3,78×.
- Contrasto C−B (P2): +4,67 = direzione (C mai alternato: nessuna cifra attribuita a P2, rilievo 1 revisione).
- Guardia arith-dq: 23,24 → 23,52/23,72 (sotto soglia, nessuna regressione).
- Conferma post-pin (pin s172 vs stash s171, R=5): D=+20,07 rumore 0,73 segni 5/5 (A=73,47 B=53,40 ns/iter): nell'intorno del D_C dell'A/B.

## Suite (coppia t18 al pin s172, DOVUTA)
- WP full (6 gambe ON, 5 pulite: leg1 SEGNALATA ictx phpr 201% ed esclusa): mediana t18 = **1,769**
  in banda [1,738;1,799] → COMPATIBILE, nessun claim (t17 1,776; t15/t16 1,746/1,749); banda_ON 0,023;
  media user-only 2,43-2,46; parità full/media OK (s172-pair-verdetto-t18.out, rc=0).
- ORM: net [7,090;7,092] vs registrato s166 [7,023;7,053] (+0,04, dentro lo storico S-164 [7,066;7,111]); Δ_norm [−0,38;−0,29] NON risolta (a cavallo di 0,293); sentinella oracle 4,87/4,88 IN banda (valida); parità ORM 16 · dbal 10 stabili — nessun claim, VOCE da rivedere in S-173 (regola 4: se resta >7,05 istruttoria) (s172-orm-coppia-verdetto.out).
Attesa dichiarata (direzione ≤0, magnitudine sotto-risoluzione): WP COMPATIBILE.

## Residuo nominato (direzione, non magnitudine)
prop-dq 52,87 vs oracle 14,00: residuo 38,9 ns/iter = dispatch 8×1,75=14 + guardie IC/borrow
(RefCell ×4 + ic.get ×2 per iterazione) + Sweep×2 (~5,8) + push/pop del Long fra PropGetSlot e
BinarySTDst + CmpJmpSC/IncDec (~7). Fetta 3 candidata: peephole PropGetSlot+BinarySTDst (niente
pila) e guardie IC a un solo borrow; poi calls (forma frame: Call/BinarySS/Ret) e str (ConcatNConst
+ CallBuiltin substr) = forme DIVERSE, con criterio proprio.
