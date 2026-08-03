# Verbale sedia 2 — Niko Matsakis (Concilio WP-92)

Perimetro: ownership/aliasing/borrow, soundness visitor census, probe discipline.
Fonti: memcensus.rs (A-MS50 blocco theap in `phys_window_dump` r.1063-1209, `peak_census_dump` r.232, `mi_bin_thread_rows` r.1425), worker_pool.rs (ProbeWindow r.74-114, teardown probe r.386-412, ramo `/__census_self` r.518-543).

## VERDETTO: CONCORDO CON EMENDAMENTI

## Q1 — Il reserve_exact pre-visita basta davvero?
Per l'ALIASING sì: guardia `len==capacity` + reserve pre-visita ⇒ push mai riallocante; il `&mut` in `theap_heap_cb` è scopato e cade prima della visita annidata (r.1152); `acc` viene rifruito solo post-visita (r.1171). Sotto Stacked/Tree Borrows regge. Ma la LETTERA ha due denti scoperti:
1. **La guardia confronta `capacity()`, non `MAX_THEAPS`** (r.1147). `reserve_exact` può concedere PIÙ di 64 slot (arrotondamento dell'allocatore): il "declared cap 64" è in realtà capacity-dipendente — `heaps_total` può superare 64 con `heap_overflow=0`. Il cap dichiarato in-band non è il cap effettivo.
2. **`TheapAgg::zeroed()` è un temporaneo di ~3,9 KB (N_BINS=192) costruito per valore sullo stack di un callback `extern "C"`** sotto i frame C di `mi_subproc_visit_heaps`. Non è UB e non è alloc, ma "no-alloc" tace su "stack headroom sotto iterazione C": rustc non garantisce costruzione in-place. Il read-back `RUST_MIN_STACK` esiste (r.1047) ma nessun pin lo lega a questo blocco.
3. **Identità heap = `last_mut()`** (r.1116): corretta SOLO se visita heap e visita aree sono strettamente sequenziali, mai interlacciate — contratto C assunto, mai dichiarato come pin. E il puntatore heap `_h` viene scartato: le righe `mi_theap_pages` non portano `heap=<ptr>`, quindi il join col dente anti-duplicato di `mi_bin_thr_sum` (r.1469) è IMPOSSIBILE — rilevante per la collision VSELF (tema 4).

## Q2 — Il ramo census_self rispetta la disciplina strumento?
Sì nella forma: ProbeWindow + catch_unwind + drop + abort (r.521-535), speculare al teardown (r.386-412). Due buchi:
1. **I due messaggi di abort sono STRINGHE IDENTICHE** ("A-MS31; instrument, not worker"): un abort non è attribuibile al SITO (in-request vs teardown). Forensics di campagna abortita = cieca sul sito.
2. **`req_t0` avvolge anche il ramo census_self** (r.507-552): con `PHPR_REQ_NS=1` un hit a `/__census_self` inietta un campione fasullo in REQ_NS — inquinamento dei per-request ns non dichiarato.
Nota dichiarata altrove (A-BB55): probe Box + BinTab::boxed() allocano sull'heap misurato PRIMA della visita — perturbazione auto-misura già a verbale, non la riapro.

## Q3 — Il secondo sito arm() indebolisce A-MS41?
La cintura A-MS41 conta le grafie `.set(`, non i call-site: NON scatta, e formalmente regge. Ma la semantica si è indebolita: `probe_active=true` non identifica più UN solo strumento; e `arm()` non è nesting-safe (bool, non contatore): un terzo sito futuro annidato farebbe cadere il flag della finestra esterna al Drop interno — falso negativo silenzioso. Oggi irraggiungibile (census_self dentro worker_loop, teardown dopo), ma è invariante di sequenza non presidiato da alcun dente.

## Q4 — Aliasing residuo nelle callback?
Nessuna violazione trovata: provenance di `arg` valida (raw derivato dal `&mut` al call-site, ri-derivazioni disgiunte nel tempo, `drop(probe)` post-visita in mi_bin_thread_rows). Il residuo è di CONTRATTO, non di borrow: sequenzialità C assunta (Q1.3).

## Emendamenti
- **A-MS-51 "cap effettivo = MAX_THEAPS"**: guardia su `len()==MAX_THEAPS` (non capacity) o `heaps: Box<[TheapAgg]>` a lunghezza fissa; il cap dichiarato deve essere quello che morde.
- **A-MS-52 "sito nell'abort"**: tag di sito distinto nei due eprintln di abort e nella riga del panic-hook (arm porta un `&'static str`).
- **A-MS-53 "heap ptr in-band nel theap census"**: emettere `heap=<ptr>` nelle righe `mi_theap_pages` per il join anti-collisione con `mi_bin_thr_sum` (serve a VSELF iter-3).
- **A-MS-54 "arm nesting-tooth"**: `debug_assert!(!flag)` (o contatore) in `arm()` — l'invariante "mai annidato" deve mordere, non essere raccontato.
- **A-MS-55 "REQ_NS esclude census_self"**: nessun campione ns per hit `/__census_self`.

## Kill-switch
- **KS-MS-92-1**: run con `heaps_total>MAX_THEAPS` e `heap_overflow=0` = census VOID (cap dichiarato non mordente).
- **KS-MS-92-2**: abort di probe senza tag di sito nel ledger = attribuzione sito-livello VOID per quella campagna.
- **KS-MS-92-3**: qualunque nuovo sito arm() senza dente anti-nesting committato nella stessa delibera = attribuzione VOID.

## Refutazioni capitali
Nessuna: le cifre S-90.0 sopravvivono. Ma la lettera "declared cap 64" e l'attribuzione per-sito degli abort sono refutate come dichiarazioni (emendate sopra).
