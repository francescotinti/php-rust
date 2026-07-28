# WP_SESSION_69 — debiti d'apertura del concilio CHIUSI (🔴 S-69.1 cascata di demozione + famiglia redeclare oracle-pinned; KH69-2 SCATTATO e chiuso) + L-68.1 forma doppia (KL69-1 SCATTA: residuo VERO ~2 KiB/req) + confine cron ISOLATO + spike ipotecato (predictions69 lockate, profilo a WP-70 su outlined)

> ⚡ **WP-69 (2026-07-28 sera, `f663765`→`76adc8c`/`07ef494`)** —
> sintesi a 9 recepita INTEGRALE in `wp69-harness/design69.md` PRIMA
> del codice; predictions69 lockate (b91b4799…) PRIMA di profilo e
> letti L-68.1. GATE69 PASS fails=0 al 2° run (run1 FAIL(1) onesto ⇒
> ri-pin sentinelle65/redecl con PROVENANCE). Stash **phpr-wp69
> (c5b1e80b…)**; census phpr-memgc69 (f034e62a).

## 🔴 S-69.1 (bloccante Stogov) — CHIUSO, fix DEFER-SIDE

s69-deep riprodotto in apertura (phpr `Failed to compile` dove Zend
esegue). Due strati: (1) **cascata di demozione assente** — fix
`demote_unbindable_chains` a FIXPOINT in `hoist_classes` (Zend
early-binda solo catene INTERAMENTE bindabili; un hoisted il cui
supertype risolto è demosso cade anch'esso al DECLARE runtime, dove
l'autoload fira in ordine Zend, shadowing incluso — zero autoload nel
lowering, KS-S69.1 ✓); (2) `Child::WHO` diventa fetch runtime (la
cascata rimuove Child da class_index). **Scoperta collaterale:
famiglia redeclare (4 forme pre-esistenti)** — dup silenziosamente
skippati (guardia per NOME), unit-redeclare-di-seed = Failed to
compile, messaggio/tipo errore non-Zend, FatalAt senza trace 8.5.
Fix: riserva first-wins PER-STATEMENT (`hoisted_class_spans`, no-op
per SPAN), `class_redeclaration_fatal` (nome CANONICO del VECCHIO —
errmsg_026 "stdClass"; sito del NUOVO), `FatalAt.trace` +
`fatal_error_backtraces` (ini nuova default 1; runner default Off
come run-tests.php:275), escaping html fedele (thrown msg+trace
ESCAPATI, backtrace fatal RAW — oracle-pinned).
**Batteria gate-ordering-s69 PASS (0✗)**: 9 unit-fixture a 2 LATI
(hit-cross asserito; s69-tuse è PURA — classe con parent
irrisolvibile ⇒ trait-use nel deferred, ipotesi concilio sbagliata
di un livello) + 6 redecl. Corpus: **−2 fail REALI chiusi**
(declare_already_in_use, no_early_binding_if_already_declared),
**+3 ex-SKIP resi runnable** dall'ini nuova (gap eval-naming
PRE-esistente: frame "()" vs "eval()", location eval'd-code non
decorata, SensitiveParameter — anche su thrown; → backlog) ⇒
**ri-pin corpus 1421→1422 con PROVENANCE**.

## Debiti puntuali

- **M-69.1/H-69.4**: `pending_unit_call` ELIMINATO — `unit_call`
  parametro di `run_linked` (staleness su FatalAt impossibile per
  costruzione). **M-69.2**: CENSUS_DEAD_TOTAL avanza solo a prune
  eseguito. **M-69.4**: parentetica FrameExt.
- **H-69.1/KH69-2 SCATTATO**: ciclo `$a[0]=&$a; $b=&$a[0]` panicava
  DAL VIVO (BorrowMutError in make_cell sotto il RefMut di
  elem_cell). **Fix H-69.2** forma (b): try_borrow_mut con semantica
  dichiarata (mid-write ⇒ promozione moot; Undef→Null preservato).
  h69-cycle BYTE-ID, fixture nel gate.
- **K-69.4**: test cargo `conditional_dup_fold_equals_mint` (cargo
  **1649/0**). **H-69.3**: assert fold==mint CONTATO al link (path
  compile v2, FATAL in ogni build). **K-69.1**: sanatorie a verbale
  (design69 §1.6). **K-69.3**: contatore meccanico corpi caldi
  consegnato (count-handler-bodies.sh, baseline PINNATA **178**).

## L-68.1 forma doppia (memgc69, 3 leg N=1000) — KL69-1 SCATTA

