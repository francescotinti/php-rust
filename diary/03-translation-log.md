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

Step non-funzionale (DevEx/stabilità), nato dalla review esterna `analysis_results.md` (oggi `external-review-2026-06-16.md`)
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

## Step 55 — batch builtin stream/file read + env/disk

> Generato con assistenza AI (Claude Opus 4.8). Scelta utente 2026-06-21: dopo lo step 54
> i lever residui cheap del dir `file` erano builtin diretti (non parser). Scope ampio:
> i 6 core read/output + env + disk. Tutti verificati byte-exact contro l'oracle PHP 8.5.7.
> Tre sotto-step. Workspace 894→**898** verde, clippy pulito. Named-args ai builtin
> **scartati** (refactor ABI + tabella ~199 funzioni, ROI basso — eval.rs:5255).

### 55a — `file` + `readfile` + `fpassthru`
`file($path,$flags)`: array di righe; di default ogni riga tiene il `\n` finale (l'ultima
senza newline no); `FILE_IGNORE_NEW_LINES`(2) strippa `\r?\n`, `FILE_SKIP_EMPTY_LINES`(4)
scarta le righe vuote; `false`+Warning "Failed to open stream" se manca. `readfile`
(file→`ctx.out`, byte-count) e `fpassthru` (resto dello stream→`ctx.out`, byte-count).
Costanti `FILE_IGNORE_NEW_LINES`/`FILE_SKIP_EMPTY_LINES` in lower.rs.

### 55b — `stream_get_contents` + `stream_copy_to_stream` + `ftruncate`
Helper condiviso `read_remaining(stream,max)`. `stream_get_contents($s,$max=-1,$off=-1)`
(seek assoluto se off≥0). `stream_copy_to_stream($from,$to,$len=null,$off=0)`: legge tutto
prima in un buffer poi scrive (così `from`/`to` non sono mai borrowed insieme, anche se
identici), ritorna il count. `ftruncate($s,$size)`: per-backend (`File::set_len` /
`Memory` Vec `resize`-con-zeri; Stdout/Stderr→false).

