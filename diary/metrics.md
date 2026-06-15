# Metriche dell'esperimento

> Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Aggiornato: 2026-06-14 (fine step 23).

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
| Unit/integration (workspace, fine step 34-7) | 624 |
| Unit/integration (workspace, fine step 34-6) | 621 |
| Unit/integration (workspace, fine step 34-5) | 617 |
| Unit/integration (workspace, fine step 34-4) | 614 |
| Unit/integration (workspace, fine step 34-3) | 611 |
| Unit/integration (workspace, fine step 34-2) | 606 |
| Unit/integration (workspace, fine step 34-1) | 601 |
| Unit/integration (workspace, fine step 33) | 594 |
| Unit/integration (workspace, fine step 32) | 589 |
| Unit/integration (workspace, fine step 31) | 582 |
| Unit/integration (workspace, fine step 30) | 575 |
| Unit/integration (workspace, fine step 29) | 567 |
| Unit/integration (workspace, fine step 28) | 545 |
| Unit/integration (workspace, fine step 24) | 512 |
| Unit/integration (workspace, fine step 23) | 497 |
| Unit/integration (workspace, fine step 22) | 462 |
| Unit/integration (workspace, fine step 21) | 433 |
| Unit/integration (workspace, fine step 20) | 408 |
| Unit/integration (workspace, fine step 19) | 377 (17 suite) |
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
| Step 19 (OOP/classi: 7 gruppi TDD — infra, write-path prop, ereditarietà+visibility, static+costanti+LSB, instanceof+interfacce+abstract, __toString+closure-bind, var_dump+recursion) | ~5h |
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

## Step 19 — OOP / classi

Il blocco più grande del corpus, 7 gruppi TDD (+54 test, 323 → 377), clippy pulito,
zero D-NEW. Semantica oracle-verificata su 8.5.7. Dettaglio completo in
`diary/02-mapping-table.md` § "Step 19 — OOP / classi (design pass)".

- **19-1** infra: `Zval::Object(Rc<RefCell<Object>>)` con **semantica handle**
  (clone condivide l'`Rc`, mutazioni visibili a tutti — contrasta l'array COW); nuovo
  modulo php-types `object` (`Object`+`Props` mappa ordinata); class table
  `Program.classes`/`ClassDecl`/`MethodDecl`; lowering classe 2-pass (nomi poi corpi);
  `new C(args)`; `__construct`; `$this`=`ExprKind::This` (no slot, letto da `cur_this`);
  `$obj->m()`=`ExprKind::MethodCall`; prop read=`ExprKind::PropGet`; write semplice via
  `PlaceStep::Prop` (entra nel `RefCell`, niente write-back COW); arm `Object` in tutti
  i funnel ops/convert/var_dump.
- **19-2** write-path proprietà completo: compound (`$o->n+=`), inc/dec
  (`ExprKind::IncDecPlace`, copre gratis anche `$a[k]++`), `??=`, `$o->arr[]`, nested
  `$a->b->c`, isset/empty/unset.
- **19-3** ereditarietà: `extends` (parent risolto a `ClassId`), risoluzione metodi
  child→ancestor (override + costruttore ereditato), prop flatten parent-first;
  `parent::`/`self::`=`ExprKind::StaticCall`+`ClassRef` (self = classe **definente**,
  no LSB); enforcement visibility public/protected/private su read+write+metodi
  (messaggi fatal esatti).
- **19-4** static + costanti + LSB: costanti di classe (`Class::C`/`self::C`/`::class`,
  valutate nel contesto della classe dichiarante); static props (cella persistente
  per-declaring-class in HashMap, `Class::$p` read/write/compound/incdec); static
  method call `Class::m()`; **late static binding** (`cur_static_class`, `new static`,
  `static::m()`, forwarding self/parent/static vs rebind per Named).
- **19-5** instanceof + interfacce + abstract: `ExprKind::InstanceOf` (transitivo su
  catena + interfacce + interface-extends); `interface`/`implements` (tabella classi
  condivisa, `is_interface`); abstract class/interface non istanziabili (fatal
  runtime); metodi abstract = solo firma (skip al lowering).
- **19-6** `__toString` + closure binding: helper `stringify` in echo/concat
  (intercettato in `apply_binop`)/`(string)` — chiude il debito step-18 di `to_zstr`;
  `Closure.bound_this` con cattura `$this` alla creazione (closure/arrow non-static,
  `static fn` no-bind); `bindTo`/`call`/`Closure::bind`/`fromCallable`.
