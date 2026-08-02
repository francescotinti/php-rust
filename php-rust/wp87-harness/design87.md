# design87 — A-DL39 (split heap per-worker, SOLO DESIGN) + A-BB50 (net window per-thread, SOLO DESIGN)

Ordine Concilio WP-88 §Sintesi p5-p6. NESSUNA riga di runtime cambia con
questo documento: entrambi i design toccano la superficie di conteggio
(A-DL39 l'allocatore, A-BB50 i contatori) e il merge è materia di delibera
di Concilio (A-DL39) o di sessione dedicata con bump ALLOC_ID (A-BB50).

## A-DL39 — split per-worker heap (via `mi_heap_new`)

**Problema (dai fatti S-86.0/WP-88)**: il default heap di mimalloc v3 è
CONDIVISO fra i thread (dente `heap=`: puntatore identico sui worker;
Leijen WP-88 Q1: le theap di default appartengono tutte a `heap_main`,
init.c:181-183/332-333). Ogni split per-thread via heap-visit sul default
heap è VOID (KL-88-1).

**Unica via pulita (Leijen Q1, tre opzioni valutate)**:
(a) `mi_heap_stats_get` — esiste ma opera sul heap condiviso: INUTILE.
(b) esporre `theap->stats` — richiede PATCH del vendored (types.h:535 non
    esportata): superficie vendored, da evitare finché esiste (c).
(c) **per-worker `mi_heap_new()` + `mi_heap_theap()` +
    `mi_theap_set_default()`** (mimalloc.h:373-376) nel prologo di ogni
    worker thread (worker_pool.rs, prima del loop): ogni worker alloca su
    un heap DISTINTO; `mi_heap_visit_blocks`/stats per-heap splittano
    DAVVERO; i free cross-thread finiscono in abandoned (visitabile:
    `mi_heap_visit_abandoned_blocks` già in memcensus.rs).

**Costi/precondizioni (per NOME)**:
1. Cambia la superficie di conteggio ⇒ **bump ALLOC_ID** (A-AH33) e
   **ricalibrazione NET_H/NET_P** (le calibrazioni correnti valgono per
   memcount-v2-s82 sul heap condiviso).
2. Il main thread resta su heap_main: le righe census distinguono
   worker-heap vs main-heap per costruzione (il dente `heap=` diventa il
   positivo: puntatori DIVERSI attesi).
3. Fallback teardown: un worker che muore senza `mi_heap_delete` orfana
   pagine → protocollo teardown da definire (delete dopo il probe, mai
   prima delle righe census).
4. Gate: battery con positive-control (due worker ⇒ due heap pointer
   distinti in-band) + il churn gate (hello 2 call) INVARIATO — lo split
   non deve toccare il fast-path.
**Delibera richiesta**: Concilio WP-89 (tocca l'allocatore; KS-DS-80-3 e
la policy no-revert impongono il design-first). Nessun merge in S-87.0.

## A-BB50 — net window PER-THREAD (contatori thread-local)

**Problema (A-BB49, ri-giudizio S-87.0 p1)**: `net_window=process-counters`
sotto span intersecanti inghiotte il lowering altrui INTERO (firma
pad_net = NET_H+NET_P+150 ESATTA 10/10 nei raw m86.ovl; KB-88-1: quei net
sono VOID come cifre per-thread). Il ×W resta sequenziale-only finché la
finestra è di processo.

**Design**: `alloc_counters()` (php-types/memcensus) affianca ai contatori
globali una coppia THREAD-LOCAL (`Cell<u64>` alloc/free per thread,
aggiornata negli stessi hook dell'allocatore dove oggi si fa fetch_add).
`CensusNetWindow` v2 legge la coppia del PROPRIO thread: il bracket
net-at-lower diventa per-thread per costruzione; gli span possono
intersecare senza inghiottimento.

**Costi/precondizioni (per NOME)**:
1. Hook allocatore: oggi il conteggio vive nei wrapper mi_* globali
   (fetch_add su atomics); la coppia TL aggiunge un incremento per evento
   ⇒ **bump ALLOC_ID** e ricalibrazione NET_H/NET_P (il costo del hook
   cambia il net osservato di se stesso).
2. Free cross-thread: un blocco allocato dal thread A e liberato dal
   thread B decrementa la finestra di B, non di A ⇒ semantica DICHIARATA
   (stessa classe della deflazione A-DL34): "net per-thread = alloc del
   thread − free ESEGUITI dal thread", con flag clamped già in essere
   (A-DL36).
3. Predizione di collaudo (A-BB53, fixture già committate): con finestra
   per-thread, conc r* deve dare net=cal AL BYTE per lato (pad87a/b
   disgiunte); con process-window la firma è netX≈calA+calB (osservata in
   questa campagna). La coppia fixture è il positivo/negativo del design.
**Precondizione di ogni canary concorrente futuro** (Concilio WP-88 p5):
nessun claim per-thread sotto concorrenza prima di questo design attuato.