### 55c — `getenv`/`putenv` + `disk_free_space`/`disk_total_space`
`getenv($name)`→string|false; `getenv()`→array di tutte le env (`vars_os`, byte grezzi).
`putenv("K=V")` set / `putenv("K")` unset → true (process-global, ok sotto `--isolate`).
`disk_free_space`/`disk_total_space` via `libc::statvfs` (`f_bavail`/`f_blocks * f_frsize`
come `f64`; `false` su path non stat'abile). Alias legacy `diskfreespace`.

### Impatto corpus (copia pulita di `ext/standard/tests/file`, 785 test)
Lezioni step 54 applicate: **runner ricostruito** prima dello sweep + **copia pulita** in
`/tmp` (una run, niente pollution in-tree):

| | pass | fail | skip | runnable |
|---|---:|---:|---:|---:|
| post-54 | 71 | 166 | 548 | 237 |
| **post-55** | **86** | 177 | **522** | 263 |

**pass +15 (71→86)**, skip −26 — risultato pulito e positivo (niente caveat di misura
questa volta). I builtin `file`/`readfile`/`fpassthru`/`stream_get_contents`/
`stream_copy_to_stream`/`ftruncate` sono **spariti** dalla lista "missing builtin"; i ~26
test ammessi passano in buona parte (a differenza dei parser dello step 54, qui la
semantica è semplice e deterministica). Lever residui ora: `parse_ini_file`(5), `ini_set`(4),
`rand`(4), `stream_get_line`(2), `set_include_path`(2) + i wrapper scope-out
(`stream_wrapper_register`/`stream_context_create`/`stream_filter_append`).

### Scope-out espliciti (debito)
- named-args ai builtin (refactor ABI + ~199 tabelle nomi-parametro) — fuori scope.
- `$context`/`$use_include_path` di `file`/`readfile`: accettati e ignorati.
- `disk_*_space` solo Unix (statvfs); Windows fuori scope.

## Step 56 — batch funzioni stringa (HTML/escape/transform/binary)

> Generato con assistenza AI (Claude Opus 4.8). Dopo 5 step nel dominio file/stream, una
> **scansione di frequenza sull'intero corpus** (9.984 `.phpt`) ha ri-orientato la scelta
> verso le funzioni stringa pure più chiamate e ancora mancanti. Scelta utente 2026-06-21:
> **batch stringhe**. Tutte funzioni pure, byte-exact contro l'oracle PHP 8.5.7. Tre
> sotto-step. Workspace 898→**903** verde, clippy pulito.

### 56a — binary/escape/transform
`bin2hex`/`hex2bin` (false+Warning "Input string must be hexadecimal string" su
lunghezza dispari/non-hex). `addslashes` (escape `'` `"` `\` NUL) / `stripslashes`
(rimuove un `\`; `\0`→NUL; backslash finale isolato rimosso). `substr_replace` (forma
scalare; start/len negativi, len=0=insert). `nl2br` (`<br />`/`<br>` prima di `\n`/`\r\n`/
`\r`). `wordwrap` (greedy, `$cut` spezza parole lunghe — port fedele dell'algoritmo
space-replace di PHP).

### 56b — HTML (nuovo `html.rs`)
`htmlspecialchars` (5 ASCII special, bit virgolette per flag; default 11 = entrambe).
`htmlentities` (+ tabella Latin-1 U+00A0–U+00FF via decode UTF-8; set HTML4 completo
greco/matematica = scope-out D-56.1). `htmlspecialchars_decode` (5 special) e
`html_entity_decode` (named Latin-1 + numerici `&#NN;`/`&#xHH;`). Costanti `ENT_*` in
lower.rs (NOQUOTES/HTML401=0, COMPAT=2, QUOTES=3, IGNORE=4, SUBSTITUTE=8, HTML5=48).

### 56c — vsprintf / vprintf
Riuso di `format_impl` (come `vfprintf`): `vsprintf`→string, `vprintf`→`ctx.out` +
byte-count, valori dall'array.

### Impatto corpus (copia pulita di `ext/standard/tests/strings`, 733 test)
Prima sweep di questa directory (runner ricostruito + copia pulita in `/tmp`, una run):
**pass 143 / fail 137 / skip 453** — pass-rate sul runnable **51% (143/280)**. Le 11
funzioni dello step 56 sono **sparite** dalla lista "missing builtin" (nessuna regressione
possibile: solo aggiunte pure). Lever residui di questa directory: `strip_tags`(27),
`pack`/`unpack`(15/10), `strrpos`/`stripos`/`strripos`(15/13/10), `strtr`(15), `crypt`(13),
`strcspn`/`strspn`(9), `chunk_split`(9), `base64_decode`(6), `md5`(6), `strtok`(6),
`levenshtein`(5).

### Scope-out espliciti (debito)
- tabella HTML4 completa di `htmlentities` (greco/matematica/symbol): solo Latin-1 (D-56.1).
- `$encoding` di htmlspecialchars/htmlentities: assunto UTF-8.
- forma-array di `substr_replace` (`$string`/`$replace` array): solo forma scalare.

## Step 57 — batch funzioni stringa #2 (search / span / translate / split / strip)

> Generato con assistenza AI (Claude Opus 4.8). Piano scelto dall'utente 2026-06-21
> (`~/.claude/plans/silly-wobbling-whisper.md`): secondo batch di funzioni stringa pure
> dai lever residui di `ext/standard/tests/strings`. Tutte byte-exact contro l'oracle PHP
> 8.5.7 (semantica inchiodata *prima* di implementare). Tre sotto-step + diario/sweep.
> Workspace 903→**918** verde, clippy pulito su `string.rs`.

### 57a — search family (rpos / case-insensitive) + span
`strrpos`/`stripos`/`strripos` con la semantica di `$offset` inchiodata sull'oracle:
- forward (`stripos`): `start=len+offset` se negativo; `start<0 || start>len` → ValueError
  "Argument #3 ($offset) must be contained in argument #1 ($haystack)".
- reverse (`strrpos`/`strripos`, helper `rpos_window`): per `offset≥0` cerca start in
  `[offset, len-nlen]` (ValueError se `offset>len`); per `offset<0` (ValueError se
  `offset<-len`) il bound alto è `len+offset`, ma se `nlen>-offset` si allarga a `len-nlen`;
  needle vuota → posizione massima del window. `rfind_window` ritorna l'ultima occorrenza.
Le varianti case-insensitive fanno fold ASCII di haystack+needle e riusano `find_sub`/
`rfind_window`. `strspn`/`strcspn` (helper `span_slice` per `$start`/`$length` con
negativo-da-fine + clamping; `byte_set` 256-bool): lunghezza del segmento iniziale del
window fatto di byte ∈/∉ `$mask`.

### 57b — strtr + chunk_split
`strtr($s,$from,$to)`: tabella di traduzione per-byte su `min(len(from),len(to))` coppie
(byte assenti passano invariati; duplicato nel `from` → vince l'ultima mappatura). Forma
2-arg con 2° non-array → TypeError "Argument #2 ($from) must be of type array, string given".
`strtr($s,$map)`: replace di sottostringhe **longest-key-first** (sort stabile per lunghezza
desc), scansione L→R senza ri-scansione dell'output; chiavi int → forma decimale; chiave
vuota → Warning "Ignoring replacement of empty string" e skip. `chunk_split($s,$len=76,
$sep="\r\n")`: separatore dopo ogni chunk incluso uno finale; `$len<1` → ValueError; stringa
vuota → comunque un separatore (fedele a PHP).

### 57c — strip_tags + quotemeta + levenshtein
`strip_tags` è un **port fedele dello scanner** di PHP (state-machine inchiodata su ~20
probe oracle): un `<` seguito da whitespace (o EOF) resta letterale; in un tag normale i `<`
annidati alzano una profondità che i `>` devono bilanciare e le virgolette (`"`/`'`)
sopprimono `<`/`>` finché aperte; `<!-- -->` è un commento la cui chiusura `-->` può riusare
i trattini di apertura (così `<!-->` è un commento vuoto); `<! …>` corre fino a `>`; `<? …?>`
corre fino a `?>`. `quotemeta` (escape di `. \ + * ? [ ^ ] $ ( )`). `levenshtein` 2-arg
(distanza di edit byte, costi unitari, DP a due righe).

### Impatto corpus (copia pulita di `ext/standard/tests/strings`, 733 test)
Runner ricostruito `--release` + copia pulita in `/tmp` (una run). **Misura con `--isolate`**
perché l'esecuzione in-process aborta su un crash *pre-esistente* di `sprintf` (vedi D-NEW):

| | pass | fail | skip | runnable | pass-rate |
|---|---:|---:|---:|---:|---:|
| post-56 (in-process, **troncato** dal crash) | 143 | 137 | 453 | 280 | 51% |
| **post-57 (`--isolate`, completo)** | **228** | 165 | **340** | 393 | **58.0%** |

Le 9 funzioni dello step 57 sono **sparite** dalla lista "missing builtin". Il salto di
runnable (280→393) è in parte reale (nuovi builtin) e in parte perché la baseline post-56 era
un conteggio **troncato**: il run in-process abortiva a `sprintf_star.phpt` (alfabeticamente
verso la fine), quindi `--isolate` è ora la misura di riferimento. Lever residui ora:
`pack`(15), `crypt`(13), `unpack`(10), `base64_decode`(6), `md5`(6), `strtok`(6),
`substr_compare`(6), `strncasecmp`(5).

**1 bug trovato e fixato dal corpus** (`strtr_variation4.phpt`): con subject vuoto PHP
ritorna `""` **senza** processare la mappa, quindi il Warning chiave-vuota non deve scattare;
`strtr_array` ora corto-circuita su subject vuoto.

### Scope-out espliciti (debito)
- `$allowed_tags` di `strip_tags` (stringa o array): non onorato, tutti i tag rimossi (D-57.1).
- forma pesata 5-arg di `levenshtein` (`$cost_ins/$cost_rep/$cost_del`): solo 2-arg unitaria (D-57.2).
- `strtok` (stateful: ricorda stringa+posizione tra chiamate): rinviato a uno step con stato
  sull'evaluator.
- coercion float→int degli argomenti `$offset` (`to_long_cast` emette Warning invece del
  TypeError "must be of type int, float given"): gap **ereditato** comune a tutti i builtin
  (`strrpos_offset`/`strripos_offset.phpt`), non specifico dello step 57.
- 2 EXPECTF (`chunk_split_variation7`, `strcspn_variation5`) sono FAIL nel runner ma l'output
  di `phpr` è **byte-identico** all'atteso: sfumatura del matcher EXPECTF del runner (probabile
  `%` letterale nei dati), non una divergenza dei builtin.

## Step 58 — chiusura del motore sprintf (crash + `*` star + `%g/%G/%h/%H`)

> Generato con assistenza AI (Claude Opus 4.8). Scelta utente 2026-06-21: dopo che lo step 57
> ha scoperto che il run **in-process** del corpus abortiva su un crash di `sprintf`, si è
> deciso di **chiudere il motore** (`crates/php-builtins/src/format.rs`) invece di lasciarlo a
> debito. Tutto inchiodato byte-exact contro l'oracle PHP 8.5.7. Quattro sotto-step. Obiettivo
> raggiunto: **`sprintf_star.phpt` ora PASSA**. Workspace 918→**922** verde, clippy pulito.

### 58a — kill del crash + validazione width/precision letterali
Il colpevole del crash dello step 57 era `sprintf_star.phpt` riga 62:
`printf("%9999999999999999999999.f\n", $f)`. Una width letterale oltre `u64`/`INT_MAX`
saturava `read_uint`→`u64::MAX`, finiva in `Vec::with_capacity` e dava **`capacity overflow`**
panic che abortiva l'intero run in-process (un test cattivo uccide il batch). PHP lancia invece
una `ValueError`. Validate width e precision contro `INT_MAX` (2147483647) **prima** di
salvarle → "Width must be between 0 and 2147483647" / "Precision must be between 0 and
2147483647". Lo sweep `strings` ora completa in-process (==`--isolate`).

### 58b — `*` star width/precision (PHP 8.4, arg-driven)
`%*d`/`%.*f`/`%*.*f`: width/precision presi da un argomento `int`, consumati L→R **prima** del
valore; binding posizionale `%*N$`/`%.*N$`. Validazione fedele (helper `read_star_arg`):
- l'arg dello star dev'essere un vero `int` (`Zval::Long`), altrimenti "Width/Precision must be
  an integer";
- width ∈ [0, INT_MAX] altrimenti "Width must be between 0 and 2147483647";
- precision ∈ [-1, INT_MAX] altrimenti "Precision must be between -1 and 2147483647"; un `-1`
  (shortest) è valido **solo** per `%g/%G/%h/%H`, altrimenti "Precision -1 is only supported
  for %g, %G, %h and %H".
`Spec.precision` diventa `i64` (-1 = shortest, trasportato fino alla famiglia g/G).

### 58c — `%g/%G/%h/%H` shortest-form (port di `php_gcvt`)
La parte deliberatamente saltata a suo tempo. Algoritmo: scelta fixed/scientific con
`decpt < -3 || decpt > P` (P = precision, default 6; **17** per la forma shortest `-1`); strip
delle zero finali; in scientific una sola cifra di testa + **almeno una** cifra frazionaria
(`1.0e+6`) ed esponente con segno senza zeri iniziali. Le cifre significative vengono dalla
formattazione float di Rust (`{:.*e}` / `{:e}` per lo shortest = round-half-to-even, **come il
dtoa di PHP**). `h`/`H` sono i gemelli locale-independent di `g`/`G` (identici sotto locale C).
`INF`/`-INF`/`NaN` e lo zero con segno (`-0`) gestiti. **Differential vs oracle byte-exact** su
24 valori × 9 varianti di formato (verifica via un `.phpt` generato dall'oracle).

### Impatto corpus + residui
Sweep `ext/standard/tests/strings` (copia pulita, in-process — niente più crash):
**228→229 pass** (sprintf_star) su 393 runnable (58.3%). Il valore vero è però **trasversale**:
`%g/%G` e il `*` sono comunissimi in tutto il corpus oltre questa dir, e il crash-fix rende
robusto **ogni** run in-process (prima un singolo `%9999…f` abortiva il batch).

Residui sprintf di questa dir (≈29) sono **ortogonali** alla chiusura del motore (erano già
fail): padding fine di `%f` (`sprintf_f.phpt`: es. `%.3f` in certe combinazioni di width/flag),
`fopen` su `__FILE__` (`sprintf_variation1.phpt`, limite harness ereditato come step 57), e la
**catchability** degli errori (`printf_error.phpt`: emettiamo `PhpError::Error` *fatale* dove
PHP lancia un `ArgumentCountError` *catchable*). Debito per un eventuale step di fedeltà
sprintf dedicato; non parte della chiusura del motore (crash + star + g/G/h/H).

## Step 59 — CLI `phpr` (era uno stub) + batch di fedeltà sprintf/printf

> Generato con assistenza AI (Claude Opus 4.8). Scelta utente 2026-06-21: "chiudiamo i 29 fail
> [residui sprintf dello step 58] e poi indaghiamo il binario CLI phpr". Indagando phpr si è
> scoperto che `php-cli/src/main.rs` era **`fn main() {}`** (uno scheletro): il binario non
> emetteva nulla. Implementarlo è prerequisito per un differential rapido, quindi è stato
> fatto per primo. Poi un batch di fedeltà sprintf guidato dal corpus. Workspace 922→**927**
> verde. **Infra**: `target-dir` di cargo spostato su disco interno (il volume "Extreme Pro"
> non sa fare hard-link della cache incrementale → binari stale/incoerenti, fonte di misure
> ballerine; vedi `.cargo/config.toml`).

### 59a — implementazione del CLI `phpr`
`php-cli` ora legge lo script, lo esegue con la registry dei builtin (`run_source_with`),
scrive lo stream **CLI-faithful** (`Outcome::rendered`: output + diagnostics + fatal non
catturato resi inline, come la CLI di PHP sotto `display_errors=1, html_errors=0`) ed esce con
lo status fedele (codice di `exit`/`die`, **255** su fatal, altrimenti 0). Diventa un `php`
drop-in **e** un differential contro l'oracle 8.5.7. Verificato byte-exact su output e exit
code (3/255/0) — al netto del path `/private` (symlink macOS) nei messaggi d'errore.

### 59b — batch di fedeltà sprintf/printf (corpus `ext/standard/tests/strings`)
Cinque fix guidati dal differential `phpr` vs oracle:
- **modificatore `l`**: `%ld`/`%lf`/`%lx`… — un singolo `l` (length modifier) prima della
  conversione è accettato e **ignorato** (`%ld`==`%d`). Prima emettevamo "d" letterale.
- **specifier sconosciuto/mancante → ValueError catchable**: conv ignota → `Unknown format
  specifier "X"`; `%`/`%l` a fine stringa → `Missing format specifier at end of string`
  (prima: silenziosamente literal).
- **errori catchable + tipo corretto**: `sprintf`/`printf` senza format → `ArgumentCountError`
  (era `Error` non catchable); `vsprintf`/`vprintf` "exactly 2 arguments" + `TypeError`
  "Argument #2 ($values) must be of type array"; `fprintf`/`vfprintf` controllano il conteggio
  prima del tipo.
- **conteggio "N arguments are required"**: PHP riporta `1 + (numero totale di specifier)` =
  max indice arg referenziato + 1 (pre-scan `max_arg_index`), non l'indice del primo specifier
  rimasto a secco.
- **threading dei diagnostics**: il motore scartava i warning di coercion in un sink throwaway
  → `%s` di un array (e `%d`/`%c` di un object) perdeva "Array to string conversion" /
  "Object … could not be converted". Ora `&mut Diags` passa per tutto il motore.
- **pad char in left-justify**: il riempimento a sinistra usa il pad char (non sempre lo
  spazio); l'unica eccezione è il flag `0` di un **intero**, che PHP declassa a spazio
  (`%-05d`→"42   "), mentre float/string lo tengono (`%-05.2f`→"3.400", `%-05s`→"hi000"); il
  pad custom `'<c>` è onorato per ogni tipo (helper `is_int` in `pad_numeric`).

### Impatto corpus + residui (debito categorizzato)
Sweep `ext/standard/tests/strings` (copia pulita, in-process): **229→242 pass / 393 runnable
(61.6%)**. Chiusi 13 dei 29 fail sprintf/printf. I 16 residui **non** sono bug del motore
(verificato byte-exact `phpr` vs oracle) ma di tre nature:
- **runner EXPECTF su contenuto binario** (~6: `fprintf_variation_004`, `sprintf_variation15/27`,
  `vprintf_variation7/9/10`): il nostro output è **identico all'oracle** (`cmp`), ma il runner
  legge il `.phpt` con `from_utf8_lossy` e i byte NUL/binari corrompono il regex EXPECTF →
  falso-negativo del *tooling*, non del motore.
- **interleaving warning/output nell'evaluator** (4: `printf_variation2`, `sprintf_variation2`,
  `vprintf_variation8`, `sprintf_rope_optimization_003`): il warning ora **viene emesso** ma
  appare *dopo* l'output di printf invece che prima (PHP lo mette prima). Richiede che ogni
  diag porti la posizione in `out` al push — modifica trasversale al sistema diagnostico,
  rischiosa, fuori dallo scope sprintf.
- **harness `fopen(__FILE__)`** (2: `sprintf_variation1`, `vprintf_variation2`): il test apre il
  proprio file `.php`, che il runner non materializza (limite ereditato, come `strtr_variation6`).
- **niche** (`sprintf_variation52`: cap precision a 53 cifre con Notice; `vprintf_variation3/5`:
  quirk di parsing `% %%d`; `sprintf_rope_optimization_001`: rendering dell'`ArgumentCountError`
  non catturato).

## Step 60 — modularizzazione di `eval.rs` (refactor, zero cambi di comportamento)

> Generato con assistenza AI (Claude Opus 4.8). Scelta utente 2026-06-21: implementare il
> suggerimento principale di una code-review esterna (Gemini, `analysis_and_suggestions.md`) —
> spezzare il monolite `eval.rs` (**6.965 righe**). Refactor puramente meccanico: spostamento di
> codice, **nessun cambio di comportamento**, con i **927 test** come rete a ogni sotto-step.

`eval.rs` → `eval/mod.rs` + cinque sottomoduli, ognuno un blocco `impl<'p> Evaluator<'p>`:
- **`eval/expr.rs`** (745) — `eval`/`eval_inner` (il cuore tree-walker), instanceof, `apply_binop`.
- **`eval/stmt.rs`** (591) — `exec_stmt*`, loop/`foreach`/`switch`, propagazione eccezioni.
- **`eval/calls.rs`** (1128) — invocazione funzioni/metodi, closure, named/spread args, runtime
  dei generatori (corosensei).
- **`eval/class.rs`** (1177) — `new`, risoluzione classi/interfacce, costanti, enum, proprietà +
  visibilità, magic method, dispatch metodi/static, destructor.
- **`eval/builtins.rs`** (1521) — builtin evaluator-dispatched: higher-order (`array_map`/
  `filter`/`walk`/`usort`/`call_user_func*`), famiglie `preg_*`/`mb_ereg*`, `json_decode`,
  `serialize`/`unserialize`, `fopen`/dir/resource, class-introspection.
- **`eval/mod.rs`** (1913) — struct `Evaluator`, macro `frame_mut!`/`slot_mut!`, le funzioni
  `run*`/`Outcome`, i metodi core (`frame`/`read_var`/`eval_isset`/array/place) e le free-fn.

**Meccanica Rust sfruttata**: un `impl` di un tipo può essere spezzato su più file dello stesso
crate; un **modulo figlio vede gli item privati dell'antenato**, quindi i metodi spostati
accedono ancora a campi e free-fn privati di `mod.rs` senza cambi di visibilità. Le uniche
annotazioni `pub(super)` servono per i metodi chiamati *dal* padre o da un altro sottomodulo (il
padre non vede i privati del figlio). I macro (`frame_mut!`) sono in scope testuale: le
dichiarazioni `mod` stanno **dopo** la loro definizione. `cargo fix` ha poi sfrondato gli import
per-modulo. Risultato: monolite da 6.965 → `mod.rs` di 1.913 righe (**−72%**), clippy pulito,
927 test verdi.

> Nota DevEx: il `target-dir` di cargo è stato spostato fuori dal volume "Extreme Pro" (vedi
> `.cargo/config.toml`), che non sa fare hard-link della cache incrementale → build stale.
> `lower.rs` (3.783 righe) resta un candidato per lo stesso trattamento in un secondo momento.

## Step 61 — DevEx tooling (Gemini E + B) + modularizzazione di `lower.rs`

> Generato con assistenza AI (Claude Opus 4.8). Completati gli altri suggerimenti della
> code-review esterna (`analysis_and_suggestions.md`): diff unificato del runner (E), flag di
> trace HIR + **trace d'esecuzione** (B), test unitari low-level su `ops` (C), e lo split di
> `lower.rs` come già fatto per `eval.rs`. Scartata solo la macro per il binding dei builtin
> (D, rischiosa come i named-args, scope-out). 934 test verdi, clippy pulito.
>
> Mea culpa su C: l'avevo prima liquidato come "copertura già forte" guardando il *totale* dei
> test, ma `ops.rs` (913 righe, port di `zend_operators`, l'anima type-juggling) aveva **zero
> test inline** — coperto SOLO da `differential.rs`, che richiede il binario oracle e si
> **auto-salta** se assente (e l'oracle non sopravvive a un reboot). Aggiunti **7 test unitari
> oracle-independent** (in `crates/php-types/src/ops.rs`) che inchiodano le semantiche PHP 8.5
> più insidiose — loose-eq PHP 8 (`0 == "foo"` → false), three-way compare, identical
> type-strict, coercion + overflow int→float, div/modulo by-zero, pow/concat, increment
> Perl-style (`az`→`ba`, `Zz`→`AAa`) — girano in ~0ms senza oracle, così una discrepanza si
> localizza in secondi.

### E — diff unificato nel `phpt-runner`
`--list-fails` non stampa più due blob `expected "…"`/`got "…"` troncati a 200 char, ma un
**line-diff compatto**: un paio di righe di contesto, poi la regione divergente (`- atteso` /
`+ reale`), con l'header `@@ <EXPECT|EXPECTF> first diff at line N @@`. È **EXPECTF-aware**: una
riga attesa "combacia" se il suo pattern (come regex) matcha la riga reale, così il diff cade
sulla **prima divergenza reale** e non su ogni riga con un `%d`/`%s`. Righe lunghe clippate,
regione cappata. Cambia solo il *detail* del fail, non la classificazione. (Avevo sofferto
direttamente la troncatura durante gli step 58-59.)

### B — flag `PHP_RUST_TRACE` (dump HIR **+ trace d'esecuzione**)
Due metà, entrambe coperte (su **stderr** → non inquina lo stdout/`rendered` confrontato; valgono
per `phpr` e per il runner):
1. **Dump HIR** (static): `run_source*` (in `eval/mod.rs`) stampano l'HIR abbassato prima
   dell'esecuzione, per triage *lowering* vs *evaluation*. `=hir|1|full` = intero `Program`;
   `=body` = solo la lista di statement top-level.
2. **Trace d'esecuzione** (runtime): `=exec|stmt|all` logga **ogni statement mentre viene
   eseguito** — `[exec] L<line> <variante StmtKind>`, **indentato per profondità di chiamata** —
   così un `.phpt` in fallimento si segue fino al punto esatto in cui l'esecuzione diverge (la
   ricorsione/le call si annidano visibilmente). Il flag è letto **una sola volta** in un campo
   dell'`Evaluator` (`trace_exec`, niente getenv per-statement); a trace spento è un solo check
   booleano. `=all` combina dump HIR completo + trace.

Il nome della variante dello statement è ricavato da `{:?}` fino al primo delimitatore (niente
enumerazione manuale di tutte le `StmtKind`).

### Modularizzazione di `lower.rs` (refactor, zero cambi di comportamento)
Stessa meccanica di eval (step 60): `lower.rs` (**3.783 righe**) → `lower/mod.rs` + tre
sottomoduli, ognuno un `impl<'f> Lowerer<'f>`:
- **`lower/stmt.rs`** (406) — `lower_stmt`/`lower_stmts` + hoist passes (funzioni/classi).
- **`lower/class.rs`** (1.090) — classi/interfacce/trait/enum, metodi, proprietà, closure,
  arrow-function.
- **`lower/expr.rs`** (932) — dispatch `lower_expr`, interpolazione/heredoc, call,
  instantiation, member-access, args, place, array-elements.
- **`lower/mod.rs`** (1.412) — `LowerError`, `lower_source`, validate_goto, `BarrierKind`,
  `Scope`, struct `Lowerer` + metodi core, `AssignFlavour` e tutte le free-fn helper.

Niente macro qui (a differenza di eval), quindi più semplice. `pub(super)` solo sui metodi
chiamati cross-modulo; `cargo fix` ha sfrondato gli import. `mod.rs` 3.783 → 1.412 (**−63%**).
Trappola incontrata e risolta: terminare il range di estrazione alla `}` dell'ultimo metodo —
includere il doc-comment del metodo *successivo* lascia un "expected item after doc comment".

## Step 62 — Famiglia hash/encoding (`base64_*`, `md5`, `sha1`, `crc32`, `hash`)

Nuovo modulo `crates/php-builtins/src/encoding.rs` (registrato in `lib.rs`), tutto
**byte-exact** (le stringhe PHP sono byte: input e output binari sono `[u8]`).

- **`base64_encode`** — port a mano (alfabeto standard, padding `=`), nessuna dipendenza.
- **`base64_decode($s, $strict = false)`** — port **fedele** di `php_base64_decode_impl`
  (`ext/standard/base64.c`): lenient salta ogni byte fuori-alfabeto; strict salta solo il
  whitespace (`\t \n \r` e spazio — **non** `\v`/`\f`, come la `base64_reverse_table`),
  fallisce su byte invalido, su dati dopo `=`, su gruppo finale di un solo carattere
  (`i % 4 == 1`) e su padding malformato (`padding > 2 || (i + padding) % 4 != 0`). La prima
  versione ingenua (con `seen_pad` + `is_ascii_whitespace`) sbagliava proprio gli edge-case di
  padding di `base64_decode_basic_003.phpt` → riscritta replicando la macchina a stati `i % 4`
  del C. Ora il test passa.
- **`md5`/`sha1`** (`$binary = false`) — digest RustCrypto (`md-5`, `sha1`), output hex
  lowercase o raw secondo il flag.
- **`crc32`** — CRC-32 zlib/IEEE (poly `0xEDB88320`, reflected) via `crc32fast`; ritorna il
  valore unsigned pieno come int positivo (`crc as i64`), come su PHP a 64-bit.
- **`hash($algo, $data, $binary = false)`** — dispatch su `md5`/`sha1`/`sha256`/`sha384`/
  `sha512`/`crc32b` (`crc32b` == output di `crc32()` in hex); algoritmo ignoto → `ValueError`.
  `crc32` (senza `b`, variante BZIP2) volutamente **non** incluso ora: poligono diverso,
  nessun test del corpus lo esercita (i `crc32*.phpt` testano la funzione `crc32()`, non
  `hash('crc32')`).

Dipendenze nuove in `php-builtins/Cargo.toml`: `md-5`, `sha1`, `sha2`, `crc32fast` (mature,
minimali). 10 test unitari inline (vettori noti: `md5("")`, `sha1("abc")`, `crc32("123456789")`
= `0xCBF43926`, round-trip base64, strict/lenient).

**Verifica `.phpt`** (oracle = output reale di PHP nelle sezioni EXPECT): verdi
`md5`/`md5_basic1`/`md5raw`, `sha1_basic`/`sha1raw`, `crc32`/`crc32_basic`/`crc32_variation2-4`,
e tutti e 6 i `base64_*` di `ext/standard/tests/url`. Gli skip residui (`sha1.phpt`,
`md5_basic2`) dipendono da `sha1_file`/`md5_file` (altri builtin), non dalle funzioni di questo
step. Workspace **943 test** verdi, clippy `--deny=warnings` pulito.

Fix incidentale: clippy si è inasprito (`suspicious_open_options`) e segnalava `touch()` in
`file.rs` (`create(true)` senza `truncate` esplicito) — aggiunto `.truncate(false)`, coerente
con la docstring ("without truncating an existing one").

## Step 63 — `pack` / `unpack`

Nuovo modulo `crates/php-builtins/src/pack.rs`: port **fedele** di `ext/standard/pack.c` (host
little-endian — l'oracle è compilato su macOS x86_64/arm64, quindi "machine endian" == LE).

**Codici supportati** (tutti quelli di PHP): stringhe `a A Z` (NUL/space-pad, `Z` NUL-terminata),
nibble hex `h H` (low/high-first), interi `c C s S n v i I l L N V q Q J P` (con endianness:
`n N J G E` big, `v V P g e` little, gli altri machine=LE; `i/I` = `sizeof(int)` = 4),
float/double `f g G d e E`, e i controlli di posizione `x X @`. Endianness implementata come
"low `size` byte di `to_le_bytes()`; se big-endian, invertiti" — equivalente al `BSWAP`+shift del C.

**Fedeltà errori** (i `.phpt` li verificano in EXPECTF): `pack` lancia `ValueError` senza prefisso
(`Type X: not enough arguments` / `too few arguments` / `unknown format code` /
`integer overflow in format string`, come `zend_value_error`) ed emette `Warning` con prefisso
`pack(): ` (`'*' ignored`, `N arguments unused`, `not enough characters in string`,
`illegal hex digit`, `outside of string`). `unpack` lancia `ValueError` `Invalid format type X`
(senza prefisso) e `unpack(): Argument #3 ($offset) ...` (con prefisso, da `zend_argument_value_error`),
e `Warning` `unpack(): Type X: not enough input values, need N values but only M was/were provided`
ecc. Chiavi nominali in `unpack` (`"f[rep]name/..."`): namelen→chiave int `i+1`; rep==1→nome;
rep>1→nome+indice; canonicalizzazione numeric-string via `Key::from_bytes` (== `zend_symtable_update`).

**Protezione anti-OOM**: replicato `INC_OUTPUTPOS` (overflow check su `INT_MAX`) così
`pack("a2000000000", …)` dà `ValueError` invece di tentare un'alloc gigante (stessa classe del crash
`capacity overflow` di sprintf, step 58). La lunghezza del risultato è `outputpos` finale (non
`outputsize`), come `ZSTR_LEN = outputpos` nel C — `out.truncate(outputpos)`.

**Verifica**: `.phpt` `pack_A`/`pack_Z`/`pack_float`/`pack_arrays`/`unpack_offset`/`unpack_error`/
`unpack_bug68225` tutti verdi (`pack`/`pack64`/`pack64_32` sono skip indipendenti — `pack.phpt` è
32-bit-only via SKIPIF, gli altri due skip per sezione non gestita dal runner). Differential diretto
`phpr` vs oracle **byte-identico** su uno sweep ampio (h/H, nvNV, c, f/G/d/E, X/@, q/Q/J/P 64-bit,
unpack nominale). 5 test unitari inline. Workspace **948 test** verdi, clippy `--deny=warnings` pulito.

## Step 64 — `crypt`

Nuovo modulo `crates/php-builtins/src/crypto.rs` su crate **`pwhash`** (DES, BSDi/ext-DES,
MD5-crypt, SHA-256/512-crypt, bcrypt — glibc-compatibile). Sopra ci sta la dispatch di
`php_crypt` (`ext/standard/crypt.c`): `pwhash::unix::crypt` riconosce l'algoritmo dal prefisso
del salt esattamente come PHP, e ci aggiungo la **convenzione `*0`/`*1`** (salt `*0`/`*1` →
fallimento immediato; su qualunque errore restituisco `*1` se il salt iniziava per `*0`, altrimenti
`*0`) e il troncamento del salt a `PHP_MAX_SALT_LEN` (123). `crypt` richiede **esattamente 2
argomenti** → `ArgumentCountError`. Costanti `CRYPT_STD_DES`/`CRYPT_EXT_DES`/`CRYPT_MD5`/
`CRYPT_BLOWFISH`/`CRYPT_SHA256`/`CRYPT_SHA512` = 1 e `CRYPT_SALT_LENGTH` = 123 in `lower/mod.rs`
(PHP bundla ogni algoritmo, quindi tutte disponibili).

**Pre-check anti-hang su `rounds` (fix importante)**: per i salt `$5$`/`$6$` con `rounds=N$`,
PHP rifiuta `N ∉ [1000, 999999999]` restituendo `NULL` → `*0`. Senza questo controllo `pwhash`
calcolerebbe *davvero* `rounds=1000000000` (≈1e9 iterazioni → interprete bloccato per minuti):
verificato che `crypt_sha256.phpt` (caso `rounds=1000000000`) faceva **hang** prima del fix, ora
restituisce `*0` istantaneo come l'oracle.

**Risultato**: differential diretto `phpr` vs oracle **byte-identico** su STD-DES (`rl.3StKT.4T8M`),
EXT-DES (`_J9..rasm…`), MD5 (`$1$…`), bcrypt `$2a$` ASCII, `$2y$`, `$2b$`, SHA-256, SHA-512 (incl.
`rounds=`), salt invalidi → `*0`, `*0`→`*1`. `.phpt` verdi: dir `crypt/` **4/4** + `crypt`,
`crypt_variation1`, `crypt_blowfish_variation1/2`, `crypt_sha256`, `crypt_sha512`,
`crypt_des_error`, `bug54721`, `bug73058`. **3 divergenze** residue (D-64.1/2/3 in
`04-divergences.md`), tutte limiti di `pwhash` su casi deprecati/non-standard: variant `$2x$`,
bcrypt 8-bit `$2a/$2b/$2y` con password high-bit, salt md5 con caratteri fuori-alfabeto. Portare
l'esatta crypt_blowfish di Openwall è un port voluminoso per edge-case deprecati → documentato,
non forzato (policy del progetto). Dep nuova: `pwhash`. 4 test unitari inline. Workspace **952
test** verdi, clippy `--deny=warnings` pulito.

## Step 65 — `strtok` (chiusura backlog builtin)

`strtok` è **stateful** (mantiene un cursore tra le chiamate) → implementato come builtin
**evaluator-dispatched**, non puro. Nuovo campo `strtok_state: Option<(Vec<u8>, usize)>`
sull'`Evaluator` (stringa in tokenizzazione + offset del prossimo token), `None` quando la
stringa è esaurita — l'equivalente di `BG(strtok_string)`/`BG(strtok_last)` di PHP. Ramo
`b"strtok"` in `dispatch_higher_order`, metodo `ho_strtok` in `eval/builtins.rs`.

Port fedele di `PHP_FUNCTION(strtok)` (`ext/standard/string.c`): la forma a **2 argomenti**
`strtok($str, $tok)` (re)inizializza lo stato (cursore a 0); la forma a **1 argomento**
`strtok($tok)` riprende dal cursore; senza stato → `Warning "strtok(): Both arguments must be
provided when starting tokenization"` + `false`. L'algoritmo: salta i delimitatori iniziali
(azzerando lo stato se la stringa finisce mentre salta), prende il token fino al delimitatore
successivo (o fine stringa), e avanza il cursore oltre quel delimitatore. Set di delimitatori
come tabella `[bool; 256]` (byte-based, come `STRTOK_TABLE`). Per evitare conflitti di borrow
uso `self.strtok_state.take()` e ripristino lo stato aggiornato a fine calcolo (il path `last >=
pe` rimette lo stato senza azzerarlo, come il `RETURN_FALSE` del C che NON libera la stringa).

**Verifica**: tutti gli `strtok_*` `.phpt` (`strtok_basic`, `strtok_variation3..7`) **verdi**;
differential diretto `phpr` vs oracle **byte-identico** su edge-case (delimitatori multipli
iniziali/finali, token set vuoto, subject vuoto, esaurimento → `false`). Workspace **952 test**
verdi, clippy `--deny=warnings` pulito.

Con questo step il **backlog builtin del piano è chiuso** (step 62–65): hash/encoding,
`pack`/`unpack`, `crypt`, `strtok`. Le uniche divergenze residue sono i 3 D-64 (limiti `pwhash`
su edge-case crypt deprecati), documentati in `04-divergences.md`.

## Step 66 — fix interleaving diagnostici/output dei builtin

Bug di **ordinamento** nel rendering: per un builtin che scrive su stdout **e** solleva un
warning (`printf`/`vprintf`), il warning compariva *dopo* l'output invece che *prima*. PHP emette
il warning nel momento in cui avviene (durante la formattazione degli argomenti), quindi **prima**
che printf scriva il risultato:

```
printf("BEFORE[%s]AFTER\n", [1,2]);
```
- **oracle**: `Warning: Array to string conversion …` poi `BEFORE[Array]AFTER`
- **noi (prima)**: `BEFORE[Array]AFTER` poi il warning

Causa in `eval/calls.rs::dispatch_value_builtin`: appendeva l'output del builtin a `rendered`
e *solo dopo* chiamava `flush_diags()`. I diag del builtin (raccolti durante la formattazione)
finivano così dopo l'output. **Fix** (riordino di due righe): snapshot dell'output prodotto →
`flush_diags()` (rende i warning su `rendered`) → poi append dell'output. Ora l'ordine è
`[warning][output]` come PHP. Il numero di riga resta corretto (`cur_line` è ancora quello della
chiamata al momento del flush).

Sicuro per costruzione: tocca l'ordine solo quando un **value-builtin produce output E diag**
nella stessa chiamata — di fatto solo `printf`/`vprintf` (gli altri builtin che scrivono su stdout
non emettono warning, e quelli che emettono warning non scrivono su stdout). Verificato
byte-identico all'oracle su più casi (`%s`/`%d` con array, conversioni multiple, `vprintf`).
Workspace **952 test** verdi, clippy `--deny=warnings` pulito. Chiude il debito "warning-position"
annotato dallo step 59.

> Nota: alcuni `*printf*_variation*.phpt` restano rossi, ma **non** per l'interleaving: usano
> `fopen(__FILE__, 'r')` e il runner esegue lo script **in-process** (non lo materializza su disco),
> quindi `__FILE__` non esiste → `fopen` fallisce con un warning spurio. È un limite dell'harness,
> ortogonale a questo fix.

## Step 67 — harness: materializzazione dello script su disco (`fopen(__FILE__)`)

Molti `*printf*_variation*.phpt` fanno `fopen(__FILE__, 'r')` per mettere una risorsa nell'array
dei valori di test. Il runner eseguiva lo script **in-process** (`run_source_with`) senza
materializzarlo: `__FILE__` puntava a un `<test>.php` inesistente su disco → `fopen` falliva con un
`Warning: fopen(...): Failed to open stream` spurio in cima all'output, mascherando le vere
divergenze.

Fix in `phpt-runner/src/lib.rs::run_phpt`: prima di eseguire, scrivo `source` nel path `name`
(il `<test>.php` accanto al `.phpt`, esattamente come fa run-tests.php), eseguo, poi rimuovo.
**Guardato**: materializzo solo se il file NON esiste già (per non sovrascrivere un file companion
reale tipo `<test>.inc`/`.php`), e rimuovo **solo** ciò che ho creato. `name` resta invariato →
zero impatto sul matching dei path EXPECTF nel resto del corpus. `printf_variation2.phpt` ora
diverge a riga 168 invece che a riga 3 (il `fopen` funziona). Sweep `strings` **289→290 pass,
164→163 fail**, nessuna regressione; **952 test** workspace verdi, clippy pulito.

> Resta un **terzo** problema, ortogonale e separato, su quei test: `%s` di `sprintf`/`printf` su
> un oggetto con `__toString()` non invoca il metodo (emette `Warning: Object of class X could not
> be converted to string` + nome classe), mentre `echo`/concatenazione/cast `(string)` lo
> chiamano già correttamente. Il motore di formato (`php-builtins/src/format.rs`) è un builtin
> **puro**: non ha accesso all'evaluator per invocare `__toString`. Fix futuro = rendere la
> famiglia sprintf/printf evaluator-dispatched (o passare un hook di stringificazione nel `Ctx`).

## Step 68 — `%s` di sprintf/printf onora `__toString`

Terzo problema emerso dopo gli step 66/67: `%s` di `sprintf`/`printf` su un oggetto non chiamava
`__toString` (emetteva `Warning: Object of class X could not be converted to string` + nome
classe), mentre `echo`/concatenazione/cast `(string)` lo facevano già. Causa: il motore di formato
(`php-builtins/src/format.rs`) è un builtin **puro** — `Ctx{out,diags}` — e non può invocare un
metodo dell'oggetto; per giunta durante la chiamata l'evaluator tiene già `&mut out`/`&mut diags`,
quindi un hook reentrante è impossibile.

**Soluzione**: rendere la famiglia `sprintf`/`printf`/`vsprintf`/`vprintf`/`fprintf`/`vfprintf`
**evaluator-dispatched** (ramo in `dispatch_higher_order`, `ho_format` in `eval/builtins.rs`).
`ho_format` valuta gli argomenti, li passa per `format_resolve_objects` (sostituisce ogni
`Zval::Object` con `Zval::Str(self.stringify(&o)?)` — ricorsivamente dentro gli array, per la lista
di valori dei `v*`), poi chiama il builtin puro via `dispatch_value_builtin` (riusa
out/diags/interleaving dello step 66). Si riusa `self.stringify` (eval/class.rs), già usato da
echo/concat/cast: invoca `__toString` o solleva il **fatale corretto** se assente — quindi è
fedele in entrambi i casi (anche `printf("%s", new stdClass)` ora è un Error fatale, non un
warning). Resource/scalari/format-string passano invariati; i named-arg ai builtin erano già
respinti a monte di `dispatch_higher_order`.

**Verifica**: `echo`/concat/`(string)`/`printf`/`sprintf`/`vsprintf`/`vprintf` con oggetto
`__toString` ora **byte-identici** all'oracle. `printf_variation2`/`sprintf_variation2` passano;
sweep `strings` **290→292 pass / 163→161 fail**, zero regressioni; **952 test** verdi, clippy
pulito. Unica divergenza nuova **D-68.1** (specifier numerico su oggetto-con-`__toString`,
patologica) in `04-divergences.md`.

## Sessione F — switch del motore al bytecode VM ed eliminazione del tree-walker

Conclude la migrazione: il **bytecode VM** (`compile.rs` + `vm/`), costruito incrementalmente
fino alla parità comportamentale con il tree-walker (`eval/`), diventa l'**unico motore di
produzione**; `eval/` viene eliminato e con esso `corosensei` e l'unico `unsafe` non-FFI del
runtime. Il payoff progettato fin dal disegno della bytecode (header `bytecode.rs`) è realizzato:
generatori e Fiber girano su uno **stack di frame esplicito** (park dell'`ip` in una tabella del
VM), senza coroutine stackful né reborrow `*mut Evaluator`.

**F1 — switch del motore** (`b45e52f`): `php_runtime::{run_source, run_source_with, Outcome}` ora
risolvono al VM; la suite linguaggio (`tests/eval.rs`, 472 test), la CLI e `differential.rs` (vs
oracle PHP 8.5.7 reale) girano sul VM. Tre test risultano **VM-più-corretto** e vengono adeguati
(argument-count → `ArgumentCountError`, generator-rewind → `Thrown`). `b45e52f` espone però gaps
che `eval.rs` non copriva: `builtins.rs` (registry builtins, engine-agnostici) esercita costrutti
che il compilatore bytecode ancora rifiutava → suite **pinnata temporaneamente** a `eval` con
l'elenco esplicito dei blocker.

**Long-tail — chiusura di tutti i gap VM** (commit per pezzo): operatore `@` di soppressione
(`827bd2d`), `exit`/`die` come espressione (`8c4c574`), `goto`/label base + within-finally
(`a1aaba1`), poi i builtin un tempo evaluator-only — `json_decode` con `stdClass` via
`alloc_stdclass` (`b5bd6e9`), `mb_split`/`mb_regex_encoding`/`mb_regex_set_options` con lo stato
`MbRegexState` sul `Vm` (`8efefcf`), `sscanf`/`fscanf` con una nuova ABI per out-param **variadici
by-ref** (`Op::CallHostBuiltinScanf`, `bce4370`), l'intera famiglia `mb_ereg*` (13 builtin,
inclusi `&$regs` by-ref e il callback higher-order via `call_callable`, `a539336`). Infine tre
divergenze emerse facendo girare l'**intera** `builtins.rs` sul VM: l'introspezione `[parameter]`
delle Closure per `var_dump`/`print_r` (`a7a510c`), la funzione indefinita come **fatale a runtime**
anziché compile-reject (PHP la differisce: il nome diventa una stringa-callable + `Op::CallValue`,
`invoke_named` solleva l'`Error` al call-site dopo l'output, `abe647c`), e il routing del `goto`
attraverso un `finally` + lo scope-out del `goto` **dentro** un blocco (D-45.1, `f34d8d5`).
Quest'ultimo ha richiesto di tracciare un **percorso di scope per blocco**: un `goto` può solo
puntare a un label il cui scope è prefisso del proprio; il router del finally usa il range di
indirizzi per «goto dentro la regione» ma la **profondità di scope** per «label fuori» — un label
marker subito prima di un `try` condivide l'indirizzo di start del try (non emette op), e un test
solo-su-indirizzi lo leggeva erroneamente come interno producendo un salto all'indietro infinito.
Con questo, **tutti i 384 test di `builtins.rs` passano sul VM**.

**F2 — eliminazione di `eval/`** (`d9dbd64`, `d1e6353`): rimosso il pin di `builtins.rs` (ora sul
VM, 411 test verdi), poi cancellate ~7000 righe del tree-walker (`pub mod eval` + la directory
`eval/`) e tutto il machinery dual-engine di `phpt-runner` (`Engine{Eval,Vm}`, flag `--engine`,
forwarding al child) — l'harness di parità VM↔eval ha esaurito il suo scopo, `run_phpt`/`run_path`
usano sempre il VM. Il test di isolamento (`--isolate`) attendeva un SIGABRT da overflow dello
stack nativo: con lo stack di frame esplicito la ricorsione illimitata è ora un fatale pulito
**«Maximum call stack depth»**, e il test lo verifica.

**F3 — drop di `corosensei` e della macchina `GenDriver`/`unsafe`** (`eb9e1d2`): rimossi il trait
`GenDriver`, l'enum `GenStep` e il campo `GenState.driver` (eval-only — il VM parcheggia il frame
sospeso nella propria tabella `generators`), la dipendenza `corosensei` (Cargo.toml + lockfile), e
aggiornati i doc dei moduli `bytecode`/`vm`/`generator` allo stato a motore unico (con fix dei
link intra-doc `crate::eval` ora rotti).

**Verifica (F4)**: `grep` di `unsafe` su `php-runtime`/`php-types` **vuoto** (l'unico `unsafe` del
workspace è l'FFI `libc` in `php-builtins`); `corosensei`/`GenDriver`/`GenStep` **assenti** da
codice e `Cargo.lock`. Workspace **1496 test verdi, 1 ignored**; clippy pulito; `differential.rs`
(VM vs PHP 8.5.7) verde; `phpt-runner` gira VM-only end-to-end. Il corpus esterno Zend/tests non è
presente in-tree, quindi il pass-count E4 non è rimisurabile, ma la parità dual-engine è ora moot
(motore unico) e la fedeltà è ancorata al PHP reale via `differential.rs`.

> **Nota metodologica** (richiesta dall'utente): provata la modalità *Serena-first rigorosa*. Su
> file UTF-8 puliti `find_symbol --include_body`/`insert_after_symbol` sono precisi e mirati, ma
> Serena ha fallito ripetutamente sui punti critici — `UnicodeDecodeError` sui file `eval/`
> (byte non-UTF8 nelle fixture) e indice LSP **stale** dopo l'eliminazione di `eval/` (continuava
> a cercare `eval/mod.rs`), proprio dove servivano i `find_referencing_symbols` di F3. Guadagno
> non degno di nota a fronte dell'attrito → ritorno alla modalità ibrida `grep`+`Read`, come
> concordato.


---

# Sessioni G–N (25 giu → 7 lug 2026) — dal linguaggio all'ecosistema

> Cento commit in due settimane, riassunti per macro-filone. Il progetto cambia natura: chiusa
> la migrazione a motore unico (Sessione F), l'obiettivo diventa far girare **software PHP
> reale** — Composer, PHPUnit, Monolog, Doctrine — con output byte-identico all'oracle, usando
> ogni suite come corpus di test gratuito. Il gate permanente resta il corpus `Zend/tests`
> (runner `--isolate`, confronto fail-list per nome, zero pass→fail ammessi): in queste sessioni
> passa da ~1900 a **2138 pass**.

## Sessione G — Composer bring-up: Reflection, references runtime, TLS, private-mangling

Il primo traguardo ecosistema: **`composer about` gira oracle-identico**. Per arrivarci servono
quattro filoni intrecciati (~40 commit):

- **Reflection framework-grade**: ReflectionProperty::getType, getAttributes con
  IS_INSTANCEOF/$flags, ReflectionClassConstant, ReflectionUnionType/IntersectionType,
  ReflectionEnum, ReflectionExtension minimale, constant()/defined() su Class::CONST. Corpus +34.
- **References a runtime**: il pezzo mancante più profondo. By-ref argument binding per le
  call **dinamiche** (l'equivalente di SEND_VAR_EX: a compile-time non sai se il parametro è
  by-ref), return-by-ref da metodi e proprietà (`$x = &$obj->method()`), elementi by-ref nei
  literal array, `$x =& $this`, by-ref attraverso static calls, host builtins che scrivono
  attraverso le ref.
- **TLS / rete**: `file_get_contents('https://…')` via **ureq+rustls**, stream contexts
  (`stream_context_create` con opzioni http onorate), header di risposta side-channel,
  `openssl_x509_parse` via x509-parser (CaBundle), costanti PHP_* di piattaforma. phpr parla
  HTTPS senza toccare OpenSSL C.
- **Terzo motore regex**: **oniguruma** come fallback per i pattern PCRE che il motore
  principale non copre (subroutine, `(?(DEFINE))`, ricorsione, `(?P<name>)`), con anchoring
  `\G` corretto per gli offset-search.
- **Stage C — private-property mangling**: le proprietà private sono storate come
  `\0Class\0prop` (dual-slot come Zend), con un resolver scope-aware (`FieldScope`) threaded
  attraverso tutti i field-path. Prerequisito del futuro property layout flat. Corpus 1915→1927.

Completano il quadro: slot delle variabili globali condivisi tra compilation unit (gli include
vedono le stesse `$GLOBALS`), fallback di namespace a runtime per le funzioni non qualificate,
`version_compare`/`parse_url`/`assert()` funzionale con AssertionError, `strnatcmp`, `strtok`
stateful, `random_int`/`random_bytes` su getrandom, uasort/uksort.

## Sessione H — `composer require` end-to-end: Monolog installa e gira

**`composer require monolog/monolog` scarica, risolve, unzippa e autoloada** dentro phpr, e
il codice Monolog gira **byte-identico** all'oracle. I pezzi: estrazione **zip nativa** (crate
Rust, niente ext/zip C), include-scope corretto, Finder/SPL (SplDoublyLinkedList/Queue/Stack),
first-class callable, e soprattutto il **binder runtime dei named arguments** esteso a *tutte*
le forme di chiamata — Monolog usa named args sui costruttori dei handler e ogni call-site
dinamico deve riordinare/completare i default a runtime.

## Sessione I — PHPUnit 13.2 verde + suite Monolog

Il moltiplicatore di tutto il resto: **PHPUnit 13.2 installa via il nostro composer e gira
verde con output byte-identico**. Da qui in poi ogni libreria porta in dote la propria suite
come differential-test gratuito. Ha richiesto:

- **ext/dom in puro Rust** (PHPUnit legge la config XML via DOM): un albero DOM con builtin
  `__dom_*` host-side e le classi PHP nel prelude.
- **proc_open + stream_select/set_blocking** end-to-end (Symfony Process), **ext/curl
  easy-API** come facade su ureq/rustls (curl_multi assente → Composer ripiega su stream),
  fsockopen Tcp/Udp, subset **pcntl** two-phase, date relative (subset timelib).
- **Docblock retention** attraverso HIR/lower/compile (getDocComment reale), serialize-hooks
  (`__sleep`/`__serialize`/`__wakeup`), **union type-hints enforced** con TypeError fedeli,
  preg byte-true, `__PHP_Incomplete_Class`.

Con PHPUnit verde, la **suite Monolog** (1150 test) passa da 291 errori a **22 err / 3 fail**
— i residui sono triagiati (curl_multi, xdebug, edge di piattaforma).

## Sessione J — Doctrine (collections, event-manager) + fix O(n²) include + GC cycle collector

Aperto il filone Doctrine: **collections 2.6.0** installa end-to-end e la suite chiude a
**257 test verdi** (da 70 err/28 fail a 0/2, poi 0). Lezioni engine: ArrayAccess con
**deferred-dispatch** su dim-write e fused-op (isset/empty/unset attraverso offsetExists/Get),
static vars di closure **per-istanza** (non per op-array), enforcement dei type-hint sui
variadici, `__set` su field-path leaf con guardia di ricorsione.

Due filoni infrastrutturali resi urgenti dalle suite grosse:

- **Costo quadratico degli include eliminato** (seed-stub): il preload di PHPUnit passava da
  5.4 GB/45 s a **1.35 GB/12 s**. Sbloccata la suite doctrine/lexer (verde) e inflector
  (**1213 test verdi in 21 s**).
- **GC cycle collector reale**: strutture O(log n) (BTreeMap per id, max-heap con
  discard-on-pop), note dei ref-drop nel plumbing eccezioni (2625→27 under-note), e infine la
  **trial deletion** sui possible-roots — `gc_collect_cycles()` è quello di Zend, non uno stub.
  Corpus 2019→2035.

Chiusura con doctrine/event-manager **8/8 verde** e la semantica engine dell'interfaccia
`Serializable` (corpus 2040).

## Sessione K — PDO/pdo_sqlite + ext/sqlite3 su rusqlite: DBAL tutto verde

Il database layer, in 8 commit pianificati (Step 1–6 + sqlite3): **PDO / PDOStatement / PDORow
e SQLite3 / SQLite3Stmt / SQLite3Result su rusqlite (bundled)**, nuovo modulo `vm/pdo.rs`.
Scelte chiave: si **ri-prepara lo statement a ogni execute** (semantica PHP), errori
SQLSTATE/errmode/attributi/metadata verificati uno a uno sull'oracle. Risultato:
**doctrine/dbal 3769 test, 0 err, 0 fail** e instantiator 49/49.

La suite DBAL ha stanato fix engine profondi che nessun corpus aveva esposto: `strict_types`
è **per-unit e si decide al call-site**, bypass dell'output-buffering per `fwrite(STDOUT)`,
nomi delle **classi anonime** unici cross-unit (il nome per-unit collideva), nullsafe
`?->` short-circuit sull'intera catena, `$GLOBALS` bare, `??=` multi-chiave, `*_exists` che
autoloadano, wrapper `data://`, `@` su trigger_error annidato e per-Fiber. Corpus 2040→2060.

## Sessione L — doctrine/orm: dal crash a 12 errori su 3484 test

La suite ORM partiva a **2353 errori + stack overflow**; chiude a **12 err / 22 fail su 3484
test**. Il grosso è stato il **modello delle reference**: rebind `=&` sul leaf (flag dedicato
in field_write), `reset()`/`current()` che **dereferenziano** il ritorno (un ritorno Ref grezzo
diventava alias involontario → celle auto-referenziali → panic RefCell), pulizia dei **residui
Ref nei temp** dopo le alias-call (`clear_temp_binding`), by-ref builtin con radice non-place.
In più: **SimpleXML completo nel prelude** sopra i builtin `__dom_*` (quirk oracle-verificati:
children() tiene il parent come contesto, __get null-on-no-match per il bool-cast) e
`Op::PropIssetDyn` (isset su proprietà dinamica via `__isset`). Corpus 2060→2071.

## Sessione M — infra runner + il bucket compile-unsupported

Due mosse di infrastruttura e poi giù di lima sul bucket dei costrutti rifiutati a compile:

- **`--run-skipif` nel phpt-runner** (+ esecuzione `--CLEAN--` sempre): lo SKIPIF gira nella
  VM come `<test>.skip.php`. Sbloccati ext/pdo 7→121 test runnable e pdo_sqlite 69→85.
- **PDO batch 1**: ATTR_STATEMENT_CLASS completo (validazione eager, ctor privato via
  invocazione host), fetchAll FETCH_GROUP/UNIQUE/FUNC/CLASSTYPE (con i **valori moderni** delle
  costanti: GROUP=32, UNIQUE=64, FUNC=10, CLASSTYPE=128), bindColumn/FETCH_BOUND, classe
  `Pdo\Sqlite` in una seconda unit namespaced del prelude. PDO phpt 57→81.
- **Il bucket** (5 commit): named/spread args su *tutte* le call dinamiche; **variable
  variables** (`$$x`, `${expr}`, `$$$x` — attenzione: Nested = un livello di indirezione, non
  due); **fatal compile-time fedeli** resi come output (`Cannot use [] for reading`, cast
  `(unset)`, elemento array vuoto) via `LowerError::Fatal` che il runner esegue e confronta;
  `C::{$expr}` dynamic class constant (quirk oracle: `::class` dinamico è "Undefined
  constant"); i **5 fatal verbatim di zend_inheritance per i trait-alias** (l'alias di un
  metodo astratto è una feature, non un errore); **foreach in place arbitrari** (key/value su
  `$c->var['k']`, `$ks[]`, by-ref su proprietà — con l'ordine osservabile VALUE-prima-di-KEY);
  **dynamic static property name** (`C::$$x`, con operandi peekati per il re-run degli
  init-thunk). Corpus 2071→**2120**.

## Sessione N — object-handle free-list LIFO (il riuso degli #N di Zend)

Zend riusa gli handle degli oggetti liberati con una free-list LIFO in `objects_store`; phpr
coniava id monotoni, e ogni test che confronta gli `#N` esatti nel var_dump falliva. Il fix:
**`impl Drop` su Object, Closure e GenState** (condividono lo spazio handle) che spinge l'id in
una free-list thread-local nel momento esatto in cui muore l'ultimo `Rc` — timing identico a
Zend, verificato oracle-identico al primo probe (LIFO, unset-order, clone, generatori,
spl_object_id). Tre insidie: le tre struct **non sono più `Clone`** (una copia implicita
porterebbe l'id del sorgente e libererebbe un handle vivo — l'unico utente, il carrier di
serialize, ora usa `copy_with_id(0)`); l'id riusato va **ripulito dal bookkeeping della vita
precedente** (destructed, lazy_*, gc_roots); i transient del prelude (oggetti Reflection
intermedi) **spostano il top della free-list** rispetto al C — de-Reflectionizzati i path caldi
PDO chiamando gli hook host direttamente. Corpus 2120→**2138** (+18 fail→pass), PDO phpt 81→84.

**Stato a HEAD `e0b5080`**: corpus Zend/tests 2138 pass; cargo test 20 crate verdi; suite
ecosistema — DBAL 3769/0/0, ORM 3484 con 12 err/22 fail, collections/event-manager/instantiator/
lexer/inflector verdi, Monolog 22 err/3 fail su 1150, PHPUnit byte-identico. Prossimo filone
pianificato: **by-ref property hooks** (`&get`, PHP 8.4, 15 test) — recon e design già scritti.

## Sessione O — by-ref property hooks (`&get`, PHP 8.4): il filone chiude 15/15

Il piano scritto la sessione scorsa ha retto quasi alla lettera. Quattro step committati
(`7bdd1ed` → `c66f2fb` → `6e5a0fc`):

- **Lowering**: `&get` accettato; il body del hook compila come `function &f()` riusando il
  plumbing `fn_by_ref`/`ReturnRef` esistente (la forma arrow `&get => $this->_p` va specchiata
  a mano sul ramo Return del lower — costruiva `StmtKind::Return` direttamente, bypassando la
  logica by-ref). `&set` si tollera (Zend deprecation, non fatal). Le due validazioni fatal
  verbatim: *"Get hook of backed property C::p with set hook may not return by reference"*
  (backed anche per **ereditarietà** — serve `ancestor_prop_backed` al lowering, prima del
  flattening di compile_class) e la variance d'interfaccia *"Declaration of A::$p::get() must
  be compatible with & I::$p::get()"* (l'abstract hook by-ref si registra come `"&get"` nella
  lista abstract_hooks; i consumer che costruiscono `$p::get` strippano il marcatore).
- **Runtime, contesti valore**: `push_hook` marca `ret_deref = func.by_ref && !is_set` — il
  `Ret` esistente dereferenzia (lezione ORM: mai residui Ref nei temp). Nei contesti place il
  nuovo `byref_hook_root` esegue il hook **sincrono** (push_hook + drive_to_return, pattern già
  usato da call_method_sync/ho_get_object_vars) con `ret_deref` azzerato, e la cell ritornata
  diventa la **radice del path**: MakeRef, FieldAssign/AssignOp/IncDec (via `field_set_in_root`,
  col drain ArrayAccess estratto da field_set_mode), BindRefTo(Checked) — che per il rebind del
  prop stesso esegue il hook per i side effect e POI fatala "Cannot assign by reference to
  overloaded object" (bug007 stampa il get prima del fatal!).
- **foreach sugli oggetti hooked**: ObjVals/ObjRefs riscritti da snapshot-chiavi a **cursore
  live** (ogni IterNext ricalcola le entry visibili e produce la prima non ancora visitata —
  le prop aggiunte nel body si visitano, foreach_002). Le entry sono Slot (storage) o Hook
  (get dispatched allo step); set-only virtual si salta, set-only backed legge il backing; la
  vista hook è scope-aware (la private del scope shadowa la ridichiarazione del figlio — la
  sezione A/B di foreach.phpt). By-ref foreach lega la cell del `&get`; get by-value = fatal
  "Cannot create reference to property C::$p". Bonus: la scrittura dynamic-name
  `$o->{$n} = v` ora dispatcha i set hook (field_write deferisce le prop hooked a
  prop_set_magic_or_dynamic, che esegue il set hook sincrono **azzerando ret_cell** — con
  ret_cell impostato il drive bounded non termina mai).
- **Lazy + typed refs**: `realize_full` segue le CATENE di proxy (proxy il cui instance è
  stato resettato lazy). E le **typed references** minime: `Vm.typed_refs` registra le cell
  che aliasano storage di prop tipate (MakeRef su singolo step Prop — copre `&$o->typed`,
  gli arg by-ref e il ReturnRef del hook su backing tipato — più il binding foreach by-ref);
  StoreSlot attraverso una cell registrata coerce/checka con l'errore Zend *"Cannot assign
  string to reference held by property C::$p of type int"*. Due sottigliezze oracle: la
  sorgente **muore con l'oggetto** (weak sull'owner; l'handle della created-table non conta —
  typed_properties_094) e **clone la eredita** come nuovo owner (typed_properties_081).