- **19-7** var_dump/print_r esatti: annotazioni visibility (`["p":protected]`,
  `["p":"C":private]`; print_r `[p:C:private]`) via `ObjectInfo`/`PropVis` portati nel
  valore (shape per-classe cache); **recursion-guard generale** (`*RECURSION*`) su
  oggetti e array (fixa anche un loop latente su array auto-referenziali).

**Validazione corpus:** `/tmp/php-src/tests/classes` ora **57 pass / 45 fail / 181
skip** (102 runnable) — prima dello step 19 erano ~tutti skip "unsupported". I fail
residui sono feature fuori Tier-1 (deprecation dynamic-prop, magic dinamici, typed
properties, ecc.).

**Scope-out / debito:** `final` enforcement (fatal *compile-time*, formato diverso);
`closure instanceof Closure`; scope-binding closure per private; sprintf `%s`
`__toString`; closure `["static"]` in var_dump; `__get`/`__set`/`__call`; traits;
enum; anonymous class; nomi membro dinamici. **Eccezioni = step 20** (riusano queste
classi).

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

## Step 23 — enum (pure + backed)

5 sotto-step TDD (+35 test, 462 → 497), clippy pulito. L'enum riusa `ClassDecl`
(`is_enum`/`enum_backing`/`enum_cases`); i case sono oggetti singleton interned
(`Evaluator.enum_cache`), così `===`/`match` funzionano per identità.

