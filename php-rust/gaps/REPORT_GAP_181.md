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
IN CORSO alla chiusura del report (catena partita 15:34; esito da pair-out/pair181-t22.done e orm-out). Attesa: direzione ≤0 sotto-risoluzione (quota chiamate utente ad arità esatta non censita in tempo). Riferimenti: WP t21 1,744 (banda [1,738;1,799]); ORM [7,008;7,060] con leg2 a filo di 7,05 (regola 4: istruttoria a t22 se persiste).

## Lettura del gap (non cifra)
calls resta il peggiore (4,0×) ma il collo è misurato: ~91 ns per chiamata contro 22, e i corpi Call/Ret pesano ancora ~55-60 ns oltre il dispatch (il Frame da 176 B viaggia per valore: with_buffers → sret → push; il pop al Ret copia di nuovo; gc_note_frame e recycle_frame percorrono i campi). Prossimi bersagli in ordine: (1) scomposizione (a)/(b)/(c) con braccio placebo (cifra per parte: rilievi 2-4); (2) CheckArity eliminato a compile-time per TUTTE le forme di chiamata (ORM: 12,9M op, oggi saltato solo sui fast path); (3) Frame davvero in place (push senza copia, drop in place al Ret) — con disasm bl/sp_refs prima e dopo; (4) str 4,1× = seconda categoria peggiore.
