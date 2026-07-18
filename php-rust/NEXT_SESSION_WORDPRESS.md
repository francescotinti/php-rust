# Rotta WORDPRESS-FIRST — WP-track (dopo WP-17: cluster maggiori chiusi)

> 🏁 **WP-17 (2026-07-18)**: **chiusura di massa dei cluster della full-suite**
> individuati dal triage WP-16. Tutti i cluster maggiori sono a parità per
> nome con l'oracle: Template 86 OK · php-ai-client 255 OK · Privacy export
> 31 OK · Kses 358 OK · wpTexturize 357 OK · BackgroundSupport 27 OK ·
> GetBookmark 46 OK · DB_Charset 100 OK · Translation 36 OK · ExportWp 11 OK ·
> WpEmailAddress 79 OK · hooks order e includesPost a parità W-per-W ·
> sitemaps a parità · zip-file tests OK. ~23 fix engine/builtin: dettaglio nel
> changelog WordPress-17 di `PHPR_DIVERGENCES_FROM_PHP.md`. I momenti ⭐:
> `$a[]` come argomento (PclZip), LSB catturata nelle closure (ai-client),
> mysqli byte-safe sul wire (DB_Charset), punycode RFC 3492 nativo,
> ZipArchive in scrittura, `\X1C` maiuscolo nel lexer (kses), classi regex
> ASCII in byte-mode + `\g` backrefs + conditional-lookbehind.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Harness full-suite (WP-16 — invariato, ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# monitorare full-out/full-phpr.rss; attendere full-phpr.done ("rc=N HH:MM:SS")
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare. ⚠️ perl exec con UN solo
#   argomento passa dalla shell e i path con SPAZIO si spezzano: usare
#   `exec { $ARGV[0] } @ARGV`.
```
- Baseline oracle: `full-out/full-oracle.{names,junit.xml}` (trunk `81b2b5b`).
- **Baseline phpr = run8 (WP-17, post-`231d96f`): 336 → 56 diff per nome —
  15E/35F/79S su 30.480 test, ~24 min.** Junit + `diff-names.txt` in
  `full-out/run8/`. Invariante: SOLO miglioramenti per nome vs run8.

## Prossimo passo: SESSIONE WP-18 (56 diff per nome, run8/diff-names.txt)
1. Classi residue in ordine di resa: theme 6F · duotone 6F · html-api goto
   5E · wpCommunityEvents 5 · themeJson 4 · wpUtf8CodePointCount 4 ·
   BlockProcessor 3+1+1 · **sitemaps P->S=3 e auth P->S=2: phpr SKIPPA test
   che l'oracle PASSA** (indagare il motivo dello skip) · wpMail 2 ·
   DateI18n 2 · code singole (~13, tra cui 1 only-oracle
   wpIsIniValueChangeable).
2. Residui NOTI (per nome, con diagnosi già fatta):
   - **Tests_Theme 6F** (`test_get_stylesheet_directory_with_filter`): dopo
     `remove_filter` il valore resta quello filtrato — un layer cache lato
     phpr trattiene il valore (closure identity ok, hooks a parità: NON è
     remove_filter; indagare wp_cache/theme_roots o riferimenti condivisi).
   - **Tests_Block_Supports_Duotone 6F**: serve il **branch reset group
     `(?|…)`** nel motore regex (nessun backend lo supporta; va emulato con
     rinumerazione dei gruppi + REMAP dei capture a livello Engine).
   - **Tests_Functions::test_wp_is_stream ftp 1F**: DECISO divergenza —
     stream_get_wrappers elenca solo i wrapper realmente apribili
     (correct-or-absent); riaprire solo se si implementa il wrapper ftp.
   - **goto-into-block html-api 5E** (D-45.1, limite compiler).
   - **refl corpus**: `internal_parameter_default_value/check_all.phpt` era
     SKIP (mancava get_defined_functions) e ora gira ma FALLISCE (vuole i
     Deprecated SUNFUNCS_RET_* al reflect dei default interni) — skip→fail
     DICHIARATO nel fail-set refl; ReflectionFunction sui builtin ora dà uno
     stub internal con parametri vuoti (residuo: metadata param reali).
   - Divergenze minori documentate: float-key deprecation non emessa in
     isset/unset e line-number al flush; display dentro ob-handler non
     scartato ("Producing output from user output handler"); stderr
     "PHP Xxx:" del CLI SAPI mai emesso.
3. **Multisite VERO** (`-c tests/phpunit/multisite.xml`): baseline oracle +
   phpr MAI fatte (i 32 test gated girano single-site).
4. Perf: full-suite phpr ~22 min vs 8:50 oracle; picco RSS ~2.7GB zona
   image/media da capire.

## Lezioni operative (nuove WP-17)
- ⭐ **PHPUnit per-file esige class==filename**: per classi come
  `Tests_AI_Client_*` in file `wpAiClient*.php` usare `--filter 'Classe'`
  (il per-file dà "Class X could not be found", il per-directory
  "No tests executed!").
- ⭐ **I probe con WP intero**: `wp9-harness/wp-mo/wp-load.php` (install
  persistente + DB wp_o) per bisezioni di funzioni WP fuori da PHPUnit —
  MA i filtri di default possono differire dal trunk wpdev: per riprodurre
  una deprecation del trunk serve il bootstrap dei TEST
  (`vendor/autoload.php` + `tests/phpunit/includes/bootstrap.php` con
  `WP_TESTS_SKIP_INSTALL=1`).
- ⭐ Il triage dei messaggi junit può MENTIRE sul colpevole: "\X1C" nel
  provider kses era un escape MAIUSCOLO del lexer, non un bug di kses;
  "Class PclZip not found" era il compile di `$this->m($list[])`; il
  "WP_User could not be converted" era array_unique(SORT_REGULAR). Risalire
  SEMPRE alla riga sorgente del provider/chiamante prima di toccare l'engine.
- `date_parse` e `strtotime`/`DateTime::__construct` hanno DUE parser
  distinti in phpr (dateparse.rs vs date.rs strtotime_in): un formato può
  funzionare in uno e mancare nell'altro — testare entrambi.
- La vista Latin1 dei subject byte-mode impone lo stesso dominio al PATTERN:
  per oniguruma tradurre `\xNN`≥80 in `\x{NN}` (onig legge \xNN come BYTE
  dell'encoding UTF-8, non come code point).
- mago pre-decodifica i literal: i fix di escape del lexer vanno nel
  bypass raw di `lower_expr` (`\u{...}`, ora anche `\X`), non solo in
  `unescape_double_quoted`.

## Invarianti (aggiornati WP-17)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  wp14-base; WP-15/16 fixano 3 corpus + 7 date: SOLO rimozioni ammesse) ·
  ORM 3E/13F per nome · http-kernel 1665 0E/0F · cargo · probe: gd 11/11,
  mysqli 11/11, media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP
  suite: option 413 · media 762 (52S) · post 906 · user 1341 · query 1889 ·
  restapi 3514 (fail-set = oracle, oEmbed+canola ambientali) · taxonomy 878
  (1W) · comment 582 · xmlrpc 316 · multisite 32 — TUTTI = oracle · misura
  fileinfo 29P/25F/8S.
- **Classi WP-17 a parità** (nuovi invarianti veloci, run per-classe):
  Template 86 OK · Privacy 31 OK · Bookmark 46 OK · ExportWp 11 OK · Kses
  358 OK · Texturize 357 OK · Background 27 OK · Email 79 OK · Charset 100
  (1S) · l10n-convert 36 OK · ai-client (`--filter Tests_AI_Client`) 255 OK ·
  includesPost 51 (6W) · actions 43 (2W) / doAction 25 (2W) /
  applyFilters 19 (2W) / filters 37 (2W).
- Full-suite: solo miglioramenti per nome vs baseline corrente
  (run7 WP-16; run8 WP-17 quando archiviata).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust, Vexp/Read per il C; Read/Write tool
  per i .php; log `tr -d '\0'`; probe MAI su wptests durante una run.
