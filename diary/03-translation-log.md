# Fase 3 — Translation log

> Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Una entry per step.

## Step 11d — Element-level references via `Zval::Ref`

### 11d-1 — variante `Zval::Ref` + rimozione `Binding` + deref-on-read (refactor a parità)

- **Riferimento C:** Zend `IS_REFERENCE`/`zend_reference`; deref pervasivo
  (`Z_DEREF`). Nessuna nuova semantica osservabile — i 185 test esistenti (incluse
  le reference 11a/b/c) fanno da guardia.
- **Target:** `php-types/zval.rs` (variante `Ref(Rc<RefCell<Zval>>)` +
  `deref_clone`/`is_ref`), `ops.rs`/`convert.rs` (arm `Ref` deref-recurse nei
  funnel: `try_to_number`/`try_to_long`/`bw_not`/`increment`/`decrement`/
  `to_bool`/`is_true_silent`/`to_long_cast`/`to_double`/`to_zstr`; `compare`
  deref all'entry), `php-runtime/eval.rs` (rimosso `enum Binding`, slot =
  `Vec<Zval>`, helper `slot_clone`/`slot_set`/`slot_cell` su `Zval`,
  read_index/foreach deref elementi), `php-builtins` (var_dump deref arm).
- **Decisioni applicate:** D-R10 (unificazione: una sola rappresentazione
  `Zval::Ref`, invariante no-ref-to-ref), D-R11 (deref-on-read: `ops.rs`/
  `convert.rs` non ricevono mai un `Ref` a runtime — i 37.835 differential
  restano intatti; gli arm `Ref` sono difensivi/deref-ricorsivi).
- **Round di iterazione AI:** 1 (il compilatore E0004 ha guidato l'esaustività:
  ~14 arm in php-types, 4 in eval.rs, 1 in builtins, 1 nel test differential).
- **Test pass al primo tentativo:** sì (185/185 invariati — parità confermata).
- **Tempo:** ~45 minuti.

### 11d-2 — element-& assignment (`$x = &$a[0]`, `$a[0] = &$x`)

- **Riferimento C:** Zend `ZVAL_MAKE_REF` su elemento di HashTable; deref-on-read
  (`Z_DEREF`) sulle letture. Oracle: ref-to-elem, vivify, elem=&var, append-ref,
  nested, write-through di elemento-ref già esistente, unset-elem-ref.
- **Target:** `hir.rs` (`AssignRef` ora `{ target: Place, source: Place }`),
  `lower.rs` (entrambi i lati via `lower_place`; rimosso `ref_var_slot`),
  `eval.rs` (`assign_ref`/`ref_source_cell`/`bind_ref_target`; nuovi free fn
  `make_cell` + `place_cell`; `slot_cell` ora = `make_cell(slot)`; **`write_into`
  ristrutturata**: deref-through di un target `Ref` in cima + scrittura nel
  child esistente al leaf → write-through di elementi-ref).
- **Decisioni applicate:** D-R12 (element-ref). `place_cell` naviga + vivifica
  (elemento mancante → NULL) + promuove a `Ref`; bind del target riusa
  `write_place(Zval::Ref(cell))`. Il caso "write-through di `$a[0]=v` quando
  `$a[0]` è già ref" cade fuori dalla nuova `write_into`.
- **Round di iterazione AI:** 1 (1 fix E0382: catch-all in `make_cell` spostava
  il `&mut` → match su `&*target`).
- **Test pass al primo tentativo:** sì (7/7 nuovi; 192 totali).
- **Divergenza/limitazione:** base scalare (`$a=5; $x=&$a[0]`) → cella detached
  (no crash) dopo il warning di `ensure_array_mut`; var_dump `&` annotation è
  11d-4 (per ora deref trasparente).
- **Test scritti:** 7 (ref-to-elem, vivify, elem=&var, append-ref, nested,
  write-through, unset-elem-ref).
- **Tempo:** ~40 minuti.

### 11d-3 — foreach-by-ref (`foreach ($a as &$v)`)

