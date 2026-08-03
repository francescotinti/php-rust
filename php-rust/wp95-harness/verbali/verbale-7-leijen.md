# Verbale sedia 7 — Leijen (allocatore mimalloc v3, footprint fisico) — Concilio WP-95

**VERDETTO**: la conclusione S-93.0 «la stat è di fatto cumulativa» è CONFERMATA, e ora ha il
meccanismo per NOME al livello di preprocessore. Ma DUE frasi agli atti sono refutate: la
nota-canale di huge-worker.out e l'inferenza «committed piatta ⇒ mi_collect non decommitta».

## Q1 — Perché il decremento non morde

Nessuna delle ipotesi elencate (theap NULL, pagina abandoned, theap diverso). Il call-site È
raggiunto: il free della huge singleton passa da `mi_free_ex` (free.c:186-204) →
`mi_free_block_local` con `track_stats=true` → chiamata a `mi_stat_free` (free.c:34). È il
CHIAMATO a essere vuoto: `mi_stat_free` è sotto `#if (MI_STAT>0)` (free.c:612); lo stub
release è free.c:635-637. E MI_STAT è 0 in OGNI build census: build.rs di libmimalloc-sys
0.1.49 definisce sempre `MI_DEBUG=0` senza feature `debug` (build.rs:108-115) ⇒ types.h:81-87
⇒ `MI_STAT 0` (types.h:85). L'INCREASE invece non è gated: `mi_theap_stat_increase(theap,
malloc_huge, …)` + `malloc_huge_count` girano incondizionati alla nascita della pagina
(page.c:935-936; macro internal.h:392 senza guardia). Asimmetria per costruzione: in release
malloc_huge conta le NASCITE di pagine huge — cumulativo, `current==total==peak` per sempre.
Asimmetria latente n.2 (per quando MI_STAT>0): il decrease va su `_mi_theap_default()` del
thread che libera (free.c:615), non sul theap che incrementò — un Σ per-theap può andare
negativo su free cross-thread.

## Q2 — Perché arena committed non scende

Semantica del contatore, non retention provata. In release `_mi_prim_decommit` su macOS fa
`madvise(MADV_FREE_REUSABLE)` ma pone `*needs_recommit=false` (prim.c:495-504, gate
`!MI_DEBUG && MI_SECURE<=2` a prim.c:503); `mi_os_decommit_ex` decrementa `committed` SOLO se
`needs_recommit` (os.c:590-591). Quindi committed non scende MAI su purge in release, per
disegno (la memoria resta riusabile senza recommit; il fisico torna all'OS via REUSABLE, con
accounting rss immediato — commento prim.c:496). Implicazione per lo slope 18814309 B/worker:
«committed piatta» NON lo attribuisce, e i sei huge (liberati e purgabili) NON possono
comporlo salvo prova fisica contraria; l'attribuzione è legittima solo sul canale fisico
on-thread (A-DL-55). La refutazione di LEVER-2 resta valida, ma sul SOLO braccio A/B fisico
(delta 0,12%), non sulla stat.

## Q3 — Strumento in-band per m91

(a) build census con `MI_STAT=1` esplicito (via CFLAGS del cc-crate; identità dichiarata nel
banner del dump insieme al livello MI_STAT); (b) coppia alloc/free IN-BAND nel GlobalAlloc
Rust (soglia ≥524288, chiave thread/theap) — è il canale che ha deciso S-93.0 in un colpo ed
è immune al gating C; (c) mai consumare una stat MI_STAT-gated senza dichiararne il livello.

## Q4 — Leva per-file, vista allocatore

Via libera: chunk ≤512 KiB evitano il path huge singleton (page.c:916-939) e il waste
os-good-size (39911424−39423200=488224 B sui sei chunk); le taglie normali finiscono nelle
page-queue e si riusano tra file. Frammentazione: rischio trascurabile sulle slice da 64 KiB;
il guadagno vero è eliminare lo spike touched 39,5 MB/thread e il 4,42× CLI.

## Q5 — Priorità S-94.0

1. Sanare il canale stat (A-DL-65/66/67) PRIMA della campagna m91; 2. leva per-file del
preludio con gate parità completi; 3. attribuzione slope solo su probe fisico on-thread.

## Emendamenti

- **A-DL-65**: census m91 = MI_STAT=1 dichiarato + coppia in-band Rust ≥524288 B; ogni cifra
  da stat gated senza livello MI_STAT nel banner è invalida.
- **A-DL-66**: vietato inferire retention/decommit da `committed` (os.c:591 ⊣ prim.c:504);
  retention si afferma solo dal probe fisico on-thread.
- **A-DL-67**: emendare huge-worker.out (nota-canale righe 6-8) e NEXT_SESSION §B con marcatura
  REFUTATA + puntatori file:riga.
- **A-DL-68**: leva per-file approvata dal perimetro allocatore (nessuna controindicazione di
  frammentazione; elimina il path huge).

## Kill-switch

- **KS-DL-95-1**: numero MI_STAT-gated in un verdetto senza livello MI_STAT dichiarato ⇒ il
  verdetto decade ad ADVISORY.
- **KS-DL-95-2**: divergenza coppia in-band vs stat mimalloc nel census ⇒ fede alla coppia
  in-band, stat solo testimone.

## Refutazioni capitali: SÌ (due)

1. huge-worker.out righe 6-8 («il decremento ESISTE … non è un contatore monco: è assenza di
   free») — FALSA: il decremento di free.c:631 è compilato VIA a MI_STAT=0, e il decremento di
   theap.c:362 vive nel blocco commentato `/*` theap.c:338 … `*/` theap.c:448 (dead code
   anche in debug). Il contatore È monco per costruzione in release.
2. «arena committed invariata a exit_collect_mi ⇒ mi_collect non decommitta quelle pagine» —
   NON SEGUE (prim.c:504 + os.c:590-591): committed non scende mai su purge in release;
   LEVER-2 resta refutata solo dalla misura fisica A/B.
