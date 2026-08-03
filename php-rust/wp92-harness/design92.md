# design92 — delibere S-92.0 p6/p7 (Concilio WP-93): canale iter-3 SBLOCCATO nel design + residui per NOME

NESSUNA riga di runtime cambia con questo documento; l'attuazione di
ogni item è la prima sessione che ne consuma l'esito. Il canale iter-3
era BLOCCATO da due refutazioni capitali (Leijen WP-93): questo doc è la
riscrittura che le consuma.

## A-DL-57 — riscrittura di A-DL-52 (census ON-THREAD alla barriera)

Refutazione consumata: `mi_heap_get_default()` NON è esportata dal
mimalloc v3 di questo tree (memcensus.rs lo dichiara due volte, r.634-637
e r.1426-1428) — il design A-DL-52 com'era scritto invocava un simbolo
inesistente sul canale. Via reale (già collaudata in `mi_bin_thread_rows`):

- snapshot per-worker eseguito SUL thread del worker: **probe
  `Box::new` ON-thread + `mi_heap_of(probe)`** — il probe stesso È il
  disambiguatore (forza la lazy-init del default heap TL: un thread che
  ha emesso il probe non è mai «mai-allocante», il rischio
  worker-idle-collisione è MOOT per costruzione).
- **Dente A-DL31 SCOPATO post-probe** (Leijen Q1): REFUSE solo per
  collisione di heap ptr fra thread DISTINTI entrambi post-probe;
  `heaps_total < W+1` da `mi_subproc_visit_heaps` è DECLARED, non
  REFUSE. Il dente scopato morde sul proprio harness PRIMA dell'uso
  (legge WP-88); la modifica va per NOME nel commit.
- **KS-DL-93-1**: riga snapshot per-worker senza probe on-thread
  in-band ⇒ VOID.
- **A-MS-58 (ordine dichiarato)**: A-MS-53 (`heap=<ptr>` sulle righe
  mi_theap_pages) ≺ A-DL-57 — il dente REFUSE anti-collisione richiede
  il ptr IN-BAND: senza, il giudice non può rifiutare la collisione
  (KS-MS-93-2: dente REFUSE esercitato senza heap=<ptr> ⇒ census VOID).

## A-DL-58 — Barrier(W) condiviso a due fasi

Refutazione consumata: con la sola architettura HTTP round-robin W
richieste in volo NON garantiscono W worker distinti (accodamento su
worker occupato ⇒ deadlock o snapshot parziale). Serve un Barrier(W)
condiviso:

1. rendezvous ARRIVO (tutti i W dentro; heap congelati dal punto di
   vista del proprio churn);
2. snapshot on-thread (probe + visit del PROPRIO heap; BinTab e buffer
   pre-allocati PRIMA della fase 2, righe formattate DOPO — A-MS50 vale
   anche qui);
3. rendezvous PARTENZA.

Timeout fail-closed OBBLIGATORIO (riga in-band, snapshot VOID — mai
campagna appesa); cardinalità thr-set/heap-set == W ASSERITA in-band;
visite concorrenti su heap propri lecite (pagine per-heap) ma il
metadato subproc è CONDIVISO — dichiarato nel header dello snapshot.
**KS-DL-93-2**: barriera in timeout o heap-set < W distinti ⇒ split
per-worker VOID (mai pubblicazione parziale).

## A-DL-60 — A-DL-54 in forma barriera (negativo controllato del clamp)

- braccio overlap = doppia richiesta A BARRIERA, **pad INVARIATI**
  (allungare i pad distrugge l'àncora cal 7.801.102 B, quinta campagna
  al byte, e confonde durata con massa);
- rendezvous FUORI dal bracket del net (l'attesa di barriera dentro il
  bracket gonfia lo span per timing, non per allocazione); span provato
  da mono_us in-band;
- **gemello serializzato** (stessa barriera, rilascio sequenziale) come
  NO-OVERLAP controllato di pari macchineria;
