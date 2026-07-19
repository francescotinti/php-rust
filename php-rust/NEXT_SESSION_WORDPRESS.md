# Rotta WORDPRESS-FIRST — WP-track (dopo WP-23: ext/tidy + xsl trampolino + fix hook-guard)

> 🏁 **WP-23 (2026-07-19, commit gated `90d7ae0`)**: **ext/tidy NATIVA su
> libtidy 5.8.0 di sistema via FFI** (44/45 phpt runnable; ini
> tidy.clean_output PHP_INI_USER → chiude `wpIsIniValueChangeable #4`) +
> **trampolino reale registerPHPFunctions/php:function** per ext/xsl
> (21→44/64 phpt; re-entry ACTIVE_VM, ritorni DOM come nodeset veri via doc
> temporanei) + 🔧 FIX ENGINE hook-guard (un hook che LANCIA non rilasciava
> il guard di ricorsione → hook bypassato in silenzio per sempre; chiude
> anche property_hooks/parent_superfluous_args nel corpus) + 🔧
> DOMAttr::$value virtuale write-through + ⚡ Props hash-index lazy e
> PhpArray::holds_containers (gc salta array scalar-only; A/B interleaved
> 84,5 vs 84,1s controllo = parità). Gate tutto verde, corpus 1489→1487.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- **Full-suite single-site: run16 LANCIATA a fine WP-23** (esito NON ancora
  letto — PRIMA COSA in WP-24: leggere `wp16-harness/full-out/`, diff per
  nome vs `full-out/full-oracle.names`; attesa: **2→1 diff** — chiuso
  `wpIsIniValueChangeable #4` via ext/tidy, resta SOLO `wp_is_stream #2`
  (stream_get_wrappers onesto — divergenza DECISA). Poi archiviare run16/.
  ⚠️ Se >1 diff: investigare PRIMA di ogni altra cosa.
- **Full-suite multisite: 2 diff per nome** (`wp19-harness/ms-out/`, baseline
  WP-19; non rilanciata in WP-22/23 — da riconfermare quando comodo: con
  tidy anche qui l'atteso scende a 1).
- Suite phpt estensioni (misura, non gate): **tidy 44/45** (residuo 010 =
  ordine riuso object-id, cosmetico) · **xsl 44/64** (residui: transcodifica
  iso-8859-1 nel DOM load → xslt004/007/008; DOMDocument::load con path
  relativi in sottodir → bug53965; document() su stream wrapper → xslt009;
  shape warning/errori minori; byref/unset su hooked prop; xsl-phpinfo).

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
```

## Prossimo passo: SESSIONE WP-24
1. **Leggere run16** (vedi sopra) e aggiornare la baseline dichiarata.
2. **CPU residua strutturale** (full-suite CPU master vs oracle 8:50 — run16
   dirà il numero aggiornato; opt (a)/(c) WP-23 = neutre sul proxy media):
   dal profilo, in ordine: run_loop leaf ~2600 (dispatch) · memmove ~600
   (concat/enter_callee) · Zval drop/clone ~500 · gc note/sweep ~500 ·
   dispatch_instance_call ~200 · identical ~180 · bind_params ~150.
   Candidati: (b) frame setup più magro (Frame ~25 campi, box dei campi
   freddi — valutata in WP-23, ROI incerto); interning stringhe/nomi;
   memoria dati vivi (footprint per-Zval / arena, backlog WP-7).
   ⚠️ METODO A/B: SOLO run interleaved nello stesso momento (la deriva
   ambientale tra giornate è ~2-3%); pkill rust-analyzer prima; ⭐ attenti
   all'INLINING: aggiungere una path fredda a una fn calda può far saltare
   l'inline dei call-site (Props::position_ro leaf da 242 tick = +5% — il
   fix: fast-path piccolo #[inline] pubblico, path grande out-of-line).
3. **Residui xsl attaccabili** (se si vuole chiudere la suite phpt):
   (a) transcodifica iso-8859-1→UTF-8 nel loader DOM (chiude 3-4 test);
   (b) DOMDocument::load con path relativi (bug53965);
   (c) "Indirect modification of X::$p is not allowed" per byref su hooked
   prop senza &get + messaggio unset ("Cannot unset" senza "hooked
   property") — gap engine hooks.
4. **Gap engine emerso (cosmetico)**: ordine di riuso degli object-id nel
   teardown ricorsivo (Zend libera i FIGLI prima del padre → LIFO riusa
   l'id del padre; phpr preorder). Chiuderebbe tidy 010.phpt e potenziali
   diff futuri su var_dump di grafi.
5. Poi: rotta post-WP (Laravel-validazione) da [[php-rust-roadmap-wp-first]]
   o residui trasversali da [[php-rust-todo-master]].

## Lezioni operative (nuove WP-23)
- ⭐⭐ **Inlining**: v. sopra (punto 2). Il leaf ranking del `sample` mostra
  subito la fn non-inlined comparsa (era il segnale: `Props::position_ro`).
- ⭐ **A/B interleaved**: 4 round dello stesso binario nella stessa ora
  scendono 93→88s (thermal/cache settling). Il controllo va rifatto OGGI,
  interleaved, mai contro il numero di ieri.
- ⭐ **FFI libtidy**: TidyBuffer SEMPRE via `tidyBufInit` (installa
  l'allocator di default; azzerare a mano → allocator NULL → segfault in
  tidyBufFree per un buffer mai scritto). Ordine di free: buffer PRIMA del
  doc (tidy_object_free_storage).
- ⭐ **Niente keeper-object nei prelude binding**: consuma un object-id e i
  test che var_dumpano confrontano gli #id — il nodo/child tenga un ref
  all'oggetto proprietario (il refcount fa da PHPTidyDoc::ref_count).
- ⭐ **Callable dall'esterno (trampolini)**: MAI pre-check is_callable — il
  path di risoluzione deve innescare l'AUTOLOAD; mappare a posteriori gli
  errori "Call to undefined function X()" → shape zend_make_callable.
- Il guard serena-vexp blocca grep/cat anche sui .phpt dentro comandi
  composti col C; i fixture .phpt restano leggibili con Read/iconv.
- var_dump/print_r nei prelude binding: __debugInfo per nascondere le prop
  interne (pattern date.php; ora anche tidy/tidyNode/XSLTProcessor).

## Invarianti (aggiornati WP-23)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline:
  **corpus 1487 (AGGIORNATA WP-23) · sess 28 · date 351 · refl 290**
  (SOLO rimozioni ammesse; fail-set in `wp18-harness/gate-out/*.fails`) ·
  ORM 3484 3E/13F per nome · http-kernel 1665 0E/0F · cargo (1556) ·
  probe: gd 11/11, mysqli 11/11, media-probe byte-id, run-http (DIFF-set
  16 = WP-14) · WP suite per-classe = oracle (option 413 · media 762 ·
  post 906 · user 1341 · query 1889 · restapi 3514 · taxonomy 878 ·
  comment 582 · xmlrpc 316 · sitemaps 132 · classi WP-17/18). Script:
  `wp22-harness/gate22.sh` (output in wp22-harness/gate-out; il gate-out
  di WP-22 è in gate-out-wp22-archived).
- Full-suite single-site: solo miglioramenti per nome vs **run16** (da
  leggere; attesa 1 diff).
- Full-suite multisite: solo miglioramenti per nome vs **ms-out (2 diff)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust (se in timeout: perl -ne via Bash,
  ma gli EDIT solo via Serena); Vexp/Read per il C; Read/Write tool per i
  .php; log `tr -d '\0'`; probe MAI su wptests durante una run; uploads
  AZZERATI prima di ogni full run.
