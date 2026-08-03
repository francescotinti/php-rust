# Verbale sedia 6 — Pedersen (Concilio WP-93, revisione S-91.0)

Perimetro: confine per-richiesta, lifecycle harness, failpath. Mandato: REFUTARE.

## VERDETTO

A-PP-63 è vivo e la sua catena è più solida di quanto il verbale S-91.0
lasciasse temere (Q2 REGGE). Ma il testimone è un **istante, non una
finestra**: la pretesa del commento in main.rs («outstanding==0 here is
the proof that no request teardown can leak into the phase Δ», righe
262-265) è REFUTATA nella sua estensione al dump. Una refutazione
capitale.

## Q1 — Testimone-istante ≠ testimone-finestra: SÌ, serve la coppia

`census_outstanding_now()` è letto a main.rs:266, la riga emessa a
267-272, il dump a 273 — tutto sul task axum del probe, che NON passa
dal pool. Fra load e fine-dump un task axum concorrente può fare
`dispatch` (inc a worker_pool.rs:653) ed eseguire DURANTE il dump.
Niente nel server impone la sequenzialità del driver. Peggio: un
campione post-dump da solo NON basta — una richiesta interamente
contenuta nella finestra riporta outstanding a 0, e sotto mem-census il
contatore monotono `REQUESTS` NON incrementa (fetch_add a riga 944 è
`cfg(census-instrumentation)`; solo la coppia OUTSTANDING è wired,
righe 179-182). `OUTSTANDING_MAX` non discrimina (watermark già 1).
Serve: coppia pre/post + monotono di arrivo compilato sul canale
mem-census.

## Q2 — La catena REGGE: il dec copre il teardown intero

Verificato: `execute_request` (worker_pool.rs:694-706) crea il
RetainSet sul proprio stack; il Vm nasce a 854 dentro
`execute_with_retain` e muore al return; il RetainSet muore al return
di execute_request — ENTRAMBI morti prima di `response_tx.send`
(572), che precede il dec (579-580). Quindi outstanding==0 certifica
request_end + teardown Vm/RetainSet COMPLETI, non solo il send. Il
send è sul oneshot; il body lo scrive hyper via il task axum dopo
`rx.await`: curl-return ⇒ send fatto ma dec forse pendente (finestra
send→dec, ammessa a 1804-1813) — direzione FAIL-CLOSED (spurio ≠0 ⇒
ADVISORY), mai fail-open. UN buco dichiarabile: il `Vec<u8>` del body
attraversa il canale e viene liberato sul task axum DOPO la scrittura
socket — quel free sfugge al testimone.

## Q3 — Trap: nessun orfano diretto, ma l'ABORT è DIFFERITO

`run_gate` esegue i gate in FOREGROUND (battery-91pre.sh:87); bash
DIFFERISCE i trap finché il figlio foreground non termina. Con `kill
-TERM/HUP` al PID dello script: il gate corre fino in fondo, POI
on_signal ledgera — `attempt_epoch` è l'istante del trap, non del
segnale, e il gate «abortito» è in realtà completato. Con INT da
tastiera l'intero process group muore: nessun orfano dello script, ma
eventuali server DETACHED avviati dai gate (setsid/nohup interni)
sopravvivono — la battery in sé non detacha nulla. L'ancora out_sha è
sana (solo lo script scrive OUTF).

## Q4 — Sede measure91-campaign: SUFFICIENTE

`date +%s` è locale-immune; sort/comm/join della battery sono
auto-consistenti (stessa env intra-run, nomi ASCII space-free per
costruzione A-AH52). A-PP-60/61/62/64 restano al campaign alla
nascita; nessuno va armato prima. Igiene gratuita, non bloccante:
`export LC_ALL=C` nel prologo battery.

## Emendamenti

- **A-PP-66**: testimone a COPPIA — seconda riga mi_quiesce POST-dump,
  stesso ckpt, campi outstanding= pre e post.
- **A-PP-67**: contatore monotono di arrivo (inc in dispatch) compilato
  `cfg(any(census-instrumentation, mem-census))`, campo `arr=` su
  entrambe le righe della coppia; uguaglianza pre/post = finestra
  pulita.
- **A-PP-68**: riga ABORT del trap porta `gate_in_flight=<label>` e
  `deferred=1` quando il trap scatta a gate concluso (la semantica
  temporale diventa un campo, non una convenzione).
- **A-PP-69**: free del body fuori-testimone DICHIARATO nel commento
  A-PP-63 (cardinalità: un Vec per richiesta, lato axum).

## Kill-switch

- **KS-PP-93-1**: Δ di fase verdict-grade SOLO con coppia A-PP-66/67
  (pre==post==0 e arr uguali); testimone a riga singola ⇒ ADVISORY.
- **KS-PP-93-2**: riga esito=ABORT senza gate_in_flight/deferred dopo
  l'attuazione di A-PP-68 ⇒ battery VOID.

## Refutazioni capitali

**SÌ, una**: il claim «outstanding==0 alla riga ⇒ nessun teardown può
inquinare il Δ di fase» è falso come garanzia di FINESTRA — vale solo
all'istante del load; il canale mem-census oggi non possiede nemmeno il
monotono per chiudere la finestra. Q2/Q3/Q4: nessuna refutazione
(catena verificata, denti minori).
