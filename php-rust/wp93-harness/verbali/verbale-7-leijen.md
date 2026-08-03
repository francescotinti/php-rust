# Verbale sedia 7 — Daan Leijen (Concilio WP-93, revisione S-91.0)

Perimetro: mimalloc, semantica contatori, canale misura iterazione 3. Mandato: REFUTARE.

## VERDETTO

La riparazione a macchina (repair90) è onesta: additività 20/20, tether al byte, gradi corretti. Ma **il piano A-DL-52 come scritto in design91.md è già refutato dal canale**: cita `mi_heap_get_default()`, simbolo che il mimalloc v3 di questo tree NON esporta (memcensus.rs r.634-637 e r.1426-1428 lo dichiarano due volte: «not exported by the tree's mimalloc v3»). Stessa classe della lezione WP-90: la lettera del design va calibrata sul canale reale.

## Q1 — heap lazy-init e collisione legittima

Il rischio del worker idle è reale su mimalloc (default heap TL lazy: pre-init tutti i thread puntano al medesimo heap statico vuoto), ma qui è MOOT per costruzione: non avendo `mi_heap_get_default`, l'unica via è quella già collaudata in `mi_bin_thread_rows` — **probe `Box::new` ON-thread + `mi_heap_of(probe)`**. Il probe stesso È il disambiguatore: forza la lazy-init, quindi un thread che ha emesso il probe non è mai "mai-allocante". Il dente A-DL31 va SCOPATO: REFUSE solo per collisione ptr fra thread DISTINTI entrambi post-probe; `heaps_total<W+1` da `mi_subproc_visit_heaps` è DECLARED, non REFUSE.

## Q2 — barriera di picco

Con la sola architettura HTTP round-robin il piano NON è eseguibile: W richieste in volo non garantiscono W worker distinti (accodamento su worker occupato ⇒ deadlock o snapshot parziale). Serve un **Barrier(W) condiviso a due fasi**: (1) rendezvous ARRIVO (tutti i W dentro, nessuno idle, heap congelati dal punto di vista del proprio churn); (2) snapshot on-thread (probe+visit del PROPRIO heap; BinTab e buffer pre-allocati PRIMA della fase 2, righe formattate DOPO — A-MS50 vale anche qui); (3) rendezvous PARTENZA. Timeout fail-closed obbligatorio (riga in-band, snapshot VOID — mai campagna appesa). Visite concorrenti su heap propri: lecite (pagine per-heap), ma il metadato subproc è condiviso — dichiararlo nel header.

## Q3 — massa invisibile

Coerente con la mia analisi WP-92: l'**intercetta** (71,9 MB) è la sede naturale di arena eager-commit/granularità 64KiB, malloc_huge (total vs current) e stato boot main-heap — tutti termini di PROCESSO, non di worker. La **pendenza** 4,48 MB/worker la spiega per prima il **page slack per-theap**: committed−used_b delle pagine parzialmente piene + free_committed delle pagine retained (page_full_retain=2 per size-class PER heap ⇒ scala esattamente con W). È già misurabile dalle righe `mi_theap_pages`/`mi_theap_bin` esistenti: joinare committed−used per heap contro la pendenza invisibile PRIMA di nuova strumentazione. A-DL-55 (current huge, abandoned, chunk_bins) chiude l'intercetta, non la pendenza.

## Q4 — armare A-DL-54

**Doppia richiesta a barriera, pad INVARIATI.** Allungare i pad distrugge l'àncora cal 7.801.102 B (quinta campagna al byte) e confonde durata con massa. Rischi dell'overlap forzato: (a) l'attesa di barriera dentro il bracket gonfia lo span per timing, non per allocazione ⇒ rendezvous FUORI dal bracket del net, span provato da mono_us in-band; (b) serve il gemello serializzato (stessa barriera, rilascio sequenziale) come NO-OVERLAP controllato di pari macchineria. Predizione ex-ante: pd=1000+overlap ⇒ clamped_rows>0 conferma bracket/counter; clamp assente ⇒ legge A-BG61 refutata.

## Emendamenti

- **A-DL-57**: riscrivere A-DL-52 senza `mi_heap_get_default`: probe on-thread + `mi_heap_of`, dente A-DL31 scopato post-probe.
- **A-DL-58**: barrier condiviso a due fasi con timeout fail-closed e thr-set/heap-set cardinalità W asserita in-band.
- **A-DL-59**: join pendenza-invisibile ⇔ Σ(committed−used_b)+free_committed per-theap dai raw esistenti, prima di ogni strumento nuovo.
- **A-DL-60**: A-DL-54 = braccio barriera + gemello serializzato, pad cal-anchored, rendezvous fuori bracket.

## Kill-switch

- **KS-DL-93-1**: riga snapshot per-worker senza probe on-thread in-band ⇒ VOID.
- **KS-DL-93-2**: barriera in timeout o heap-set < W distinti ⇒ split per-worker VOID (mai pubblicazione parziale).
- **KS-DL-93-3**: run A-DL-54 con pad diversi dall'àncora cal ⇒ braccio VOID.

## Refutazioni capitali

**SÌ, due**: (1) A-DL-52 come scritto invoca un simbolo inesistente sul canale; (2) l'architettura a canali attuale non porta W worker alla barriera — serve il Barrier condiviso, non è opzionale.

— Leijen, sedia 7
