# Prossima sessione: symfony/http-kernel — CHIUSURA da 0E/25F (poi WordPress)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 7 (2026-07-13) ha chiuso
il big rock **TIMEZONE D-DT3** in un commit gated `78a2ea1`:
**0E/38F → 0E/25F**, ext/date 160→**212** (+52), ORM 3E/14F→**3E/13F**.
Dettaglio in memoria: `php-rust-symfony-http-kernel` (sezione SESSIONE-7).

> 🎯 **Contesto roadmap (WP-first, memoria `php-rust-roadmap-wp-first`)**:
> questa chiusura è PROPEDEUTICA al WP-track — non lasciarla a metà. Quando
> la coda qui sotto è vuota (o ridotta ai soli test server-based fuori
> scope), si passa a `NEXT_SESSION_WORDPRESS.md`.

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **0E/25F** (24 nomi).
- Zend corpus **2469** · ext/session 161 · ext/date **212** ·
  ext/reflection 175 · ORM **3E/13F** · cargo **1539/0**.
- Workspace suite: 56c2e188 `…/scratchpad/symfony/http-kernel`. ORM:
  77b21d67/scratchpad/orm-work.
- **Baselines gate correnti in 3991dcd8/scratchpad** (gate-e, sessione 7):
  corpus-e.norm, sess-e.norm, date-e.norm, refl-e.norm, orm-e.names,
  hk-run12.log/names, gate-e.sh (ricetta completa). Probe p7_tz1.php
  (timezone, byte-id vs oracle).

## Coda run12 (25F / 24 nomi) in ordine di lavoro
1. **Cluster resolver (~14F)**: ContainerControllerResolver 4
   (StaticController#2/#3, UndefinedController#14, RemovedControllerService),
   QueryParameterValueResolver 4 (2 nomi @parameter), ControllerResolver 2
   (StaticController#2/#3), RequestAttributeValueResolver 2 (out-of-range
   int → 404), ServiceValueResolver 1 + NotTagged 1 (ControllerNameIsAnArray),
   BackedEnum 1 (ResolveThrowsOnTypeError) — riclassificare su hk-run12.log:
   guardare i diff PHPUnit, alcuni potrebbero cadere insieme.
2. **HttpCache 3+1**: 2 ESI embedded-response, DegradationWhenCacheLocked,
   ResponseCacheStrategy LastModifiedIsMerged.
3. **ErrorListener 2** (LogLevelAttribute su interfaccia, HttpAttribute#3) ·
   singoli: LoggerTest testLogsWithoutOutput, InlineFragmentRenderer
   testRenderWithObjectsAsAttributes, CacheAttributeListener @closure,
   MergeExtensionConfigurationPass testFooBundle, ExceptionDataCollector
   testCollect.

## Divergenze residue documentate (changelog PHPR_DIVERGENCES sessione-7)
- Timezone: epoch ≥2037 per zone DST usano l'ultimo tipo della tabella TZif
  (footer POSIX non valutato); nomi IANA/abbreviazioni ("EST") DENTRO le
  stringhe datetime non parsati (solo UTC/GMT/Z/±offset); DateTimeZone ctor
  senza validazione (no DateInvalidTimeZoneException).
- Sessione 6: unit eval `eval()'d code`; messaggio fromCallable; __set dopo
  unset su typed prop.

## Lezioni operative (cumulative)
- df PRIMA dei run pesanti (gate corpus ~4GB temp; la sessione 7 ha dovuto
  cancellare la build debug a metà gate — ricostruirla se serve).
- ⚠️ gm* e locali ora DIVERGONO: mai delegare una gm-variante alla variante
  locale (bug gmmktime→mktime beccato dal probe in sessione 7).
- Probe timezone SEMPRE con tz fissata (l'oracle gira nella zona di sistema).
- Probe con vendor (Data, MockClock): eseguirli NEL workspace della suite.
- pgrep -fl; MAI cargo test/build durante un gate phpt; gate per NOME sempre
  (`--list-fails`), mai solo conteggio.
- I fail phpt nei .txt del runner sono righe `--- /path.phpt ---`.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (baseline `corpus-e.norm`) · ext/session+date+reflection per nome
  (sess-e/date-e/refl-e.norm) · ORM (3E/13F, orm-e.names) se
  ref/arg/reflection/date · cargo test (1539/0).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
