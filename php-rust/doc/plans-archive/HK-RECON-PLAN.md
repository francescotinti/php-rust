# symfony/http-kernel — recon 2026-07-12

Workspace: scratchpad 56c2e188 `symfony/http-kernel` (v8.1.1, composer via oracolo,
phpunit dev-main + symfony/phpunit-bridge 8.2.x-dev — SENZA bridge sleep(3600) REALE
in KernelTest: l'oracolo ci ha messo 1h05!).

## Baseline
- Oracle (con bridge): 1663 test, 0 fail, ~2 min (hk-oracle2.log).
- phpr: **1663 test, 286E / 84F** (hk-phpr-base3.log) dopo i 2 fix engine pre-baseline:
  1. fn &() => (arrow by-ref) + &$f()/&expr() dinamico + notice line-attribution +
     Ref-wrap dei value-return da fn by-ref + ReflectionFunction::returnsReference().
  2. serialize/unserialize r:/R: back-reference (numerazione Zend pre-order:
     r: consuma un numero, R: no; ref+pointee condividono il numero; unserialize
     con registry slot + cell-wrap dei target R:, shell registrata pre-props per i cicli).
  Senza il fix 2 l'intera suite moriva (stack overflow su serialize del kernel ciclico
  in HttpKernelBrowserTest::testGetScript via $client->insulate()).

## Cluster (per ROI)
- **A (~143 test)**: `ReflectionException: Function {closure}() does not exist` —
  ho_reflect_closure_info non risolve cl.named "Class::method" (Closure::fromCallable
  / first-class method callable). Fix host_reflect.rs: split su `::`, resolve metodo,
  build_func_descriptor. ArgumentResolver/ControllerResolver ovunque.
- **B (~36 test)**: `VM compile: unsupported static property field path` —
  compile/assign.rs field_path() rifiuta PlaceBase::StaticProp; morde su
  `self::$freshCache[$k] ?? …` (KernelTrait::initializeContainer, isFresh).
  Stessa famiglia del residuo `fn &() => S::$sp` (lower_place bare static prop).
- **C (22 test)**: `Cannot modify protected(set) readonly` su
  `public public(set) readonly array Cache::$variables` — phpr ignora l'override
  esplicito public(set) sui readonly (default protected(set) applicato sempre).
- **D (15)**: `array_values(): bool given` — indagare (builtin a monte che torna false).
- **E (11+2)**: `closure from a trait used across files is not yet supported` +
  PriorityTaggedServiceTrait compile fail — divergenza §3.3 trait-closure relocation.
- **F (batch builtins mancanti ~25)**: stream_context_get_params (11),
  get_cfg_var (7+4+2), ob_get_status (4), DateTimeImmutable::getLastErrors (6+2),
  ReflectionFunction::getClosureCalledClass (2), ErrorException::getSeverity (2).
- **G (10)**: Closure::fromCallable "not callable" — probabilmente callables
  privati/metodo dal proprio scope.
- **H (6)**: "Session is not active" — SessionListener, coda ext/session.
- **I (~15)**: timezone/DateTime (Input timezone/Default timezone in DataCollector,
  `+ 1 hour` strtotime relativo con spazio, getLastErrors).
- **J (7)**: spread unpacking non supportato in una forma di call VM (`get`).
- Coda varia: ControllerEvent callable TypeError (2), ob non chiusi (3), ecc.

## Residui dichiarati oggi (fuori scope)
- `&` marker nel var_dump di (array)$obj con prop ref-condivisa.
- R: markers persi nei grafi CON hook __serialize/__sleep (prepare_serialize
  dereffa le celle; identità oggetti preservata via memo → r: ok).
- bare `Class::$p` come Place in lower_place (fn &() => S::$sp) — nota: il fix B
  potrebbe coprirlo.

## Gate in corso
corpus-base/new, sess-base/new, date-base/new in scratchpad 9eb21ed6 + cargo test;
diff per NOME prima del commit dei fix engine.