| Sotto-step | Contenuto | Test |
|---|---|---|
| 23-1 | enum puro: lowering, case singleton, `->name`, `instanceof`, `::class`, no-instantiate (+ fix object `===`) | 8 |
| 23-2 | backed `:int`/`:string`: `->value`, `from`/`tryFrom`, `BackedEnum`, `ValueError` | 7 |
| 23-3 | `cases()` + metodi/costanti utente (`$this`=case, `match($this)`, `self::Case`) | 6 |
| 23-4 | `var_dump`/`print_r` + fix corpus (object `==`, costanti d'interfaccia) | 9 |
| 23-5 | immutabilità case: readonly / no-dynamic / no-unset | 4 |

**3 D-NEW (gap generali pre-esistenti, non enum-specifici):** D-NEW-11 object
`===` (mai implementato), D-NEW-12 object `==`, D-NEW-13 ereditarietà costanti
d'interfaccia. Dettaglio in `04-divergences.md`.

**Validazione corpus:** `/tmp/php-src/Zend/tests/enum` **43 pass / 18 fail / 91
skip** (152 tot, 70.5% dei runnable) — prima dello step 23 ~tutti skip
"unsupported". Fail residui: by-ref readonly, operatori d'ordine fra oggetti,
validazioni compile-time, Reflection*/SPL/WeakMap, stack-trace frames.

## Step 24 — Stringable + __destruct

3 sotto-step TDD (+15 test, 497 → 512), clippy pulito, **zero D-NEW**. Tutto
intercettato nei punti esistenti (instanceof, dispatch di `new`, boundary di
statement); nessuna modifica all'HIR/lowerer salvo una riga di PRELUDE.

| Sotto-step | Contenuto | Test |
|---|---|---|
| 24-1 | **Stringable** auto-interface: `interface Stringable {}` nel PRELUDE; `is_instance_of` special-case → true se la classe ha `__toString` risolvibile (auto-impl PHP 8), `implements` esplicito gratis | 3 |
| 24-2 | **__destruct** shutdown: tracking degli oggetti creati (`Evaluator.created`), `run_destructors` a fine script in ordine LIFO, dopo il fatal eventuale | 4 |
| 24-3 | **__destruct** refcount-zero immediato: tracking passato a `Rc` forti, sweep ai boundary di statement global-scope (`Rc::strong_count==1` ⇒ irraggiungibile), loop a fixpoint per il rilascio transitivo | 8 |

**Meccanismo 24-3:** un oggetto il cui unico `Rc` forte residuo è quello di
tracking non è più raggiungibile dal programma → `__destruct` dovuto. Lo sweep
gira ai boundary di statement con `locals.is_none()` (i corpi dei dtor girano
con un frame locale, quindi niente rientranza). Copre `unset`, riassegnazione,
temporanei scartati, uscita di scope di funzione e rilascio transitivo
(array/proprietà che teneva l'ultimo riferimento). A fine script
`run_destructors` finalizza i sopravvissuti (tenuti dai global), ordine LIFO.

**Validazione corpus:** `/tmp/php-src/Zend/tests/magic_methods` **18 pass / 22
fail / 117 skip** (45% dei runnable). I 4 test mirati (`bug29368_2`, `bug43175`,
`bug72177`, `dtor_scope`) falliscono solo su feature fuori scope (Reflection*,
`array_push` by-ref su non-lvalue), non sul `__destruct`. `tests/classes`
**`factory_and_singleton_002`** conferma il timing: la nostra sequenza
interlacciata "Destruct x"/"Destruct y" combacia byte-per-byte con PHP (diverge
solo sul Warning di visibilità per la chiamata esplicita a `__destruct()`
protetto). Stringable: `stringable_automatic_implementation` produce il primo
`var_dump(... instanceof Stringable)` = `bool(true)` corretto, fallisce solo
sul `ReflectionClass` successivo.

**Scope-out (debito esplicito):** eccezione lanciata dentro un `__destruct`
(PHP la trasforma in fatal di shutdown; noi la inghiottiamo); timing
intra-statement dentro funzioni (i temporanei per-iterazione di un loop in
funzione sono finalizzati al boundary global racchiudente, non per statement
interno); oggetti creati durante lo sweep di shutdown non ri-finalizzati;
check di firma/visibilità sulla chiamata esplicita a `__destruct()`;
`implements Stringable` senza `__toString` non è un errore compile-time da noi.

## Step 25 — string interpolation

`Expression::CompositeString` (prima `Unsupported`) lowerata a una catena di
concatenazioni seeded con stringa vuota (forza il risultato a stringa). Parti:
literal, simple (`$x`/`$a[k]`/`$o->p`), braced (`{$e}`). `Concat` onora già
`__toString`. La chiave bareword `$a[k]` è riscritta da mago a `Identifier`
(segnale presente solo in interpolazione) → mappata a chiave stringa. +8 test
(512→520). Scope-out: `${name}` deprecato, heredoc indentation, backtick.

## Step 26 — json_encode / json_decode

`json_encode` builtin puro (php-builtins/src/json.rs): scalari, array (list →
array JSON, assoc/sparse → oggetto con chiavi stringa), oggetti (prop pubbliche).
Float con formato shortest-roundtrip (serialize_precision=-1) ed esponente
minuscolo; float non-finiti e UTF-8 invalido → `false`. Flag `JSON_PRETTY_PRINT`,
`JSON_UNESCAPED_SLASHES`, `JSON_UNESCAPED_UNICODE` (default: escape di `/` e
non-ASCII con `\uXXXX` + surrogate pair). `json_decode` intercettato nel
valutatore (deve costruire `stdClass`): parser recursive-descent in
php-runtime/src/json.rs; `assoc=true` → array, default → `stdClass`; JSON invalido
→ `null`. +10 test (520→530). Scope-out: `JSON_THROW_ON_ERROR`, depth, altri
flag, `JsonSerializable`, `json_last_error`.

## Step 27 — preg_* (regex)

Modulo `preg` (php-runtime/src/preg.rs) traduce i pattern PCRE delimitati
(`/body/flags`, delimitatori `(){}[]<>`) al crate `regex`; flag i/m/s/x mappati.
Backreference/lookaround non supportati dal motore → il pattern non compila e la
funzione ritorna `false`/`null` (scope-out documentato). Sei funzioni
intercettate nel valutatore (preg_match/match_all hanno `$matches` come 3° arg
by-ref; replace_callback ha una callable): `preg_match`, `preg_match_all`
(PREG_PATTERN_ORDER), `preg_replace` (backref `$1`/`${1}`/`\1` tradotti),
`preg_replace_callback`, `preg_split`, `preg_quote`. +11 test (530→541).
Scope-out: pattern/subject array, gruppi nominati, flag PREG_*, limit/count,
subject non-UTF-8 (match lossy), testo esatto del warning di compilazione PCRE.

## Step 28 — stack-trace frames reali

Call stack runtime (`Evaluator.call_stack`): `call_user_fn`/`invoke_method`
pushano un frame (nome callee + classe/tipo + linea del call-site) per la durata
del body. Alla costruzione di un Throwable (`eval_new` + `synthesize_throwable`)
lo stack è snapshottato via `capture_trace` che costruisce sia l'array di
`getTrace()` (file/line/function/class/type/args-vuoti, innermost-first) sia la
stringa di `getTraceAsString()` (`#0 file(line): Class->m() … #N {main}`). Il
prelude Exception/Error porta `$trace`/`$traceString` privati; i getter li
ritornano. `render_fatal` usa il `traceString` catturato per il blocco uncaught.
Validato byte-esatto contro l'oracle (EXPECTF `.phpt` nested-trace = pass).
+4 test (541→545). Scope-out: cattura argomenti reali (args sempre `[]`), frame
include/require e closure, trace per errori engine fuori da una call.

## Step 29 — espansione builtin data-driven + `(object)` cast

Pattern collaudato degli step 10/17: builtin **PURI** (ABI `fn(&[Zval], &mut
Ctx)` invariata, **ZERO modifiche all'evaluator**), TDD-isolato per gruppo,
ognuno oracle-verificato byte-per-byte. +22 test totali (545→567), clippy
pulito. Quattro sotto-step.

- **29-1 string** (`crates/php-builtins/src/string.rs`, +7): `strrev`,
  `str_contains`/`str_starts_with`/`str_ends_with` (byte-oriented; needle vuoto
  sempre trovato), `str_split` (chunk≥1, stringa vuota→array vuoto PHP 8.2+,
  chunk<1→ValueError), `substr_count` (non-overlapping; needle vuoto→ValueError),
  `number_format`. `number_format` usa **arrotondamento decimale half-away** sulla
  rappresentazione shortest round-trip (`format!("{:e}")` → cifre intere/frac,
  carry propagato a mano) per matchare PHP 8.4+ (`2.675→2.68`, dove il naïve
  float darebbe `2.67`); grouping a tre, separatori custom, soppressione del
  segno su `-0`. Scope-out: `substr_count` offset/length, non-finiti, multibyte.

- **29-2 array puri** (`crates/php-builtins/src/array.rs`, +10):
  `array_key_exists`/`key_exists` (null-aware, ≠ isset), `array_search`
  (loose/strict, ritorna la chiave o false), `array_fill` (chiavi consecutive
  anche con start negativo, count<0→ValueError), `array_flip` (solo valori
  int|string diventano chiavi), `array_combine` (length-mismatch→ValueError),
  `array_pad` (left/right; chiavi int rinumerate, string preservate),
  `array_product` (fold numerico da 1; vuoto→1), `array_key_first`/`_last`
  (null su vuoto), `array_diff`/`array_intersect` (confronto per stringa, chiavi
  preservate, variadici). Helper nuovo `zval_to_key` (regole chiave PHP).
  Scope-out: `array_diff_key/assoc`, `*_udiff/uintersect`, `array_walk/splice`
  (by-ref → step dedicato).

- **29-3 `(object)` cast** (HIR + lowerer + evaluator, +4): aggiunta variante
  `CastKind::Object`; `P::ObjectCast` ora lowera (prima `Unsupported`). In eval
  `object_cast`: array→stdClass (chiavi stringificate, numeriche→`"1"`),
  oggetto→identità (stessa istanza), null→stdClass vuoto, scalare→singola
  proprietà `scalar`. Riusa `make_stdclass` (già con created-tracking per
  `__destruct`). `(unset)`/`(void)` restano `Unsupported`.

- **29-4 fix D-NEW interpolazione** (`lower.rs`, +1): il **corpus** ha scoperto
  che i segmenti literal di una stringa interpolata (`CompositeString` →
  `StringPart::Literal`, valore grezzo da mago) venivano emessi **non
  unescaped** — `echo "x $v\n"` stampava un backslash-n letterale. Lo step 25
  trattava solo le parti di interpolazione. Fix: `unescape_double_quoted()`
  processa il set di escape double-quoted (`\n \r \t \v \f \e \\ \$ \"`, `\x..`
  hex, `\u{..}` codepoint, `\0..\777` ottale) su ogni segmento literal. I
  literal non interpolati (`Literal::String`) arrivano già unescaped da mago e
  restano intatti.

**Corpus** (`ext/standard/tests/{strings,array}`, batch mirato sulle nuove
funzioni): ogni funzione è byte-corretta. Le `_basic` di `strrev`/`array_fill`
divergono **solo** sul valore heredoc (`"Hello\n"` vs `"Hello"`): la **coda
newline dell'heredoc non viene strippata** — bug pre-esistente (era 25),
catalogato come **D-NEW differito** allo step heredoc/nowdoc del backlog, NON di
competenza dello step 29. `array_search.phpt` diverge per l'encoding placeholder
del byte NUL in EXPECTF (artefatto dell'harness, non un bug). Quindi 1 D-NEW
trovato+fixato (29-4) e 1 D-NEW trovato+differito (heredoc trailing newline).

## Step 30 — heredoc / nowdoc (chiude D-NEW-15)

mago restituisce heredoc/nowdoc come `CompositeString::Document` con il corpo
**grezzo** (niente dedent, niente strip della newline finale) più
l'indentazione del marker di chiusura (`DocumentIndentation`
None/Whitespace(n)/Tab(n)/Mixed) e il `DocumentKind` (Heredoc/Nowdoc). Nuovo
`lower_document()` replica il lexer PHP:
1. **dedent** — toglie l'indentazione del marker dall'inizio di ogni riga del
   corpo (flexible heredoc/nowdoc PHP 7.3+), tracciando `at_line_start`
   attraverso i segmenti literal/interpolati (l'indent è sempre literal);
2. **strip newline finale** — rimuove l'unica newline prima del marker;
3. **heredoc**: interpola le parti + processa gli escape, MA `\"` resta letterale
   (le doppie virgolette non sono speciali in un heredoc) →
   `unescape_double_quoted` ha ora il flag `process_quote` (true per
   double-quoted, false per heredoc);
4. **nowdoc**: ogni byte verbatim (niente interpolazione, niente escape).

Instradare `Document` fuori da `lower_interpolation` corregge **en passant** la
regressione del 29-4 (i corpi nowdoc venivano erroneamente unescapati: `\t`→TAB
reale). **+8 test (567→575)**, clippy pulito.

**Corpus**: `strrev_basic`/`array_fill_basic` ora **passano** (batch
string+array 5/5 = 100%); `Zend/tests/heredoc_nowdoc` 7 pass / 0 fail / 58 skip
(100% dei runnable; gli skip sono feature di linguaggio non correlate o test di
compile-error intenzionali). **D-NEW-15 chiuso.** Scope-out: i casi di
parse-error PHP (indent tab+spazi misti, righe meno indentate del marker) non
sono modellati (dedent lenient); backtick/shell-exec invariati.

## Step 31 — preg named groups + flag PREG_*

Estende lo step 27. Costanti `PREG_*` aggiunte a `resolve_constant`
(PATTERN_ORDER/SET_ORDER/OFFSET_CAPTURE/UNMATCHED_AS_NULL,
SPLIT_NO_EMPTY/DELIM_CAPTURE/OFFSET_CAPTURE). `captures_array` rifatto (prende il
`Regex` + flags):
- **gruppi nominati** `(?<name>..)`/`(?P<name>..)` emessi come chiave-nome
  seguita dall'indice numerico (via `re.capture_names()`), nell'ordine PHP;
- **PREG_OFFSET_CAPTURE** → ogni valore diventa `[stringa, offset-byte]`
  (`[_, -1]` se non matchato);
- **PREG_UNMATCHED_AS_NULL** → gruppi non matchati = `null`, tutti i gruppi
  tenuti; di default i gruppi trailing non matchati sono **omessi** (gli interni
  restano `""`).

`preg_match_all`: **PREG_SET_ORDER** (un `$matches` completo per match) vs default
**PREG_PATTERN_ORDER** (colonne per-gruppo, ora con chiavi-nome + offset/null).
`preg_split`: `$limit` rispettato, **PREG_SPLIT_NO_EMPTY** /
**PREG_SPLIT_DELIM_CAPTURE** (delimitatori catturati reinseriti) /
**PREG_SPLIT_OFFSET_CAPTURE** via walk manuale su `captures_iter`. **+7 test
(575→582)**, clippy pulito. Corpus `ext/pcre/tests`: 38 pass / 45 fail / 82 skip
(45.8% runnable; i fail sono lo scope-out di motore già dichiarato a step 27 —
backreference/lookaround/recursion del crate `regex` vs PCRE — non le feature di
questo step, verificate byte-esatte coi 7 test TDD). Scope-out: pattern/subject
array, offset di ricerca (5° arg), offset su subject non-UTF-8 (lossy).

## Step 32 — array by-ref family (array_splice + array_walk)

- **array_splice** (builtin `RefFirst`, `php-builtins/src/array.rs`): splice
  posizionale by-reference. Offset/length negativi, length null (fino a fine),
  `$replacement` scalare o array. Il risultato rinumera le chiavi intere ma
  **preserva le chiavi stringa** degli elementi tenuti; ritorna gli elementi
  rimossi (reindicizzati). Registrato con sort/array_push/pop/shift.
- **array_walk** (intercettato nel valutatore, `php-runtime`): applica la
  callback a ogni elemento. Nuovo `callable_first_by_ref()` ispeziona il primo
  parametro della callback — se è `&$value` l'elemento passa attraverso una
  cella condivisa (`Zval::Ref`) e la mutazione è **riscritta** nell'array;
  altrimenti passa by-value (read-only). 3° arg opzionale inoltrato. Le chiavi
  non cambiano mai.

**+7 test (582→589)**, clippy pulito. Corpus `array_splice*`/`array_walk/`:
10/15 runnable; i 5 fail sono **scope-out documentati**: primo arg lvalue
complesso (elemento d'array — stessa limitazione bare-`$var` di usort/preg),
var_dump di oggetti dentro l'array, sostituzione dell'array durante il walk via
`$GLOBALS` (wart di re-entrancy PHP), reference a proprietà tipizzate. Il core
(by-ref modify, by-value, extra arg) è verificato byte-esatto.

## Step 33 — array key/assoc set-ops + array_column

Builtin PURI (ABI invariata, ZERO modifiche evaluator), TDD, oracle-verificati.
- **array_diff_key** / **array_intersect_key**: confronto per **chiave** (assente
  da ogni altro / presente in ogni altro); interrogano gli altri array
  direttamente via `contains_key` (niente `HashSet<Key>` → evita il lint
  `mutable_key_type`).
- **array_diff_assoc** / **array_intersect_assoc**: confronto della coppia
  (chiave, valore-come-stringa) via helper `assoc_match`.
- **array_column**($rows, $column, $index_key=null): estrae un campo da ogni
  riga (riga senza il campo → saltata; column null → riga intera); `index_key`
  ri-chiavizza il risultato. Le righe possono essere array o oggetti (prop
  pubbliche via `Props::get`) — helper `row_get`.

**+5 test (589→594)**, clippy pulito. Corpus
`array_diff_*`/`array_intersect_*`/`array_column_*` 6/6 runnable (100%).

**`mb_*` DIFFERITO**: il build dell'oracle non ha il modulo `mbstring`
(`php -m` → solo `standard`), quindi le `mb_*` non sono validabili
differenzialmente — e `mb_strtoupper` richiederebbe le tabelle Unicode di case
mapping (proprio dove si annidano le divergenze). Serve un oracle con mbstring
compilato. Catalogato nel backlog, non implementato blind.

## Macro-step 34 — DateTime / date()

Design pass: `diary/NEXT-datetime-macro-step.md`. Decisioni del Decider a inizio
sessione: **D-DT1** crate `time` 0.3 (aritmetica civile pure-Rust, Strategy A
adapter come `regex` allo step 27); **D-DT3** scope solo UTC + offset fissi;
**D-DT5** `now`/`time()` scope-out dai differenziali (orologio reale non
deterministico). D-DT2 (classi native intercettate, stato in prop interna) e
D-DT4 (subset strtotime) per i sotto-step OOP.

### Step 34-1 — `date()` / `gmdate()` core formatting

Builtin PURI in `php-builtins/date.rs` (ABI invariata, zero modifiche
evaluator). Il crate `time` fornisce gli accessor di calendario
(`OffsetDateTime::from_unix_timestamp`, `year/month/day/hour/minute/second`,
`weekday`, `ordinal`, `iso_week`, `to_iso_week_date`); la mappatura dei format
char PHP→byte è scritta a mano (i format char PHP ≠ quelli di `time`).
- Coperti tutti i format char: giorno `d/j/D/l/N/w/S/z`, settimana `W`, mese
  `F/M/m/n/t`, anno `L/o/Y/y`, ora `a/A/g/G/h/H/i/s/u/v/B`, timezone (solo UTC)
  `e/T/I/O/P/Z`, compositi `c/r/U`.
- Escape backslash (`\Y` → letterale `Y`), char non-format passano inalterati.
- `gmdate` == `date` con scope UTC. `u`/`v` sempre `000000`/`000` (epoch i64,
  niente frazioni). `B` = Swatch internet time (BMT = UTC+1).
- `now` (ts omesso) legge l'orologio reale (`SystemTime`) → funzionante ma non
  differential-tested (D-DT5).

**+7 test (594→601)**, clippy pulito. Edge case oracle-verificati: suffissi
ordinali (1st/2nd/3rd/11th/21st/23rd), single-digit padding, leap year (`t`/`L`
feb 2023 vs 2024), `z` 0-based agli estremi anno, ISO week edge (2023-01-01 =
W52/o2022), 12h vs 24h a mezzanotte/9:00/13:00.

### Step 34-2 — `mktime` / `gmmktime` / `checkdate`

Builtin PURI in `date.rs`. `mktime(h,m,s,month,day,year)` costruisce un epoch
UTC con **normalizzazione completa degli overflow** alla maniera PHP: i mesi
fuori range riportano sull'anno (`div_euclid`/`rem_euclid`), poi giorno/ora/
minuto/secondo si sommano come durata in secondi sull'epoch del 1° del mese
(così `day 0` → ultimo giorno mese precedente, `hour 25` → +1 giorno +1h,
`second -1` → secondo precedente). Helper `civil_to_epoch`.
- **Fixup anno a 2 cifre** (legacy PHP): 0..69 → 2000..2069, 70..100 →
  1970..2000; altri valori invariati (`fixup_two_digit_year`, oracle-verificato).
- Argomenti omessi → componenti dell'ora corrente (orologio reale, non
  differential-tested, D-DT5). Overflow d'anno non rappresentabile → `false`.
- `gmmktime` == `mktime` (scope UTC). `checkdate(month,day,year)`: valida
  `1<=month<=12`, `1<=year<=32767`, giorno entro la lunghezza del mese (riusa
  `days_in_month`/leap).

**+5 test (601→606)**, clippy pulito.

### Step 34-3 — `strtotime` (subset, D-DT4)

Builtin PURO in `date.rs`. `strtotime($s, $base=now)`. Subset coperto:
- `@N` → epoch diretto (ignora base/tz), incl. `@0`.
- `now` → base.
- **Assoluti**: `Y-m-d` / `Y/m/d`, con time opzionale dopo ` `/`T` (`H:i[:s]`).
  Parser manuale (`parse_absolute`), niente dipendenza regex in php-builtins.
- **Relativi**: `[+-]N unit` ripetibili (`+1 day +2 hours`). Unit:
  sec/min/hour/day/week/month/year (sing/plur). `parse_relative` decompone il
  base in componenti civili, accumula i delta e ricostruisce via
  `civil_to_epoch` → month/year usano l'aritmetica di calendario (e l'overflow
  normalizza come PHP: jan 31 +1 mese → 2 marzo). week = 7 giorni.
- Token non riconosciuto / stringa vuota → `false`.

**Scope-out** (documentato): formati relativi testuali (`next monday`, `first
day of next month`), assoluti esotici/locale, combinazione assoluto+relativo
nella stessa stringa. **+5 test (606→611)**, clippy pulito.

### Step 34-4 — `DateTime` core (D-DT2)

**Decisione chiave**: `DateTime` è una **classe del prelude** (PHP puro in
`lower.rs::PRELUDE_SRC`), NON una classe nativa intercettata nell'evaluator.
Lo stato è una prop privata `$__ts` (epoch i64); i metodi delegano ai builtin
puri già scritti (`date`/`mktime`/`strtotime`/`time`). **Zero modifiche
all'evaluator** — riusa interamente la macchina OOP degli step 19+ (object
handle semantics, dispatch metodi, fluent `return $this`). È la traduzione più
idiomatica e nello spirito dell'esperimento.
- `__construct($datetime="now")`: `now`/`""`/`null` → `time()`; altrimenti
  `strtotime()` (throw `Exception` su parse-fail, come PHP). 
- `format` → `date($fmt, $__ts)`; `getTimestamp`/`setTimestamp`.
- `setDate`/`setTime`: ricompongono l'epoch via `mktime`, estraendo le
  componenti da preservare con `date('G'|'i'|'s'|'n'|'j'|'Y')`. Mutabile
  (modifica `$this`, alias condiviso) + fluent.
- Nuovo builtin **`time()`** (orologio reale, non differential-tested, D-DT5).

**Scope-out**: `var_dump($dateTimeObj)` diverge (PHP mostra `date`/
`timezone_type`/`timezone`; noi una prop privata `$__ts`) — si testano i
metodi/`format()`, non il dump dell'oggetto. **+3 test (611→614)**, clippy
pulito.

### Step 34-5 — `DateTimeImmutable` + `modify`

Granularità rivista (Decider): `add`/`sub` spostati al 34-6 perché richiedono
`DateInterval`; il 34-5 copre la distinzione **mutabile vs immutabile** via
`modify` (che richiede solo `strtotime`).
- `interface DateTimeInterface {}` nel prelude; `DateTime` e
  `DateTimeImmutable` la implementano (`instanceof` corretto).
- `DateTime::modify($mod)` → `strtotime($mod, $__ts)`, **muta** `$this` e lo
  ritorna (fluent).
- `DateTimeImmutable`: gemella di DateTime ma `modify`/`setTimestamp`/`setDate`/
  `setTime` **ritornano una NUOVA istanza** (`new DateTimeImmutable("@$ts")`,
  sfrutta il parsing `@N` dello step 34-3) e lasciano l'originale invariato.

**+3 test (614→617)**, clippy pulito.

### Step 34-6 — `DateInterval` + `add`/`sub` + `diff`

Mix prelude + helper builtin interni (`__interval_parse`/`__date_diff`/
`__interval_format`, prefisso `__`, back-end della API OOP):
- **`DateInterval`**: prelude class con prop pubbliche `y/m/d/h/i/s/f/invert/
  days`. Costruttore → `__interval_parse(spec)` (parser ISO 8601
  `P[nY][nM][nW][nD][T[nH][nM][nS]]` in Rust; settimane → giorni; `false` →
  throw). `days = false` quando costruito da spec (come PHP). `format()` →
  `__interval_format` (specifier `%y/%Y/%m/%M/.../%a/%R/%r/%%`, padding sulle
  maiuscole, `%a`=`(unknown)` da spec).
- **`add`/`sub`** (su DateTime e DateTimeImmutable): **PHP puro via `mktime`** —
  somma `±componenti` (rispettando `invert`) alle componenti civili estratte con
  `date()`; `mktime` normalizza l'overflow di calendario (mesi/anni calendar-
  aware, jan 31 +1m → mar 2). Immutable ritorna nuova istanza.
- **`diff`**: `__date_diff(ts1,ts2)` in Rust → `y/m/d/h/i/s/invert/days`.
  `invert=1` se ts2<ts1; `days` = totale assoluto. Breakdown y/m/d con
  l'**algoritmo di prestito di PHP/timelib** (presta la lunghezza del mese
  precedente camminando la data più recente all'indietro, `base_m` decrementa a
  ogni prestito) — verificato su 8 casi limite oracle (incl. jan31→mar1 = 30d
  0m, 2020-02-29→2024-02-28 = 3y11m30d).

