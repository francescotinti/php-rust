# Prossima sessione: symfony/http-kernel — coda 29E + mappa 103F (da 29E/103F)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 2026-07-13 (sessione 4) ha
CHIUSO il gap engine SEND_VAR_EX in 2 commit gated: `a4f2209` (ArgPlace
differito per argomenti place a callee dinamici + costruttori by-ref) e
`f0b0f4e` (class_exists/class_uses coerenti per Closure/Generator, guardia
PRIMA dell'autoload). Dettaglio in memoria: `php-rust-symfony-http-kernel`.

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **29E/104F → 29E/103F**
  (SessionListenerTest::testSessionCookieWrittenNoCookieGiven fixata, zero
  nuove rotture). Run ~3-5 min, DETACHED, cwd = workspace.
- Zend corpus **2445 pass** identico per nome · ext/session **68 fail**
  (006.phpt guadagnato) · ext/date 435 / ext/reflection 294 invariati ·
  **ORM 3E/14F = NUOVA BASELINE** (+testCreateVersionedField) · cargo 1530/0.
- Workspace: 56c2e188 `…/scratchpad/symfony/http-kernel` (v8.1.1).
  **Baselines gate correnti**: corpus-n/sess-n/date-n/refl-n.norm +
  orm-n.names + hk-run8.names/hk-run8.log in **35578a1b/scratchpad**
  (fallback: le -l in 3520338b). ORM workspace: 77b21d67/scratchpad/orm-work.
- Probe: e_probe di sessione-3 in 3520338b/scratchpad; p_ref4/p_ref5 (bordi
  SEND_VAR_EX), p_sl_mini1-3, p_cc1-3 in 35578a1b/scratchpad.

## Obiettivo primario: coda 29E (mappa nota) + prima mappa sistematica dei 103F
Errors in ordine di ROI:
- **getLastErrors (6)**: `DateTimeImmutable::getLastErrors()` mancante —
  diagnostica del date parser (HARD, differito due volte: valutare se
  implementare almeno il contratto minimo warnings/errors che i test usano).
- **Dom\HTML_NO_DEFAULT_NS (5)**: costante ext/dom mancante.
- **"Using $this when not in object context" (3)**: da indagare (repro
  minima prima di toccare).
- **Store::requestsMatch arg #3 null (3)**, ControllerEvent callable
  TypeError (2), BadRequestException "Closure() is not allowed" (2),
  strtotime `'+ 1 hour'` (2+1), Constraint::$groups typed-prop (2),
  `::class` su null (1), ReflectionMethod::isClosure (1), is_uploaded_file (1).
- I **103 FAILURES**: mai analizzati — mappare per firma con perl su
  hk-run8.log (35578a1b/scratchpad); grosso cluster "Failed asserting that
  exception of type InvalidArgumentException is thrown" (9×).

## Gap engine residui (documentati in PHPR_DIVERGENCES, non urgenti)
- SEND_VAR_EX: place con step **PropDyn** (`->$n`) o base call-result non
  differiti (restano by-value silenti); manca l'Error runtime "Argument #N
  ($p) could not be passed by reference" per un NON-place passato a un param
  by-ref di callee dinamico; ordine warning R-fetch al bind (dopo gli arg
  successivi) in casi patologici.
- Shadowing METODI privati parent/child (workaround __dicur/__disync nel
  prelude; fix vero in resolve_method_runtime).

## Lezioni nuove di sessione-4
- Un fix può far MIGRARE un errore latente one-shot (es. $checkedClasses
  statico di DebugClassLoader) su un test prima verde: sembra una regressione
  del fix, è un difetto preesistente rivelato dall'ordine della suite —
  indagare la catena (lì era class_exists('Closure') che autoloadava).
- MAI killare i phpr "duplicati" durante una suite: PHPUnit 13 spawna i job
  process-isolated come processi `phpr vendor/bin/phpunit` omonimi pipati via
  stdin; killarli blocca il parent a 0% CPU. Diagnosi hang: `ps` + `sample
  PID` (0% = blocked su pipe, non un loop).
- phpr bufferizza stdout: log a 0B durante il run è normale.
- Artifact 32-hex VUOTI compaiono nel root php-rust durante i run (ora in
  .gitignore; fonte da indagare — controllare comunque `git status` prima di
  `git add -A`).

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (`--list-fails`, baseline `corpus-n.norm`) · ext/session+date+reflection
  per nome · ORM (**3E/14F**) se ref/arg/reflection · cargo test. MAI
  `cargo build` durante un gate phpt. MAI CARGO_TARGET_DIR in /private/tmp.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
