# Verbale sedia 2 — Matsakis (Concilio WP-93, revisione S-91.0)

Perimetro: ownership/aliasing, cfg/feature soundness, disciplina probe.

## VERDETTO: PASS CON EMENDAMENTI — nessuna refutazione capitale; una SOVRA-DICHIARAZIONE da emendare (il commento «outstanding==0 here is the proof» in main.rs:263-265 promette più di ciò che una lettura singola Relaxed garantisce).

## Q1 — cfg-split A-PP-63: COERENTE

Evidenza (worker_pool.rs): modulo census su `any(census-instrumentation, mem-census)` (r.184); inc OUTSTANDING + note_outstanding nel dispatch sotto `any(...)` (r.646-656) con QUEUE_DEPTH annidato sotto solo census-instrumentation (r.649-653); dec dopo `response_tx.send` sotto `any(...)` (r.578-588); err-undo sotto `any(...)` con QUEUE_DEPTH interno cfg-ristretto e OUTSTANDING incondizionato (r.673-678). Sotto mem-census ogni inc ha esattamente un dec (post-send) o un undo (send fallito), sullo stesso thread del dispatch nel caso undo (sequenced, niente wrap). Il dec del pickup QUEUE_DEPTH (r.498-508) è correttamente census-instrumentation-only: simmetrico al suo inc. Percorsi senza dec: solo panic worker/probe → abort di processo (A-PP9/A-MS31), il contatore muore col canale — non è un leak osservabile. `note_outstanding` (r.246-254) compila sotto mem-census e panica SOLO su wrap (o==fetch_add+1==0 impossibile in aritmetica vera): stesso contratto fail-closed KH81-2 del canale instrumented, nessun abort NUOVO su run sani. Nota minore: l'undo (r.677) non ha tripwire-su-0 — accettabile perché sequenced-after il proprio inc.

## Q2 — happens-before del testimone: FALSIFICATORE VALIDO fail-closed, garanzia formale INCOMPLETA

`census_outstanding_now` = `load(Relaxed)` sul router (r.290-292); dec = `fetch_sub(Relaxed)` sul worker DOPO il send (r.580). Il curl sequenziale dà hb solo fino al send (oneshot recv sincronizza col send, r.572; il dec è sequenced-AFTER): quindi outstanding può leggere 1 stantio su client genuinamente sequenziale — finestra send→dec già dichiarata (A-TH15, r.204-211) e fail-closed (≠0 ⇒ ADVISORY): direzione giusta. Il teardown Vm/RetainSet completa PRIMA del send (execute_request ritorna prima di r.572), e la sua VISIBILITÀ al router viaggia sulla catena oneshot+TCP, non sul contatore. Falso 0: dentro il protocollo (client unico sequenziale, inflight_max≤1) è escluso dalla catena syscall; FUORI protocollo un load Relaxed può leggere 0 pre-inc di una richiesta concorrente — il testimone NON è machine-proof contro traffico estraneo. Con Relaxed manca l'edge formale dec→load: emendare a Release/Acquire (costo zero, chiude il gap di carta).

## Q3 — sequenza A-MS-51/52+54/53/55: RISPETTATA, UN ordine interno MANCANTE

A-TH-57 è in forma PREFISSO `==2==narm` già in S-91.0 (sigilli v9, WP_SESSION_91 p4) e design91.md r.76-79 dichiara pin narm aggiornato nello STESSO commit del cambio firma: vincolo rispettato. MANCANTE: A-MS-53 (`heap=<ptr>` su mi_theap_pages) è PREREQUISITO del dente REFUSE di A-DL-52 («due thr con lo stesso heap ptr ⇒ REFUSE», design91 r.13-24) — senza ptr in-band il giudice non può rifiutare la collisione; l'ordine non è dichiarato. Inoltre A-MS-55 è confermato NECESSARIO dal codice: oggi `req_t0` campiona ANCHE i hit `/__census_self` (worker_pool.rs r.525 e r.567-571, il branch self non lo salta).

## Q4 — finestra testimone→dump: ESISTE, guardia solo di protocollo

main.rs r.266-273: load outstanding → eprintln mi_quiesce → `peak_census_dump`. Il handler census NON incrementa OUTSTANDING (ritorna prima del dispatch, r.246-279): una richiesta nuova arrivata FRA testimone e dump porta il contatore a 1 senza alzare il watermark oltre il normale — INVISIBILE alla macchina. Oggi garantisce solo il protocollo (curl sequenziale). Serve il bracket: rileggere outstanding DOPO il dump ed emettere `outstanding_post=` sulla stessa riga ckpt; pre==post==0 ⇒ nessun dispatch è entrato nella finestra (con client unico sequenziale un transito completo intra-finestra è escluso).

## Emendamenti

- **A-MS-56**: dec OUTSTANDING → `Release`, load del testimone → `Acquire` (edge formale dec→load; commento main.rs riformulato: «proof» solo con l'edge).
- **A-MS-57**: bracket del testimone — seconda lettura POST-dump, campo `outstanding_post=` in-band sulla riga mi_quiesce.
- **A-MS-58**: dichiarare in design l'ordine A-MS-53 ≺ A-DL-52 (il REFUSE anti-collisione richiede heap ptr in-band).

## Kill-switch

- **KS-MS-93-1**: claim «teardown-complete» da testimone a lettura SINGOLA (senza bracket pre/post) ⇒ il Δ resta ADVISORY.
- **KS-MS-93-2**: dente REFUSE di A-DL-52 esercitato senza `heap=<ptr>` in-band (A-MS-53) ⇒ census VOID.

## Refutazioni capitali: NO.
