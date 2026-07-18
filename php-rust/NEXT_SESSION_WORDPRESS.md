# Rotta WORDPRESS-FIRST — WP-track (dopo WP-20: memoria include dimezzata)

> ⚡ **WP-20 (2026-07-19, commit gated `2e66cc9`)**: **memoria del macchinario
> include −50% di footprint REALE** (A/B media-group, binario vecchio
> ricostruito: 9,4GB → 4,6GB). Ogni include ricompilava e leakkava le ~1070
> funzioni del prelude (~1,5MB fissi per file), rigenerava gli stub di TUTTE
> le classi seed (quadratico) e la unit-cache riteneva l'intero Rc<Program>.
> Fix: prelude compilato Rc-shared dal modulo main (relocate salta le shared
> via Rc::get_mut; static prelude condivise = semantica Zend), stub-class
> internati per nome, SeedDelta al posto del Program. run11 full-suite:
> 2 diff per nome = SOLO i dichiarati, ~23,5 min. Gate20 tutto verde.
> ⚠️ LEZIONE: la RSS macOS MENTE (compressor) — giudicare la memoria con
> `vmmap --summary → Physical footprint`, mai con ps rss.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- **Full-suite single-site: 2 diff per nome** (`wp16-harness/full-out/run11/`,
  baseline oracle `full-out/full-oracle.names` trunk `81b2b5b`).
- **Full-suite multisite: 2 diff per nome CONFERMATI su WP-20**
  (`wp19-harness/ms-out/`, run del 2026-07-19: 31.277 test 2F/86W/75S;
  baseline WP-19 archiviata in `ms-out/wp19-baseline/`).
- I 2 diff sono ENTRAMBI dichiarati e stabili: `wp_is_stream #2`
  (stream_get_wrappers onesto — divergenza DECISA) · `wpIsIniValueChangeable
  #4` (dataset generato solo con ext/Tidy).

## Harness full-suite (WP-16 — invariato, ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec { $ARGV[0] } @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# monitorare full-out/full-phpr.rss (APPEND tra run: usare tail); attendere full-phpr.done
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/)
# ⚠️ MAI probe su wptests durante una run (WP-20 docet: run appesa 50 min).
```

## Prossimo passo: SESSIONE WP-21 (perf, fase 2)
1. **CPU 2,7× vs oracle** (full-suite ~23,5 min vs 8:50): il profilo WP-20
   (`wp20-harness/prof-out/sample-*.txt`) dice run_loop + memmove/memcmp
   (linear-scan `Props` + Key eq in resolve_prop_access/Props::get) + Zval
   clone/drop + gc_sweep/collect_cycles + allocator. Candidati concreti:
   (a) indice hash/interning dei nomi di proprietà sopra N entry (Props è
   linear-scan con memcmp); (b) interning stringhe/chiavi; (c) arena
   per-request. Lezione WP-3: profilare la zona SPECIFICA prima di toccare.
2. **Memoria residua**: il footprint della suite ora è dominato da DATI PHP
   vivi (fixture + data provider trattenuti da PHPUnit per tutta la run) ×
   il footprint per-Zval di phpr. Stessa cura del punto 1 (interning/arena,
   PhpArray più densa). Misurare con vmmap Physical footprint (NON ps rss).
3. **Divergenza redeclare** (catalogata WP-20 in PHPR_DIVERGENCES): il
   re-require di un file con funzione non-condizionale deve fatalare
   "Cannot redeclare function f()" — check in `run_linked` quando il nome è
   già in `linked_functions`/modulo corrente e non-condizionale. Fix
   piccola, gate corpus obbligatorio.
4. **ext/tidy minimale** SOLO se emergono altri consumatori (chiuderebbe
   wpIsIniValueChangeable #4; oggi non vale la superficie).
5. Valutare misura suite phpt `ext/xsl` (aggiungere "xsl" a
   SUPPORTED_EXTENSIONS di phpt-runner) — misura, non gate.
6. Poi: rotta post-WP (Laravel-validazione) da [[php-rust-roadmap-wp-first]]
   o residui trasversali da [[php-rust-todo-master]].

## Lezioni operative (nuove WP-20)
- ⭐ **RSS macOS ≠ memoria**: pagine leaked FREDDE finiscono nel compressor
  (RSS 510MB con 9GB di footprint). Giudicare con `vmmap --summary` →
  "Physical footprint"; per gli A/B ricostruire il binario vecchio in un
  worktree (`git worktree add /tmp/phpr-old <sha>`) — ma copiare
  `crates/php-server/` e `Cargo.lock` dal working tree (sono gitignored,
  senza lock il mago drift rompe la build).
- ⭐ **Pattern Rc::get_mut per shared-vs-fresh**: strutture compilate
  condivise (prelude, stub) si saltano in rilocazione con
  `let Some(x) = Rc::get_mut(x) else { continue }` — unicità = fresh =
  relocate, condivisione = già globale = skip, senza flag esterni.
- Il costo di un include era FISSO (~1,5MB) e indipendente dal contenuto:
  quando un costo per-evento non scala col contenuto, cercare l'immagine
  globale ricompilata per evento (prelude/seed), non il file stesso.
- I probe `--group X` di phpunit CARICANO comunque tutti i file di test
  (group-filter post-load): un probe "piccolo" paga l'intera discovery.

## Invarianti (aggiornati WP-20)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline INVARIATA
  WP-18: **corpus 1489 · sess 28 · date 351 · refl 290** (SOLO rimozioni
  ammesse; fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F
  per nome · http-kernel 1665 0E/0F · cargo (1556) · probe: gd 11/11,
  mysqli 11/11, media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP
  suite per-classe = oracle (option 413 · media 762 · post 906 · user 1341 ·
  query 1889 · restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 ·
  sitemaps 132 · classi WP-17/18). Script pronto: `wp20-harness/gate20.sh`.
- Full-suite single-site: solo miglioramenti per nome vs **run11 (2 diff)**.
- Full-suite multisite: solo miglioramenti per nome vs **ms-out (2 diff)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust (se in timeout: perl -ne via Bash),
  Vexp/Read per il C; Read/Write tool per i .php; log `tr -d '\0'`; probe
  MAI su wptests durante una run.
