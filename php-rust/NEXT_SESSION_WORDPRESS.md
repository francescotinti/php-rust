# Rotta WORDPRESS-FIRST — WP-track (la suite INTERA gira: 336 test-diff residui)

> 🏁 **WP-16 (2026-07-17)**: **la WP core suite INTERA (30.480 test, 4,55M
> assertion, ~22 min) arriva IN FONDO su phpr per la prima volta** — prima
> moriva silenziosamente. Percorso: 5 blocker seriali (mysqli senza
> `__destruct` → deadlock a 3 processi sull'install dei test isolati; RSS →
> jetsam-kill, fix `MIMALLOC_PURGE_DELAY=0`; panic std-sort su ordine
> non-totale → merge sort tollerante; foreach by-ref su static prop iterava
> una copia; catena utf8.php) + cluster: BIG-5 nativo byte-id (port tabella
> libmbfl), mb_substitute_character/mb_scrub, JSON_UNESCAPED_LINE_TERMINATORS,
> preg_last_error+costanti, chown family, ini access/NULL/open_basedir.
> **Da 572E/199F a 123E/198F/15S = 336 test differenti per nome** (oracle:
> 30.321P/86W/73S/1F upstream wpPostsListTable; -1 test = dataset tidy
> ambientale). Dettagli: changelog WordPress-16 in
> `PHPR_DIVERGENCES_FROM_PHP.md`.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario =
100% compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER
NOME del filone. Laravel dopo.

## Harness full-suite (WP-16 — ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
# run DETACHED (sopravvive alla sessione; il task-manager in background della
# sessione UCCIDE silenziosamente le run phpr lunghe — visto in WP-15 e WP-16):
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# - crea LUI wptests (drop+create nello stesso processo), esporta
#   MIMALLOC_PURGE_DELAY=0, scrive full-out/full-phpr.{txt,junit.xml,rss,pid}
#   e il marker full-phpr.done con "rc=N HH:MM:SS" alla fine.
# - monitorare: tail full-out/full-phpr.rss (RSS+CPU ogni 20s; CPU ferma =
#   hang → sample <pid>); attendere il .done.
# oracle: "$H/run-full-detached.sh" oracle  (8:50, junit 11.6MB)
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare (il launcher fa rm).
```
- Baseline WP-16 archiviate: `full-out/full-oracle.{names,junit.xml,txt}` +
  `full-out/run7/` (l'ultima phpr) + run6. Il clone è quello WP-15
  (`be003709…/scratchpad/wpdev`, trunk `81b2b5b`); se sparisce, riclonare e
  RIFARE la baseline oracle (il trunk cambia!).
- gate15.sh (invariante per commit) resta in wp15-harness/ e ora è "gate16":
  4 suite phpt per nome vs worktree wp14-base + ORM + hk + cargo + batterie +
  10 gruppi WP + misura fileinfo.

## Prossimo passo: SESSIONE WP-17 = chiudere i 336 test-diff della full-suite
Triage per nome già fatto (diff dei .names, run7): cluster in ordine di resa:
1. **Tests_Template 38** — da aprire (junit in run7).
2. **Tests_AI_Client_PromptBuilder 28** (+ Discovery) — php-ai-client nuovo
   nel trunk; probabilmente 1-2 feature mancanti condivise.
3. **Privacy export 27** — `Class "PclZip" not found`: class-pclzip.php non
   carica (indagare cosa lo rompe; probabilmente feature parser/engine).
4. **Tests_Kses 21** · **WpRenderBackgroundSupport 20** · **GetBookmark 19**
   · **DB_Charset 16** (ora che BIG-5 c'è, restano conversioni lato wpdb) ·
   **Translation_Controller_Convert 12** · **wpTexturize 11** ·
   **ExportWp 11** · coda lunga (~100 su ~40 classi).
5. Trasversali noti: **WPDieException 26** (probabile UNA causa comune) ·
   "Expecting E_*" (16 assertion su Deprecated attese) · **Hooks order 8+8**
   (ordine callback filter/action) · **goto-into-block** html-api (D-45.1,
   limite compiler — 5E) · "two variables reference the same object" 15.
6. **Multisite VERO** (`-c tests/phpunit/multisite.xml`): baseline oracle +
   phpr MAI fatte (i 32 test gated girano single-site). Il bootstrap
   multisite phpr FUNZIONA (installa la network — verificato col filtro
   wp_validate_site_data 11/11 = oracle).
7. Perf: full-suite phpr ~22 min vs 8:50 oracle (2.5x, in linea col 2.7x di
   restapi); il picco RSS ~2.7GB nella zona image/media va capito (con
   PURGE_DELAY=0 regge, ma su 16GB è tanto).

## Lezioni operative (nuove WP-16)
- ⭐ **Le run phpr lunghe (>15') nel task-manager della sessione muoiono
  senza traccia** (nemmeno il wrapper stampa; il task resta "running"):
  SEMPRE detached con perl POSIX::setsid (macOS non ha setsid(1)) + marker
  done + telemetria RSS. Lo stdout di phpr è bufferizzato: 0B nel log NON
  significa fermo — guardare la CPU nel .rss (ferma = hang vero → `sample`).
- ⭐ **Triage full-suite = contare i messaggi distinti PRIMA di tutto**
  (`grep -A1 "^[0-9]*) " | sort | uniq -c`): 413 dei 572E erano UNA funzione
  (e il fix ne ha svelata un'altra a catena: utf8.php chiama
  mb_substitute_character POI mb_scrub — controllare TUTTE le funzioni di un
  file prima di rilanciare 25 min di suite).
- ⭐ **I deadlock cross-processo si leggono da MySQL**: performance_schema
  metadata_locks (PENDING) + information_schema.innodb_trx +
  events_statements_history del thread incriminato = la storia completa
  senza indovinare.
- `sort_by` std con semantiche PHP = bomba a orologeria ("not a total
  order"): qualunque sort su confronto loose deve passare da
  `php_types::ops::stable_sort_by`.
- I probe con `posix_*` non girano su phpr (assenti): usare getenv/`id -un`.
- macOS `sort`/`uniq` con byte non-UTF8 → "Illegal byte sequence": LC_ALL=C.
- RTK intercetta grep in pipeline complesse: usare perl per i filtri sui log.
- Archiviare junit/txt di ogni run PRIMA del rilancio (il launcher fa rm).

## Invarianti (aggiornati WP-16)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  wp14-base; WP-15/16 fixano 3 corpus + 7 date: SOLO rimozioni ammesse) ·
  ORM 3E/13F per nome · http-kernel 1665 0E/0F · cargo · probe: gd 11/11,
  mysqli 11/11, media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP
  suite: option 413 · media 762 (52S) · post 906 · user 1341 · query 1889 ·
  restapi 3514 (1E oEmbed upstream) · taxonomy 878 (1W) · comment 582 ·
  xmlrpc 316 · multisite 32 — TUTTI = oracle · misura fileinfo 29P/25F/8S.
- **Full-suite (nuovo)**: 30.480 test, ≤123E/199F per nome vs run7 (solo
  miglioramenti; junit di riferimento in wp16-harness/full-out/run7/).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED (vedi
  harness WP-16) sotto watchdog/telemetria; Serena per Rust, Vexp/Read per
  il C; Read/Write tool per i .php; log `tr -d '\0'`; probe MAI su wptests
  durante una run.
