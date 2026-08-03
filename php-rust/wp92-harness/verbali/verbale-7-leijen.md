# Verbale sedia 7 — Daan Leijen (mimalloc, footprint fisico, semantica contatori)

VERDETTO: **CONCORDO CON EMENDAMENTI**

## Q1 — Collisione mi_heap_of (VSELF REFUSED 20/20)

Meccanica, dai raw: `mi_bin_thr_sum` riporta per thr=0..15 lo **stesso**
`heap=0x105c3ae80` (w16.r1) e `0x10562ee80` (w4.r1) — indirizzo diverso
per RUN, non per thread: è un heap **statico nell'immagine** (ASLR sposta
la base), cioè `_mi_heap_main`. Su mimalloc v3 `mi_heap_get_default()` è
**TLS-only**: chiamata dal thread del census (o via wrapper cross-thread)
restituisce W volte il main-heap del chiamante. Prova interna: gli
`used_b` per-thr jitterano ±20 KB **non monotoni** (246.220.912 …
246.241.984) = un solo heap vivo letto in sequenza mentre i worker
mutano. La stat `heaps.total=1` vs `theaps.total=26` (w16 peak) dice che
i 26 thread-heap ESISTONO ma non sono enumerabili dall'API pubblica.
Non è "pagine condivise": è il wrapper che legge la TLS sbagliata.

**Iter-3 (in ordine):** (a) **census ON-THREAD**: alla barriera di picco
ogni worker chiama `mi_heap_get_default()+mi_heap_visit_blocks` sul
PROPRIO thread e spedisce lo snapshot su canale — zero API nuove;
(b) heap **espliciti** per worker (`mi_heap_new` + allocazioni VM
heap-scoped) → puntatori distinti e stats per-heap; (c) censimento
at-exit per thread (merge stats tld — richiede patch al crate).

## Q2 — P-CLAMP-PD refutata: la lettura "bracket/counter" regge, ma va scopata

pd=1000 **non** rende il run purge-free: dt1.afirst.pd1000 ha
`purge_calls=274`, `purged=85.721.088` già a exit_mi (mono_ms=1064 > pd)
e 127.467.520 dopo collect. Difende solo la FINESTRA: lo span clamped è
[6.059, 20.236] µs — nessuna pagina matura il deadline 1000 ms
in-bracket. Il discriminatore nei raw è **OVERLAP⇔clamp: 6/6 righe
clamped (pd0+pd1000) hanno spans=OVERLAP; 4/4 NO-OVERLAP hanno clamp
zero**; la coppia dt5.pd1000 (afirst NO-OVERLAP clamped=0 vs bfirst
OVERLAP clamped=1, stessi dt e pd) è l'esperimento naturale. E nel
clamped dt1.pd1000: `df=89.955.518 > da=86.862.318` eppure net=0 — il
bracket su contatori **di processo** sottrae il traffico concorrente
dell'altro lato (KB-88-1). Colpevole = finestra process-wide + overlap.
Debolezza: purged≡0 in-window è ARITMETICA, non lettura (A-DL50
unavailable); e 3/4 vs 4/4 è condizionato dallo scheduler.

## Q3 — b_boot 2.252.800 B (550 pagine ×4096, se=6.345 B)

Il contatore è il **commit mimalloc** (mi stats): gli stack pthread
(mmap di sistema) NON ci sono dentro ⇒ l'ipotesi "stack 2 MiB/thread" è
refutata dalla semantica del contatore, non serve misurarla.
Composizione candidata: bootstrap theap per-worker + stato phpr
per-worker (unit-cache TL, scaffolding VM). Intercetta a=12.386.304 =
runtime condiviso (boot W=4: threads=15, theaps=14). Discriminatore:
braccio **bare-thread** (W thread che toccano mimalloc, zero init phpr)
→ b_boot_bare; la differenza è lo stato phpr.

## Q4 — Coverage 0,576: dov'è il 42%

A w16.r1 peak il visit copre 253.420.432 su commit 399.638.528 (0,634).
L'invisibile, per NOME dai raw: (1) **malloc_huge**: 96 chiamate, total
638.582.784 — le huge non stanno nelle pagine-bin di alcun heap, il
visit non le vede (e il census logga il TOTAL cumulativo, serve il
CURRENT); (2) **pages_abandoned=2481** — pagine senza heap visitabile;
(3) free-committed in arena in attesa di purge (purged cumulativo
844.627.968 al picco); (4) pagine degli altri 25 theap. L'arena si
cammina **senza TLS**: estendere `mi_arena_chunk_bins` con classe
per-chunk (huge/abandoned/free-committed/heap-owned) chiude il bilancio.

## Emendamenti

- **A-DL-52 "census on-thread"**: snapshot per-worker eseguito SUL thread
  del worker alla barriera di picco; due thr con lo stesso heap ptr ⇒
  REFUSE (A-DL31 resta il dente).
- **A-DL-53 "purged ai bordi dello span"**: lettura read-only
  purged/purge_calls a t0/t1 dello span sul worker path (chiude A-DL50).
- **A-DL-54 "overlap forzato"**: braccio clamp con doppia richiesta a
  barriera — l'overlap diventa controllato, non emergente.
- **A-DL-55 "huge/abandoned nel census"**: current (non total) di
  malloc_huge + conteggio pagine abandoned + classi per-chunk in
  mi_arena_chunk_bins.
- **A-DL-56 "bare-thread arm"**: scomposizione b_boot = thread-runtime +
  stato phpr.

## Kill-switch

- **KS-DL-92-1**: census per-worker con heap ptr duplicato ⇒ REFUSE, mai
  media.
- **KS-DL-92-2**: qualunque claim sul clamp senza riga overlap-status
  per span ⇒ VOID.

Refutazioni capitali: no. La refutazione P-CLAMP-PD regge (ri-scopata:
vale per la finestra, non per il run); l'ipotesi stack su b_boot cade
per semantica del contatore.