- predizione ex-ante: pd=1000+overlap ⇒ clamped_rows>0 conferma
  bracket/counter; clamp assente ⇒ legge A-BG61 refutata.
- **KS-DL-93-3**: run A-DL-54 con pad diversi dall'àncora cal ⇒ braccio
  VOID.

## A-DL-59 — join pendenza-invisibile (ESEGUITO in S-92.0, ADVISORY)

Sede: `wp92-harness/dl59-join.sh` + `dl59-join.out` (analisi sui 20 raw
m90 COMMITTATI, zero strumenti nuovi — team-engine: eseguibile subito,
grado ADVISORY; VERDICT-grade solo dal canale barriera A-DL-57/58).
Ipotesi Leijen Q3: pendenza invisibile ≈ page slack per-theap
(Σ(committed−used_b) + free_committed; page_full_retain=2 per
size-class PER heap ⇒ scala con W). Esito: vedi il .out (cifre SOLO da
lì).

## A-MS-55 — req_t0 salta `/__census_self`

Confermato NECESSARIO dal codice (worker_pool r.525 e r.567-571: il
branch self non salta il campione REQ_NS). Attuazione: col canale
iter-3 (stesso commit del probe on-thread), mai in un commit misto.

## Ordine residuo del canale (vincolante, dal team-engine)

A-MS-53 (heap=<ptr>) → A-DL-57 (probe on-thread + dente scopato) →
A-DL-58 (Barrier due fasi) → A-DL-60 (braccio overlap + gemello) →
**solo dopo: measure91-campaign/verdict91 con grammatica ledger v2**
(A-AH63 già ARMATO a HEAD: qualunque riga m91 fuori grammatica v2 =
campagna VOID alla consumazione; A-PP-60/61/62/64 al campaign alla
nascita, `LC_ALL=C` nel prologo battery — igiene Pedersen Q4).

## Promozione b_work — stato aggiornato S-92.0

Condizioni: A-PP-63 ✓ (S-91.0) · stimatore non-tautologico ✓ (repair90)
· **coppia testimone UNIFICATA ✓ (S-92.0 p4: outstanding_pre/post +
arr_pre/post, dec Release/load Acquire)** · census per-worker senza
collisione (A-DL-57) — APERTA. b_work resta ADVISORY finché A-DL-57 non
gira in campagna; il Δ di fase è verdict-grade SOLO con
pre==post==0 ∧ arr_pre==arr_post (KS-PP-93-1).

## Promozione dei doc GitHub a judge=yes (da A-SK-80)

README/COVERAGE/TODO/FOOTPRINT_CPU_ROADMAP/PHPR_DIVERGENCES sono nel
perimetro come esclusioni DICHIARATE (judge=no per NOME, manifest
S-92.0 p1). La promozione a judge=yes richiede: output di
`scripts/measure-coverage.sh` COMMITTATO come fonte corpus (+ glob nel
giudice + budget nello stesso commit, A-SK61) e l'emenda di
gh-status-sync che lo rigenera a ogni sync. Backlog, per NOME.

## Invariati / deferred

A-MS-51/52+54 (design91, invariati) · A-MS46 · A-TH55 · A-BB60/A-PP49 ·
A-MS27/A-PP18/A-PP27/A-AH38 · KS-DS-80-3 (mai innescata).
A-DS51: fase 0 ✓ (S-91.0) · fase 1 in S-92.0 (fixture v4 committate
PRIMA del checker; esenzione ctor keyed sul PROTOTIPO, A-DS59; cinque
regole di formattazione A-DS60 in unit dedicate) · fase 2 = wiring
HOISTED con le esenzioni nello STESSO commit + w7 anti-esenzione per
NOME + gate ORM 3E/13F + hk 1665 + corpus per NOME (KS-DS-92-3,
KS-DS-93-1..3) · fase 3 = bind-registry.