Due regressioni intercettate dal gate per-nome e fixate al volo: il marcatore `&` di var_dump
sulle prop (mancava del tutto — aggiunto specchiando la regola degli elementi array) ha
scoperto che `public private(set)` non era modellato: `$r = &$foo->bar` da fuori deve legare
una **copia** (object_reference.phpt) — aggiunto `set_visibility` a PropDecl/PropInfo con il
copy-on-ref in MakeRef.

Corpus Zend 2138→**2163** (+25: i 15 del filone + 10 bonus, tra cui 5 typed_properties, due
gh di property_hooks, foreach_010); zero pass→fail; cargo test 20 crate verdi.

## Sessione P — i sei residui lazy-reset chiudono (stessa giornata del filone by-ref)

I 6 test `reset_*` deferiti dal filone lazy objects erano tutti root-caused da una settimana;
due dei pezzi costruiti stamattina (realize_full a catena, la tabella typed_refs) li hanno
resi improvvisamente a portata. Un commit (`9215f54`), corpus 2163→**2170**, lazy_objects
90→97:

- **Forwarding transitivo** (`lazy_prop_forward`): trigger_lazy + proxy_redirect in loop fino
  a stabilità nei quattro siti prop-access — un proxy la cui instance è stata ri-resettata
  lazy si ri-triggera (reset_as_lazy_real_instance).