- **Riferimento C:** Zend `ZEND_FE_FETCH_R`/`_RW` (by-ref fetch promuove
  l'elemento a reference). Oracle: mutazione sorgente, **lingering ref gotcha**
  (`1,2,2`), key+by-ref, foreach-by-ref su array temporaneo (tollerato).
- **Target:** `hir.rs` (`Foreach.by_ref: bool`), `lower.rs`
  (`foreach_value_slot` rileva `&` sul value-target), `eval.rs`
  (`exec_foreach_by_ref`: snapshot delle chiavi, `place_cell` promuove ogni
  elemento a `Ref`, value slot = alias; **niente unset finale** → lingering).
- **Decisioni applicate:** D-R13. Insight chiave: il foreach **by-value**
  ora snapshotta i **clone raw** degli elementi (non deref) e deref-a al bind →
  un elemento-ref condivide la cella e viene letto *live*, ed è ciò che fa
  funzionare il gotcha (`1,2,2`). I valori plain restano congelati (semantica
  snapshot invariata). Builtin come `implode` deref-ano gli elementi-ref
  gratis via `convert::to_zstr` (arm Ref di 11d-1) — nessuna modifica per-builtin.
- **Round di iterazione AI:** 1.
- **Test pass al primo tentativo:** sì (4/4 nuovi; 196 totali).
- **Limitazione:** by-ref su non-lvalue (`foreach([1,2,3] as &$v)`) degrada a
  by-value (mutazioni perse, nessun errore) — coerente con l'oracle.
- **Test scritti:** 4 (mutazione sorgente, lingering gotcha, key+by-ref,
  temporaneo tollerato).
- **Tempo:** ~35 minuti.

### 11d-4 — var_dump `&` annotation per elementi-reference

- **Riferimento C:** Zend `php_var_dump` stampa `&` quando
  `Z_ISREF && GC_REFCOUNT(ref) > 1`. Oracle: `&int(5)` per elemento condiviso,
  **nessun** `&` dopo `unset` dell'altro alias (refcount 1), `&array(...)` per
  ref-to-array, print_r sempre trasparente.
- **Target:** `php-builtins/lib.rs` (`dump`: elemento `Zval::Ref` con
  `Rc::strong_count >= 2` → prefisso `&` + deref; altrimenti deref trasparente.
  `print_r_into`: arm `Ref` che deref-a e ricorre, niente `&`).
- **Decisioni applicate:** D-R14 + raffinamento oracle: il marker `&` dipende da
  `Rc::strong_count(cell) >= 2` (cella effettivamente condivisa), non dal solo
  essere reference — `$a[0]=&$x; unset($x); var_dump($a)` stampa `int(5)` senza
  `&`.
- **Round di iterazione AI:** 1.
- **Test pass al primo tentativo:** sì (5/5 nuovi; 201 totali).
- **Test scritti:** 5 (shared `&int`, no-marker post-unset, `&array`,
  print_r no-`&`, print_r recurse-into-ref-array).
- **Tempo:** ~30 minuti.

## Step 11c — Builtin by-reference (`array_push`/`sort`/`array_pop`/`array_shift`)

- **Riferimento C:** `ext/standard/array.c` (`php_array_push`, `php_sort`,
  `array_pop`, `array_shift`). Oracle `/tmp/php-src/sapi/cli/php`: write-through
  sull'array del caller, conteggio di ritorno, reindicizzazione di `array_shift`
  vs preservazione chiavi di `array_pop`, errori su arg0 mancante / non-array /
  non-variabile.
- **Target:** `crates/php-runtime/builtin.rs` (ABI: `BuiltinRefFn`,
  `enum Builtin { Value, RefFirst }`, `Registry` ora mappa a `Builtin`),
  `eval.rs` (`call_ref_builtin`: bind della cella di arg0, mirror output/diag
  come per i builtin by-value), `crates/php-builtins/array.rs` (4 builtin +
  helper `as_array_mut`), `lib.rs` (`add_ref`).
- **Decisioni applicate:** D-R7 ("Opzione minima"). Scelta ABI: prima-arg
  by-ref via `fn(&mut Zval, &[Zval], &mut Ctx)` invece di un `&mut [Arg]`
  generico — i quattro builtin condividono "arg0 by-ref, named `$array`,
  required", quindi l'evaluator può sollevare gli errori di famiglia
  (`Argument #1 ($array) could not be passed by reference`,
  `expects at least 1 argument`) senza conoscenza per-builtin. La cella di arg0
  è promossa con lo stesso `slot_cell` di 11a/11b.
- **Round di iterazione AI:** 1 (compila e passa al primo tentativo; nessun lint
  introdotto).
- **Test pass al primo tentativo:** sì (7/7 nuovi; 185 totali, +7 dal 178 di
  11b).
- **Divergenze/limitazioni intenzionali:** `sort` implementa solo SORT_REGULAR
  (flag accettato ma ignorato); `array_pop` non resetta `nNextFreeElement`
  (irrilevante finché non si rifa append dopo pop — non nei test); `str_replace
  $count` by-ref resta scope-out (raro). Arg0 non-variabile o mancante → errori
  oracle-verificati (Error / ArgumentCountError), superano la nota D-R7 originale
  (Warning).
- **Test scritti:** 7 (push+count, push type-error, sort+reindex, pop, shift,
  shift reindex int/preserva string, pop preserva chiavi).
- **Tempo:** ~40 minuti.

## Step 11b — Parametri by-reference (`function f(&$x)`)

- **Riferimento C:** Zend `ZEND_RECV` / `zend_call_function` (binding by-ref di
  argomento), `ZEND_SEND_REF`. Oracle `/tmp/php-src/sapi/cli/php`: mutazione del
  caller, definizione di variabile indefinita, swap a due ref, argomento
  non-variabile → Error fatale.
- **Target:** `crates/php-runtime` — `hir.rs` (`Param.by_ref: bool`),
  `lower.rs` (lettura `p.ampersand`; rimossa la `LowerError` su by-ref),
  `eval.rs` (`enum Arg { Val(Zval), Ref(Rc<RefCell<Zval>>) }`, `slot_cell`
  estratto da `assign_ref`, `eval_call_args`, `call_user_fn`/`run_user_fn_body`
  passano `Vec<Arg>`).
- **Decisioni applicate:** D-R6. Il caller promuove lo slot-argomento a `Ref`
  (riusando `slot_cell`, stessa promozione lazy di 11a) prima del frame-swap; il
  callee installa `Binding::Ref(Rc::clone)` nello slot del parametro, così la
  cella è condivisa tra i due frame.
- **Round di iterazione AI:** 1 (più 1 fix di un test esistente:
  `by_reference_and_variadic_params_are_unsupported` splittato in
  `by_reference_param_lowers_with_flag` + `variadic_params_are_unsupported`).
- **Test pass al primo tentativo:** sì (5/5 nuovi; 178 totali, +6 includendo lo
  split del test di lowering).
- **Divergenza dalla mappa Fase 2:** D-R6 prevedeva un Notice/Warning + pass
  by-value per argomenti non-variabili; l'oracle 8.5 emette invece un **Error
  fatale** (`f(): Argument #N ($p) could not be passed by reference`) — seguito
  l'oracle. Argomenti by-ref complessi (`$a[0]`, proprietà) restano scope-out
  (richiedono element-ref, step 11d): per ora solo variabili bare.
- **Test scritti:** 5 (mutazione caller, definizione variabile indefinita, swap,
  contrasto by-value, argomento non-variabile fatale).
- **Tempo:** ~30 minuti.

## Step 11a — Reference semantics a livello di variabile (`$b = &$a`)

- **Riferimento C:** Zend/zend_types.h (`IS_REFERENCE`/`zend_reference`),
  Zend `ZEND_ASSIGN_REF` / `ZVAL_MAKE_REF`. Verifica semantica contro l'oracle
  `/tmp/php-src/sapi/cli/php` (write-through bidirezionale, ref→undef definisce
  NULL, catena di alias, `unset` rompe solo il legame).
- **Target:** `crates/php-runtime` — `hir.rs` (nuovo `ExprKind::AssignRef`),
  `lower.rs` (rilevazione `$x = &$y` + `ref_var_slot`), `eval.rs`
  (`enum Binding { Value(Zval), Ref(Rc<RefCell<Zval>>) }`, helper
  `slot_clone`/`slot_set`, `assign_ref`, write-through in tutti i ~13 access site
  agli slot).
- **Decisioni applicate:** D-R1 (Binding enum, non `Zval::Ref` → blast radius
  minimo), D-R2 (read by-value con deref), D-R3 (write-through), D-R4
  (promozione lazy a `Ref`, undef→NULL alla creazione), D-R5 (`unset` rimpiazza
  il binding con `Value(Undef)`, rilascia solo quel clone dell'`Rc`), D-R8
  (write annidato via ref riusa `write_into`), D-R9 (var_dump/print_r
  trasparenti).
- **Round di iterazione AI:** 1 (compila e passa al primo tentativo dopo la
  conversione degli access site; unica iterazione: 2 lint `explicit_auto_deref`
  su `&mut *cell.borrow_mut()` inline → forma `let z = &mut *…;`).
- **Test pass al primo tentativo:** sì (4/4 nuovi; 172 totali, +4).
- **Divergenza intenzionale dalla mappa Fase 2:** D-R4 modellava `source` come
  `Place`; per 11a `AssignRef { target: Slot, source: Slot }` usa due slot bare
  (reference *dentro* array = step 11d scope-out). Promozione undef→NULL aggiunta
  dopo conferma oracle (`$b=&$a` con `$a` indefinito → NULL, nessun warning).
- **Test scritti:** 4 (write-through bidirezionale, ref→undef=NULL, catena
  `$c=&$b`, `unset` rompe solo l'alias nei due versi).
- **Tempo:** ~35 minuti.

## Step 10 — Espansione builtin per frequenza nei test

- **Riferimento C:** ext/standard (array.c, string.c, formatted_print.c, math.c),
  Zend/zend_operators.c (compare/identical per max/min/in_array).
- **Target:** crates/php-builtins (nuovi moduli `array.rs`, `string.rs`,
  `format.rs`, `math.rs`; `print_r` accanto a `var_dump` in `lib.rs`).
- **Builtin aggiunti (8 commit TDD-isolati, uno per gruppo):**
  - `count`/`sizeof` (incl. `COUNT_RECURSIVE`, TypeError sugli scalari PHP 8)
  - `array_keys` (con `$search`/`$strict`) / `array_values`
  - `in_array` / `array_merge`
  - `implode`/`join` / `explode` (limit ±, multichar)
  - `substr` / `strpos` / `str_replace` (search/replace scalari o array)
  - `sprintf`/`printf` (d/i u f/F e/E s x/X o b c %%, flag `- + 0 '<c>`,
    width, `.precision`, posizionale `%N$`)
  - `abs` / `max` / `min`
  - `print_r` (scalari + array ricorsivo, modalità `$return`)
- **Decisioni applicate:** ABI builtin di Step 5 invariata (`fn(&[Zval], &mut Ctx)`),
  zero modifiche all'evaluator. Coercizioni via `convert::*`, confronti via `ops::*`.
- **Estensioni a php-types (additive, nessuna regressione):**
  - `PhpError::ValueError` — `explode("")`, `strpos` offset fuori range, `max([])`
  - `PhpError::ArgumentCountError` — `sprintf`/`max` con troppi pochi argomenti
  - Entrambe renderizzate via `class_name()`/`message()` esistenti.
- **Round di iterazione AI:** 1 per gruppo (tutti i test verdi al primo run dopo
  RED; unica eccezione il test `printf` riscritto perché usava interpolazione
  `"$n"` non ancora lowered — bug del test, non del builtin).
- **Test pass al primo tentativo:** sì (ogni gruppo verificato prima contro
  l'oracle `/tmp/php-src/sapi/cli/php`, poi TDD RED→GREEN).
- **Scope-out espliciti (debito):**
  - `array_push` e la famiglia by-reference (`sort`, `array_pop`, `array_shift`):
    l'ABI passa gli argomenti per valore e il lowerer rifiuta i parametri `&$x`
    (`lower.rs:367`). Richiede uno step dedicato alle reference.
  - `sprintf` `%g`/`%G` (forma shortest diverge da PHP, raro nel corpus).
  - `str_replace` `$count` by-ref (4° parametro).
- **Divergenze nuove (D-NEW):** nessuna. Ogni builtin combacia byte-per-byte
  con l'oracle in tutti i casi testati.
- **Test scritti:** 44 nuovi test funzionali (totale workspace 131 → 168).
- **Baseline .phpt (corpus completo `Zend/tests` + `tests`, 6172 file):**
  pass 126 → **135** (+9), fail 62 → 64, skip-`builtin` 114 → 103 (gli 11 test
  prima non-eseguibili ora girano: 9 verdi, 2 falliscono su gap *non*-builtin —
  `$GLOBALS` e scrittura su string-offset, ora raggiungibili). Nessuna regressione.
- **Tempo:** ~2h.

## Step 9 — Rendering dei diagnostici e dei fatal (interleaved sullo stdout)

- **Riferimento C:** `main/main.c:1493` (formato `%s: %s in %s on line %d`),
  `Zend/zend_exceptions.c:756` (display di un throwable uncaught).
- **Target:** `crates/php-types/src/diag.rs`, `crates/php-runtime/src/{hir,lower,eval}.rs`,
  `crates/phpt-runner/src/lib.rs`.
- **Round di iterazione AI:** 1 (più triage del corpus + 1 fix Classe A).
- **Test pass al primo tentativo:** sì (7 nuovi test `rendered_*` + 3 nel runner).

### Modello scelto: rendering al punto di occorrenza, non collezione

Fino a step 8 i diagnostici erano *raccolti* in `Outcome.diags` (side channel) e
mai resi su stdout; il phpt-runner skippava ogni test che ne attendesse
(`diag-or-fatal`, ~176 file). PHP invece interleava il diagnostico **nel byte
stream di stdout, nel momento esatto in cui viene sollevato** (verificato con
`od -c` sull'oracle: `\nWarning: {msg} in {file} on line {N}\n`, newline iniziale
e finale; il fatal uncaught chiude lo stream con il blocco
`\nFatal error: Uncaught {Class}: {msg} in {file}:{line}\nStack trace:\n#0 {main}\n  thrown in {file} on line {N}\n`).

Per renderlo fedelmente serve sapere la **linea** di ogni operazione: l'HIR già
porta `line` su ogni `Stmt`/`Expr` (predisposto a step 3), quindi non è servito
alcun cambiamento al front-end se non aggiungere `Program.file` (il nome
sorgente) per la parte `in <file>`.

**Scelta additiva (non distruttiva):** invece di mutare `Outcome.stdout` (che
avrebbe rotto i ~6 test che asseriscono stdout puro + `diags`/`fatal`), ho
aggiunto un nuovo campo `Outcome.rendered`: lo stream CLI-fedele = `stdout` con
diagnostici e fatal interleaved. `stdout`/`diags`/`fatal` restano invariati come
side channel per le asserzioni fine-grained; il phpt-runner confronta contro
`rendered`. Tutti i 122 test preesistenti restano verdi.

### Meccanica (eval.rs)

- `Evaluator` guadagna `file`, `cur_line`, `rendered`, e un watermark
  `diags_rendered` (quanti `diags` sono già stati resi).
- `eval` è ora un wrapper attorno a `eval_inner` che (a) stampa `cur_line =
  e.line`, (b) esegue, (c) `flush_diags()` rende i diag di *questo* livello
  stampati con `e.line` (i sotto-eval hanno già reso i propri). Sul ramo `Err`
  **non** ripristina `cur_line`, così quando il fatal risale al top punta ancora
  alla riga che l'ha lanciato.
- `exec_stmt` analogamente imposta `cur_line = stmt.line` e flush a fine.
- `emit(bytes)` = `flush_diags()` poi scrive su `out` **e** `rendered`: garantisce
  che un warning sia reso *prima* dei byte che lo seguono (es. `echo [1]` →
  `\nWarning: Array to string conversion …\nArray`).
- Path builtin: flush prima, esegui (scrive su `out` via `Ctx`), copia la coda
  fresca di `out` in `rendered`, flush dopo (output-poi-diagnostici).
- `flush_diags()` rende `\n{severity}: {message} in {file} on line {cur_line}\n`;
  `render_fatal()` aggiunge il blocco uncaught in coda a `rendered`.

### phpt-runner

- Rimossi gli skip `diag-or-fatal` e la funzione `expects_diagnostic`; il
  confronto ora è contro `outcome.rendered`. Resta lo skip `builtin` per
  "Call to undefined function" (scope gap reale, non difetto).
- **Nuovo skip `compile-error`**: l'EXPECT che inizia con `Parse error:` o un
  `Fatal error:` *non*-`Uncaught` è una diagnostica **compile-time** del motore
  (validazione attributi/tipi, strictness del parser) che non modelliamo (mago fa
  da front-end). Se non produciamo un fatal corrispondente, skip onesto invece di
  un falso fail. Sposta **104** file da fail→skip motivato.

### Fix Classe A trovato dal corpus

- **null come array offset**: PHP 8.1+ emette `Deprecated: Using null as an array
  offset is deprecated, use an empty string instead` (la chiave resta `""`).
  Mancava in `coerce_key`. Aggiunto (read/write/array-literal); le varianti
  `isset`/`??` passano per `coalesce_index` (free fn silenziosa) e restano fuori
  scope. Regressione: `eval.rs::rendered_null_array_offset_deprecation`.

### Risultato sul corpus completo (tests/ + Zend/, 6172 file)

| | pass | fail | skip | runnable | pass-rate |
|---|---|---|---|---|---|
| fine step 8 | 114 | 2 | 6056 | 116 | 98.3% |
| **fine step 9** | **126** | **62** | **5984** | **188** | **67.0%** |

I diag-test sono ora *runnable* (+72 netti): **+12 pass** (11 diag + 1 null-offset)
e **62 fail** che il corpus ora **espone onestamente** invece di nascondere. Il
crollo del pass-rate è atteso e voluto: prima quei 176 file erano skippati, ora
sono confrontati. La triage dei 62 è in `04-divergences.md` (quasi tutti scope
gap di feature non implementate, non difetti di rendering).

- **Test:** 131 totali (da 122: +6 `rendered_*` in eval, +1 null-offset, +3 nel
  runner, −1 test obsoleto rimpiazzato). Clippy `--all-targets --all-features
  --deny=warnings` pulito.
- **Tempo:** ~2h (incluse verifica oracle byte-level e triage del corpus).

## Step 8 — Funzioni utente (dichiarazione, parametri, return, scope, ricorsione)

- **Riferimento concettuale:** Zend `zend_execute.c` (ZEND_DO_FCALL, frame di
  esecuzione), `zend_compile.c` (hoisting delle dichiarazioni top-level).
  Tradotto come *call-frame swap* nel tree-walker, non come VM.
- **File target:** `hir.rs` (`Program.functions`, `FnDecl`, `Param`),
  `lower.rs` (`hoist_function`/`lower_function`/`lower_function_body`,
  arm `Statement::Function`), `eval.rs` (`call_user_fn`/`run_user_fn_body`,
  resoluzione user-prima-di-builtin nel `Call`).
- **Decisioni di step (Fase 2 locale):**
  - **D 8.1** — `Program.functions: Vec<FnDecl>`; ogni `FnDecl` possiede la
    *propria* slot-table locale (le funzioni PHP non catturano lo scope
    esterno). `Param { slot, default }`, con `params[i].slot == i`.
  - **D 8.2** — **hoisting** delle dichiarazioni top-level: pre-pass su
    `program.statements` che le registra prima di lowerare il body, così una
    call può precedere testualmente la definizione (anche mutua ricorsione).
    La dichiarazione produce `Ok(None)` (nessuno statement runtime).
  - **D 8.3** — solo parametri **by-value posizionali** + default opzionali.
    By-ref (`&$x`), variadici (`...$x`), promoted-property, redeclaration,
    return-by-ref → `Unsupported` (SKIP motivato). Le **type hint** sono
    accettate ma **non enforced** (nessuna coercizione / TypeError) →
    divergenza D-NEW-6 documentata.
  - **D 8.4** — risoluzione `Call`: prima la tabella user (case-insensitive
    ASCII), poi il registry builtin, poi "Call to undefined function". Nuovo
    frame per call (swap di `slots` + `names`, ripristino a fine call);
    ricorsione sullo stack host. Argomenti extra ignorati; troppo pochi →
    fatale `ArgumentCountError`-style.
- **Round di iterazione AI:** 1 (compila e passa al primo tentativo dopo la
  stesura dei test).
- **Errori / scoperte:**
  - [eval-order] L'import .phpt ha fatto diventare *runnable* due test che
    prima erano skip (usano funzioni): `engine_assignExecutionOrder_005/006`.
    Hanno scoperto un **bug reale di step 7** (classe A): `AssignPlace`
    valutava la RHS **prima** degli indici dell'lvalue, mentre PHP valuta gli
    offset del target da sinistra a destra *prima* della RHS. Output invertito
    a coppie (`i5 i6 i3 i4 i1 i2` invece di `i1..i6`). Fix di 1 riga
    (resolve-steps-first), allinea `AssignPlace` a `AssignOpPlace` che era già
    corretto. Regressione: `eval.rs::assignment_evaluates_lvalue_offsets_before_rhs`.
- **Test scritti:** 11 eval (declare+call, hoisting, case-insensitive, scope
  isolato, default, extra-args, missing-arg-fatale, fattoriale, mutua
  ricorsione, fall-off→NULL, type-hint-non-enforced) + 3 lowering (tabella
  hoisting, by-ref/variadic unsupported, conditional-decl unsupported) + 1
  regressione eval-order = **15 nuovi test**. Totale workspace: **122**.
- **Baseline phpt aggiornata:** 6172 file → **114 pass / 2 fail / 6056 skip =
  98.3% dei runnable** (116 runnable, da 72). `unsupported` scende 5215 → 5028
  (−187). I 2 FAIL residui sono entrambi noti: `unicode_escape` (D-NEW-4, mago)
  e `scalar_float_with_integer_default_weak` (D-NEW-6, type-hint non enforced).
- **clippy** `--all-targets --all-features --deny=warnings`: pulito (exit 0).
- **Tempo:** ~1.5h.

## Step 6 — phpt-runner (capability scan + import testsuite, Fase 4c)

> Eseguito DOPO lo step 7 (gli array rendono il runner molto più utile: ~quintuplicano
> i test in-scope). Questo è lo step "Fase 4c — import original testsuite" della
> metodologia, materializzato come **tool ri-eseguibile** invece che come conversione
> one-shot.

- **Target:** nuovo crate `crates/phpt-runner` (lib + bin). Dipende da `php-runtime`
  + `php-builtins` + `regex`. Niente copia della testsuite in repo (licenza PHP):
  il runner punta a `/tmp/php-src` a runtime; le fixture committate sono scritte da noi.
- **Architettura:**
  - `parse_sections`: split del formato `.phpt` (`--NAME--` header `[A-Z_]+`).
  - **Capability scan** (il cuore, mantiene la promessa del doc-comment di `lower.rs`):
    si prova a `lower_source` il `--FILE--`; `LowerError::Unsupported{what,line}` →
    SKIP categorizzato, `Parse` → SKIP. Poi si esegue con `run_source_with(registry)`.
  - **Honest classification**: l'unico **FAIL** è una divergenza di output su uno
    script *clean* (no diag, no fatal). Scope-gap → SKIP con categoria:
    `unsupported` (lowering), `section` (sezioni non modellate: SKIPIF/EXTENSIONS/
    INI/POST/GET/STDIN/ARGS/…), `builtin` ("Call to undefined function"),
    `diag-or-fatal` (warning/fatal non renderizzati su stdout — step 9; include
    l'euristica "l'EXPECT contiene `Warning:`/`Deprecated:`/… → skip"), `parse`,
    `expectregex`, `expectf-%r`, `malformed`.
  - **Matcher**: `--EXPECT--` esatto (CRLF→LF + trim); `--EXPECTF--` → regex
    (`%d %s %S %a %A %w %i %x %f %c %e`, fedele a run-tests.php); `--EXPECTREGEX--`
    e `%r` → skip.
  - **CLI** (`phpt-runner [--list-fails] <path>...`): walk ricorsivo (skip dei
    dotfile `._*` AppleDouble macOS), summary con breakdown skip-by-category e
    pass-rate dei runnable; exit code ≠ 0 sse c'è un FAIL. Il lavoro gira su un
    **thread con stack da 1 GiB**: il front-end recursive-descent (mago) e il
    tree-walker ricorsivo overfloano lo stack di default su test patologici
    (es. `Zend/tests/bug64660.phpt`, migliaia di `[` annidate) — ora gestiti.
- **Run completo (`tests/` + `Zend/tests/`, 6172 file):** **71 pass, 1 fail,
  6100 skip → 98.6% dei runnable (71/72)**. Breakdown skip: unsupported 5215,
  section 660, builtin 88, parse 67, diag-or-fatal 59, malformed 6, expectregex 4,
  expectf-%r 1.
- **Bug reali trovati dall'import (classe A, fixati qui — vedi `04-divergences.md`):**
  - **D-NEW-2 (bug #69889):** `??` su offset di stringa restituiva `""`/char errato
    invece di "not set" → fix `coalesce_index`/`coerce_key_silent`/`string_offset_silent`
    in `eval.rs` (path `??` separato da quello di `isset()`-construct, che era già corretto).
  - **D-NEW-3 (bug #74947):** literale intero gigante → `~1.8e19` (valore clampato da
    mago a `u64::MAX`) invece di `INF` → fix `lower_int` ri-parsa il testo decimale grezzo.
  - **D-NEW-4 (classe D, ereditata):** mago 1.30 non decodifica `\u{...}` nelle stringhe
    doppie → unico FAIL residuo, documentato (non correggibile a valle).
- **Fix collaterale (corretto inline, fedele al lexer Zend):** `?>` mangia un singolo
  newline (`\n`/`\r\n`) dell'inline-HTML che segue → `lower.rs::strip_one_newline` +
  flag `after_closing_tag` (sblocca tutti i test con `?>\n…`, es. bug44654).
- **Verifica:** `cargo test` **107/107** verde (era 94; +11 phpt-runner: parser,
  matcher, le 6 regole di classificazione, walker su fixtures + 2 regressioni dei bug);
  clippy `--all-targets --all-features --deny=warnings` pulito.
- **Out-of-scope (debito):** rendering diagnostici (step 9, sblocca ~60 test
  `diag-or-fatal` + l'euristica diventa esatta); `--EXPECTREGEX--`/`%r`; sezioni
  I/O/INI; decodifica `\u{}` (a monte in mago); guard di ricorsione esplicito
  (oggi mitigato dallo stack da 1 GiB).
- **Tempo:** ~2.5h.

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

---

## Step 39 — Generators (`yield`)

- **File originale:** Zend/zend_generators.c (~1500 LOC), zend_compile.c (detezione
  generatore), Zend/zend_execute.c (ZEND_GENERATOR_*).
- **File target:** `php-types/src/generator.rs` (GenState/GenStatus/GenKey/GenStep/
  GenDriver), `php-runtime/src/eval.rs` (GenDriverImpl, make_generator,
  resume_generator, generator_method, gen_suspend, eval_yield_from,
  foreach_generator), `php-runtime/src/hir.rs` (ExprKind::Yield/YieldFrom,
  FnDecl.is_generator), `php-runtime/src/lower.rs` (lowering yield + flag
  fn_saw_yield), `php-builtins/src/lib.rs` (var_dump/print_r).
- **Motore:** `corosensei` 0.3 (`Coroutine`, non `ScopedCoroutine` — vedi metrics
  D-GEN-1). Stackful: il `yield` sospende la ricorsione nativa di `eval()`.
- **Round di iterazione AI:** ~1 per sub-step (8 sub-step). Build-error driven per
  i match esaustivi su Zval (5 in convert/ops, 4 in eval, 1 in differential test).
- **Test pass al primo tentativo:** sì per 39-2..39-7 (l'infra 39-1 li copriva);
  39-1 al primo build verde dopo la chiusura dei match non-esaustivi.
- **Test scritti:** 22 unit (eval.rs) + 2 (builtins) — tutti oracle-verificati.
- **Errori incontrati:**
  - [layering] `Zval::Generator` in php-types non può nominare Evaluator/corosensei
    → type-erasure dietro `GenDriver` + `*mut ()`.
  - [lifetime] `Coroutine: 'static` vs `Evaluator<'p>` → cancellazione del lifetime
    (riborrow `Evaluator<'static>`), unsafe confinato e documentato.
  - [borrow] driver e corpo vogliono lo stesso `&mut Evaluator` → passato via
    `resume(*mut ())`, guard di non-rientranza per-generatore.
  - [bug corpus] closure-generator non passava da `call_user_fn` → aggiunto branch
    in `call_closure`. getReturn non auto-primava → `ensure_started`.
- **Differenze idiomatiche dalla mappa Fase 2:** D-GEN-1 raffinato (Coroutine vs
  ScopedCoroutine); swap-contesto confinato in `GenDriverImpl::resume` invece che
  in helper sull'Evaluator (php-types resta pulito).
- **Tempo:** sessione dedicata (lo step più complesso finora).

## Step 40 — Argument unpacking / spread `f(...$arr)`

- **File originale:** Zend/zend_compile.c (check compile-time
  `zend_compile_args` — "positional after unpacking" / "unpacking after named"),
  Zend/zend_execute.c (unpacking SPREAD + `zend_handle_named_arg` a runtime).
- **File target:** `php-runtime/src/hir.rs` (`ExprKind::Spread`),
  `php-runtime/src/lower.rs` (`lower_args` — wrapping spread + ordering fatals),
  `php-runtime/src/eval.rs` (`expand_spread`, `place_named_arg`,
  `apply_named_args`, `eval_call_args`/`eval_value_args` ridisegnati,
  `Arg::Named`, `bind_params` variadic keyed, `reject_named`).
- **Strategia:** estensione del modello step-38 (positional `Vec<Arg>` + named
  trailing). Un `ExprKind::Spread(Box<Expr>)` "finto" vive solo come elemento di
  arg-list (mai valutato dal match generico → errore). L'espansione è **two-phase**
  (espandi → piazza), uniforme su Call/New/MethodCall/StaticCall.
- **Round di iterazione AI:** ~1; build-error driven per i call-site della firma
  cambiata (`eval_*_args` ora ritorna `(positional, SpreadNamed)`).
- **Test pass al primo tentativo:** 18/20 spread + 3/3 named-into-variadic. I 2
  fail erano **bug dei test** (usavano `count()`/`array_sum()`, builtin non
  implementati) — riscritti con `foreach` manuale.
- **Test scritti:** 23 (20 spread + 3 named-into-variadic), tutti oracle-verificati.
- **Sub-step:** 40-1a lowering+compile-fatals · 40-1b runtime spread (Call) ·
  40-1c New/Method/Static · 40-2 named-into-variadic (`Arg::Named` collezionato
  con chiave string dalla branch variadic di `bind_params`).
- **Errori/decisioni:**
  - [chiavi int] il *valore* della chiave int è ignorato: appese posizionalmente
    in ordine d'iterazione (oracle `[5=>'x',2=>'y',9=>'z']` → x,y,z).
  - [ordering] int-key dopo string-key durante l'unpacking → `Error` catchable.
  - [type] spread di non-array/non-Traversable → `TypeError`.
  - [generatori] spread di Traversable iterato via `cur_key`/`cur_val` (chiave
    `Zval::Str` → named, altrimenti posizionale).
  - [clippy] gate `--all-features --all-targets --deny=warnings` ha fatto
    emergere 3 lint **pre-esistenti** (step 39 `mem_replace_option_with_some` ×2,
    step 18 `too_many_arguments` su `push_closure`, step 37 test `_D_` non
    snake_case) — sistemati en passant (idioma `Option::replace`, `#[allow]`).
- **Differenze idiomatiche dalla mappa:** nessuna nuova D-G; riusa il binding
  step-38. `SpreadNamed` type-alias per il tipo di ritorno composto.
- **Tempo:** ~mezza sessione.

## Tooling hardening — depth-guard + phpt-runner isolation

Step non-funzionale (DevEx/stabilità), nato dalla review esterna `analysis_results.md`
(punti 1A + 3B). Nessun cambio di semantica osservabile; +2 test.
- **Oracle**: ricompilato `/tmp/php-src` con `--enable-mbstring` (richiede oniguruma,
  installato via `brew install oniguruma`; `pkg-config` assente → passati
  `ONIG_CFLAGS`/`ONIG_LIBS` espliciti). Ora `mb_strlen`/`mb_strtoupper`/`mb_substr`/
  `mb_convert_encoding` disponibili → **sblocca la validazione differential di mb_***
  (era BLOCCATO senza oracle mbstring). Configure preservata: `--disable-all
  --enable-cli --disable-cgi --disable-phpdbg --without-pear --enable-mbstring`.
- **1A — depth-guard** (`eval.rs`): l'evaluator ricorre sullo stack nativo (Rust non
  protegge da overflow) → ricorsione runaway = SIGABRT del processo host. Nuovo
  `MAX_CALL_DEPTH = 25_000` + `guard_call_depth()` ai due ingressi che spingono un
  frame (`call_user_fn`, `invoke_method_args`); supera la soglia → `Error` catchable
  "Maximum call stack depth of 25000 exceeded" invece del crash. **Calibrato
  empiricamente** sullo stack da 1 GiB del worker del runner (overflow nativo misurato
  ~38k frame; 25k = margine ~35%, e ben oltre qualsiasi ricorsione realistica).
  Test (`deep_recursion_yields_clean_error_not_host_crash`) gira su un thread da 1 GiB
  (proietta il fatal a `String` perché `PhpError`/`Zval` sono `Rc`-based, non `Send`).
  **Scope-out**: la ricorsione di **closure** non passa da quei due ingressi (path
  proprio, non pusha `call_stack`) → non guardata da 1A; coperta da 3B. Su stack
  piccoli l'overflow nativo può precedere il guard (presuppone un worker ampio).
- **3B — isolamento `--isolate`** (`phpt-runner/main.rs`): flag opt-in (il path
  in-process veloce resta default). In modalità isolata il parent enumera i `.phpt`
  (`collect_phpt` reso `pub`) e per ognuno fa spawn di un figlio `self --run-one <path>`
  che esegue il singolo test su un worker da 1 GiB e serializza il risultato
  (`STATUS\tCATEGORY\n` + detail). Un figlio che muore (signal da overflow, o panic)
  → exit non-success → registrato come **un FAIL "isolated worker crashed (signal …)"**
  invece di abortire l'intero batch. Verificato: la ricorsione di closure (crasher
  non coperto da 1A) senza `--isolate` dà exit 134 (batch abortito), con `--isolate`
  il batch completa (test successivi eseguiti, crash contenuto). Test d'integrazione
  `tests/isolation.rs` (via `CARGO_BIN_EXE_phpt-runner`, fixture in tempdir).
- **Tempo:** ~mezza sessione (gran parte sulla ricompilazione oracle + calibrazione).

## Step 41 — mbstring batch 1 (funzioni stringa UTF-8 code-point)

Primo batch di `mb_*`, sbloccato dalla ricompilazione oracle con mbstring. Design
pass: `diary/NEXT-mbstring.md`. Pattern builtin PURO (modulo
`php-builtins/src/mbstring.rs`, ABI `fn(&[Zval],&mut Ctx)`, zero modifiche
all'evaluator), come step 17/29. **+18 test** oracle-verificati (734→752).
- **23 funzioni in 4 sotto-step**: mb-1 `mb_strlen`/`mb_substr`/`mb_str_split`
  (+ helper `units`); mb-2 `mb_strtoupper`/`mb_strtolower`/`mb_convert_case`
  (UPPER/LOWER/TITLE/FOLD + alias SIMPLE)/`mb_ucfirst`/`mb_lcfirst`; mb-3
  `mb_strpos`/`stripos`/`strrpos`/`strripos`/`strstr`/`stristr`/`strrchr`/
  `strrichr`/`mb_substr_count`; mb-4 `mb_ord`/`mb_chr`/`mb_str_pad`/`mb_trim`/
  `mb_ltrim`/`mb_rtrim`/`mb_check_encoding`. Costanti `MB_CASE_*` aggiunte a
  `resolve_constant` (lower.rs).
- **Scoperta abilitante (D-MB3)**: il case-mapping Unicode di Rust std
  (`char::to_uppercase`/`to_lowercase`) **combacia con PHP** anche sui casi
  difficili (`ß→SS`, `ı→I`, `İ→i̇` 2 cp, final-sigma `ς→Σ`) → mb-2 quasi
  interamente std-backed, zero tabelle. `str::chars().count()` = `mb_strlen`.
- **Helper**: `units` (decode lenient: scalare UTF-8 valido = 1 unità, byte
  invalido = 1 unità → `mb_strlen("a\xFF\xFEb")==4` come oracle); `cps`
  (char + byte_start/len per offset↔byte); `fold` (case-fold semplice per
  ricerca case-insensitive).
- **Encoding (D-MB1)**: solo UTF-8 (+ alias UTF8/US-ASCII/ASCII). Encoding
  diverso → `ValueError` "must be a valid encoding, "X" given" (oracle-esatto).
- **Errori RED dei test** (non bug d'impl): `var_export()`/`count()`/`array_sum()`
  NON sono builtin implementati → riscritti con `var_dump`/`implode`.
- **Divergenze dichiarate (scope-out, in `04-divergences.md` sez. mbstring)**:
  encoding non-UTF-8 *validi* riportati come invalidi (D-MB1, serve `encoding_rs`);
  `mb_convert_case` TITLE non onora le Case_Ignorable Unicode (apostrofo:
  `o'brien`→noi `O'Brien` vs PHP `O'brien`); FOLD ≈ `to_lowercase`; `*_SIMPLE`
  trattati come full; offset sul ramo reverse di `mb_strrpos` non gestito;
  rendering byte invalidi (il conteggio/offset è corretto). Famiglia `mb_ereg*`
  (oniguruma), `mb_convert_encoding`/`detect`/`strwidth` → batch futuri.
- **Corpus** `ext/mbstring/tests` (420): **417 tutti SKIP categoria "section"**
  — il phpt-runner scarta a monte i test con `--EXTENSIONS--` (397), `--SKIPIF--`
  (123), `--INI--` (120). NON è una regressione né un difetto delle funzioni:
  la validazione differenziale è fatta dai 18 unit test (ogni atteso preso
  dall'oracle ricompilato). Rilassare `--EXTENSIONS--` per le estensioni
  supportate è un item tooling **cross-cutting** a sé (sbloccherebbe anche
  ext/standard ecc.) — non incluso qui.
- **Tempo:** ~mezza sessione.

## Step 42 — mbstring batch 2A (encoding + width)

Secondo batch `mb_*` (traccia A encoding + traccia B width). Pattern builtin
PURO come step 41, zero modifiche all'evaluator. **+8 test** oracle-verificati
(752→760). Unica nuova dipendenza: `encoding_rs = "0.8.35"` (pure-Rust, no C).
Traccia C (oniguruma `mb_ereg*`/`mb_split`) rinviata a uno Step 43 dedicato
(richiede FFI alla libreria C, fuori dal pattern pure-Rust del batch).
- **5 funzioni in 2 sotto-step**:
  - **42b width** `mb_strwidth`/`mb_strimwidth`/`mb_strcut`. Tabella EAW
    (`FIRST_DOUBLEWIDTH=0x1100` + 124 range) **portata verbatim** da
    `ext/mbstring/libmbfl/mbfl/eaw_table.h`; `character_width()` binary-search →
    2 se in tabella, 1 altrimenti. `mb_strcut` è **byte-oriented** (start
    arrotonda giù al confine del char che lo contiene; length dal rounded start;
    include solo char interi che ci stanno). `mb_strimwidth`: start in
    **code-point**, marker conta verso il limite, out-of-range→`ValueError`.
  - **42a encoding** `mb_convert_encoding`/`mb_detect_encoding`. `enum Codec`
    {Ascii,Utf8,Latin1,Utf16Be,Utf16Le,Rs(&Encoding)} + `resolve_encoding`
    (canonical PHP name per detect). `decode_bytes`/`encode_str` (substitute
    `?`=0x3F char-per-char, NON entità HTML); `validates` per detect.
- **Scoperte abilitanti**:
  - **`unicode-width` è SBAGLIATO** per `mb_strwidth`: PHP/mbfl dà width 1 a
    combining/zero-width/control (unicode-width dà 0). Solo la tabella EAW
    portata riproduce l'oracle → zero crate width esterni.
  - **`ISO-8859-1` ≠ `encoding_rs`**: la label WHATWG `iso-8859-1` mappa a
    windows-1252 (`\x80`→€). PHP usa true Latin-1 (`\x80`→U+0080) → Latin1
    hand-rolled. Idem UTF-16 (encoding_rs non *codifica* UTF-16) → hand-rolled.
- **Encoding (D-MB1 invariato)**: le funzioni batch-1 e le width restano
  UTF-8-only; solo `mb_convert_encoding`/`mb_detect_encoding` accettano encoding
  arbitrari. `mb_list_encodings`/`mb_encoding_aliases` non implementate (mbfl ne
  elenca ~79, nessun driver dal corpus).
- **Errori RED dei test** (non bug): `bin2hex` NON è implementato → aggiunto
  helper `out_bytes()` per asserzioni byte-esatte; risultati `mb_strcut`
  (char interi) confrontati via echo diretto.
- **Divergenze dichiarate** (`04-divergences.md` sez. Step 42): D-MB-enc-latin1
  (parità), D-MB-enc-subst (parità), D-MB-enc-utf16 (parità), D-MB-enc-list
  (scope-out), D-MB-enc-htmlent (scope-out), D-MB-enc-detect (approssimazione),
  D-MB-width-eaw (parità), D-MB-width-enc (dichiarata), D-MB-strimwidth-neg
  (scope-out). **D-NEW: nessuna.**
- **Clippy** strict gate (`--all-features --all-targets --deny=warnings`) pulito.
- **Tempo:** ~mezza sessione.

## Step 43 — mbstring batch 2B (famiglia regex `mb_ereg*`)

Chiude mbstring (traccia C). **Primo step del batch che tocca il core
dell'evaluator** (41/42 erano pure builtins): la famiglia ha stato persistente
e out-param by-ref all'argomento #3, fuori dall'ABI builtin. Strategia **Adapter**
(legacy-port Strategy A) su **oniguruma reale** via crate `onig 6.5.3`. **+9 test**
oracle-verificati (760→769). ~16 funzioni.
- **Gate 0 (build feasibility)**: `onig`/`onig_sys` compila la libreria C
  oniguruma *bundled* via `cc` + genera i binding con `bindgen`/libclang →
  **build pulito in ambiente** (clang presente). Nessun pkg-config richiesto.
- **Architettura**: nuovo `php-runtime/src/mbregex.rs` (adapter `onig` confinato:
  `MbRegexState`, `compile`, `exec`, `replace`, `split`, `find_all`,
  `matches_at_start`, `search_from`; ritorna `Zval`/byte owned, nessun borrow
  `onig` esce). Campo `mb_regex: MbRegexState` sull'`Evaluator` (precedente:
  `statics`/`static_props`); le funzioni sono **higher-order builtins** in
  `eval.rs` (mirror di `ho_preg_match` + `write_out_param`), così accedono allo
  stato e scrivono `$regs` (arg #3). `GenCtx` è un save/restore di *sottoinsieme*
  → `mb_regex` resta condiviso, niente scope-out per i generatori.
- **Dialetto**: PHP mbregex usa di default **Ruby syntax + opzioni `"pr"`**
  (`p` = MULTILINE|SINGLELINE: `.` matcha newline, `^`/`$` ancorano la stringa).
  `parse_options` traduce la stringa opzioni PHP (i/x/m/s/p/l/n + selettori
  syntax r/z/d/b/j/u/g/c) in `RegexOptions`+`Syntax`. Classi POSIX `[[:digit:]]`,
  named group `(?<n>)`, backref `\1` funzionano (verificati vs oracle).
- **43a** (stateless + stato globale): `mb_ereg`/`mb_eregi` (return **bool** PHP 8;
  `$regs` arg #3: 0=match, 1..=gruppi con **`false`** per gruppo non
  partecipante, named appesi per chiave stringa; no-match→false+`$regs=[]`),
  `mb_ereg_replace`/`mb_eregi_replace` (backref `\0`-`\9`, `\\`→`\`),
  `mb_ereg_replace_callback` (callable PHP), `mb_split` (campi vuoti preservati,
  limite), `mb_ereg_match` (ancorato all'inizio, non full-match),
  `mb_regex_encoding`/`mb_regex_set_options` (getter→"UTF-8"/"pr").
- **43b** (cursore stateful): `mb_ereg_search_init/search/search_pos/search_regs/
  search_getregs/search_getpos/search_setpos`. Cursore in byte su `MbRegexState`;
  `mb_search_step` prende il `Regex` con `Option::take` (non è `Clone`), avanza a
  `end` (o `end+1` per match zero-width). `regs_from_region` costruisce `$regs`
  dalle posizioni assolute del match.
- **Warning su pattern invalido**: `Diag::Warning "{func}(): mbregex compile err:
  {msg}"` (messaggio oniguruma), return false. NB lo stdout dei test è `ev.out`
  puro (i Warning vanno sul canale renderizzato) → i test vedono solo `false`.
- **Errori RED**: nessun CLI standalone (php-cli è stub `fn main(){}`) → niente
  spot-check via binario; la validazione differenziale resta gli unit test
  oracle-derivati (come step 41/42) + i probe oracle manuali.
- **Divergenze** (`04-divergences.md` sez. Step 43): D-MB-ereg-enc (UTF-8-only,
  scope-out coerente D-MB1), D-MB-ereg-syntax (opzioni avanzate/encoding non
  validati a fondo). **D-NEW: nessuna.**
- **Clippy** strict gate pulito. **Tempo:** ~una sessione.

## Step 44 — phpt-runner `--EXTENSIONS--` relax + import corpus ext/mbstring (Phase 4c)

Step **tooling + validazione** (metodologia legacy-port Phase 4c "import original
testsuite"): finora mbstring (41/42/43) era validato SOLO da unit test scritti a
mano, perché il phpt-runner scartava a monte ogni test con `--EXTENSIONS--`.
Questo step lo sblocca selettivamente e fa girare il corpus reale `ext/mbstring/
tests` contro la nostra implementazione. **+4 test** (769→773). Commit unico.
- **Gating selettivo** (`phpt-runner/src/lib.rs`): `EXTENSIONS` rimosso da
  `UNSUPPORTED_SECTIONS`; nuovo `SUPPORTED_EXTENSIONS` = `[core, standard,
  mbstring, pcre, json, date]` (le estensioni che modelliamo davvero). Un test
  gated su sole estensioni supportate ora **gira**; altrimenti SKIP categoria
  `extension`. I test che usano funzioni non implementate restano SKIP `builtin`
  (non FAIL) → i FAIL sono **divergenze reali**, non gap di funzioni.
- **Sblocco**: 163 test mbstring-only diventano raggiungibili (erano 20 runnable).
  Test runner.rs aggiornati (supported→runs, unsupported→skip `extension`),
  fixture `skip_section.phpt` json→intl.
- **Run corpus** (`--isolate`): 417 totali → **30 pass / 37 fail / 350 skip**
  (350 skip = 252 section [SKIPIF/INI] + 70 builtin + 28 unsupported; 67 runnable,
  pass-rate 44.8%).
- **3 BUG CLASSE A trovati e FIXATI** (in `php-builtins/src/mbstring.rs`, surfacing
  dal corpus, +3 unit test oracle-verificati): (1) `mb_strpos/stripos/strrpos/
  strripos` con `$offset` fuori da `[-len,len]` → ora `ValueError "Argument #3
  ($offset) must be contained in argument #1 ($haystack)"` (prima clampava
  silenziosamente); (2) `mb_detect_encoding($s, '')` e (3) `mb_convert_encoding($s,
  to, '')` con lista encoding stringa-vuota → ora `ValueError "...must specify at
  least one encoding"` (`parse_enc_list` filtra le voci vuote → `''` = zero
  encoding; convert distingue from-null=UTF-8 da from-vuoto=errore). Pass 27→30.
- **37 fail residui = scope-out dichiarati** (non bug): ~21 encoding non-UTF-8
  (D-MB1: EUC-JP/SJIS/cp936/UTF-16/HTML-ENTITIES/UTF7), case fold/sigma/apostrofo
  (D-MB3a/b/c), funzioni config non implementate (mb_internal_encoding/
  detect_order/substitute_character/convert_kana). **2 D-NEW documentati** (vedi
  04-divergences): mb_convert_encoding/check_encoding **array input** (conversione
  ricorsiva, gap di feature) e **mb_convert_case TITLE titlecase** (usiamo
  uppercase: digrammi Dž/Lj/Nj U+01C4 invece di U+01C5 titlecase; Rust std non ha
  `to_titlecase`).
- **Headline metrics SALVE**: il "37.835 casi a 0 mismatch" è il differential
  OPERATORI (step 2), NON il corpus phpt; il phpt-runner è uno strumento
  informativo (no gate CI). `php-cli` resta stub. **Clippy** strict gate pulito.

## Step 45 — `goto` + label

Ultima feature di control-flow mancante. Il parser **mago la riconosce già**
(`Statement::Goto`/`Statement::Label`) ma `lower.rs` la scartava nel catch-all
`LowerError::Unsupported` → i 10 test `Zend/tests/*goto*` erano SKIP. **+14 unit
test** (773→787), tutti oracle-verificati. Clippy strict pulito.

- **HIR** (`hir.rs`): 2 varianti `StmtKind::Label(Box<[u8]>)` (marker no-op) e
  `StmtKind::Goto(Box<[u8]>)`.
- **Lowering** (`lower.rs`): 2 arm `Statement::Goto/Label` (la `LocalIdentifier.
  value` dà i byte della label).
- **Runtime** (`eval.rs`): nuova variante `Flow::Goto(Box<[u8]>)`. `exec_stmts`
  rifattorizzato da `for` a **`while`+indice** così un goto può ri-entrare a un
  indice diverso: se la label è in questo blocco salta (`i = j; continue`),
  altrimenti **propaga su** (`return Ok(Flow::Goto)`). Il destructor-sweep tra
  statement è preservato. `loop_step` e lo `switch` aggiungono l'arm
  `Flow::Goto(l) => propaga` (= un goto esce naturalmente da loop/switch). `Label`
  → no-op, `Goto` → `return Ok(Flow::Goto)`. **Try/finally**: il path generico
  già esistente (`flow => flow` → il `finally` gira sempre, poi propaga) gestisce
  `Flow::Goto` **senza modifiche** — un goto che esce dal `try` fa girare il
  `finally` prima del salto (caso `finally_goto_005`), esattamente come PHP.
- **Validazione compile-time** (`lower.rs`, `validate_goto` su ogni scope di
  funzione: body globale + ogni `lower_function`/`lower_method`/`lower_closure`).
  PHP rileva 3 errori **a compile time** (nessun output parziale), riprodotti come
  `LowerError::Fatal` (reso senza output, identico all'oracle):
  - `'goto' to undefined label 'X'`;
  - `Label 'X' already defined`;
  - `'goto' into loop or switch statement is disallowed` **e** (scoperto dal
    corpus, barriera distinta) `jump into a finally block is disallowed`.
  La legalità dell'into-jump è decisa da **stack di barriere**: ogni loop/
  `switch`/`finally` riceve un id; un `Label`/`Goto` registra lo stack di id che
  lo racchiude; un goto raggiunge la label sse lo stack della label è **prefisso**
  di quello del goto (ogni barriera attorno alla label racchiude anche il goto).
  `if`/`try`-body/`catch`/blocchi nudi sono **trasparenti** (PHP-fedele: il goto
  può entrarci).
- **Scope-out D-45.1**: il tree-walker non può atterrare a **metà** di un blocco
  trasparente, quindi un goto che salta *dentro* un `if`/`try`-body/`catch`/blocco
  (PHP-valido ma raro, mai nel corpus) non è supportato. Per non fallire in
  silenzio, un `Flow::Goto` che sfugge al body di funzione / top-level diventa un
  errore deterministico (`unsupported_goto`, "D-45.1"). I salti same-block e
  out-of-block (tutti i casi del corpus + i comuni) funzionano.
- **Corpus** `Zend/tests/*goto*` (10): **5 PASS** (finally_goto_001/002/003/004,
  goto_in_foreach), **5 SKIP** non-goto (finally_goto_005 = `print` non
  implementato; 4× `exit/define_goto_label_*` = **Parse error** atteso su parola
  riservata `die`/`exit` usata come label → strictness del parser, non modellata),
  **0 FAIL**.
- **phpt-runner — 2 fix di fedeltà** (sbloccano 001/002/004, prima FAIL solo per
  cosmetica dell'harness): (1) run-tests.php gira ogni test con
  `fatal_error_backtraces=Off`, quindi un `Fatal error:` semplice **non** ha la
  coda `Stack trace:\n#0 {main}` che il nostro engine aggiunge sempre → quando
  l'EXPECTF non contiene `Stack trace:` la togliamo dal nostro output (gated, mai
  tocca le eccezioni `Uncaught` che la traccia ce l'hanno per davvero; monotòno:
  può solo trasformare falsi-FAIL in PASS). (2) Il runner ora nomina lo script
  col **path reale `.php`** (`php_script_name`) invece dell'hardcoded `test.phpt`,
  così gli EXPECTF che incorporano il basename (`%sfinally_goto_001.php`)
  combaciano (run-tests usa un file temp `<test>.php`).

## Step 46 — costrutti di linguaggio: `print` + `exit`/`die`

Tre costrutti molto comuni che cadevano nel catch-all `Construct`
(`"language construct"`) di `lower.rs`. **+12 unit test** (787→799),
oracle-verificati, clippy strict pulito.

- **HIR** (`hir.rs`): `ExprKind::Print(Box<Expr>)` e `Exit(Option<Box<Expr>>)`
  (entrambi *espressioni*; `print` ritorna `int(1)`, `exit`/`die` non ritornano).
- **Lowering** (`lower.rs`): 3 arm `Construct::Print/Exit/Die` (`die` = alias
  esatto di `exit`) + helper `lower_exit_arg` (0/1 argomento posizionale).
- **Decisione di canale**: `exit`/`die` sono espressioni → si propagano via
  **`Err(PhpError::Exit(u8))`**, NON via `Flow` (un'espressione non può
  ritornare un `Flow`). Vantaggio: il `?` esistente la propaga fino al top, e
  niente modifiche a `Flow`/`loop_step`/`switch`.
- **Runtime** (`eval.rs`): `ExprKind::Print` → `emit(stringify) ; Long(1)`.
  `ExprKind::Exit` → `Err(PhpError::Exit(code))`. Nuovo campo
  `Outcome.exit_code: Option<u8>` (`None` = script completato senza `exit`); arm
  in `run` che tratta `Err(Exit)` come terminazione pulita (NON un fatal).
  `handle_thrown`: `Exit` passa attraverso → **non catchable** (un `catch` non lo
  vede mai).
- **`exit` NON fa girare i `finally`** (verificato con oracle: `try { exit; }
  finally { … }` NON esegue il finally — a differenza di `return`/`throw`). Quindi
  il try handler intercetta `Err(Exit)` **prima** del finally e propaga subito.
- **Coercion `string|int $status`** (`exit_status` + `exit_type_error`,
  oracle-verificata): `int` → exit code; `bool`/`float`/`null` → coerciti a int
  code (`true`→1, `1.9`→1, `null`→0) via `to_long_cast`, **nessun output**;
  `string` e oggetto con `__toString` → **messaggio** stampato, code 0; `array` o
  oggetto non-stringabile → `TypeError "exit(): Argument #1 ($status) must be of
  type string|int, X given"` (catchable, distinto dalla terminazione `exit`).
  Codice normalizzato a `0..=255` (`exit(256)`→0, `exit(-1)`→255).
- **Corpus**: `finally_goto_005` ora **PASS** (era SKIP, sbloccato da `print`);
  `Zend/tests/exit` `die_string_cast_exception`/`define_class_members_exit_die`
  **PASS**. Unico FAIL residuo `exit_as_function` = sintassi first-class-callable
  `exit(...)` + reflection dei parametri Closure in `var_dump` (gap pre-esistente,
  estraneo alla semantica di `exit`).
- **Scope-out D-46.1**: i Deprecated notice di PHP sulla coercion (float→int
  loses precision, null→`string|int` deprecated) non sono emessi; l'exit code è
  comunque corretto. `eval`/`include`/`require` restano `Unsupported`. Il codice
  di uscita resta su `Outcome.exit_code` (la CLI è uno stub, niente
  `process::exit`).

## Step 47 — `var_export` + reflection (`get_object_vars`, `get_class_methods`)

Tre builtin di introspezione/debug fra i più richiesti dal corpus. **+14 unit
test** (799→813… al netto: workspace a 812), oracle-verificati, clippy pulito.

- **`var_export`** (builtin PURO in `php-builtins/src/lib.rs`): port di
  `php_var_export_ex`. Modalità return col 2° arg (pattern di `print_r`).
  Indentazione esatta (membri array a `level+1`, oggetti a `level+2`,
  prefisso/chiusura a `level-1`, ricorsione a `level+2`). Float via
  `dtoa::double_to_shortest` + regola `.0` (sempre un literal float valido:
  `1.0`, `-0.0`, `1.0E+20`, `INF`/`NAN`). Stringhe single-quoted, escape solo
  `'`/`\`; un **byte NUL** non può stare in una single-quote → split su NUL e
  join con `. "\0" .` (`'' . "\0" . 'Hi'`). `stdClass`→`(object) array(...)`,
  user→`\Class::__set_state(array(...))` (tutte le prop by value). Riferimento
  **circolare** → `Warning: var_export does not handle circular references` +
  `NULL` (emesso via `ctx.diags`; `export_into` prende `&mut Diags`).
- **`get_class_methods`/`get_object_vars`** (introspezione in `eval.rs`,
  famiglia `dispatch_class_introspection` accanto a `get_class`): hanno bisogno
  della class-table → non possono essere builtin puri. **Scope-aware**: filtrano
  per `visible_from(vis, decl_class)` rispetto a `self.cur_class` → da global solo
  `public`, da dentro la classe anche `protected`/`private`. `get_class_methods`
  cammina la chain `parent` child→parent, ogni nome una volta (la classe più
  derivata vince: il nome è marcato `seen` **anche se non visibile**, così un
  metodo astratto/omonimo del genitore non "filtra" — fix scoperto da
  `bug32296`). `get_object_vars` itera `props` con `resolve_prop_decl` per la
  visibilità; prop dinamiche/non dichiarate = public.
- **HIR**: nuovo campo `ClassDecl.abstract_methods: Vec<Box<[u8]>>` (i metodi
  astratti — interfacce/`abstract` — non hanno body, quindi non erano in
  `methods`; ora memorizzati così `get_class_methods` li riporta). Popolato nei 3
  siti di costruzione (interface = i suoi metodi; class = gli astratti non
  implementati; enum = vuoto). Sbloccato `get_class_methods` su interfacce
  (corpus `get_class_methods_001/002/003`, `bug32296`, `bug43483` PASS).
- **Corpus**: `Zend/tests/get_class_methods` 5/6 PASS (FAIL solo `bug64239_1` =
  ordine dei metodi alias di trait, ortogonale); `ext/standard/.../general_functions`
  var_export 7 PASS / 0 FAIL fra i runnable; `class_object` get_object_vars con
  edge di reference-aliasing FAIL (scope-out D-47.2).
- **Scope-out D-47.1**: un metodo `abstract protected` mai overridden e
  interrogato da global è riportato (lo trattiamo come public); raro.
  **D-47.2**: `get_object_vars` su proprietà-riferimento — l'aliasing fine nel
  var_dump dell'array risultante diverge in casi limite. `var_export` di
  closure/generator → `NULL`.

## Step 48 — micro-step (runner breakdown) + dynamic class references + `@`

Tre sotto-step coesi (commit separati). **+9 unit test** (812→821), clippy pulito.

### 48a — micro-step: breakdown dei costrutti non supportati (commit `344bc69`)
Il catch-all di lowering riportava un generico `"expression"`/`"statement"`. Ora
`expr_variant_name`/`stmt_variant_name` riportano il tipo di nodo mago
(`expr:Instantiation`, `stmt:...`). Il phpt-runner aggrega due breakdown nel
summary — **"unsupported by construct"** e **"missing builtins"** (top 20) — sia
in-process sia in `--isolate` (sopravvive ai test patologici). Strumento per
guidare data-driven la scelta dei prossimi costrutti/builtin.

### 48b — dynamic class references (commit `fdafb4c`)
Nuova variante `ClassRef::Dynamic(Box<Expr>)`. `class_ref_of` (ora **metodo** del
Lowerer) lowera qualunque espressione in posizione-classe non statica. A runtime
`resolve_class_ref` (ora `&mut self`) valuta l'espressione → nome **stringa** (con
`\` iniziale strippato) risolto via `class_index`, oppure **oggetto** → il suo
`class_id`, altrimenti `TypeError`. `Dynamic` è **non-forwarding** per il late
static binding (come `Named`). Copre `new $cls` / `new $obj` / `$cls::CONST` /
`$cls::m()` / `$cls::$prop` / `$obj::m()` / `$x instanceof $cls`. Helper condiviso
`resolve_class_name`. Scope-out minori: `$cls::bind()` su `Closure`, generator
`instanceof $dyn`.

### 48c — `@` error-control operator (commit `e6b405a`)
Nuova `ExprKind::Suppress(Box<Expr>)`. **Punto delicato**: `eval()` chiama
`flush_diags()` dopo *ogni* `eval_inner`, quindi un warning dell'operando sarebbe
renderizzato prima di poterlo droppare. Soluzione: un contatore
`suppress_depth` che rende `flush_diags` un **no-op** durante la valutazione
dell'operando; al termine i diagnostici accumulati vengono **troncati**. I
**throwable/Error NON sono soppressi** (viaggiano sul canale `Err`, come PHP che
silenzia solo `error_reporting`): verificato con `@(1%0)` → `DivisionByZeroError`
ancora catchable. Scope-out **D-48.1**: un diagnostico già renderizzato a metà
valutazione (operando che emette output) non è ritrattabile (raro).

## Step 49 — constant expressions (magic + named) + hardening del runner

Scelta **data-driven** dal breakdown dello step 48: dopo un run completo del
corpus (9.117 `.phpt`) i due bucket `unsupported` dominanti erano
`expr:MagicConstant` (758) e `named constant` (381) — ~1.140 test bloccati su
un'unica famiglia. Tre sotto-step coesi (commit separati). **+11 unit test**
(821→832), clippy pulito, workspace 829 verde.

### 49-pre — runner: timeout per-test in `--isolate` (hardening)
Far girare il corpus pieno piantava il Mac (OOM): un `.phpt` che porta
l'evaluator in un loop illimitato (`while (true) $a[] = 1;`) girava all'infinito
mentre `--isolate` attendeva — su macOS non c'è `timeout(1)`. Ora ogni child gira
sotto un budget wall-clock (default 10s, `PHPT_TIMEOUT_SECS` per override/`0`=off):
oltre il limite è ucciso e contato come un FAIL `timed out`. Lo stdout è drenato
su un thread separato così un diff grande non causa deadlock nel ciclo
wait/kill. Verificato con un test sintetico `while(true)` (ucciso al cap).

### 49a — magic constants (commit `feat step49a`)
Le 9 varianti mago `MagicConstant` (`__LINE__ __FILE__ __DIR__ __CLASS__
__FUNCTION__ __METHOD__ __TRAIT__ __NAMESPACE__ __PROPERTY__`) sono risolte **a
lowering time** a literal: PHP le sostituisce a compile-time dallo *scope
lessicale*, quindi nessun supporto runtime. Il Lowerer traccia
`cur_class`/`cur_function`/`cur_trait` con lo stesso idioma save/restore di
`fn_by_ref` (function, method, closure/arrow=`{closure}`, class, trait).
`__LINE__`→`Int(line)`, `__FILE__`→`prog_name`, `__DIR__`→`dirname`,
`__METHOD__`→`Class::m` (nome nudo in funzione libera, `""` a top level),
`__NAMESPACE__`/`__PROPERTY__`→`""` (Tier 1: niente namespace; hook non
supportati).

### 49b — named constants predefinite estese (commit `feat step49b`)
`resolve_constant` ora folda anche la famiglia `E_*` (E_ALL=32767),
`DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_SAPI='cli'`.

### 49c — costanti utente: `define`/`constant`/`defined` (commit `feat step49c`)
Un bare `NAME` non-engine non è più uno SKIP: lowera a `ExprKind::Const(name)` e
si risolve a runtime contro una tabella `define()` sull'`Evaluator`, con il fatal
PHP 8 `Undefined constant "NAME"` se assente. I tre builtin sono dispatchati
**nell'evaluator** (serve la tabella) prima del registry stateless, sia sul path
diretto sia su `call_named` (chiamate dinamiche/stringa). `define()` avvisa e
ritorna `false` su ridefinizione; `defined()`/`constant()` consultano anche la
tabella engine (`resolve_constant`, ora `pub(crate)`).

### Impatto sul corpus (9.117 test)
`pass 1180→1231` (**+51** passano del tutto), `skip 6744→6389` (**−355**),
bucket `unsupported` `2926→1933` (**−993**: MagicConstant e named-constant spariti
dalla classifica). I ~993 test sbloccati: +51 passano, +304 ora **eseguono e
falliscono** su un gate successivo (prima non partivano), il resto migra a un
altro skip — soprattutto **builtin mancanti** (1473→2110, ora il bucket #1 e il
prossimo lever naturale). Il pass-rate "of runnable" cala (49,7%→45,1%) solo
perché il denominatore cresce: 355 test in più ora girano.

## Step 50 — `serialize()` / `unserialize()`

Scelta data-driven (builtin = bucket #1). Coppia auto-contenuta e ben
specificata, verificata **byte-exact contro l'oracle PHP 8.5**. Due sotto-step.
**+12 unit/functional test** sul parser e round-trip, workspace 841 verde.

### 50a — `serialize()` builtin puro (commit `feat step50a`)
Nuovo modulo `php-builtins/src/serialize.rs`. Walk del `Zval` → byte string:
`N;` / `b:N;` / `i:N;` / `d:<shortest>;` / `s:<bytelen>:"…";` / `a:<n>:{…}` /
`O:<len>:"class":<n>:{…}`. Float con `serialize_precision=-1`
(`dtoa::double_to_shortest`, riuso step 47); stringhe a **lunghezza in byte**;
`Closure`/`Generator` → `Error` "Serialization of 'X' is not allowed". È un
builtin puro: non serve stato dell'evaluator.

### 50b — `unserialize()` evaluator-dispatched (commit `feat step50b`)
Parser recursive-descent **puro** (`php-runtime/src/unserialize.rs`, intermedio
`enum Ser`, 4 unit test) + conversione `Ser`→`Zval` **nell'evaluator** (come
`json_decode`): ricostruire un oggetto richiede la class table / id allocator,
fuori portata di un builtin puro. Nuovo `make_object(class, fields)` istanzia la
classe per nome col suo `class_id` e shape reali e setta le proprietà
direttamente (**il costruttore NON gira**, come PHP); classe sconosciuta →
fallback `stdClass`. Input malformato o con garbage finale → `false` + Warning.
**Punto delicato**: il nome-classe in `O:` è `<len>:"class":` (terminato da `:`,
non `;`) — diverso dalle stringhe-valore; risolto con `quoted_bytes()` separato
da `string_body()`. Le lunghezze sono in byte: `;`/`"` interni sono dati.

### Impatto sul corpus (9.117 test)
`pass 1231→1243` (**+12**), `skip 6389→6285` (**−104**), bucket `builtin`
`2110→2006`: `serialize`/`unserialize` spariti dai builtin mancanti. I ~104 test
sbloccati: +12 passano, +92 ora **eseguono e falliscono** su un gate successivo.
Il prossimo lever è ora schiacciante: **`fopen` (297)** — l'intero sottosistema
filesystem/stream (decisione architetturale a sé).

## Step 51 — `fopen` + sottosistema filesystem-stream (spina)
Lever data-driven (builtin = bucket #1; `fopen` 297 file). Scelta utente
2026-06-21: **"spina fopen"** — introdurre il tipo risorsa + stream su file
reali + `php://` base, scope-out dei wrapper di rete/context/filter. Tre
sotto-step + un fix corpus-driven. Tutti i formati osservabili verificati
**byte-exact contro l'oracle PHP 8.5.7** (probe diretti). Design in
`02-mapping-table.md` (D-51.1…51.6). Workspace 845→**864** verde, clippy pulito.

### 51a — `Zval::Resource` + tipo stream + fopen/fread/fwrite/fputs/fclose
Mancava del tutto un tipo risorsa. Nuovo `Zval::Resource(Rc<RefCell<Resource>>)`
con handle semantics come `Object` (il clone condivide l'`Rc`: `$g=$f` aliasa,
`fclose($g)` chiude `$f`). Modulo `php-types::stream` (`Resource`/`ResKind`/
`Stream`/`StreamBackend` con I/O `std` puro + EOF flag sticky). **Arm `Resource`
in ~14 match esaustivi** (la parte più laboriosa, scoperti via `cargo build`):
gettype/error_type_name, convert (to_bool/is_true_silent=true, to_long_cast/
to_double=id, to_zstr="Resource id #N"), ops (try_to_number/try_to_long=None →
TypeError aritmetico, increment/decrement=TypeError, compare per id, identical
per handle), var_dump/print_r/var_export/serialize, coerce_key(+Warning)/
coerce_key_silent/php_type_name/match_case_repr. `fopen` è
**evaluator-dispatched** (`ho_fopen`, possiede il contatore `next_resource_id`
base 5 come la CLI); fread/fwrite/fputs/fclose sono **builtin puri**
(`php-builtins/src/file.rs`) che operano sull'`Rc` condiviso. Modi r/w/a/x/c
con `+`; b/t ignorati; fallimento → Warning "Failed to open stream: <strerror>"
(suffisso " (os error N)" di Rust strippato). 9 test.

### 51b — fgets/fgetc/feof/fseek/ftell/rewind/fflush + `php://`
`ho_fopen` apre `php://memory`/`temp` (buffer `Cursor` in-process; spill-to-disk
di temp = scope-out), `php://stdout` (→ buffer di output dell'evaluator, così
interleava con echo ed è catturato; **non** lo stdout reale), `php://stderr`
(→ stderr reale). Wrapper ignoti → Warning "no suitable wrapper" + false.
Costanti `SEEK_SET/CUR/END` = 0/1/2. `fgets($f,$len)` legge ≤ `$len-1` byte
(convenzione C). `feof` riflette l'EOF flag sticky; closed → TypeError. `fseek`
whence SET/CUR/END, offset assoluto negativo → −1. 8 test.

### 51c — file_get_contents / file_put_contents (builtin puri)
Nessuna risorsa: I/O diretto su `std::fs`. `file_get_contents` legge tutto poi
applica offset (negativo = dalla fine)/length; mancante → Warning + false.
`file_put_contents` accetta string | array (concatenato) | stream resource
(drenato); `FILE_APPEND`(8) appende, `LOCK_EX`(2) accettato e ignorato.
Costanti FILE_USE_INCLUDE_PATH/LOCK_EX/FILE_APPEND. 5 test.

### Fix corpus-driven (Fase 4c) — fwrite $length clamp
`ext/standard/tests/file/fwrite.phpt` ha rivelato un bug **classe A**: il terzo
arg `$length` va clampato a `[0, len]` — `fwrite($f,"data",-1)` scrive 0 byte
(scrivevo tutti e 4). Fix + 1 test → **fwrite.phpt passa end-to-end**. Conferma
collaterale: i testi d'errore dell'oracle combaciano esatti (errno=9 "Bad file
descriptor", ValueError "$length must be greater than 0", TypeError "must be an
open stream resource").

### Impatto corpus (bounded — `ext/standard/tests/file`, 897 test)
Sweep mirato (`--isolate`, `PHPT_TIMEOUT_SECS=5`) sulla directory più toccata:
**pass 1→2, fail 43→42, skip 853** dopo il fix fwrite. Pass-rate basso atteso:
la suite `file` dipende massicciamente da predicati FS non implementati
(`unlink`/`tempnam`/`mkdir`/`stat`/`fileperms`…, scope-out esplicito) e da
helper di setup, quindi 853/897 sono SKIP per capability-scan e molti dei 42
FAIL falliscono **a monte** (il path costruito dal setup non esiste → `fopen`
fallisce), non per bug delle primitive stream. Lo sweep full-corpus (delta del
bucket "missing builtin") è rinviato al prossimo run batch per la policy
anti-freeze (mai tutto il corpus in un colpo). Lever successivo naturale:
**predicati filesystem** (`file_exists`/`is_file`/`unlink`/`mkdir`/`stat`…),
che sblocca la maggioranza dei FAIL di `ext/standard/tests/file`.

## Step 52 — sottosistema predicati/operazioni filesystem (il lever di fine step 51)

> Generato con assistenza AI (Claude Opus 4.8). Continuazione diretta del lever
> dichiarato a fine step 51 ("predicati filesystem … sblocca la maggioranza dei
> FAIL di `ext/standard/tests/file`"). Scelta utente 2026-06-21: **scope A–E
> completo** (≈30 funzioni). Tutti i formati osservabili (array `stat` a 26 voci,
> messaggi di Warning per-funzione, formato `pathinfo`, ordini `scandir`)
> verificati **byte-exact contro l'oracle PHP 8.5.7** via probe diretti. Cinque
> sotto-step; ogni sotto-step commit + push. Workspace 864→**882** verde, clippy
> pulito. Nuova dep `libc` (già nel lockfile) per `access(2)` e `utimes(2)`.

### 52a — path-string puri: `basename` / `dirname` / `pathinfo` (commit 617b17c)
Nessun I/O: manipolazione di byte pura, quindi byte-exact testabile senza FS.
`php_basename` (strip trailing `/`, suffix rimosso solo se l'output resta più
lungo del suffix: `basename(".php",".php")`→`.php`), `php_dirname` con il param
`$levels` (clamp ≥1, "/" assorbente), `pathinfo` con i selettori
`PATHINFO_DIRNAME/BASENAME/EXTENSION/FILENAME` = 1/2/4/8 e la regola del dot
iniziale (`.htaccess`→filename `""`, extension `htaccess`). 27 asserzioni.

### 52b — predicati esistenza/tipo + `access(2)` + `clearstatcache`
`file_exists`/`is_file`/`is_dir` (segue symlink via `metadata`), `is_link`
(no-follow via `symlink_metadata`: un symlink rotto è ancora `true`), `filetype`
(lstat → file/dir/link/fifo/char/block/socket/unknown). **`is_readable`/
`is_writable`/`is_executable` rifatti su `libc::access(2)`** (euid-aware, segue
symlink): un file `chmod 0` legge come *non* leggibile anche per l'owner che lo
può stat'are (D-52.7) — il vecchio euristico su `metadata().readonly()` non
distingueva. `realpath` (`canonicalize`, `false` se manca un componente),
`getcwd`/`chdir` (cwd di processo), `sys_get_temp_dir` (senza slash finale).
`clearstatcache` = no-op `null`: non teniamo cache di stat per-richiesta, niente
da invalidare (D-52.8). 5 test nuovi + i 3 preesistenti di 52a.

### 52c — `stat` / `lstat` / `fstat` + accessor a campo singolo
Builder condiviso dell'array a 26 voci (chiavi intere `0..=12` poi le nominali
`dev,ino,mode,nlink,uid,gid,rdev,size,atime,mtime,ctime,blksize,blocks` nello
stesso ordine, D-52.9) da `std::os::unix::fs::MetadataExt`. `stat` segue symlink,
`lstat` no (mode 0120xxx vs 0100xxx, verificato). `fstat` su un resource: backend
File → metadata reale; backend in-memory/std → array sintetico mode 0100666 con
`size`=lunghezza buffer e zeri altrove (D-52.10, l'oracle dà 33206 per
`php://memory`). `filesize/filemtime/fileatime/filectime/fileperms/fileinode/
fileowner/filegroup` via helper condiviso (tutti seguono symlink); messaggio
"`%s(): stat failed for %s`" (`lstat` usa "Lstat failed"). 3 test.

### 52d — mutatori
`unlink`, `mkdir` (`$permissions`+`$recursive`, mode via `DirBuilderExt::mode`
mascherato dall'umask come PHP), `rmdir`, `rename` (sovrascrive dest), `copy`
(sovrascrive dest), `touch` (create-senza-troncare + `utimes(2)`; `$mtime` null →
now, `$atime` null → `$mtime`), `symlink`, `link` (hard), `readlink`, `chmod`
(`PermissionsExt::from_mode`). Ogni mutatore emette il **Warning esatto** di PHP
in fallimento — ognuno incornicia path/strerror diversamente (oracle-verified):
`unlink(%s)`/`rmdir(%s)`/`rename(%s,%s): %s`; `mkdir(): %s` (senza path!);
`copy(%s): Failed to open stream: %s`; `touch(): Unable to create file %s
because %s`; `symlink/link/readlink/chmod(): %s`. Nuovo helper `out_diags` nei
test per asserire il testo grezzo dei diag. 3 test.

### 52e — `scandir` / `glob` / `tempnam` / `tmpfile`
`scandir($dir,$sort)`: voci incluse `.`/`..`, sort byte ascendente(0)/
discendente(1)/none(2); in fallimento PHP emette **due** Warning ("Failed to open
directory" + "(errno N)") poi `false` — replicati entrambi. `glob` è un
**globber shell self-contained** (no crate): `*`/`?`/`[...]` su tutti i segmenti
di path, regola del dot iniziale, espansione `GLOB_BRACE`, flag `GLOB_MARK/
NOSORT/NOCHECK/ONLYDIR`; array vuoto se nessun match (D-52.11). `tempnam` crea un
file 0600 unico e ritorna il path canonicalizzato (l'oracle risolve `/var`→
`/private/var` su macOS). **`tmpfile` è evaluator-dispatched** (conia un resource
come `fopen`): crea un file temp unlinkato aperto r+ (riassorbito dall'OS alla
chiusura/uscita). 4 test.

### Impatto corpus (bounded — `ext/standard/tests/file`, 897 test)
Sweep `--isolate` `PHPT_TIMEOUT_SECS=5` sulla directory più toccata, prima/dopo
lo step:

| | pass | fail | skip | runnable |
|---|---:|---:|---:|---:|
| fine step 51 | 2 | 42 | 853 | ~44 |
| **fine step 52** | **63** | **81** | **753** | **144** |

**pass 2→63 (+61)**, skip −100 (il capability-scan non salta più i test che
usano unlink/mkdir/stat/scandir/…), fail +39 (più test arrivano *fino* alle
asserzioni invece di fallire a monte su un `fopen` impossibile). Il lever dei
predicati FS è **speso**: i 753 skip residui sono ora dominati da *altri*
builtin mancanti — `fprintf`(35), `strstr`(32), `stream_wrapper_register`(14),
`opendir`(9), `stream_context_create`(8), `fscanf`(7), `ftruncate`(7),
`get_resource_type`(6), `fputcsv`/`fgetcsv`/`parse_ini_file` — e da 498 skip di
tipo "section" (multi-sezione `--FILE_EXTERNAL--`/`--CLEAN--`/`--INI--`,
harness-level, non gap di builtin). Lever successivi naturali entro questa
directory: **`fprintf`/`fscanf`**, la **famiglia `opendir`/`readdir`/`closedir`**
(D-52.12 scope-out di questo step), **`get_resource_type`**, e i **CSV**
(`fputcsv`/`fgetcsv`).

### Scope-out espliciti (debito)
- **D-52.12**: `opendir`/`readdir`/`closedir`/`rewinddir` (iterazione directory
  basata su resource) — `scandir` copre la forma moderna/comune; 9 test skippati.
- Wrapper di rete/context/filter (`stream_context_create`, `stream_wrapper_register`,
  `stream_filter_append`) restano fuori (continuità con lo scope-out di step 51).
- `SCANDIR_SORT_NONE` ritorna l'ordine `readdir` grezzo (non garantito uguale
  all'ordine OS dell'oracle); ascendente/discendente sono byte-exact.

## Step 53 — lever cheap/medi che finiscono `ext/standard/tests/file`

> Generato con assistenza AI (Claude Opus 4.8). Scelta utente 2026-06-21 dopo lo
> step 52: implementare **i quattro lever a basso rischio** che restavano sul
> bucket `ext/standard/tests/file` (i parser veri — `fscanf`, CSV — rinviati a
> sessione dedicata con design pass). Tutti i formati verificati byte-exact
> contro l'oracle PHP 8.5.7. Quattro sotto-step + un fix. Workspace 882→**888**
> verde, clippy pulito.

### 53a — `strstr` / `strchr` / `stristr` / `strrchr`
String puri (in `string.rs`, riusano `find_sub`). `strstr($h,$n,$before=false)`
ritorna la fetta da/prima dell'occorrenza; `strchr` = alias. `stristr`
case-insensitive (match in lowercase, fetta in case originale). `strrchr` usa
**solo il primo byte** del needle e cerca l'ultima occorrenza. `false` se assente.

### 53b — `get_resource_type`
Ritorna l'etichetta `dump_type` del resource ("stream" aperto / "Unknown"
chiuso) — esattamente la stringa che PHP dà per file e dir handle; TypeError su
un non-resource.

### 53c — iterazione directory: `opendir`/`readdir`/`closedir`/`rewinddir`
Nuova `ResKind::Dir(DirHandle)` (snapshot delle voci `.`/`..` + resto in ordine
OS, più un cursore). PHP modella i dir handle come `php_stream`, quindi riportano
le stesse etichette "resource"/"stream" di un byte stream (chiude D-52.12).
`opendir` è **evaluator-dispatched** (conia resource come `fopen`/`tmpfile`);
Warning "opendir(%s): Failed to open directory: %s" + false in errore. `readdir`
ritorna i byte grezzi (una voce "0" trippa ancora `=== false`), `closedir` →
resource chiuso, `rewinddir` resetta il cursore.

### 53d — `fprintf` / `vfprintf`
Riusano l'engine `sprintf` esistente (`format_impl`/`first_format` resi
`pub(crate)`): formattano e scrivono sul resource stream, ritornando il conteggio
di byte (come `printf`). `vfprintf` prende gli argomenti da un array.

### Fix (D-53.1) — dir handle in un builtin di stream non panica più
Bug latente scoperto dal corpus (`directory_wrapper_fstat_basic`): un
`ResKind::Dir` passato a `fstat` colpiva `as_stream_mut().expect(...)` →
**panic** (e con `--isolate` abortiva il worker). Ora `stream_arg` ammette solo
`ResKind::Stream(_)` (rigetta Dir + Closed con il TypeError "must be an open
stream resource"), mantenendo sani gli 8 `.expect()` dei builtin byte-stream;
`fstat` risolve il resource da sé e ritorna `false` su un dir/closed handle (non
abbiamo il path per ricostruire lo stat). Test di regressione aggiunto.

### Impatto corpus (bounded — `ext/standard/tests/file`, 897 test)
Sweep `--isolate`. Segnale **robusto**: lo skip scende **753→716** (−37): i test
che lo capability-scan saltava per `strstr`/`get_resource_type`/`opendir`/
`fprintf` ora vengono ammessi. La composizione degli skip residui conferma che
il lever è speso — ora dominano *parser veri* ancora mancanti: **`fscanf`(50)**,
`stream_wrapper_register`(14), `stream_context_create`(8), `ftruncate`(7),
**CSV** (`fputcsv`(6)/`fgetcsv`(4)) — più i 498 skip "section" (multi-sezione,
harness-level).

**Caveat di misura**: il conteggio *pass* osservato in questo run è confondato
da **accumulo di artefatti in-tree**. Il nostro phpt-runner esegue i test
*in loco* nell'albero sorgente ma **non** esegue le sezioni `--CLEAN--`, quindi
sweep ripetuti lasciano `*.tmp`/directory generate che fanno fallire con
"File exists" test altrimenti verdi (es. `bug45181`, `007_variation7`,
`copy_variation11` — tutti leftover, non regressioni di codice: lo step 53 non
tocca `mkdir`/`fopen`). Su albero pulito i ~37 ammessi sono in larga parte
fscanf/CSV-dipendenti (ancora fuori scope) e quindi falliscono comunque; il
delta-pass netto reale è piccolo e positivo. Lever successivo naturale:
**`fscanf`/`sscanf`** e **CSV** (`fgetcsv`/`fputcsv`/`str_getcsv`), entrambi
parser che meritano il loro design pass.

## Step 54 — parser families: `sscanf`/`fscanf` + CSV (`str_getcsv`/`fgetcsv`/`fputcsv`)

> Generato con assistenza AI (Claude Opus 4.8). Scelta utente 2026-06-21: dopo lo
> step 53 i due lever residui di `ext/standard/tests/file` erano *parser veri*;
> l'utente ha scelto di farli **entrambi** in un design pass. Due engine distinti
> (scanf in `php-runtime`, CSV in `php-builtins`) con semantica byte-exact verificata
> contro l'oracle PHP 8.5.7. Quattro sotto-step. Workspace 888→**894** verde, clippy
> pulito.

### Vincolo di layering (decisione)
`sscanf`/`fscanf` hanno il **modo by-reference** (`sscanf($s,$fmt,&$a,...)` assegna e
ritorna il count): i parametri by-ref si fanno **solo** higher-order/evaluator-dispatched
(come `preg_match` → `write_out_param`). Quindi il motore scanf vive in `php-runtime`
(`crates/php-runtime/src/scanf.rs`), non in `php-builtins` (che `php-runtime` non può
importare). Le funzioni CSV ritornano array / scrivono su stream → **builtin puri**,
motore in `crates/php-builtins/src/csv.rs`.

### 54a — motore scanf + `sscanf`
`run_scanf(input,fmt) -> Vec<Option<Zval>>`: una slot per conversione non soppressa
(None quando una conversione fallisce o non viene raggiunta — lo scanning si ferma alla
prima conversione fallita o al primo mismatch di un literal). Conversioni: `%d` (decimale
stretto), `%i` (**auto-base C**: 0x→16, 0→8, else 10 — distinto da `%d`), `%u`/`%x`/`%X`/
`%o`/`%b`, `%f`/`%e`/`%g`, `%s` (fino a ws), `%c` (esattamente `width` byte, **non** salta
ws), `%[..]`/`%[^..]` (char class), width `%2d`, `%*` suppress, `%%`; ws-matcha-ws. Riusa
parse i64 saturante + parse f64 std. `ho_sscanf` (eval.rs): senza out-var → array (NULL
per non-match); con `&$var` → assegna e ritorna il count (D-54.1: solo `$var` bare, come
preg_match).

### 54b — `fscanf`
`ho_fscanf`: legge **una riga** (`Stream::read_line`) poi riusa `run_scanf` +
`scanf_finish` (condivisi con sscanf). `false` a EOF (così `while($r=fscanf(...))`
termina); array o count by-ref altrimenti.

### 54c+54d — motore CSV + `str_getcsv` / `fgetcsv` / `fputcsv`
`csv.rs`: `parse_csv_line` (doppia-enclosure `""`→`"`, escape char dentro le quote,
sep/newline embedded nelle quote) e `format_csv_line`. Set di qualifica di `fputcsv`
(oracle-verified) = `{sep, enclosure, escape, space, tab, \r, \n, NUL}` → quota e
raddoppia l'enclosure. Solo il **primo byte** di sep/enclosure/escape è usato; escape
stringa vuota = disabilitato (come PHP). `str_getcsv` (puro; input vuoto → `[null]`);
`fgetcsv` (legge una riga → array; `false` a EOF) e `fputcsv` (scrive un record, ritorna
il byte-count) in file.rs via `stream_arg`. **Fedeltà 8.5 (D-54.2)**: emesso il
`Deprecated: <fn>(): the $escape parameter must be provided as its default value will
change` quando `$escape` è omesso (testo oracle-verified).

### Impatto corpus (copia pulita di `ext/standard/tests/file`)
Per evitare la pollution in-tree dello step 53, sweep su una **copia pulita** in `/tmp`
(solo `.phpt`+`.inc`, una sola run col binario aggiornato):

| | pass | fail | skip | runnable |
|---|---:|---:|---:|---:|
| pre-54 (engine assenti) | 66 | 115 | 604 | 181 |
| **post-54** | **71** | 166 | **548** | 237 |

Segnale robusto: **skip −56** — il bucket "missing builtin: `fscanf`/`fgetcsv`/`fputcsv`"
è **eliminato** (i ~56 test ora vengono ammessi). I net-new pass sono modesti (~5): i test
ammessi falliscono in larga parte per ragioni **ortogonali** al motore (verificato
ispezionando i diff):
- **named arguments ai builtin** non supportati (limite pre-esistente del runtime, es.
  `fgetcsv_variation1`);
- **fixture / `__FILE__`** non risolti nella copia pulita (es. `fscanf_variation10` fa
  `fopen` di un path derivato dal proprio file);
- **messaggi d'errore edge** non implementati: "Variable is not assigned by any conversion
  specifiers" / "Bad scan conversion" per mismatch numero-var/spec (scope-out 54).
I motori in sé sono validati byte-exact dai test unit. Lever successivi naturali per
questa directory: `ftruncate`, `stream_get_contents`/`stream_copy_to_stream`,
`parse_ini_file`, `readfile` — più il supporto named-args ai builtin (trasversale).

### Scope-out espliciti (debito)
- by-ref `sscanf`/`fscanf` su `&$a[0]`/`&$o->p`: ignorati come `preg_match` (D-54.1).
- record CSV multi-riga (campo quotato con `\n` che attraversa più righe in `fgetcsv`):
  leggiamo una riga sola (D-54.3); `str_getcsv` su stringa con `\n` embedded funziona.
- messaggi d'errore di mismatch var/spec di sscanf/fscanf (vedi sopra).
- argomento `$length` di `fgetcsv` ignorato (leggiamo la riga intera).
