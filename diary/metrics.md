# Metriche dell'esperimento

> Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Aggiornato: 2026-06-13 (fine step 10).

## LOC (target Rust, escluso codice di test)

| Crate / modulo | LOC (≈) | Note |
|---|---|---|
| php-types/zstr.rs | 100 | PhpStr binary-safe + hash lazy |
| php-types/zval.rs | 80 | enum Zval |
| php-types/array.rs | 230 | PhpArray ordered hash |
| php-types/numstr.rs | 220 | is_numeric_string |
| php-types/dtoa.rs | 150 | zend_gcvt |
| php-types/convert.rs | 230 | conversioni |
| php-types/ops.rs | 620 | operatori (full port semantico) |
| php-types/diag.rs | 45 | Diag / PhpError |
| **Totale step 1–2** | **~1.675** | core types + operatori |

## Test

| Tipo | Conteggio |
|---|---|
| Unit/integration (workspace, fine step 18) | 323 (17 suite) |
| Unit/integration (workspace, fine step 17) | 264 (17 suite) |
| Differential vs oracle (php-types) | 37.835 casi, 0 mismatch |
| phpt-runner su testsuite PHP completa | 6172 file: 135 pass / 64 fail / 5973 skip (67.8% dei runnable) |

## phpt-runner — skip per categoria (run completo `tests/` + `Zend/tests/`, fine step 10)

| Categoria | Conteggio | Significato |
|---|---|---|
| unsupported | 5028 | confine Tier 1 (OOP, namespace, by-ref/variadic, …) — atteso |
| section | 660 | sezioni I/O/INI/SKIPIF/EXTENSIONS non modellate |
| builtin | 103 | builtin non ancora implementato (era 114; −11 sbloccati a step 10) |
| compile-error | 104 | diagnostica compile-time del motore (validazione attributi/tipi, strictness parser) non modellata — **nuova** in step 9 |
| parse | 67 | sintassi che mago non parsa nel nostro path |
| malformed | 6 | `.phpt` senza FILE/EXPECT |
| expectregex | 4 | `--EXPECTREGEX--` non supportato |
| expectf-%r | 1 | placeholder `%r` non supportato |

**Step 9** ha eliminato la categoria `diag-or-fatal` (176): quei file sono ora
*runnable* (+72 netti dopo lo skip `compile-error`). Il pass-rate scende a 67.0%
perché il corpus ora **confronta** i diag-test invece di skipparli: **+12 pass**
(11 diag + 1 null-offset Classe A) e **62 fail** esposti, tutti triagiati in
`04-divergences.md` (quasi tutti scope gap di feature, non difetti di rendering).
Tra i 62 ci sono ancora i 2 fail storici D-NEW-4 (`\u{}`) e la famiglia D-NEW-6
(type-hint non enforced).

## Differential — convergenza (step 2)

| Iterazione | Mismatch |
|---|---|
| Prima implementazione | 2.711 / 37.835 |
| Dopo fix conversione int (saturazione, doppi diagnostici) | 8 / 37.835 |
| Dopo fix pow-overflow + bitnot value-name | **0 / 37.835** |

## Compressione C → Rust (parziale, solo moduli portati)

| Modulo C | LOC C | LOC Rust | Compressione |
|---|---|---|---|
| zend_operators.c (subset osservabile) | ~3.900 | ~620 (ops) + ~450 (numstr+convert) | ~73% |
| zend_strtod.c → zend_gcvt | ~120 (la sola gcvt) | ~150 | n/a (più esplicito su Ryū) |

## Divergenze catalogate (D-NEW)