ON 62,610 KiB/req (= B-68.3 alla cifra) con counter censusown 56,79
(90,7% attribuito ⇒ counter RICONCILIATORE, non giudice — KL69-2 non
scatta); nk OFF **0,0000** (L-69.P2 DENTRO ESATTO — engine ≈0 sul
fixture, Leijen); OFF wpdev 181 finestra-INVALIDA (gradino cron
+158 MiB DENTRO la finestra — L-69.2 applicata a noi stessi).
**Discriminatore = run cron-free + HITS-off (probe cron): +2,235
pre / +2,021 post KiB/req ⇒ 🔴 KL69-1 SCATTA — sotto i ~60 KiB/req
di census-own c'è un residuo VERO ≈2 KiB/req per-request su wpdev**
(62,6 = ~60,5 census-own + ~2,1 reale). **Fronte axum FERMO** finché
attribuito (protocollo pulito ora disponibile). Lezioni protocollo:
leg L-68.1 SEMPRE con DISABLE_WP_CRON + UNIT_CACHE_LOG acceso.

## B-69.5 confine cron — ISOLATO (FAIL(1) informativo)

DISABLE_WP_CRON isola PERFETTAMENTE: vivi steady **528** (un
contesto, [500,540] ✓), zero segmenti spontanei, Δconfine +498
una-tantum ∈ [480,540] ✓, coda PIATTA, body byte-id, ways Δsteady=0
(KB69-3 ok — ma 8 evizioni AL confine). **P69-C-b FUORI con firma**:
4 re-miss front post-confine TUTTI `miss fp` su version.php,
una-tantum R501..R504 [⚠️ rettifica WP-70/B-70.2: TUTTI in R501] — input per B-67.4/ways. **B-69.2 audit fp a
codice**: la chiave È context-sensitive PER COSTRUZIONE
(unit_chain_fp seminato con l'identità del MAIN — vm/mod.rs ~495);
il secondo set cron è costo strutturale del salt, non un bug;
opzione digest-dello-stato-seed per il concilio.

## Spike dispatch — in esecuzione CONFORME (KG69-2)

predictions69 lockate PRIMA (bande P69-S-a/b, tabella-decisione
ex-ante, MDE 0,52% IR) + K-69.3 consegnato. Profilo NON eseguito per
vincolo B-69.4: il binario outlined-per-corpi NON esiste (WP-65
outlinava solo helper) — costruirlo tocca il loop caldo e non si
improvvisa. **WP-70 apre con build outlined feature-gated validata
in coppia → profilo → decisione da tabella.** Né richiuso né aperto
oltre mandato.

## ⭐ Lezioni

- ⭐⭐ **La demozione è una proprietà di CATENA, non di nodo**: Zend
  early-binda solo catene interamente bindabili — ogni demozione
  deve propagarsi a fixpoint o il figlio resta legato a un
  placeholder.
- ⭐⭐ **L'identità di uno statement è lo SPAN, mai il nome**: la
  guardia per nome del main pass mascherava TUTTA la famiglia
  redeclare (dup silenziosi, seed-redeclare, messaggi non-Zend).
- ⭐⭐ **Un kill-switch che scatta su una run PULITA vale doppio**:
  KL69-1 a 2,0-2,2 su protocollo cron-free+HITS-off ha trovato il
  residuo vero che 60 KiB/req di rumore census nascondevano.
- ⭐⭐ **La finestra di regime va dichiarata rispetto agli EVENTI del
  workload** (L-69.2): il gradino cron dentro la finestra ha reso
  181 una pendenza-fantasma; la stessa banda su finestra pulita vale
  2,0.
- ⭐ Un ini mancante è una maschera: aggiungere
  `fatal_error_backtraces` ha reso runnable 3 phpt ex-skip — i gap
  che espongono erano lì da sempre.
- ⭐ La falsificazione di Hoare (H-69.1) era giusta: il borrow_mut
  "sicuro per adiacenza" panicava al primo ciclo di riferimenti.

## Parità e stash

Release **phpr-wp69 (c5b1e80b…, tree 76adc8c/07ef494)**, stash
ADDITIVO accanto a wp68 (8eea807e…) — 25° in archivio. Census
phpr-memgc69 (f034e62a). GATE69 PASS fails=0 run2 (run1 FAIL(1)
onesto → ri-pin sentinelle65/redecl `base.php:174` con PROVENANCE:
la 2ª include del fixture ora COMPILA per effetto del fix). Delta
engine vs wp68: demote_unbindable_chains + hoisted_class_spans +
class_redeclaration_fatal + FatalAt.trace/ini backtraces + esc_html
+ make_cell try_borrow_mut + run_linked(unit_call) + fold==mint
assert + CENSUS_DEAD_TOTAL nel ramo eseguito + PHPR_CENSUS_HITS +
tag=censusown.

## Prossimo (WP-70) — vedi NEXT_SESSION §WP-70
