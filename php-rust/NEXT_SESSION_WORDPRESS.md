# Rotta WORDPRESS-FIRST — WP-track (dopo WP-18: i 56 diff di run8 chiusi)

> 🏁 **WP-18 (2026-07-18)**: **chiusi tutti i cluster e singleton attaccabili
> dei 56 diff per nome di run8.** Fix ⭐: `(object)$closure` IDENTITARIO
> (spl_object_id stabile → remove_filter, Tests_Theme) · branch reset `(?|…)`
> emulato (riscrittura + `Engine::Remap`, Duotone) · goto DENTRO blocchi
> trasparenti (if/try/catch) sul bytecode piatto (html-api) · string-offset
> chiave non-integrale: isset false + read TypeError (themeJson/style-engine) ·
> `static::${$n}` runtime · flush dei diag di FetchDim AL punto di raise
> (PHPUnit expectWarning) · __unset magic su path multi-step · strtotime con
> orario nelle espressioni relative + "next week"=lunedì · DateTime("−06:00")
> tz-only · 'B' zone-independent · mb_strlen maximal-subpart · json_decode
> assoc-null/scrub UTF-8 · escape ottali nelle classi regex (PHPMailer) ·
> htmlspecialchars &apos; per XML1/XHTML/HTML5 · iconv_mime_decode(_headers) ·
> ext/xml start_ns + default handler (commenti; prologo mai consegnato) ·
> argon2i/argon2id (crate `argon2`) · subset ext/intl (Normalizer +
> normalizer_*). Dettaglio nel changelog WordPress-18 di
> `PHPR_DIVERGENCES_FROM_PHP.md`.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Harness full-suite (WP-16 — invariato, ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec { $ARGV[0] } @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# monitorare full-out/full-phpr.rss; attendere full-phpr.done ("rc=N HH:MM:SS")
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare.
```
- Baseline oracle: `full-out/full-oracle.{names,junit.xml}` (trunk `81b2b5b`).
- Baseline phpr = run8 (WP-17): 56 diff per nome. **run9 (WP-18) lanciata a
  fine sessione — LEGGERE full-out/ e archiviare come run9/ prima di tutto.**
- Attesi residui run9 (~5-6 per nome, tutti DICHIARATI): sitemaps 3
  (XSLTProcessor assente), wpIsIniValueChangeable #4 (ext/Tidy assente),
  ftp wp_is_stream (divergenza decisa), ±oEmbed/canola ambientali.

## Prossimo passo: SESSIONE WP-19
1. **Leggere run9** (`wp16-harness/full-out/`), archiviare, aggiornare le
   baseline qui e in memoria. Se compaiono regressioni inattese → priorità 1.
2. **XSLTProcessor via libxslt FFI** (chiude sitemaps 3): macOS ha
   libxml2/libxslt di sistema; pattern php_types::zlibio/gd — parse del DOM
   serializzato + xsltApplyStylesheet + serializzazione. Byte-parity gratis
   (stessa lib C dell'oracle).
3. **Multisite VERO** (`-c tests/phpunit/multisite.xml`): baseline oracle +
   phpr MAI fatte (i 32 test gated girano single-site).
4. Perf: full-suite phpr ~22-24 min vs 8:50 oracle; picco RSS ~2.7GB zona
   image/media da capire.
5. Valutare ext/tidy minimale SOLO se emergono altri consumatori (per ora 1
   solo data set — non vale la superficie).

## Lezioni operative (nuove WP-18)
- ⭐ **La diagnosi tramandata può essere sbagliata**: "NON è remove_filter"
  (WP-17) era falso — il probe minimale in contesto WP_UnitTestCase ha
  smentito la nota. Ricostruire SEMPRE la riproduzione minima prima di
  fidarsi del triage precedente.
- ⭐ **`_wp_filter_build_unique_id` usa `spl_object_id((object)$cb)`**: ogni
  wrapper/copia non-identitaria dell'oggetto rompe add/remove dei filtri in
  modo silenzioso (il valore resta "filtrato").
- ⭐ **I diag phpr sono accodati, PHP li consegna al punto di raise**: con un
  error-handler che LANCIA (PHPUnit expectWarning/expectDeprecation) il
  differimento cambia il PUNTO DI UNWIND e il test fallisce. Se un
  expect*() fallisce solo in phpr, cercare il flush mancante sull'op
  colpevole (fatto per FetchDim; altri op potrebbero riemergere).
- Il tokenizer xml consegna al DEFAULT handler i commenti INTERI
  (`<!--…-->`) e NON consegna mai il testo del prologo; start_ns con prefix
  default = `false`; end_ns MAI chiamato (libxml compat, oracle-pinned).
- `--filter 'Classe::test'` senza file evita il mismatch class≠filename di
  PHPUnit MA fa il bootstrap dell'intera suite (Installing... = reinstalla
  wptests: MAI durante una run in corso).
- pwhash≠argon2: il crate `argon2` (PHC string) verifica gli hash PHP
  byte-compatibili; PASSWORD_ARGON2* sono STRINGHE in PHP 8.

## Invarianti (aggiornati WP-18)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — **nuova baseline
  WP-18: corpus 1489 · sess 28 · date 351 · refl 290** (SOLO rimozioni
  ammesse; fail-set per nome in `wp18-harness/gate-out/*.fails`) ·
  ORM 3484 3E/13F per nome · http-kernel 1665 0E/0F ·
  cargo (1554) · probe: gd 11/11, mysqli 11/11, media-probe byte-id,
  run-http (DIFF-set 16 = WP-14) · WP suite per-classe = oracle (option 413 ·
  media 762 · post 906 · user 1341 · query 1889 · restapi 3514 · taxonomy
  878 · comment 582 · xmlrpc 316 · multisite 32 · classi WP-17).
- **Classi WP-18 a parità** (nuovi invarianti veloci): duotone 30 ·
  wpHtmlProcessor 117 · wpCommunityEvents 13 · wpThemeJson 229 ·
  wpUtf8CodePointCount 20 · BlockProcessor 222 · AtomParser 1 ·
  auth argon2 2 · RemoveAccents NFD 1.
- Full-suite: solo miglioramenti per nome vs baseline corrente (run8; run9
  quando archiviata).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust, Vexp/Read per il C; Read/Write tool
  per i .php; log `tr -d '\0'`; probe MAI su wptests durante una run.