**+4 test (617→621)**, clippy pulito.

### Step 34-7 — `createFromFormat` + tz no-op + corpus

- **`DateTime::createFromFormat`** / **`DateTimeImmutable::createFromFormat`**
  (static, per-classe per evitare la dipendenza da `new static`): back-end Rust
  `__date_from_format($fmt,$val)` → epoch | false. Subset di format char
  espliciti (`Y y m n d j H G h g i s` + literali + escape `\`); `!` iniziale
  resetta tutti i campi all'epoch Unix, `|` resetta i campi non ancora letti;
  campi non specificati senza reset → ora corrente (non-det, D-DT5). Valore
  interamente consumato o `false`.
- **`date_default_timezone_set`/`get`**: `set` ritorna sempre `true`, `get`
  sempre `"UTC"` (D-DT3). Settare una zona ≠ UTC è un **no-op documentato**
  (sblocca molti test del corpus che iniziano con `date_default_timezone_set`).

**+3 test (621→624)**, clippy pulito.

#### Corpus `ext/date/tests` — 37 pass, 0 bug di logica

`phpt-runner` su tutta `ext/date/tests/`: **37 pass / 155 fail / 497 skip**
(192 runnable). I 155 fail sono **tutti scope-out già dichiarati**, verificati
campionando (nessun bug nelle funzioni implementate):
- **Timezone DB / DST / abbreviazioni** (D-DT3): es. `004`, `date_basic1`
  (`T`→"GMT" a Londra d'inverno vs nostro "UTC"); il blocco UTC combacia
  byte-esatto.
- **API procedurale** non implementata: `date_create`, `date_diff`,
  `date_format`, `getdate`, `localtime`, `strftime`, `date_interval_*` ecc.
  (abbiamo solo l'API OOP).
- **`DateTimeZone`/`getTimezone`/`getOffset`**, `getLastErrors`,
  serializzazione, `var_dump`/`print_r` della rappresentazione esatta degli
  oggetti (Date* e DateInterval, incl. la prop interna `from_string`), parsing
  di stringhe con zone nominali ("CET"), formati relativi testuali.

I 28 test unit 34-1..34-7 sono tutti oracle-derived e passano byte-esatti.




