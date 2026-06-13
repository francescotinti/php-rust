# Fase 3 — Translation log

> Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Una entry per step.

## Step 7 — Array end-to-end + foreach / switch / match

> Step 6 (phpt-runner) deliberatamente saltato con l'utente: gli array danno più
> sostanza e rendono il phpt-runner più utile dopo.

- **Riferimento C/AST:** mago 1.30 (`mago_syntax::ast`) per il front-end — nodi
  `Array`/`LegacyArray`/`ArrayElement`, `ArrayAccess`/`ArrayAppend`, `Construct`
  (`isset`/`empty`), `Foreach`/`ForeachTarget`, `Switch`/`SwitchCase`, `Match`/
  `MatchArm`, `Unset`. Semantica array da `php-types::PhpArray` (già portato dallo
  step 1, Zend/zend_hash.c) + COW via `Rc::make_mut` (D-G2).
- **Target:** `crates/php-runtime/src/{hir.rs, lower.rs, eval.rs}`; test
  `php-runtime/tests/{lowering.rs, eval.rs, differential.rs}` + corpus array di
  `php-builtins/tests/differential.rs`.
- **HIR esteso:**
  - `ExprKind`: `Array(Vec<ArrayElem>)`, `Index{base,index}`, `AssignPlace`/
    `AssignOpPlace`/`AssignCoalescePlace`, `Isset(Vec<Place>)`, `Empty(Place)`,
    `Match{subject,arms}`.
  - `StmtKind`: `Foreach{iter,key,value,body}`, `Switch{subject,cases}`,
    `Unset(Vec<Place>)`.
  - Nuovi tipi: `ArrayElem{key:Option,value}`, `MatchArm{conditions,body}`
    (conditions vuote = arm `default`), `Case{test:Option,body}`, `Place{slot,
    steps}` + `PlaceStep::{Index,Append}` — l'**lvalue** è modellato come uno slot
    base + catena di step (gestisce `$a[k]`, `$a[]`, e write annidati con
    auto-vivification).
- **Lowering:** `lower_place` generalizza il vecchio `assign_target`; una variabile
  nuda resta sull'encoding leggero `Assign(slot,…)` (preserva i diagnostici), un
  elemento array passa alle varianti `*Place`. `[...]` e `array(...)` lowerano
  identici. `isset`/`empty` sono `Construct` (espressioni), `unset` è uno
  `Statement`. Out-of-scope → `LowerError::Unsupported`: spread `...$x`, `list()`,
  foreach `&$v` by-ref, `$a[]` in read context.
