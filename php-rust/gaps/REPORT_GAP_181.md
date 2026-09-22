# REPORT_GAP_181 — gap perf oracle↔phpr al pin s181 (SOLO sessione S-181, 2026-09-22 pomeriggio)

Pin: phpr 19a2faa83a492745 + server b2802f08c5887e77 (leva L-CR1 «Call/Ret magri», commit 172814ae; wp181-harness/s181-promo-verdetto.out, tutti i gate rc=0). Una leva: il Δ su calls è attribuito (A/B proprio ×3 + conferma post-pin); gli altri Δ sono tick o effetto trasversale replicato (quota non ripartita).

## Micro (run-micro.sh R=5, pavimenti sottratti per binario; E2 PASS t8, quiescenza t3) — rapporto phpr/oracle
| categoria | s180 | **s181** | Δ |
|---|---|---|---|
| arith | 2,2 | **2,1** | −0,1 (arith-dq +0,72 sotto rumore: tick/trasversale) |
| prop | 2,4 | **2,3** | −0,1 (prop-dq +2,40/+2,47 5/5 replicata post-pin: trasversale deterministico, NON ripartito) |
| calls | 4,5 | **4,0** | **−0,5 = LEVA** (calls-dq 98,83→90,83 ns/iter, oracle 21,67: 4,56×→4,19×; CIFRA [+8,00;+8,17]) |
| str | 4,1 | 4,1 | = |
| arr | 2,9 | **2,8** | −0,1 (tick) |
| re | 2,5 | 2,6 | +0,1 (run-to-run del denominatore: companion 0,92/0,36) |

## Giudice calls-dq (nuovo, wp181-harness/calls-dq.php, N=60M dal driver)
cr1c di RECORD (finestra pulita): A 98,83 · Z 99,00 · B 90,83 ns/iter; D colonna +8,00 / appaiata +8,17; soglia 4; rumore drop-1 ≤0,67; |A−Z| 0,17; segni 5/5. Repliche a guardia: cr1 [+8,17;+8,50], cr1b [+8,33;+8,50]. Conferma post-pin (pin s181 vs stash s180, stessa toolchain, layout diverso): +8,00 (98,50→90,50), rumore 0,67, 5/5. Attesa [4;12]: centrata.

## WordPress coppia t22 + Doctrine ORM/DBAL (s181-pair.sh / s176-orm-coppia.sh E3/E4)
WP t23 mediana **1,799** = bordo superiore della banda [1,738;1,799] ⇒ giudizio canonico REGRESSIONE SEGNALATA (coppie proprie 1,735/1,783/1,799/1,799/1,809/1,814, banda_ON 0,079 vs 0,030 a t21; media user-only 2,44-2,53 vs 2,42-2,44: finestra più lenta su TUTTE le gambe, macchina in uso con carico 4-5,5; t22 abortita rc=8 per loadavg 7,9) — VOCE, non cifra (s181-pair-verdetto-t23.out, rc=0; t22 abortita rc=8 per loadavg 7,9 alla gamba 3). ORM sul pin s181 FINESTRA CONTAMINATA: leg1 net 6,721 · leg2 8,250 SEGNALATA dal gate ictx (phpr2 5250/s vs 188, user 43,3 s vs 34,9, sys 3,28), sentinella oracle 5,25/5,30 FUORI banda [4,84;5,04] ⇒ nessun claim, regola 4 NON giudicabile; parità ORM 16 · dbal 10 ok; dbal [7,472;7,909] (wp181-harness/s181-orm-coppia-verdetto.out). Attesa era direzione ≤0 sotto-risoluzione. ISTRUTTORIA in apertura di S-182 (criterio t22 p.1: «indagine PRIMA di ogni altra leva»): rerun WP t24 + ORM E3/E4 in finestra NOTTURNA quieta (carico <3, nessuna app utente); se t24 rientra in banda e ORM torna ≤7,05 ⇒ t23/ORM-s181 = contaminazione ambientale dichiarata; se persiste ⇒ reperto CONTRO L-CR1 (attesa era ≤0), istruttoria vera prima di ogni leva.

## Lettura del gap (non cifra)
calls resta il peggiore (4,0×) ma il collo è misurato: ~91 ns per chiamata contro 22, e i corpi Call/Ret pesano ancora ~55-60 ns oltre il dispatch (il Frame da 176 B viaggia per valore: with_buffers → sret → push; il pop al Ret copia di nuovo; gc_note_frame e recycle_frame percorrono i campi). Prossimi bersagli in ordine: (1) scomposizione (a)/(b)/(c) con braccio placebo (cifra per parte: rilievi 2-4); (2) CheckArity eliminato a compile-time per TUTTE le forme di chiamata (ORM: 12,9M op, oggi saltato solo sui fast path); (3) Frame davvero in place (push senza copia, drop in place al Ret) — con disasm bl/sp_refs prima e dopo; (4) str 4,1× = seconda categoria peggiore.