- **Reflected-scope** in `install_lazy`: si lazificano solo gli slot del layout della classe
  RIFLESSA; le prop dichiarate dalla subclass si preservano (ordine mantenuto iterando il
  layout completo dell'oggetto), le **dinamiche si droppano sempre** (la triage vecchia
  diceva il contrario — reset_as_lazy_resets_dynamic_props l'ha smentita al gate), e le
  readonly GIÀ inizializzate dichiarate da una classe diversa dalla riflessa restano valore+
  marca (l'unlock readonly è per classe dichiarante). `lazy_reset_scope` per-oggetto si è
  rivelato inutile: `lazy_props` filtrata all'install codifica già tutto per la realize.
- **Ordine zend_object_make_lazy**: i contenuti spiazzati si rilasciano DENTRO la reset con
  uno **sweep sincrono** (`gc_sweep_impl(None)` — variante di gc_sweep che drive_to_return-a
  ogni dtor invece di schedulare col rewind dell'ip) e il marker lazy si imposta DOPO: il
  destructor osserva il target svuotato ma non-lazy. Il primo tentativo è panicato
  "a non-baseline Ret has a caller": il frame dtor aveva `ret_cell` — la stessa lezione
  drive_to_return che morde per la terza volta.
- **Il vero bug di side_effect_destruct** non era il timing del dtor ma un buco dei path
  misti: `$this->obj->b = v` (FieldAssign) scriveva lo storage del ghost SENZA trigger
  dell'init. Ora field_write deferisce i leaf-write su oggetti lazy a
  prop_set_magic_or_dynamic (stessa condizione dei hooked). Restano i gap su read-path e
  mid-path, oggi non osservati da nessun test.
- **Realize ghost**: i default di classe si riapplicano SOLO agli slot ancora still-lazy —
  fix generale che smette di clobberare i valori raw-set.

Il gate per-nome ha di nuovo pagato: la prima versione "dinamiche preservate" passava i 6
target ma rompeva resets_dynamic_props, invisibile al conteggio (+7 netto comunque).

## Sessione Q — lazy objects, lo sweep grande: 97→152/223

Dopo i sei reset_* la dir lazy_objects restava a 97/223 con ~117 fail in famiglie nette.
Quattro commit (`…→b8d4be4`), corpus 2170→**2225**:

- **La superficie di trigger completa**: `lazy_prop_access` gate-a il forwarding sul TIPO di
  accesso — se la proprietà dispatcha un hook o un magic accessor, l'init NON scatta (la
  famiglia `fetch_*_may_not_initialize`: sono gli accessi interni del body a triggerare).
  Cablata su tutte le op prop (get/set/silent/dynamic/isset/unset/compound/incdec) e, via
  `field_lazy_root`, sulle op path-based (FieldAssign*/MakeRef) che ora radicano il walk
  sull'oggetto realizzato invece di scarabocchiare i placeholder del wrapper.
- **La mappa Zend di cosa inizializza**: sì per read/write/isset/unset/`??`/compound/&-fetch/
  foreach/`==`/var_export/serialize/clone/json; NO per dispatch hook-magic, `(array)` cast
  (solo redirect della catena proxy), `===`, confronto same-handle, var_dump senza
  __debugInfo, prop skipped. Tre trappole scoperte coi gate: il same-handle compare non
  inizializza; serialize bypassava il realize sul pure-path (gate: has_serialize_hooks →
  true per i lazy); il raw-set su proxy INIZIALIZZATO scrive l'instance.
- **Clone**: realize della sorgente prima; il clone di un proxy inizializzato è un NUOVO
  wrapper proxy attorno al clone dell'instance, __clone una volta sola sulla copia.
- **Rollback**: eccezione nell'initializer → il ghost torna lazy byte-per-byte (snapshot di
  props/readonly/still-lazy) con initializer reinstallato; idem la factory del proxy.
- **Regole di contorno**: materialize ricostruisce la prop-table in ordine di dichiarazione;
  &-fetch su typed Undef non-nullable = errore verbatim; ReflectionProperty rigetta le
  private degli antenati; classe senza prop eleggibili = mai lazy (stdClass); classi interne
  (= prelude) rifiutate con i due wording Zend.

Residui 63, triagiati nel topic file di memoria (validazioni setRaw/skipLazy, famiglia
gh18038, serialize __sleep, __debugInfo, superficie Reflection isLazy/getInitializer).
