# Prossima sessione: symfony/http-kernel — cluster E (trait cross-unit) e coda (da 68E/105F)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 2026-07-12/13 ha chiuso il
cluster F + un batch engine (3 commit gated: `8bf6daf`, `d5fee4f`, + `b627175` class_*-su-trait). Dettaglio completo in
memoria: `php-rust-symfony-http-kernel`.

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **114E/93F → 68E/105F**
  (errori −46; molti ex-Error ora arrivano agli assert → +12 F "onesti").
  ⚠️ La run ora dura **~2 minuti**: il phpunit-bridge BOOTSTRAPPA (ClockMock
  attivo) da quando `self::${$n}` compila (DebugClassLoader).
- Zend corpus **2443 pass** / 1609 fail / 1253 skip · ext/session 78 fail /
  ext/date 435 fail / ext/reflection 294 fail (tutti invariati per nome) ·
  ORM **3e/15f** identico per nome · cargo **1530/0**.
- Workspace: 56c2e188 `…/scratchpad/symfony/http-kernel` (v8.1.1). Se evaporato:
  clone v8.1.1 + `php composer.phar install` con l'oracolo brew +
  `composer require --dev phpunit/phpunit:dev-main symfony/phpunit-bridge:8.2.x-dev`.
- Probe oracle sessione-2 in 77155ebc/scratchpad: p_f2/p_f3, p_spd*, p_g1,
  p_j2, p_gpc*, p_dcl1.

## Fatto nella sessione precedente
1. `8bf6daf` — cluster F builtins: ob_get_status (shape oracle-pinned; user
   1/113, default e ob_gzhandler 0/112; buffer 16384-raddoppio o chunk),
   get_cfg_var (IniEntry.global; phpr ≡ `php -n`), stream context params
   (ResKind::Context{options,params} + get/set_params + 2° arg di create),
   ErrorException ctor 6-arg reale + getSeverity, ReflectionFunction::
   getClosureCalledClass. getLastErrors DIFFERITO (serve il date-parser vero).
2. `d5fee4f` — engine: **dynamic static prop name** `self::${$expr}` come place
   (PlaceBase::StaticPropDyn; SpName Lit/Dyn in compile/assign.rs; op DynName
   già esistenti; push_class_value esteso a self/parent via Op::ClassNameScope,
   `static::` = Unsupported esplicito) → DebugClassLoader compila → bridge OK;
   Closure::fromCallable(oggetto invokable) → [$obj,'__invoke'] (cluster G);
   spread su fn sconosciuta: Op::CallNsFallbackArgs + PushConst+CallValueArgs
   (cluster J, trigger_deprecation di ParameterBag).
3. `b627175` — class_* su NOMI di trait: get_parent_class→false,
   class_implements/parents→[], class_uses→[] (residuo trait-di-trait
   documentato in PHPR_DIVERGENCES §3.3-bis). Sbloccava il TypeError di
   DebugClassLoader::checkClass su ogni trait autoloadato.

## Obiettivo primario: Cluster E — trait cross-unit (~33 errori: 22+11)
Due firme, UNA radice (la macchineria trait/closure/deferred cross-unit):
- **22× `include(): Failed to compile 'PriorityTaggedServiceTrait.php'`** —
  il file compila standalone; nel flusso reale fallisce nel percorso
  "**deferred decl re-lower**" (vm/mod.rs ~2632: `lower failed … Unsupported {
  what: "class/interface redeclaration", line: 165 }` — line 165 =
  `class PriorityTaggedServiceUtil` nello STESSO file del trait: il re-lower
  del decl differito ri-abbassa l'unità e collide col già-registrato).
  🔧 Diagnosi: `PHPR_LOG=warn PHPR_LOG_FILE=… phpr vendor/bin/phpunit
  --no-configuration --filter testBindScalarValueToControllerArgument
  Tests/DependencyInjection/RegisterControllerArgumentLocatorsPassTest.php`.
- **11× "closure from a trait used across files is not yet supported"**
  (vm/run.rs ~825, Op::MakeClosure: fn_idx del trait punta all'unit del trait,
  frame.module = unit del consumer). NOTA: `LoweredTrait` HA GIÀ
  closures/closure_base/external per lo shift cross-unit (hir.rs:111) — il
  lowering lo prevede, manca il pezzo runtime/linking. Cfr. PHPR_DIVERGENCES
  §3.3 (deferral disattivata nei corpi dei trait, `DeferConf::No`).

## Poi, in ordine di ROI (dalla mappa 68E)
- **H (6)**: "Session is not active" (SessionListener; coda ext/session).
- **getLastErrors (6)**: HARD differito — diagnostica del date parser
  (false iniziale; shape warning_count/warnings/error_count/errors).
- **Dom\HTML_NO_DEFAULT_NS (5)**: costante ext/dom mancante.
- **"Using $this when not in object context" (3)**: nuovo, da indagare.
- **Store::requestsMatch arg #3 null (3)**, ControllerEvent callable TypeError
  (2), strtotime `'+ 1 hour'` (2+1, spazio dopo il segno), Constraint::$groups
  typed-prop (2), `array + bool` (1), `::class` su null (1),
  is_uploaded_file (1).
- E i **105 FAILURES** (assert falliti) — mai analizzati sistematicamente:
  estrarre la mappa con lo stesso perl sul log (`hk-f.log` in 77155ebc).

## Residui dichiarati (non regressioni)
- class_uses(trait-che-usa-trait) → [] (§3.3-bis); class_implements(enum)
  senza interfaccia esplicita (da verificare se pre-esistente).
- Attribuzione RIGA dei warning dentro metodi statici (riga del chiamante
  invece che interna) — PRE-esistente, vale anche per nomi letterali.
- `&self::${$n}` (ref-bind su nome dinamico) = errore esplicito; `static::`
  come class-value = Unsupported esplicito.
- fn &() => S::$sp non aliasa; `&` marker perso in (array)$obj; R: persi con
  __serialize/__sleep (ereditati dalla sessione 1).

## Invarianti (identici) + 2 lezioni nuove
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (`--list-fails`; baseline .norm in 77155ebc/scratchpad: corpus-i.norm ecc.) ·
  ext/session+date+reflection per nome · ORM (3e/15f) se ref/arg/reflection ·
  cargo test. MAI `cargo build` durante un gate phpt.
- ⚠️ **MAI CARGO_TARGET_DIR dentro /private/tmp** (la build release da ~10GB
  ha RIEMPITO il disco: ENOSPC blocca perfino i comandi del harness; se serve
  un binario di prova mentre gira un gate, usare un target-dir su
  /Volumes/Extreme Pro).
- ⚠️ Suite appesa con CPU-time fermo (`ps -o etime,cputime`) = sleep reale →
  bridge non bootstrappato.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena per
  Rust; Read tool per i .php; log con `LC_ALL=C tr -d '\0'`.