Vedi `04-divergences.md`. Allo step 2: 0 divergenze residue (tutte le scoperte
del differential sono state riconciliate verso il comportamento dell'oracle).

## Tempo

| Fase | Tempo (≈) |
|---|---|
| Phase 0 + Fase 1 + Fase 2 (diary) | ~1.5h |
| Step 1 (php-types core) | ~0.5h |
| Step 2 (operatori + oracle build + differential) | ~2.5h |
| Step 3 (bridge mago→HIR) | ~0.5h |
| Step 4 (evaluator tree-walking) | ~1h |
| Step 5 (builtins registry + nucleo) | ~1h |
| Step 7 (array + foreach/switch/match) | ~2h |
| Step 6 (phpt-runner + Fase 4c import + 2 bugfix) | ~2.5h |
| Step 8 (funzioni utente + Fase 4c re-import + 1 bugfix eval-order) | ~1.5h |
| Step 9 (rendering diagnostici/fatal + skip compile-error + triage corpus + 1 fix null-offset) | ~2h |
| Step 10 (espansione builtin: 8 gruppi TDD + ValueError/ArgumentCountError) | ~2h |
| Step 11a/b/c (reference semantics: `$b=&$a` + `f(&$x)` + builtin by-ref) | ~1.75h |
| Step 11d (element-ref + foreach-by-ref via `Zval::Ref`, 4 sotto-step) | ~2.5h |
| Step 12 (`global $x` + `$GLOBALS['literal']`: frame overlay + Zval::Ref alias + Place.base, 3 sotto-step) | ~1.75h |
| Step 13 (return-by-reference `function &f()`: ReturnRef + AssignRefCall + 2 Notice, 2 sotto-step) | ~1.25h |
| Step 14 (type-hint enforcement scalare weak: coercion engine + TypeError + deprecation + return/default, 2 sotto-step; chiude D-NEW-6) | ~1.75h |
| Step 15 (static variables: StaticVar + store persistente Rc<RefCell> + init-once, 1 sotto-step) | ~0.75h |
| Step 16 (`declare(strict_types=1)`: parsing declare + flag strict + coerce_strict int→float widening, 1 sotto-step) | ~0.75h |
| Step 17 (espansione builtin per frequenza: 24 fn pure in 5 gruppi TDD — case/build/trim/math/array) | ~1.25h |
| Step 18 (closures/callables: 7 gruppi TDD — infra+use, arrow, is_callable/call_user_func, ConstFetch, array_map/filter/usort, first-class callable, var_dump esatto) | ~3h |
| **Totale a fine step 18** | **~30h** |

## Step 10 — espansione builtin

18 builtin nuovi (count/sizeof, array_keys, array_values, in_array, array_merge,
implode/join, explode, substr, strpos, str_replace, sprintf, printf, abs, max,
min, print_r) in 8 commit TDD-isolati, +44 test funzionali (131 → 168), tutti
verificati contro l'oracle CLI. Baseline .phpt: **126 → 135 pass** (+9), gli 11
test prima skippati come `builtin` ora girano. Zero divergenze D-NEW: ogni builtin
combacia byte-per-byte. ABI di Step 5 invariata, zero modifiche all'evaluator.
Scope-out: famiglia by-reference (`array_push`/`sort`/…), `%g`/`%G`.

## Step 17 — espansione builtin per frequenza (gruppi)

24 builtin nuovi (pure, by-value) in 5 commit TDD-isolati, +20 test (244 → 264),
ognuno verificato byte-per-byte contro l'oracle CLI. ABI invariata, zero
modifiche all'evaluator, clippy pulito, zero D-NEW.

- **case**: strtoupper, strtolower, ucfirst, lcfirst, ucwords (ASCII-only).
- **build**: str_repeat, str_pad, chr, ord.
- **trim**: trim, ltrim, rtrim (charlist + range `c1..c2`).
- **math**: intdiv, pow, sqrt, floor, ceil, round.
- **array**: range, array_slice, array_reverse, array_unique, array_sum.

Builtin registrati totali: ~41 → ~65. Scope-out: Deprecation 8.5 chr/ord,
array_map/array_filter (richiedono closures), costanti named (`STR_PAD_*`,
`PHP_INT_MIN`: i test usano i valori literali), mb_*.

## Step 18 — closures / callables

Prima feature di "funzioni come valori", 7 gruppi TDD (+59 test, 264 → 323),
clippy pulito, zero D-NEW. Tutta la semantica oracle-verificata su 8.5.7.

- **18-1**: `Zval::Closure(Rc<Closure>)` (variante dedicata, no OOP); tabella piatta
  `Program.closures` + `ExprKind::Closure{fn_idx, captures}`; `function() use($a,&$b){}`
  (cattura by-value snapshot / by-ref via cella); chiamata dinamica `$f()` /
  `$a['k']()` / IIFE (`ExprKind::CallDynamic` + `call_value`/`call_closure`/`call_named`);
  `gettype`="object". Arm `Closure` nei funnel ops/convert/zval (object→scalar edge).
- **18-2**: arrow `fn()=>expr` con auto-cattura by-value (analisi free-var via
  `Node::children()` ∩ slot del padre); cattura transitiva per arrow annidate.
- **18-3**: builtin higher-order intercettati nell'evaluator (no registry):
  `is_callable`, `call_user_func`, `call_user_func_array`; callable stringa e hint
  `callable` (accettato/non-enforced) già funzionanti da 18-1.
- **18-4**: `ConstFetch` (`Expression::ConstantAccess`) + tabella costanti engine
  (PHP_INT_*, PHP_FLOAT_*, STR_PAD_*, ARRAY_FILTER_USE_*, SORT_*, COUNT_*, M_*,
  PHP_EOL, …). Sblocca i modi di array_filter e retro-sblocca l'ergonomia dei
  builtin con flag (step 17). Chiude backlog #3.
- **18-5**: `array_map` (single preserva chiavi / multi reindicizza / null=zip),
  `array_filter` (truthy / callback + modi USE_KEY/USE_BOTH), `usort` (by-ref arg0,
  merge sort stabile guidato dalla callback, reindex, ritorna true).
- **18-6**: first-class callable `name(...)` (`Expression::PartialApplication`) →
  closure `Named` che incapsula il nome.
- **18-7**: var_dump/print_r esatti — `object(Closure)#N (P) { name/file/line |
  function, parameter[] <required>/<optional> }` con contatore object-id e metadati
  di render embedded (`Rc<ClosureInfo>`).

Scope-out (debito): `Closure::bind/bindTo/call/fromCallable` + static closures
(OOP/`$this`); argomenti by-ref ai dynamic call; string-call di builtin by-ref;
spread `...$args`; callable array `[$o,'m']`/`['C','m']` (OOP); object-id non
riciclati (closure effimere numerano più alto di PHP); first-class callable di un
builtin senza array `parameter` (manca la signature → P differisce di 1);
`uasort`/`uksort`/`array_walk`/`array_reduce`; user `const`/`define()`.

**Divergenza var_dump catturanti (scoperta dal corpus, debito noto):** PHP 8.5
aggiunge una pseudo-proprietà `["static"]` con l'array delle variabili catturate
(`use`/arrow) — es. `object(Closure)#N (4){ name, file, line, static }`. La nostra
var_dump mostra solo `(3)` (name/file/line) per le closure catturanti. Ometterla è
deliberato: renderla richiede un recursion-guard in `dump` (una closure che cattura
sé stessa per riferimento — `use(&$f)` — andrebbe in loop infinito; PHP stampa
`*RECURSION*`). Rivedere insieme al recursion-tracking generale di var_dump.
Validazione corpus: `Zend/tests/closures` ora gira (6 pass / 5 fail / 124 skip;
i 5 fail sono `["static"]` mancante o il nome-file sintetico `test.phpt` del
harness, non regressioni) — prima dello step 18 erano tutti skip "unsupported".

## Step 11 — reference semantics (a livello di variabile)

Reference `&` portate in tre sotto-step TDD (+17 test, 168 → 185), tutte
verificate contro l'oracle CLI:

- **11a** `$b = &$a`: gli slot diventano `enum Binding { Value(Zval),
  Ref(Rc<RefCell<Zval>>) }` con promozione lazy (solo quando `&` lega una
  variabile). Read by-value con deref, write-through su tutti gli alias, `unset`
  che rompe solo il legame. Blast radius minimo: nessun `Zval::Ref` variant,
  ~13 access site instradati su due helper `slot_clone`/`slot_set`.
- **11b** `function f(&$x)`: `Param.by_ref`, `enum Arg { Val, Ref }`; il caller
  promuove la cella dell'argomento (riuso `slot_cell`) e il callee la condivide
  tra frame. Argomento non-variabile → Error fatale (oracle 8.5).
- **11c** builtin by-ref: ABI `BuiltinRefFn` + `enum Builtin { Value, RefFirst }`;
  `array_push`/`sort`/`array_pop`/`array_shift` ricevono `&mut Zval` su arg0.
- **11d** element-ref: unificato su `Zval::Ref(Rc<RefCell<Zval>>)` (rimosso
  `Binding`), deref-on-read (ops/convert intatti). `$x=&$a[0]`/`$a[0]=&$x`
  (`place_cell`+`write_into` ref-aware), `foreach ($a as &$v)` (+lingering
  gotcha `1,2,2`), var_dump `&int(5)` solo se `Rc::strong_count>=2`. +16 test
  (185→201) in 4 sotto-step.

Zero divergenze D-NEW. Scope-out residuo: return-by-ref (`function &f()`),
array-literal con elemento-ref (`[&$x]`), `sort` flags ≠ SORT_REGULAR,
`str_replace $count`.
