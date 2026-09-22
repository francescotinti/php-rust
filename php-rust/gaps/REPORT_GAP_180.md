# REPORT_GAP_180 — gap perf oracle↔phpr al pin s180 (SOLO sessione S-180, 2026-09-21/22 notte)

Pin: phpr 884399fc52277119 + server 045fe03356ee73e5 (ri-pin: stesso sorgente del pin s175 + gc-idle S-176, L-CM1 revertata, toolchain Rust 1.96.0→1.98.1; wp180-harness/s180-promo-verdetto.out, tutti i gate rc=0). Nessuna leva: ogni Δ vs s175 è «direzione toolchain+gc-idle, NON ripartita».

## Micro (run-micro.sh R=5, pavimenti sottratti per binario; E2 PASS t3, quiescenza t1) — rapporto phpr/oracle
| categoria | s175 (1.96.0) | **s180 (1.98.1)** | Δ |
|---|---|---|---|
| arith | 2,4 | **2,2** | −0,2 (toolchain/gc-idle, non ripartito) |
| prop | 2,6 | **2,4** | −0,2 (idem; prop-dq 36,20→34,20 +2,00 5/5 rumore 0,40, sola direzione) |
| calls | 4,6 | **4,5** | −0,1 (tick) |
| str | 4,1 | 4,1 | = |
| arr | 3,0 | **2,9** | −0,1 (tick) |
| re | 2,5 | 2,5 | = |

## WordPress coppia t21 (s180-pair.sh, rc=0, s180-pair-verdetto-t21.out)
mediana **1,744** su N=6 coppie proprie ON pulite (1,728-1,759; banda_ON 0,030) — **COMPATIBILE** in [1,738;1,799]; t20 1,765; media user-only 2,42-2,44; ictx contesa ok; deriva nessuna. Parità: media vuota 6/6; full 5/6 «solo wp_is_stream #2», leg4 DIVERSA su test da rete (Fonts GetData + REST Attachments from_url, ~10 nomi) = voce esterna transitoria. Attesa: nessuna (toolchain + gc-idle sotto-risoluzione).

## Doctrine ORM + DBAL (s176-orm-coppia.sh canone E3/E4, rc=0, s180-orm-coppia-verdetto.out)
ORM net **[7,008;7,060]** (t20 [6,936;7,014]; registrato S-162 [7,023;7,053]; storico S-164 [7,066;7,111]); Δ_norm oracle-normalizzato [−0,14;+0,19] s dentro il rumore ±0,293 ⇒ COMPATIBILE, nessun claim; **leg2 7,060 >7,05 a filo ⇒ VOCE riaperta (regola 4: istruttoria a t22 se persiste)**. Sentinella oracle 4,96/4,93 IN banda [4,84;5,04]. dbal net [7,440;7,463]. Parità ORM 16 nomi · dbal 10 nomi stabili.

## Lettura del gap (non cifra)
Il gap resta nei CORPI (calls 4,5 e str 4,1 i peggiori). Il cammino «scritture di proprietà private» pesa ≈1,1 % della suite ORM (S-179, ricalcolo confermato): non è dove sta il 7×. Prossimo bersaglio S-181: handler di chiamata (census Call/Ret/args-Vec, giudice calls-dq, A/B same-toolchain).
