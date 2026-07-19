# Rotta WORDPRESS-FIRST — WP-track (dopo WP-24: multisite al minimo teorico + xsl 57/64 + fix engine id-reuse/cast/error-handler)

> 🏁 **WP-24 (2026-07-19, commit gated `d3cf0e3`)**: **multisite RICONFERMATA a 1 SOLO diff per nome
> (minimo teorico: resta solo `wp_is_stream #2`, divergenza DECISA;
> `wpIsIniValueChangeable #4` chiuso da ext/tidy)** su 31.278 test
> (`wp19-harness/ms-out/`, baseline WP-21 archiviata in `wp21-baseline/`).
> **ext/xsl 44→57/64 phpt**: transcodifica iso-8859-1↔UTF-8 (load+saveXML,
> byte-id), `file://` strip + `documentURI` canonica (xmlPathToURI FFI —
> bug53965 byte-id ANCHE con spazi nel path), doc vuoto → xmlNewDoc
> (bug71571), shape trampolino (throw_in_autoload con previous chained,
> TypeError su ritorni non-DOM, registerPHPFunctions validazioni 8.4).
> 🔧 ENGINE: `set_error_handler(null)` = default handler; `(string)` di
> Closure/Generator LANCIA (Op::Stringify; funnel echo resta D-19.18);
> **Drop di Object POSTORDER** (riuso LIFO = id del padre, come Zend) +
> **`next_id` ripulisce TUTTE le tabelle per-id** (⭐ un Fiber/generator
> morto non deve rivivere sul riuso dell'id — fibers/destructors_002 era il
> sintomo). Corpus 1487→**1485** (bug60738, closure_015 chiusi).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- **Full-suite single-site: 1 diff per nome — minimo teorico** (run16,
  `wp16-harness/full-out/run16/`; solo `wp_is_stream #2`). Non rilanciata in
  WP-24 (i cambi sono xsl/dom/engine-cosmetici, coperti da gate22 + probe);
  al prossimo cambio sostanzioso rilanciarla e attendersi SOLO il diff
  dichiarato.
- **Full-suite multisite: 1 diff per nome — minimo teorico** (WP-24,
  `wp19-harness/ms-out/`, 31.278 test; solo `wp_is_stream #2`).
