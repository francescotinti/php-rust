# Prossima sessione: symfony/http-kernel — da 0E/38F (TIMEZONE D-DT3 è il lavoro)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 2026-07-13 (sessione 6) ha
chiuso TUTTI gli errori e 16 failure in un commit unico gated `2660ee0`:
**3E/54F → 0E/38F**, e il corpus Zend è salito **2450 → 2469 (+19)**.
Dettaglio in memoria: `php-rust-symfony-http-kernel` (sezione SESSIONE-6).

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **0E/38F**.
- Zend corpus **2469 pass** · ext/session 161 · ext/date 160 ·
  ext/reflection 175 · ORM 3E/14F · cargo 1530/0.
- Workspace suite: 56c2e188 `…/scratchpad/symfony/http-kernel`. ORM:
  77b21d67/scratchpad/orm-work.
- **Baselines gate correnti in 92692ea3/scratchpad**: corpus-d.norm,
  sess-d.norm, date-d.norm, refl-d.norm, orm-d.names, hk-run11.log/names.
  Probe sessione-6: p6_out (out-param), p6_tramp (trampolino magic),
  p6_cname/p6_magic (nomi closure 8.4), p6_eval (scope-bridge),
  p6_rounset (readonly unset); p6_data/2/3 nel workspace http-kernel.

## Coda run11 (38F) in ordine di lavoro
1. **⭐ TIMEZONE (13F) — DateTimeValueResolverTest**: piano architetturale
   PRONTO in memoria `php-rust-timezone-ddt3-plan`. Sintesi: (a) parser TZif
   da /usr/share/zoneinfo in Rust (`offset_at` + `wall_to_epoch`, pinnare
   gap/fold con l'oracle), (b) `default_timezone` VM + date_default_timezone_
   set/get + INI date.timezone, (c) prelude DateTime: __tz col NOME zona +
   host fn `__tz_offset`/`__tz_mkts` in ctor/format/getTimezone,
   (d) setTimezone/diff/createFromInterface, (e) date()/strtotime nel default
   tz. ⚠️ gate ext/date per NOME a ogni step (baseline 160); probe SEMPRE con
   tz fissata (l'oracle gira nella zona di sistema).
2. **Cluster resolver (~14F)**: ContainerControllerResolver 4,
   QueryParameterValueResolver 4, ControllerResolver 2,
   RequestAttributeValueResolver 2, ServiceValueResolver 1, NotTagged 1,
   BackedEnum 1 — riclassificare su hk-run11.log: erano "arrays-equal",
   alcuni potrebbero essere caduti di riflesso; guardare i diff PHPUnit.
3. **HttpCache 3+1** (ESI/stale) · **ErrorListener 2** · singoli:
   LoggerTest, InlineFragmentRenderer, CacheAttributeListener,
   MergeExtensionConfigurationPass, ExceptionDataCollector.

## Divergenze residue documentate (changelog PHPR_DIVERGENCES sessione-6)
- Unit eval si chiama `eval()'d code`; Zend `file(line) : eval()'d code`
  (tocca nomi closure in eval, backtrace, getFileName).
- Messaggio fromCallable invalido: phpr "is not callable", Zend "Failed to
  create closure from callable: …".
- field_aa_walk è gated (prop dichiarata, ≥2 Index, container ArrayAccess);
  UnsetPath NON walka gli intermedi AA (semantica indirect-modification).
- __set dopo unset su typed prop non scatta (residuo sessione-5).

## Lezioni operative (cumulative)
- df PRIMA dei run pesanti (gate corpus ~4GB temp); `cargo clean` se serve.
- isset($a[k][k2]) e isset($o->p[k][k2]) = OP DIVERSI (IssetPath/FieldIsset):
  un fix ai path annidati va fatto in ENTRAMBI.
- Probe con vendor (Data, MockClock): eseguirli NEL workspace della suite.
- pgrep -fl (non ps|perl); MAI cargo test/build durante un gate phpt;
  gate per NOME sempre (`--list-fails`), mai solo conteggio.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (baseline `corpus-d.norm`) · ext/session+date+reflection per nome ·
  ORM (3E/14F, orm-d.names) se ref/arg/reflection · cargo test.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
