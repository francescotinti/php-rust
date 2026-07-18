# Rotta WORDPRESS-FIRST — WP-track (dopo WP-19: XSLT chiuso, multisite baselined)

> 🏁 **WP-19 (2026-07-18, commit gated `76cdb99`)**: **ext/xsl su libxslt DI
> SISTEMA via FFI** (`php_types::xsltio` → /usr/lib/libxslt+libexslt+libxml2,
> le stesse dylib dell'oracle ⇒ byte-parity; probe 10 sezioni diff-zero) —
> XSLTProcessor nel prelude dom, costanti XSL_*, "xsl" loaded. Chiusi i 3
> skip sitemaps (gruppo 132 IDENTICO per nome). **Multisite VERO
> (-c multisite.xml): PRIMA BASELINE, già a parità effettiva** — oracle
> 31.278 1F/86W/75S vs phpr 31.277 2F/86W/75S, diff per nome = 2 (entrambi
> dichiarati). **run10 full-suite: 5 → 2 diff per nome** (0E/2F/86W/73S su
> 30.480, ~24 min) — restano SOLO wp_is_stream #2 (deciso) e
> wpIsIniValueChangeable #4 (dataset ext/Tidy). Dettaglio nel changelog
> WordPress-19 di `PHPR_DIVERGENCES_FROM_PHP.md`.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- **Full-suite single-site: 2 diff per nome** (`full-out/run10/`, baseline
  oracle `full-out/full-oracle.names` trunk `81b2b5b`).
- **Full-suite multisite: 2 diff per nome** (`wp19-harness/ms-out/`,
  harness `wp19-harness/run-multisite-detached.sh` — oracle ~11 min, phpr
  ~28 min). L'1F oracle (wpPostsListTable upstream) fallisce identico.
- I 2 diff sono ENTRAMBI dichiarati e stabili su entrambe le modalità:
  `wp_is_stream #2` (stream_get_wrappers onesto — divergenza DECISA) ·
  `wpIsIniValueChangeable #4` (dataset generato solo con ext/Tidy).

## Harness full-suite (WP-16 — invariato, ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec { $ARGV[0] } @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# monitorare full-out/full-phpr.rss (APPEND tra run: usare tail); attendere full-phpr.done
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/)
```

## Prossimo passo: SESSIONE WP-20
1. **Perf full-suite**: phpr ~24 min vs 8:50 oracle (2.7×) e **picco RSS
   ~4,2 GB già in fase di INSTALL** (poi ~2,6 GB; run9 annotava ~2,7 GB in
   zona image/media). Profilare con `sample` (lezione WP-3: PRIMA di
   ottimizzare) sia l'install sia la zona calda; candidate note dal profilo
   WP-7: churn COW/arena per-request/interning.
2. **Watch item 70E fantasma**: UNA run `--group sitemaps` ha dato 70E
   ("Undefined constant \"\"" da save_mod_rewrite_rules → require file.php,
   canonical tests) MAI riprodotta (poi 132/132 OK due volte, run10 pulita).
   Se ricompare: indagare PRIMA di tutto (smell: nome-costante vuoto = op
   con stringa sbagliata — sospetto unit-cache o race una-tantum).
3. **ext/tidy minimale** SOLO se emergono altri consumatori (oggi 1 dataset
   → chiuderebbe wpIsIniValueChangeable #4; non vale la superficie).
4. Valutare misura suite phpt `ext/xsl` (aggiungere "xsl" a
   SUPPORTED_EXTENSIONS di phpt-runner) — misura, non gate.
5. Poi: rotta post-WP (Laravel-validazione) da [[php-rust-roadmap-wp-first]]
   o residui trasversali da [[php-rust-todo-master]].

## Lezioni operative (nuove WP-19)
- ⭐ **Callback variadic C senza `c_variadic`**: per xsltGenericError basta
  `xsltSetGenericErrorFunc(ctx, NULL)` con ctx = FILE* di `open_memstream` —
  il default handler vfprintf-a nel contesto; si legge il buffer a fine
  chiamata e si splitta per riga (pattern riusabile per ogni lib C con
  generic-error "context+default-handler").
- ⭐ **open_memstream conserva i PUNTATORI alle celle buf/size**: se le
  celle vivono in una struct che poi viene MOSSA, flush/close scrivono su
  stack morto = SIGSEGV. Slot in `Box` (indirizzo stabile).
- FFI a lib del dyld shared cache: i simboli si linkano via SDK .tbd
  (`cargo:rustc-link-lib=dylib=xslt` basta); `xmlFree` è una VARIABILE
  globale di tipo fn-pointer (extern static, non fn).
- I test cargo che toccano globali C di processo (error-func, xsltMaxDepth)
  vanno serializzati con un Mutex: l'harness cargo è multi-thread.
- Il gate19 confronta i fail-set contro i FILE baseline di WP-18
  (`wp18-harness/gate-out/*.fails`) — niente più worktree/binario baseline.
- hk può dare 1F transiente su ResponseCacheStrategyTest ('60'≠'61') =
  drift di secondo: rilanciare prima di gridare alla regressione.

## Invarianti (aggiornati WP-19)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline INVARIATA
  WP-18: **corpus 1489 · sess 28 · date 351 · refl 290** (SOLO rimozioni
  ammesse; fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F
  per nome · http-kernel 1665 0E/0F · cargo (1556) · probe: gd 11/11,
  mysqli 11/11, media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP
  suite per-classe = oracle (option 413 · media 762 · post 906 · user 1341 ·
  query 1889 · restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 ·
  sitemaps 132 · classi WP-17/18). Script pronto: `wp19-harness/gate19.sh`.
- Full-suite single-site: solo miglioramenti per nome vs **run10 (2 diff)**.
- Full-suite multisite: solo miglioramenti per nome vs **ms-out (2 diff)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust, Vexp/Read per il C; Read/Write tool
  per i .php; log `tr -d '\0'`; probe MAI su wptests durante una run.