- Suite phpt estensioni (misura, non gate): **xsl 57/64** (residui 7, tutti
  strutturali: bug49634 trace con frame prelude · bug69168 identity/liveness
  nodi = divergenza dichiarata · registerPHPFunctionNS (functionURI FFI) ·
  xsl-phpinfo · xslt008/-mb/009 stream-wrapper dentro l'I/O libxml) ·
  **tidy 44/45** (010: id-reuse nei grafi trattenuti in gc_roots — lo sweep
  rilascia in ordine di nota, non di cascata; il Drop postorder ha chiuso la
  parte in cascata).
  ⚠️ Misurare le suite phpt con INVOCAZIONE ASSOLUTA del path: con path
  relativi `__DIR__` diventa "." nel runner e i test file://+__DIR__
  falliscono per artefatto (xslt007/bug53965 passano SOLO con path assoluto).

## Harness full-suite (WP-16 — invariato, ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec { $ARGV[0] } @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# monitorare full-out/full-phpr.rss (APPEND tra run: usare tail); attendere full-phpr.done
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/)
# ⚠️ MAI probe su wptests durante una run (WP-20 docet).
# ⚠️⚠️ AZZERARE wpdev/src/wp-content/uploads PRIMA di ogni full run (WP-21).
# ⚠️ NON ricompilare mentre una run/gate usa il binario (le fasi successive
#   spawnerebbero il binario nuovo → run mista) e NON lanciare un secondo
#   gate22 finché il primo non ha scritto gate22.done: due istanze
#   interleavate mescolano progress.txt/output e il marker done del primo
#   sveglia i watcher del secondo (successo in WP-24).
```

## Prossimo passo: SESSIONE WP-25
1. **CPU residua strutturale** (full-suite CPU master 16:45 vs oracle 8:50):
   dal profilo WP-23 (`wp22-harness/prof-out/`), in ordine: run_loop leaf
   ~2600 (dispatch) · memmove ~600 (concat/enter_callee) · Zval drop/clone
   ~500 · gc note/sweep ~500 · dispatch_instance_call ~200 · identical ~180
   · bind_params ~150. Candidati: interning stringhe/nomi; frame setup più
   magro (ROI incerto, valutata WP-23); memoria dati vivi (arena, backlog
   WP-7). ⚠️ METODO A/B: SOLO run interleaved nello stesso momento; pkill
   rust-analyzer; ⭐ attenzione all'INLINING (fast-path piccolo #[inline],
   path grande out-of-line — lezione Props::position_ro).
2. **Residui strutturali xsl/tidy** (se si vogliono chiudere le suite phpt):
   (a) stream wrapper PHP dentro l'I/O libxml (xmlRegisterInputCallbacks →
   bridge verso gli stream phpr; chiude xslt008/-mb/009 e apre document()
   su wrapper arbitrari); (b) registerPHPFunctionNS: leggere function/
   functionURI dal xmlXPathParserContext via offset ABI e mappa (ns,name)→
   callable; (c) trace senza frame prelude (bug49634 — nasconderli in
   backtrace/getTrace quando file=="prelude", impatto largo, gate pesante);
   (d) sweep release-order per tidy 010 (rischioso: cambia l'ordine dei
   distruttori — valutare bene).
3. **Roadmap post-WP** da [[php-rust-roadmap-wp-first]]: validazione Laravel,
   oppure pescare dai residui trasversali di [[php-rust-todo-master]].
4. Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (355→351
   baseline). Se si tocca ref/arg/reflection: gate ORM+hk obbligatorio.

## Lezioni operative (nuove WP-24)
- ⭐⭐ **Riuso degli object-id**: `next_id` DEVE ripulire OGNI tabella VM
  keyed per id (fibers, generators, gc marks, cache transienti) — il Drop
  postorder ha cambiato QUALI id si riusano e ha esposto lo stato stale dei
  Fiber ("Cannot start a fiber that has already been started"). Quando si
  aggiunge una nuova mappa per-id, aggiungerla anche lì.
- ⭐ **Suite phpt con path ASSOLUTO** (v. sopra): `__DIR__`="." è un
  artefatto del runner con invocazione relativa, non una divergenza phpr.
- ⭐ **xmlPathToURI/xmlCanonicPath via FFI** per documentURI/base: l'oracle
  non tiene mai lo scheme file:// nella documentURI e %-escapa gli spazi;
  con la base canonica gli xsl:include relativi risolvono anche sotto
  "/Volumes/Extreme Pro" (spazio nel path).
- ⭐ **ErrCapture (open_memstream) perde i confini di chiamata** di libxslt:
  il report di ricorsione è UNA chiamata con \n interno → ricucire la riga
  "You can adjust …" al messaggio precedente (bug71571). Pattern per altri
  messaggi multi-riga futuri.
- ⭐ **Serena timeout ≠ edit fallito**: dopo un TimeoutError di
  replace_content verificare lo stato del file PRIMA di riprovare (il primo
  "timeout" di WP-24 aveva GIÀ applicato l'edit).
- Un DOMDocument mai caricato ha serializzazione "solo dichiarazione" —
  round-trip verso libxslt = xmlNewDoc, non xmlReadMemory (che rifiuta
  input senza root).

## Invarianti (aggiornati WP-24)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline:
  **corpus 1485 (AGGIORNATA WP-24) · sess 28 · date 351 · refl 290**
  (SOLO rimozioni ammesse; fail-set in `wp18-harness/gate-out/*.fails`) ·
  ORM 3484 3E/13F per nome · http-kernel 1665 0E/0F · cargo (1556) ·
  probe: gd 11/11, mysqli 11/11, media-probe byte-id, run-http (DIFF-set
  16 = WP-14) · WP suite per-classe = oracle (option 413 · media 762 ·
  post 906 · user 1341 · query 1889 · restapi 3514 · taxonomy 878 ·
  comment 582 · xmlrpc 316 · sitemaps 132 · classi WP-17/18). Script:
  `wp22-harness/gate22.sh` (gate-out WP-23 in gate-out-wp23-archived).
- Full-suite single-site: solo miglioramenti per nome vs **run16 (1 diff:
  wp_is_stream #2)**. Full-suite multisite: solo miglioramenti vs **ms-out
  WP-24 (1 diff: wp_is_stream #2)** — entrambe al minimo teorico finché la
  divergenza resta DECISA.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust (se in timeout: perl -ne via Bash,
  ma gli EDIT solo via Serena); Vexp/Read per il C; Read/Write tool per i
  .php; log `tr -d '\0'`; probe MAI su wptests durante una run; uploads
  AZZERATI prima di ogni full run.
