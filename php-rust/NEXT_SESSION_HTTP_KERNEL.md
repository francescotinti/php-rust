# Prossima sessione: symfony/http-kernel — da 3E/54F (timezone è il big rock)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 2026-07-13 (sessione 5) ha
chiuso TUTTA la coda 29E e metà delle failure in un commit unico gated
`c8f03c9`: **29E/103F → 3E/54F**. Dettaglio in memoria:
`php-rust-symfony-http-kernel` (sezione SESSIONE-5).

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **3E/54F**. Run ~4 min ora che
  ClockMock intercetta davvero gli sleep (fix CallNsFallback).
- Zend corpus **2450 pass** · ext/session 161/68 · ext/date **160 pass** (+3)
  · ext/reflection invariato · ORM 3E/14F · cargo 1530/0.
- Workspace suite: 56c2e188 `…/scratchpad/symfony/http-kernel`. ORM:
  77b21d67/scratchpad/orm-work.
- **Baselines gate correnti in 3312a66f/scratchpad**: corpus-c.norm,
  sess-c.norm, date-c.norm, refl-c.norm, orm-c.names (= orm-n.names),
  hk-run10.log/names; f-map.txt = mappa storica dei 103F. Probe: p_b2.php
  (matrice typed-unset/is_callable/include-$this), p_dom1, p_intval, p_pop,
  p_hk, p_store2, p_prof.

## Coda run10 (3E/54F) in ordine di ROI
1. **flock &$wouldBlock (3F)**: out-param non cablato → "Undefined variable
   $wouldBlock". Stessa ricetta di headers_sent: tabella
   host_builtin_out_param. QUICK WIN. Stesso giro: indagare `$cchrCount` (2F,
   altra out-param?) — grep nel log run10.
2. **3E**: `Closure::fromCallable([$obj,'magicMethod'])` deve creare la
   closure-trampolino __call (2E, RequestDataCollectorTest #6/#7);
   LazyClosure ctor da eval (1E, RegisterControllerArgumentLocatorsPass).
3. **LoggerDataCollector (7F)**: getCompilerLogs/getProcessedLogs — la radice
   NON è la regex possessiva (verificata byte-id); indagare con il binario
   nuovo (probabile fixture Compiler.log / Data-cloning).
4. **arrays-equal 10F** (QueryParameter 4, Container/ControllerResolver 4+2,
   ControllerEvent 4, ControllerArguments 2): riclassificare con perl su
   hk-run10.log — alcuni potrebbero essere caduti con i fix callable.
5. **⭐ TIMEZONE (13F, il big rock)**: DateTimeValueResolverTest "Default
   timezone"/"Input timezone" = gap architetturale D-DT3 (phpr è UTC-only:
   niente date_default_timezone_set reale, niente tz dal parse testuale).
   Valutare un piano dedicato: rappresentare il tz nel DateTime prelude
   (__tz già esiste) + offset lookup per le zone IANA usate dai test
   (America/New_York, Europe/Paris…) via tabella o crate tzdb — decidere
   PRIMA il perimetro (solo DateTime o anche date()/strtotime).
6. Residui minori: float-string precision (2), validator deprecation (3),
   ESI/HttpCache (3), InlineFragmentRenderer (1), ecc.

## Gap engine residui documentati (PHPR_DIVERGENCES changelog sess-5)
- Call ns non qualificate: i builtin **RefFirst** (sort…) e gli **host
  builtin** (fopen, is_callable…) restano direct-bind (shadowing non visto).
- __set dopo unset su TYPED prop non scatta (su untyped funziona).
- SEND_VAR_EX: PropDyn/call-result non differiti; Error by-ref mancante.
- Shadowing metodi privati parent/child (workaround __dicur/__disync).
- getLastErrors: aggiornato solo da createFromFormat; messaggi generici.

## Lezioni operative fresche (sessione 5)
- **df PRIMA dei run pesanti**: il gate corpus tira ~4GB di temp; il disco
  era al 98% → ENOSPC che uccide harness e gate. `cargo clean` libera ~10GB
  (rebuild completo 45s).
- CPU-frozen su nanosleep ≠ suite appesa: erano sleep(65)+… REALI (ClockMock
  inerte, ora fixato). Prima di killare: sommare gli sleep attesi.
- `ps aux | perl` può NON vedere processi: usare `pgrep -fl`.
- MAI `cargo test` mentre un gate phpt gira (può relinkare i bin release).
- Il gate per NOME ha beccato uno SWAP (+2 conteggio ma 4 rotture) sul primo
  modello di typed-unset: mai fidarsi del conteggio.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (`--list-fails`, baseline `corpus-c.norm`) · ext/session+date+reflection
  per nome · ORM (**3E/14F**, orm-c.names) se ref/arg/reflection · cargo
  test. MAI `cargo build` durante un gate phpt.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
