# Prossima sessione: symfony/http-kernel — cluster F e coda (da 114E/93F)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 2026-07-12 ha fatto il recon
di **symfony/http-kernel** e chiuso 3 batch gated (`133c2cc` → `5541772` →
`020f523`, tutti pushati). Dettaglio completo in memoria:
`php-rust-symfony-http-kernel`.

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **286E/84F → 114E/93F**.
- Zend corpus **2438 pass / 1614 fail / 1253 skip** · ext/reflection 294 fail
  (+3 pass) · ext/session 78 fail / ext/date 435 fail (invariati) · cargo **1530/0**.
- Workspace: scratchpad sessione 56c2e188 →
  `…/56c2e188-…/scratchpad/symfony/http-kernel` (v8.1.1, composer via oracolo,
  phpunit dev-main + **symfony/phpunit-bridge 8.2.x-dev**).
  ⚠️ Se il workspace è evaporato: ricetta = clone v8.1.1 + `php composer.phar
  install` con l'oracolo brew + `composer require --dev phpunit/phpunit:dev-main
  symfony/phpunit-bridge:8.2.x-dev`. **SENZA il bridge non c'è ClockMock e
  KernelTest fa sleep(3600) REALE** (oracle da 2min diventa 1h05).
- Suite: `phpr vendor/bin/phpunit -c phpunit.xml.dist` DETACHED;
  log con `LC_ALL=C tr -d '\0'`; hang → `--log-events-text FILE`.

## Fatto nella sessione precedente (3 commit gated)
1. `133c2cc` — fn &() => arrow by-ref; `$t = &$f()` dinamico (BindRefToChecked);
   notice flush alla riga dell'op; Ref-wrap dei value-return da fn by-ref;
   returnsReference(); **serialize/unserialize r:/R:** (numerazione Zend:
   pre-order, ogni slot conta, r: consuma un numero, R: no; unserialize con
   registry + cell-wrap dei target R: + shell registrata pre-props per i cicli).
   Il kernel ciclico (resolver=$this) stack-overflowava l'intera suite.
2. `5541772` — ReflectionFunction su closure method-backed (`Class::method` in
   cl.named → find_method_reflect); getName() dal descriptor; "Closure [" in
   __toString. ~143 test (ControllerResolver/ArgumentResolver).
3. `020f523` — ref dentro static prop: `$x = &self::$arr[$k]` via static_prop_rmw
   (Ref cell Rc-shared sopravvive al write-back), bare `$x = &Class::$sp` via
   nuovo `Op::StaticPropRef` (cella viva); **`public(set) readonly`** (PublicSet
   mancava in set_visibility_of; messaggi aviz esatti: esplicita SENZA
   "readonly", implicita "protected(set) readonly").

## Obiettivo primario: Cluster F — batch builtin mancanti (~25 test)
Probe oracle GIÀ catturati nel workspace (`symfony/p_bfns.php`, `p_obst.php`):
- `get_cfg_var`: valori dal file ini (per phpr: INI table di startup);
  `cfg_file_path`; unknown → `false`. Usato da ErrorRenderer/VarDumper/DataCollector.
- `ob_get_status($full)`: shape 7 chiavi `name`("default output handler"/nome
  handler)/`type`(0 user)/`flags`(112)/`level`/`chunk_size`(0)/`buffer_size`
  (16384)/`buffer_used`; senza buffer → array(0); (true) → lista per livello.
- `stream_context_get_params($ctx)`: `['notification' => …, 'options' => […]]`.
- `ErrorException::getSeverity()` (prelude, ctor già riceve severity).
- `ReflectionFunction::getClosureCalledClass()` → ReflectionClass|null
  ("Q" per `$obj->m(...)` e `Q::s(...)`; null per closure pura).
- `DateTime(Immutable)::getLastErrors()` — HARD (serve diagnostica dal date
  parser: false iniziale; shape warning_count/warnings/error_count/errors;
  messaggi tipo "Double timezone specification"). Valutare se differire.

## Poi, in ordine di ROI (conteggi dalla baseline, molti già scalati)
- **D (15)**: `array_values(): Argument #1 must be of type array, bool given`
  — builtin a monte che torna false; da indagare su un test singolo.
- **G (10)**: `Closure::fromCallable(): Argument #1 is not callable` —
  probabile callable privato/metodo dal proprio scope.
- **E (11+2)**: closure-from-trait cross-file (divergenza §3.3) +
  PriorityTaggedServiceTrait "Failed to compile".
- **H (6)**: "Session is not active" (SessionListener; coda ext/session).
- **I (~15)**: timezone/DataCollector; strtotime `'+ 1 hour'` (spazio dopo +);
  getLastErrors. **J (7)**: spread in una forma di call VM (`get`).
- Coda: ControllerEvent callable TypeError (2), ob non chiusi (3).

## Residui dichiarati (non regressioni)
- corpus: typed_properties_083 = fail onesto ex-skip (gap pre-esistente
  array-promotion TypeError "Cannot auto-initialize an array inside property");
  4 fail by-ref-corner ex-skip (arrow 006/007, closure_014, first_class_refs).
- `fn &() => S::$sp` non aliasa (ReturnRef/field_path senza base StaticProp);
  `&` marker perso nel var_dump di (array)$obj; R: persi nei grafi con hook
  __serialize/__sleep (r: ok via memo).

## Invarianti (identici alle sessioni precedenti)
- Gate per OGNI commit: probe byte-id vs oracle 8.5.7
  (`/opt/homebrew/opt/php/bin/php`) · Zend corpus diff per NOME (`--list-fails`;
  nuovi fail ammessi SOLO se ex-skip — verificare con i conteggi skip) ·
  ext/session + ext/date + ext/reflection per nome quando si toccano le aree ·
  cargo test · ORM gate se si toccano ref/arg/reflection (baseline 3e/15f).
- I `--list-fails` dell'ultimo gate sono riusabili come baseline del giro dopo
  (niente stash se la suite era già misurata su HEAD).
- ⚠️ **MAI `cargo build` mentre un gate phpt gira** (sostituisce i binari a
  metà run: gate da buttare).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED;
  Serena per nav/edit Rust; Read tool per i .php/.tpl (RTK collassa i body).