- **Evaluator:**
  - **COW writes:** `resolve_steps` pre-valuta le chiavi (evita conflitti di borrow),
    poi `write_into` naviga `&mut Zval` con `Rc::make_mut` — auto-vivifica
    `Null`/`Undef` ad array, scalare → Warning "Cannot use a scalar value as an
    array" + no-op (sull'oracle è un *fatal* `Error`: resta debito di rendering
    step 9). Verificata la semantica a valore: `$b=$a; $b[0]=…` non tocca `$a`.
  - **foreach:** itera su uno **snapshot** `Vec<(Key,Zval)>` (by-value PHP: mutare
    l'array nel body non estende l'iterazione). Key→Zval per il binding di `$k`.
  - **switch:** match loose `==`, fall-through, `default` in qualunque posizione;
    `break`/`continue` livello 1 escono entrambi dallo switch (lo switch conta come
    un livello per `continue`, semantica PHP).
  - **match:** `===` strict, arm multi-condizione, `default`; nessun match e nessun
    default → `UnhandledMatchError` (`PhpError::Error("Unhandled match case <v>")`,
    repr stringhe quotate come l'oracle).
  - **isset/empty/??/??=/unset:** traversal **silenzioso** condiviso (`silent_get`):
    chiave mancante → not set, valore `null` → isset false. Esteso `eval_isset`
    (LHS di `??`) per `Index` ricorsivo → `$a['x'] ?? d` non emette warning
    (verificato: 0 diags).
  - **read `$a[k]`:** array → lookup (mancante → Warning "Undefined array key" +
    null); string offset intero (negativi da fondo, fuori range → "" + warning);
    altro scalare → Warning "Trying to access array offset…" + null.
  - **coercizione chiave:** int/bool→Int, string canonicalizza (`"8"`→Int(8)),
    null→`""`, float→trunc con Deprecated "loses precision" se frazionario,
    array→`TypeError`.
- **Differential vs oracle (php 8.5.7, `php -n -r`):** +20 snippet runtime (array
  build/index/append/nested, COW, compound su elemento, foreach k/v, switch
  fall-through/default/loose, match strict/multi/default, isset/empty/unset, `??`)
  + 6 snippet `var_dump` array (ricorsivo/annidato/keyed/post-unset) in php-builtins.
  Tutti byte-identici.
- **Verifica:** `cargo test` **94/94** verde (era 79; +15 nuovi: lowering, eval,
  differential); clippy `--all-targets --all-features --deny=warnings` pulito.
- **Out-of-scope (debito esplicito):** string-offset **write** (`$s[0]='x'`),
  foreach by-reference + `list()` destructuring, spread `...$x`, variable-variables,
  incremento su elemento (`$a[k]++`), rendering fatal/warning su stdout (step 9),
  builtin array (`count`/`array_*`/`implode`, step 10), funzioni utente (step 8).
- **Nessuna D-NEW:** la semantica array era già coperta dal port fedele di
  `PhpArray` (step 1, oracle-verified); il differential di step 7 ha confermato
  parity senza scoprire nuove divergenze.
- **Tempo:** ~2h.

## Step 5 — Builtins registry + nucleo + float shortest-roundtrip

- **Riferimento C:** ext/standard (selective port, frequenza nei test);
  `php_var_dump` (ext/standard/var.c) per il formato; `zend_gcvt` mode 0
  (serialize_precision=-1) per i float di var_dump.
- **Target:** `crates/php-builtins/src/lib.rs` (funzioni + `registry()`),
  `crates/php-runtime/src/builtin.rs` (ABI), + Call in hir/lower/eval;
  test `php-builtins/tests/{builtins.rs, differential.rs}`.
- **Decisioni applicate:** D-G16 (trait/registry builtin), risolto il vincolo di
  dipendenza: **il grafo è php-builtins → php-runtime** (non viceversa), quindi:
  - php-runtime definisce l'**ABI**: `Ctx { out, diags }`, `BuiltinFn = fn(&[Zval],
    &mut Ctx) -> Result<Zval, PhpError>`, `Registry = HashMap<Vec<u8>, BuiltinFn>`;
    l'evaluator tiene `&Registry` **iniettata** (`run_with`/`run_source_with`;
    `run`/`run_source` usano registry vuota → retro-compatibili).
  - php-builtins implementa le funzioni + `registry()`; i test end-to-end vivono
    qui (vede sia runtime che builtins).
- **HIR/lowering esteso:** `ExprKind::Call { name, args }`; lowering accetta solo
  `FunctionCall` con callee `Identifier` e argomenti **posizionali** (no
  named/variadic → Unsupported); `function_name` risolve all'ultimo segmento dopo
  `\` (Tier 1 senza namespace). Metodi/static/dynamic call → Unsupported.
- **Builtins (nucleo):** `var_dump` (variadico, ricorsivo su array, formato
  esatto), `strlen`, `gettype`, `is_int/integer/long`, `is_float/double`,
  `is_string`, `is_bool`, `is_null`, `is_array`, `is_scalar`, `is_numeric`,
  `intval`, `floatval/doubleval`, `strval`, `boolval`.
- **php-types esteso (additivo):** `PhpError::Error(String)` per la classe base
  `Error` (es. "Call to undefined function f()"); differential 37.835 invariato.
- **Float formatting:** `dtoa::double_to_shortest` (mode 0, serialize_precision=-1)
  **già presente e oracle-verified** dallo step 2 → riusato per var_dump. Nessun
  nuovo codice di formattazione necessario.
- **Differential vs oracle (php 8.5.7, `php -n -r`):** 34/34 snippet byte-identici,
  inclusi `var_dump` di INF/-INF/NAN/-0.0/`0.1+0.2`/`1/3`/`1e20`, array via
  `(array)` cast, `is_*`, `gettype`, cast `*val`.
- **Verifica:** `cargo test` 79/79 verde (10 nuovi php-builtins); clippy
  `--workspace --all-targets --deny=warnings` pulito.
- **Out-of-scope (debito):** array literali + foreach (step 7, ora gli array si
  costruiscono solo via `(array)` cast), funzioni utente (step 8), rendering
  diagnostici (step 9), espansione builtin per frequenza — implode/count/substr/
  sprintf/array_* (step 10), arity-error con messaggio PHP esatto.
- **Tempo:** ~1h.

## Step 4 — Evaluator tree-walking (v1)

- **Riferimento C:** sostituzione architetturale di `zend_execute.c` + VM generata
  (D-G9): tree-walk su HIR con `match`, NON opcode. La semantica dei valori è
  delegata a `php-types::ops`/`convert` (D-G11, l'unico modulo portato fedele).
- **Target:** `crates/php-runtime/src/eval.rs` (+ `lib.rs`); test
  `tests/eval.rs` (24) e `tests/differential.rs` (corpus 66 vs oracle).
- **Decisioni applicate:** D-G9 (evaluator tree-walk), D-G11 (dispatch a ops),
  D-G13 (diagnostica raccolta in `Outcome.diags`), D-G15 (exit/return: `Outcome`
  porta `return_value` per il `return` top-level e `fatal` per PhpError uncaught).
- **Architettura evaluator:**
  - store a slot: `Vec<Zval>` indicizzato per slot (HIR), init a `Undef`.
  - `enum Flow { Normal, Break(u32), Continue(u32), Return(Zval) }` per la
    propagazione del controllo; helper `loop_step` traduce il segnale al livello
    del loop corrente (Break/Continue N decrementano e propagano).
  - output bufferizzato (`Vec<u8>`), diagnostici raccolti (`Diags`), errori
    fatali via `?` che risalgono a `run()` → `Outcome.fatal`.
  - API: `run(&Program) -> Outcome`, `run_source(name, src) -> Result<Outcome, LowerError>`.
- **Dettagli di semantica (verificati col differential):**
  - `echo` usa `to_zstr` (implicito, precision=14): `0.1+0.2` → `0.3`.
  - lettura di variabile non definita → Warning "Undefined variable $x" + NULL;
    `??` e `??=` leggono in modalità isset-like (nessun warning).
  - `&&`/`||` short-circuit (RHS non valutato), `xor` non short-circuit.
  - `>`/`>=` mappati a `smaller(b,a)`/`smaller_or_equal(b,a)`; `<=>` → `compare`.
  - unario `+` = `1 * v` (stessa superficie TypeError della coercizione numerica).
  - inc/dec: post ritorna il vecchio valore, pre il nuovo; `Undef` → warning + NULL
    prima dell'incremento.
- **Differential vs oracle (php 8.5.7 CLI, `php -n -r`):** 66/66 snippet
  byte-per-byte identici (aritmetica, formato float, bitwise, concat/coercion,
  comparazioni, cast, assegnamenti, if/while/do-while/for, break 2/continue,
  ternario, fattoriale 10!).
- **Scoperta che valida il differential:** `$x='a'; $x++;` → valore `b` corretto,
  ma in 8.5 l'oracle stampa "Deprecated: Increment on non-numeric string..." su
  stdout (display_errors). Il mio evaluator **cattura** il `Diag::Deprecated`
  (test dedicato) ma non lo renderizza ancora → confine esplicito verso lo
  step 9 (fedeltà diagnostica). Rimosso dal corpus "warning-free".
- **Errori incontrati:**
  - [test] due aspettative errate (non bug del codice): `'10' < '9'` è
    confronto **numerico** (10<9 = false), e il caso string-increment non è
    warning-free. Codice corretto, test corretti.
- **Verifica:** `cargo test` 69/69 verde; `clippy --workspace --all-targets
  --deny=warnings` pulito.
- **Out-of-scope (debito esplicito):** rendering/interleaving dei diagnostici su
  stdout (step 9), array end-to-end + foreach/switch (step 7), funzioni utente
  (step 8), builtin + var_dump (step 5/10).
- **Tempo:** ~1h.

## Step 3 — Bridge mago → HIR

- **Riferimento C:** nessuno (sostituzione architetturale, D-G8 + D-G9: il lexer
  re2c + parser Bison + `zend_ast` + `zend_compile.c` sono rimpiazzati da mago +
  lowering, non tradotti riga-per-riga).
- **Target:** `crates/php-runtime`: `hir.rs` (tipi HIR owned), `lower.rs`
  (bridge), `lib.rs`; `tests/lowering.rs` (20 smoke test).
- **Front-end scelto:** `mago-syntax` 1.30.0 (+ `mago-database`, `mago-span`,
  `bumpalo`). Strategia A — Adapter.
- **Decisioni applicate:** D-G8 (mago come front-end + bridge isolato),
  D-G9 (AST→HIR con slot variabili risolti + span→line), D-G13 (`slots[]`
  porta il nome per la diagnostica "Undefined variable $x").
- **Round di iterazione AI:** 1 (più 1 fix di test — vedi sotto).
- **Test pass al primo tentativo:** 19/20 (il 20° era un *test errato*, non codice).
- **Scoperte sull'API di mago (verificate leggendo il sorgente nel registry, non
  solo docs.rs):**
  - mago 1.30 NON ha interner: l'AST è arena-allocato (`bumpalo::Bump`,
    lifetime `'arena`) e il testo è inline come `&'arena [u8]` (nomi di
    variabile includono il `$`). → l'HIR deve essere **owned** per sopravvivere
    all'arena (coerente con D-G10: processo residente tiene l'HIR in memoria).
  - Entry point: `parse_file(&arena, &file) -> &Program`; errori in
    `program.errors` (parsing error-recovering, mai panica), non in un `Result`.
  - `Position` ha solo `offset: u32`; la linea si ottiene da
    `File::line_number(offset)` (0-based → +1 per PHP).
  - `IfBody`/`WhileBody`/`ForBody` espongono helper (`statements()`,
    `else_if_clauses()`, `else_statements()`) che astraggono la forma a graffe
    da quella `:`/`endif` — usati per lowering uniforme di entrambe.
  - `mago-syntax` 1.30 richiede **rustc ≥ 1.96**: toolchain bumpata da 1.90 → 1.96
    (`rustup update stable`). Lint clippy 1.96 più severi → 5 fix triviali di
    stile in php-types (nessun cambio di semantica; differential 37.835 invariato).
- **Decisioni di lowering (registrate qui, non nuove D-G):**
  - Slot: ogni `$nome` *diretto* distinto → slot stabile in ordine di incontro;
    `$$x`/`${expr}` (variable-variables) → `Unsupported`.
  - Overflow di letterale intero (> i64::MAX) → promosso a `Float` come fa il
    lexer PHP.
  - `( expr )` è trasparente (nessun nodo HIR dedicato).
  - `&&`/`and` → `And`, `||`/`or` → `Or`, `xor` → `Xor`, `??` → `Coalesce`
    (short-circuit gestito dall'evaluator allo step 4); resto via `map_binop`.
  - **Scope-out esplicito** (non droppato in silenzio → `LowerError::Unsupported`,
    diventerà SKIP motivato nel phpt-runner): foreach/switch/match (step 7),
    funzioni/classi/try (step 8/Tier 2), target di assegnazione non-variabile
    (`$a[0]=`, step 7), `@`, `&`, instanceof, cast object/unset/void.
- **Test scritti:** 20 (echo singolo/multiplo, slot create+reuse, aritmetica +
  precedenza delegata a mago, overflow→float, if/elseif/else, if senza graffe,
  while, for con `$i++`, do-while, ternario pieno+corto, &&/||/??, compound
  assign, cast+unari, break/continue con livello, inline HTML, linea 1-based,
  foreach unsupported, target array unsupported, parse error).
- **Errori incontrati:**
  - [test] `while(1){break 2;}`: il corpo a graffe è un `Block`, quindi il
    `Break` è un livello più sotto — il test assumeva `body[0] == Break`; HIR
    corretto, test corretto.
- **Verifica:** `cargo test` 44/44 verde (20 nuovi + 24 php-types);
  `cargo clippy --workspace --all-targets -- --deny=warnings` pulito.
- **Tempo:** ~1h (gran parte: ricognizione API mago + lettura sorgente registry).

## Step 2 — Operatori e conversioni + oracle + differential

- **Riferimento C:** Zend/zend_operators.c (full port semantico: is_numeric_string :3620,
  compare :2306, compare_long_to_string :2260, smart_strcmp :3421, smart_streq :3373,
  increment_string :2613, pow_function_base + safe_pow, zendi_try_get_long :378),
  zend_operators.h (dval_to_lval/safe/cap, THREEWAY :516), zend_strtod.c (zend_gcvt),
  zend_smart_str.c:116
- **Target:** php-types: numstr.rs, dtoa.rs, convert.rs, ops.rs, diag.rs
  + tests/differential.rs
- **Oracle:** php 8.5.7 compilato dal sorgente locale (`/tmp/php-src`, build
  `--disable-all --enable-cli`, copia in /tmp per evitare lo spazio nel path che
  rompe autoconf)
- **Differential: 37.835 casi (47 valori × 47 × 17 binop + 6 unari), 0 mismatch**
  byte-per-byte, diagnostica inclusa. Iterazioni: 2.711 → 8 → 0 mismatch.
- **Errori dei report di seconda mano corretti leggendo il C / sondando l'oracle:**
  - [spec] trailing whitespace È ammesso nelle stringhe numeriche PHP 8 (l'agente diceva il contrario)
  - [spec] int vs stringa non-numerica in `<` → confronto come stringhe (non `l!=0`)
  - [spec] NAN→bool è truthy CON warning 8.5 "unexpected NAN value was coerced to bool"
- **Scoperte non documentate trovate dal differential (sarebbero state bug):**
  - stringa numerica con overflow intero → int **satura** a LONG_MAX/MIN (emula strtol),
    silenziosamente se `zend_is_long_compatible` (es. "9223372036854775808"|0 silente,
    "1e100"|0 deprecato)
  - double non rappresentabile in contesto int → Warning "not representable as int";
    NAN|0 emette **due** diagnostici (Warning + Deprecated, per FITS_LONG(NAN)=true)
  - NAN→string: warning solo nel cast esplicito, NON in concat
  - `pow` int overflow: il loop square-multiply **continua in double dal punto di
    overflow** (5**100 e MIN**MAX divergono da `pow(base,exp)` ricalcolato)
  - `~true` → "Cannot perform bitwise not on true" (value name, non type name)
  - conversione operandi sequenziale: op1 fallisce → niente warning da op2
- **Test:** 24 unit/integration + 37.835 differential
- **Tempo:** ~2.5h (inclusa build oracle in parallelo)

## Step 1 — php-types: PhpStr, Zval, PhpArray

- **Riferimento C:** Zend/zend_types.h:335-432, Zend/zend_string.h:114-133,
  Zend/zend_hash.c:257,1099,1182-1183,3300, Zend/zend_long.h:112
- **Target:** crates/php-types (zstr.rs, zval.rs, array.rs)
- **Decisioni applicate:** D-G1, D-G2, D-G3, D-G4
- **Round di iterazione AI:** 1 (più una correzione pre-compilazione)
- **Test pass al primo tentativo:** sì (12/12)
- **Errori incontrati / scoperte:**
  - [semantica] Il modello iniziale di `nNextFreeElement` (flag overflow) era
    impreciso: il C inizializza a `ZEND_LONG_MIN` (zend_hash.c:257), tratta MIN
    come "append parte da 0" (zend_hash.c:1099) e **satura** a `LONG_MAX`
    (zend_hash.c:1183); l'errore "next element is already occupied" deriva dal
    fatto che lo slot saturo è occupato, quindi dopo `unset($a[PHP_INT_MAX])`
    l'append a MAX **riesce di nuovo**. Verificato sul C prima del commit,
    test dedicato aggiunto. Conseguenza osservabile della RFC 8.3
    "negative array index": `$a[-5]=1; $a[]=2;` → chiave -4 (test coperto).
- **Test scritti:** 12 (3 zstr, 2 zval, 7 array: canonicalizzazione chiavi,
  collisione "8"/"08", ordine post-unset/update, next_free, append-at-MAX,
  compattazione)
- **Tempo:** ~25 minuti
