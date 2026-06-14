# Fase 2 — Mapping table: PHP (C) → Rust

> Generato con assistenza AI (Claude Fable 5). Data: 2026-06-13.
> Le decisioni D-Gn sono il contratto della Fase 3. Status: `confermato` salvo nota.

## Decisioni globali

| ID | Costrutto C / sottosistema | Scelta Rust | Razionale | Status |
|---|---|---|---|---|
| D-G1 | zval (union+tag, 16B) | `enum Zval { Undef, Null, Bool(bool), Long(i64), Double(f64), Str(Rc<PhpStr>), Array(Rc<PhpArray>) }` | ADT nativo; stessa taglia; nessun unsafe; `Undef` serve per la diagnostica "Undefined variable" | confermato |
| D-G2 | refcount + COW (`SEPARATE_*`) | `Rc` + `Rc::make_mut` su ogni scrittura | semantica di separazione identica, gratis | confermato |
| D-G3 | zend_string | `PhpStr { hash: Cell<u64>, bytes: Box<[u8]> }`; mai `String` | stringhe PHP = byte binari; hash lazy come Zend (non osservabile) | confermato |
| D-G4 | HashTable | `PhpArray` proprio: `Vec<Option<(Key,Zval)>>` (tombstone) + `HashMap<Key,u32>` + `next_free: i64` | indexmap non modella chiavi duali/canonicalizzazione/next_free; semantica osservabile > layout | confermato |
| D-G5 | zend_alloc (3.6K LOC) | allocator di sistema + ownership | per-request pool irrilevante per un processo Rust | confermato |
| D-G6 | zend_gc ciclico (2.4K LOC) | scope-out Tier 1; `Rc` basta | senza `&$x`/oggetti/closure il PHP procedurale non crea cicli (array sono valori COW) | confermato |
| D-G7 | TSRM | nulla (Send/Sync) | thread-safety by type system | confermato |
| D-G8 | lexer re2c + parser Bison + zend_ast (~25K LOC) | dipendenza **mago** (Apache-2.0) + bridge isolato in un modulo di php-runtime | Strategia A; parse error message divergenti → skip-list | confermato |
| D-G9 | zend_compile + opcodes + VM generata (~158K LOC) | lowering AST→**HIR** (slot variabili risolti, funzioni hoisted, span) + evaluator tree-walking `match` | decisione utente: comportamento, non architettura; escape hatch bytecode futuro | confermato |
| D-G10 | Optimizer + opcache (~72K LOC) | niente | un processo residente non ri-parsa; rustc ottimizza l'evaluator | confermato |
| D-G11 | zend_operators.c | **porting fedele** in `php-types::ops` (~1.5K LOC) | è l'anima osservabile di PHP; unico modulo tradotto riga-per-riga | confermato |
| D-G12 | double→string | due funzioni: `to_str_precision14` (echo) e `to_str_shortest` (var_dump, via Ryū di `format!("{}")` con correzioni) | rischio n.1; differential dedicato | da-validare |
| D-G13 | errori/warning | canale `Diag` nel contesto di esecuzione, scritti su stdout interleaved col formato `main/main.c:1493` | metà degli EXPECTF li asserta | confermato |
| D-G14 | riferimenti `&$x` | **slot-level** `enum Slot { Value(Zval), Ref(Rc<RefCell<Zval>>) }` (NON un variant di `Zval`); promozione on-demand; vedi sezione "Step 11" per le sotto-decisioni D-R* | ROI: sblocca la famiglia builtin by-ref + by-ref param | in-progress (step 11) |
| D-G15 | exit codes | fatal → 255, `exit(n)` → n, default 0 | `Zend/zend.c:1625` | confermato |
| D-G16 | builtin | trait `Builtin` + registry `HashMap<&[u8], fn>` in php-runtime; implementazioni in php-builtins | evita ciclo di dipendenze; espansione incrementale | confermato |

## Decisioni per modulo (strategie legacy-port)

| Modulo C | LOC | Strategia | Note |
|---|---|---|---|
| Zend/zend_operators.c | 3.9K | **C — full port** (semantico) | unico full port del progetto |
| Zend/zend_hash.c, zend_string.* | 4.5K | D — scoped port | solo semantica osservabile (§3 semantic model) |
| Zend lexer/parser/ast | ~25K | A — adapter (mago) | bridge isolato |
| zend_compile + VM | ~158K | sostituzione architetturale (HIR+evaluator) | non è porting: design nuovo |
| zend_alloc, zend_gc, TSRM, Optimizer, opcache, win32 | ~88K | scope-out totale | sostituiti dal linguaggio/architettura |
| ext/standard (subset) | 74K | F — selective port | funzione per funzione, guidato dalla frequenza nei test |
| ext/pcre, ext/date, ext/json, ext/hash, ext/mbstring… | ~400K | A — adapter su crate (Tier 3) | vedi piano, fuori scope Tier 1 |

## Step 11 — Reference semantics (design pass)

> Design pass scritto a fine step 10 (Claude Opus 4.8) **prima** dell'implementazione,
> radicato nel modello di storage reale dell'evaluator. L'implementazione TDD parte
> in una sessione dedicata. Le D-R* sono il contratto di quella sessione.

### Modello attuale (cosa cambia)

Oggi le variabili vivono in `slots: Vec<Zval>` (`eval.rs:141`), un `Zval` **per valore**
per slot. Lettura: `read_var` clona (`eval.rs:819`). Scrittura: `self.slots[slot] = v`
(`eval.rs:978`). Le chiamate utente fanno frame-swap con un `Vec<Zval>` fresco
(`eval.rs:471-478`). Gli heap-type (Str/Array) sono già `Rc` con CoW via `Rc::make_mut`
(D-G2). L'assegnamento `$a = $b` è una copia di valore (Rc-clone), semantica PHP corretta.

Il vincolo di ownership (Layer 1): un `Vec<Zval>` piatto **non può** esprimere "due slot
condividono lo stesso valore mutabile" — Rust vieta due `&mut` allo stesso dato. La
reference PHP (`zend_reference`, `IS_REFERENCE`) è esattamente aliasing mutabile
condiviso. In un interprete **single-thread** (D-G7: nessun thread) lo strumento
idiomatico è `Rc<RefCell<Zval>>` — non `Arc<Mutex>` — coerente con l'uso di `Rc` già
presente nel codebase.

### Reasoning chain

```
+-- Layer 1: aliasing mutabile condiviso (no due &mut su un Vec<Zval>)
|   Problema: $b = &$a deve far vedere a entrambi le scritture dell'altro
|       ^
+-- Dominio: interprete single-thread, modello Rc+CoW già in uso (m02/m03)
|   Vincolo: niente thread (D-G7) -> Rc non Arc; serve interior mutability
|       v
+-- Layer 2: scelta di design
    Decisione: enum Slot { Value(Zval), Ref(Rc<RefCell<Zval>>) },
               promozione lazy (come IS_REFERENCE wrappa solo quando serve)
```

### Decisioni

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-R1** | Rappresentazione | `enum Slot { Value(Zval), Ref(Rc<RefCell<Zval>>) }`; `slots: Vec<Slot>`. Il caso comune resta `Value` (zero overhead, nessun cambio di comportamento sui test esistenti). | Mirror fedele di Zend: una zval è un valore semplice e diventa `IS_REFERENCE` solo quando si applica `&`. Penalizza zero il 99% non-reference. **Scartato** "tutti gli slot `Rc<RefCell>`" (overhead su ogni read + rischio di sbagliare value-vs-ref) e **scartato** `Zval::Ref` come variant (blast radius enorme: ogni match in ops/convert/var_dump/builtin dovrebbe deref-are). |
| **D-R2** | Lettura variabile | `Value(z) → z.clone()`; `Ref(cell) → cell.borrow().clone()`. `read_var`/`silent_get`/`read_place_value` acquisiscono un `&Zval` via un helper `slot_value(slot) -> Zval` (o `with_slot`). | La lettura è sempre per valore (CoW preserva la semantica copy). |
| **D-R3** | Scrittura variabile | `$x = v`: se `Value` → rimpiazza con `Value(v)`; se `Ref(cell)` → `*cell.borrow_mut() = v` (**write-through**, visibile a tutti gli alias). | È la semantica PHP cruciale: assegnare a una variabile che *è* una reference scrive attraverso il legame. |
| **D-R4** | Creazione `$b = &$a` | Nuovo nodo HIR `AssignRef { target: Slot, source: Place }` (oggi `lower.rs:618` rifiuta l'operatore `&`). Eval: assicura che lo slot sorgente sia `Ref` (se `Value(z)` → promuovi a `Ref(Rc::new(RefCell::new(z)))`), poi `target` slot = `Ref(Rc::clone(cell))`. | Promozione lazy: la sorgente diventa reference solo qui. |
| **D-R5** | `unset($b)` su reference | Lo slot torna `Value(Undef)` (rilascia il suo `Rc`-clone della cella); gli altri alias mantengono il valore. | Semantica PHP: unset rompe **solo** quel legame, non il dato condiviso. Già esiste `unset_place` (`eval.rs:1036`). |
| **D-R6** | Parametri by-ref `f(&$x)` | `lower.rs:367` e `FnDecl`/`Param` guadagnano `by_ref: bool`. Il call path (`eval.rs:647` `Call`) per ogni arg by-ref **non** clona un valore ma lega la cella della variabile-argomento (promuovendola a `Ref` se serve) nello slot-parametro del callee. `argv: Vec<Zval>` diventa insufficiente → introdurre `enum Arg { Val(Zval), Ref(Rc<RefCell<Zval>>) }` (o risolvere gli arg by-ref separatamente prima del frame-swap). Un arg by-ref che non è una variabile (es. literal) → Error PHP "Only variables should be passed by reference" (Notice/Warning, poi passa per valore). | Sblocca la base per i builtin by-ref. |
| **D-R7** | Builtin by-ref (`array_push`/`sort`/`array_pop`/`array_shift`/`str_replace $count`) | Estendere l'ABI builtin (D-G16): una tabella di **arity by-ref** per builtin (quali posizioni sono `&`), e una nuova signature o un `Ctx` arricchito che dà accesso `&mut Zval` allo slot dell'argomento. Opzione minima: un secondo registry `RegistryRef` con signature `fn(&mut [Arg], &mut Ctx)`. Da rifinire in implementazione. | È il driver primario dello step (la famiglia è molto usata nel corpus). |
| **D-R8** | Scrittura annidata via reference (`$ref[0] = 1`) | `write_place`/`unset_place` (`eval.rs:976,1036`) ottengono `&mut Zval` dallo slot tramite l'helper di D-R2: per `Ref(cell)` usano `&mut *cell.borrow_mut()` passato a `write_into`. `write_into` resta invariato (lavora su `&mut Zval`). | Riusa tutta la logica CoW/auto-vivify esistente. |
| **D-R9** | var_dump / print_r | Le reference a livello di variabile sono **trasparenti**: si deref-a e si stampa il valore (PHP non annota `&` per le reference top-level). Nessun cambio a `dump`/`print_r_into`. | Mantiene il blast radius minimo. L'annotazione `&` compare solo per reference *dentro* array/oggetti → vedi scope-out. |

### Step 11d — Element-level references via `Zval::Ref` (design pass, sessione 2026-06-13)

> Brainstorming (architettura) → decisioni utente: **unificare** su `Zval::Ref`
> (rimuovere `Binding`); scope = **foreach-by-ref + element-&**, defer
> return-by-ref. Semantiche tutte verificate contro l'oracle
> `/tmp/php-src/sapi/cli/php` (foreach-by-ref `[1,2,3]→[10,20,30]`, lingering
> gotcha `1,2,2`, `$x=&$a[0]`, `$a[0]=&$x`, `&int(5)` in var_dump, ref-collapse,
> ref-survives-copy).

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-R10** | Rappresentazione (unificata) | Aggiungo `Zval::Ref(Rc<RefCell<Zval>>)`. **Invariante**: l'interno non è mai un `Ref` (ref-to-ref collassa; `slot_cell` riusa la cella esistente). **Rimuovo `enum Binding`**: gli slot tornano `Vec<Zval>`, una variabile-reference contiene `Zval::Ref(cell)`. Helper 11a/b/c rimappati su `Zval`: `slot_clone`→`deref_clone`, `slot_set`→write-through se `Ref`, `slot_cell`→promuove/clona la cella, `&mut Zval` (IncDec/`write_into`)→`&mut *c.borrow_mut()`. | Fedele a Zend (un solo IS_REFERENCE), rimuove un caso speciale. Scartato additivo (Binding+Zval::Ref) per non avere due rappresentazioni della stessa cosa. |
| **D-R11** | Deref-on-read (contenimento) | Nuovo `Zval::deref_clone(&self) -> Zval`. Un `Ref` esiste solo come slot/elemento e si dereferenzia appena materializzato. Siti (~9): `read_index`, snapshot `foreach` by-value, `var_dump`, `print_r`, builtin che leggono *valori* d'array (implode/in_array/array_values/array_merge/sort). | **`ops.rs`/`convert.rs` non cambiano** — non ricevono mai un `Ref` (zero rischio sui 37.835 differential). |
| **D-R12** | Element-& assignment | `AssignRef { target, source }` generalizza `Slot`→`enum { Var(Slot), Elem(Place) }` su entrambi i lati. `$x=&$a[0]`: promuovo l'elemento a `Ref(cell)` in-place (CoW), lego `$x` a clone della cella. `$a[0]=&$x`: scrivo `Ref(cella di $x)` nel place. lower.rs abbassa entrambi i lati come `Place`. | Riusa `slot_cell` + `write_into`. |
| **D-R13** | foreach-by-ref | `StmtKind::Foreach` guadagna `by_ref: bool`; lower accetta `&` sul value-target. eval: iterabile = variabile con array; snapshot delle **chiavi**; per ogni chiave promuovo `$a[k]` a `Ref(cell)` e lego il loop-var a `Ref(clone(cell))`. **Niente auto-unset** → lingering gotcha emerge naturalmente. | Mutazione propaga alla sorgente; fedele a PHP. |
| **D-R14** | var_dump / print_r | var_dump: elemento `Ref` → prefisso `&` + deref dell'interno. print_r: deref trasparente (NESSUN `&`, verificato oracle). Ref top-level restano trasparenti (D-R9). | Solo var_dump annota le reference *dentro* container. |
| **D-R15** | Cicli | `$a[0]=&$a` crea un ciclo; `Rc<RefCell>` lo leak-a. Accettato (D-G6, nessun GC ciclico Tier 1), documentato. | Coerente con la scelta `Rc` senza weak/GC. |

**Scope-out di 11d:** return-by-ref (`function &f()`), array-literal con elemento-ref (`[&$x]`), foreach-by-ref su non-lvalue.

**Sotto-suddivisione TDD 11d:** **11d-1** `Zval::Ref` + rimozione `Binding` + deref (refactor a parità di comportamento: i 185 test restano verdi); **11d-2** element-& (`$x=&$a[0]`, `$a[0]=&$x`); **11d-3** foreach-by-ref (+ lingering gotcha); **11d-4** var_dump `&` annotation.

### Step 12 — `global $x;` + `$GLOBALS['literal']` (design pass, sessione 2026-06-14)

> Dialogo di design → decisioni utente: fare **`global $x` + `$GLOBALS['literal']` insieme** (stessa infrastruttura, `global` ha più valore sul corpus e mappa su `Zval::Ref`), scope **nomi statici** (defer indici dinamici), meccanismo **refactor del frame** (overlay globals/locals). Semantiche verificate sull'oracle: `global` rw (`59`), `global` crea global (`7`), `$GLOBALS` rw (`38`), `$GLOBALS['n']=5` crea nuovo global (`5`), `isset($GLOBALS['z'])` indefinito → false senza warning.

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-12.1** | Frame model (refactor) | Sostituire lo swap in blocco di `self.slots` con `globals: Vec<Zval>` (sempre il frame globale) + `locals: Option<Vec<Zval>>` (Some mentre gira una funzione). Accesso via `frame()`/`frame_mut()` = `locals.as_mut().unwrap_or(&mut globals)`. Idem `names`. `call_user_fn` setta `locals = Some(fresh)` e ripristina al return. **Stesso refactor in `lower.rs`**: tenere `global_slots`/`global_index` in campi dedicati + overlay locale durante `lower_function` (oggi `mem::take`), così il global index è raggiungibile mentre si abbassa il corpo di una funzione. | È l'unico modo per raggiungere il frame globale per nome da dentro una funzione. Scartato l'hack "campo aggiuntivo" (due percorsi, debito). I ~13 access-site agli slot (già maneggiati in 11d) passano per `frame_mut()`. |
| **D-12.2** | `global $x;` | Nuovo `StmtKind::Global(Vec<GlobalBinding>)` con `GlobalBinding { local: Slot, global: Slot }`. Lowering: per ogni var, slot locale (scope corrente) + slot globale (scope globale, **pre-registrato** se assente). Eval: `let cell = make_cell(&mut self.globals[global])`; `self.frame_mut()[local] = Zval::Ref(Rc::clone(&cell))`. A scope globale è un no-op (local == global). | Riusa interamente `Zval::Ref`/`make_cell` di 11d: `global $x` *è* un alias-by-reference del globale. |
| **D-12.3** | `$GLOBALS['literal']` | `Place` guadagna `base: PlaceBase` con `enum PlaceBase { Local(Slot), Global(Slot) }` (oggi `slot: Slot` → `base`). Lettura: nuovo `ExprKind::GlobalVar(Slot)` (base d'espressione, così `$GLOBALS['x'][0]` = `Index{base: GlobalVar, ..}`). Scrittura/compound: il place con `base: Global(slot)` opera sul frame `globals`. I siti place (`resolve_steps`/`write_place`/`read_place_value`/`silent_get`/`unset_place`) leggono `self.frame_for(base)` invece di `self.slots[slot]`. | `$GLOBALS['x']` *è* la variabile globale `x`; modellarla come base globale gestisce nested/compound (`$GLOBALS['x'][0]+=1`) gratis. |
| **D-12.4** | Pre-registrazione slot globali | In lowering, incontrando `global $x` o `$GLOBALS['literal']`, garantire uno slot nel global index (anche da dentro una funzione). Così un globale referenziato solo via `$GLOBALS['n']` (mai come bare `$n`) ottiene comunque uno slot → `$GLOBALS['n']=5` crea il global (oracle `5`). | Evita l'overflow `HashMap` finché gli indici sono literal. |
| **D-12.5** | Diagnostica | `$GLOBALS['undef']` in lettura → come una lettura di global indefinito (verificare sull'oracle in implementazione: probabile "Undefined variable" o "Undefined array key"). `isset($GLOBALS['z'])` → false silenzioso. | Da rifinire contro l'oracle nel sotto-step 12-3. |
| **D-12.6** | Scope-out | `$GLOBALS[$dynamic]` (indice non-literal), `$GLOBALS` come array intero (`foreach($GLOBALS)`, passarlo, `count($GLOBALS)`), globali engine (`argc`/`argv`/`_SERVER`…). | Richiedono risoluzione nome→slot a runtime + overflow `HashMap<Vec<u8>,Zval>` per globali non nella slot table. Deferiti. |

**Sotto-suddivisione TDD step 12:** **12-1** refactor frame overlay globals/locals (parità: i 201 test restano verdi); **12-2** `global $x;` (ref-based, riusa `Zval::Ref`) — TDD da `$x=5; function f(){global $x; $x=9;} f(); echo $x;` → `9`; **12-3** `$GLOBALS['literal']` read/write/compound + pre-registrazione (`Place.base`) — TDD da `$x=3; function f(){$GLOBALS['x']=8;} f(); echo $x;` → `8`.

**Step 12 IMPLEMENTATO (sessione 2026-06-14, TDD, zero D-NEW):** +12 test (201→213), tutto oracle-verificato, clippy pulito.
- **12-1 `9a8b69d`** (refactor a parità): eval.rs `slots`→`globals: Vec<Zval>` + `locals: Option<Vec<Zval>>`, `names`→`global_names`+`local_names`; macro `frame_mut!` (macro, non metodo, così il borrow tocca solo locals/globals e `diags` resta prendibile in parallelo), accessor `frame()`/`names()`; `call_user_fn` installa/ripristina l'overlay locale. lower.rs: estratto `struct Scope { slots, index }`, Lowerer con `globals: Scope` + `locals: Option<Scope>`, `slot_for` sullo scope attivo, `lower_function` installa overlay fresco. I 201 test restano verdi.
- **12-2 `a20f832`** (`global $x;`): `StmtKind::Global(Vec<GlobalBinding{local,global}>)`. Lowering registra slot locale (alias) + slot globale **pre-registrato**. Eval: `make_cell(&mut globals[g])` + `frame_mut!(self)[l] = Zval::Ref(clone)` — riusa interamente lo `Zval::Ref` di 11d; global indefinito promosso a cella NULL → la scrittura *crea* il global. No-op a scope globale (`locals.is_none()`). `global $$x` → Unsupported. +5 test (9, 42, 7, 3, 3_99).
- **12-3 `da509fb`** (`$GLOBALS['literal']`): `Place.slot`→`Place.base: PlaceBase{Local|Global}`; nuovo `ExprKind::GlobalVar(Slot)` per le letture. Lowering riconosce `$GLOBALS['stringa-literal']` (`globals_key`), pre-registra lo slot globale → `$GLOBALS['n']=5` crea il bare global. Fast-path assegnazione bare-var gated su base `Local`. Eval: macro `slot_mut!` + `base_clone` instradano i 6 place-helper (write_place/read_place_value/silent_get/unset_place/ref_source_cell/bind_ref_target) al frame globale per base `Global`. Lettura di `$GLOBALS['undef']` → warning distinto "Undefined global variable $name"; `isset($GLOBALS['z'])` falso silenzioso. +7 test (8, 10, 5, 5, 9, nY, 7).
- **Scope-out confermati (D-12.6):** `$GLOBALS[$dynamic]`, `$GLOBALS` come array intero (`count`/`foreach`/passaggio), globali engine — richiedono overflow `HashMap` runtime. Bonus emerso: `$x = &$GLOBALS['y']` funziona gratis (ref_source_cell base-aware).

### Step 14 — type-hint enforcement (scalari, weak mode) (design pass, sessione 2026-06-14)

> Chiude D-NEW-6 (step 8: hint accettati ma non enforced). L'utente ha scelto type-hint enforcement come prossimo step. Semantiche verificate sull'oracle PHP 8.5.7 (weak mode, default). **Coercion param è più stretta del cast `(int)`**: `f(int $x); f("12abc")` → **TypeError** (non `12`); solo stringhe numeriche ben formate coercono. Risultati chiave: `int<-"123"`=123, `int<-3.0`=3 (no dep), `int<-3.7`=Deprecated "Implicit conversion from float 3.7…"+3, `int<-"1.5"`=Deprecated "…from float-string \"1.5\"…"+1, `int<-"1.0"`=1 (no dep), `int<-true`=1, `int<-null`=TypeError, `int<-[1]`=TypeError; `float<-"1e3"`=1000.0, `float<-"abc"`=TypeError; `string<-42`="42", `string<-true`="1"; `bool<-0`=false, `bool<-"x"`=true; `?int<-null`=NULL; return `:int` coerce uguale ma messaggio diverso. Messaggi: arg = `f(): Argument #1 ($x) must be of type int, string given, called in <file> on line <L> and defined in <file>:<DL>`; nullable mostra `?int`; return = `f(): Return value must be of type int, string returned in <file>:<DL>`.

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-14.1** | Scope | Enforcement SOLO dei 4 hint scalari (`int`/`float`/`string`/`bool`) + nullable `?T`, in **weak mode** (default). Ogni altro hint (array/iterable/object/callable/nome-classe/union/intersection/mixed/void/self/…) → nessuna enforcement (accettato as-is = comportamento attuale). | I fail D-NEW-6 sono quasi tutti coercizione scalare. Union/classi/strict richiedono molto più lavoro (e OOP). |
| **D-14.2** | Rappresentazione | `enum ScalarType { Int, Float, String, Bool }`, `struct TypeHint { kind: ScalarType, nullable: bool }`. `Param.hint: Option<TypeHint>`, `FnDecl.ret_hint: Option<TypeHint>`. Lowering mappa `Hint::Integer/Float/String/Bool` e `Hint::Nullable(inner-scalare)`; ogni altro Hint → `None`. | HIR-level; un `None` significa "non enforced" (uniforma scope-out e hint assenti). |
| **D-14.3** | Motore coercizione (weak) | `Evaluator::coerce_to_hint(value, &TypeHint) -> Result<Zval, GivenType>` (Err porta il nome-tipo PHP del valore per il messaggio). Regole sotto. Riusa `numstr::parse_numeric_ex(s,false)` (rifiuta `trailing`), `convert::{dval_to_lval_safe,to_double,to_zstr,to_bool}`. `null`→ok solo se `nullable`; array/object→sempre Err. | Le primitive numeriche/convert esistono già da step 10; il motore è orchestrazione. |
| **D-14.4** | Param TypeError | In `run_user_fn_body`, dopo aver calcolato il binding by-value (NON per `Arg::Ref` né per i default), applica `coerce_to_hint`. Err → `PhpError::TypeError("{fn}(): Argument #{n} (${pname}) must be of type {hint}, {given} given, called in {file} on line {callline} and defined in {file}:{defline}")`. `callline = self.cur_line` (linea della call, già impostata quando si valuta il `Call`); `pname = f.slots[param.slot]`; `defline = f.line`. | La coercizione avviene al bind, prima del corpo. |
| **D-14.5** | Return TypeError | In `run_user_fn_body`, dopo `exec_stmts`, se `ret_hint` Some coerce il valore di ritorno (by-value). Err → `"{fn}(): Return value must be of type {hint}, {given} returned in {file}:{defline}"` (formato diverso: no "called in", suffisso "returned in F:DL"). | Solo by-value; un `function &f(): int` con return-by-ref resta scope-out. |
| **D-14.6** | Diagnostica deprecation | float→int con frazione → Deprecated "Implicit conversion from float {repr} to int loses precision" (riusa `dval_to_lval_safe`). float-string→int con frazione → Deprecated "Implicit conversion from float-string \"{orig}\" to int loses precision" (messaggio custom: "float-string" + stringa originale quotata). | Verificato: `3.0`/`"1.0"` NON deprecano, `3.7`/`"1.5"` sì. |
| **D-14.7** | Scope-out | `declare(strict_types=1)`, hint union/intersection/classe/object/array/iterable/callable/mixed/void/self/parent/static, param variadici tipati (già unsupported), coercizione su param by-ref. | Richiedono strict-mode engine, OOP, o sono rari. |

**Tabella coercizione weak (target ← sorgente):**

| target | Long | Double | Bool | Str (numerica ben formata) | Str (non num.) | Null | Array |
|---|---|---|---|---|---|---|---|
| **int** | as-is | frac==0→trunc; else Dep+trunc | 0/1 | int→val; float→(frac==0→val; else Dep-float-string+trunc) | **Err** | Err* | Err |
| **float** | →f64 | as-is | 0.0/1.0 | →f64 | Err | Err* | Err |
| **string** | to_zstr | to_zstr | "1"/"" | as-is | as-is | Err* | Err |
| **bool** | to_bool | to_bool | as-is | to_bool | to_bool | Err* | Err |

(*) `null` con `nullable=true` → resta `Null` (ok). Nome-tipo per "{given}": Long→`int`, Double→`float`, Str→`string`, Bool→`bool`, Null→`null`, Array→`array`.

**Sotto-suddivisione TDD step 14:** **14-1** rappresentazione (`TypeHint`/`ScalarType` + lowering) + motore coercizione param (successi int/float/string/bool/nullable) + Param TypeError; **14-2** deprecation float→int (float e float-string) + return type enforcement.

**Step 14 IMPLEMENTATO (sessione 2026-06-14, TDD, chiude D-NEW-6):** +11 test (220→230 net, -1 test obsoleto sostituito), oracle-verificato, clippy pulito.
- **14-1 `8dd9331`**: nuovi tipi HIR `ScalarType{Int,Float,String,Bool}` + `TypeHint{kind,nullable}` con `display_name()`. `Param.hint` + `FnDecl.ret_hint` via `lower_hint` (mappa `Hint::Integer/Float/String/Bool` + `Nullable` scalare; ogni altro → `None`). Motore `coerce_to_hint` + `coerce_to_{int,float,string,bool}` (free fn in eval.rs) applicano la coercizione weak al bind by-value in `run_user_fn_body`; più stretta del cast `(int)` (solo stringhe numeriche ben formate, riusa `numstr::parse_numeric_ex(s,false)`). Fallimento → `arg_type_error` con messaggio PHP esatto. Sostituito il test "hint accettati ma non enforced". +5 test.
- **14-2 `7b4e5a1`**: return type coercion (in `run_user_fn_body` dopo `exec_stmts`, skip se `by_ref`) + `return_type_error` (formato "Return value must be of type … returned in F:DL"). Deprecation float→int (riusa `dval_to_lval_safe`) e float-string→int (messaggio custom "float-string") già cablate in 14-1, qui testate. +5 test.
- **Default coercion (chiude D-NEW-6 completamente):** anche i default sono coercizzati (`float $n = 0` → `float(0)`). +1 test. 
- **Scope-out confermati (D-14.7):** `declare(strict_types=1)`, hint union/intersection/classe/array/iterable/mixed/void, param variadici tipati, coercizione su param by-ref.

### Step 13 — return-by-reference (`function &f()`) (design pass, sessione 2026-06-14)

> Dialogo → l'utente ha scelto return-by-ref come prossimo step (piccolo, il modello `Zval::Ref` è pronto da 11d/12). Semantiche verificate sull'oracle PHP 8.5.7: `function &f(){ global $x; return $x; } $y=&f(); $y=99;` → global a `99`; `$y=f()`/`echo f()` (contesto valore) → **copia** (`1`/`5`); `return <non-lvalue>` o `return;` in fn by-ref → Notice "Only variable references should be returned by reference" + valore (NULL per bare return); `$y=&normalfn()` (fn NON by-ref) → Notice "Only variables should be assigned by reference" + valore; `$y=&byref_fn_che_ritorna_nonplace()` → **solo** il Notice interno (no outer).

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-13.1** | Accettare la sintassi | `FnDecl.by_ref: bool` (lowering legge `func.ampersand`); rimosso il reject "function returning by reference" in `lower.rs`. | Prerequisito; il flag serve anche al call-site per decidere il Notice outer. |
| **D-13.2** | Return di un place | Nuovo `StmtKind::ReturnRef(Place)`. Eval: `ref_source_cell(place.base, steps)` → `Flow::Return(Zval::Ref(cell))`. | Riusa interamente la macchina cell di 11d/12 (`make_cell`/`place_cell`): un return-by-ref *è* la promozione del place a cella condivisa. |
| **D-13.3** | Quando abbassare a ReturnRef | Lowerer flag `fn_by_ref` (settato in `lower_function`). `return <expr>` → se `fn_by_ref` && `<expr>` è lvalue (`Variable::Direct` / `ArrayAccess` / `Parenthesized`) → `ReturnRef(lower_place)`; altrimenti `Return(lower_expr)`. | La detection lvalue va fatta a lowering (ha `lower_place`); il runtime riceve già la forma giusta. |
| **D-13.4** | Notice "Only variable references…" | Runtime field `fn_returns_ref: bool` (save/restore in `call_user_fn` come `locals`). Un `StmtKind::Return` (NON ReturnRef) eseguito con `fn_returns_ref==true` emette il Notice. | Copre in un colpo `return;` e `return <nonplace>` dentro una fn by-ref (entrambi non sono `ReturnRef`). |
| **D-13.5** | Call-site `$y = &f()` | Nuovo `ExprKind::AssignRefCall { target: Place, call: Box<Expr> }`. Lowering: nel ramo `&`-rhs esistente, se `u.operand` è una `Call` → `AssignRefCall` invece di `AssignRef`. Eval: chiama **raw** (no deref); `Zval::Ref(cell)` → bind target alla cella; valore → se il callee NON è by-ref emette "Only variables should be assigned by reference", poi bind a cella fresca col valore. | Un call non è un `Place`: variante dedicata, lascia intatto l'`AssignRef` di 11d. Il flag `by_ref` del callee (via `fn_index`) decide l'outer Notice (oracle F: solo inner se callee by-ref). |
| **D-13.6** | Contesto valore | `eval(ExprKind::Call)` deref-a il risultato della user-fn (`Zval::Ref` → copia). I builtin non ritornano mai `Ref`. | `$y=f()`/`echo f()` devono copiare; solo `$y=&f()` (AssignRefCall) prende la cella raw. |
| **D-13.7** | Scope-out | `static $x` (feature separata, serve per i contatori), return-by-ref di proprietà (no OOP), return-by-ref dentro `foreach`. | Fuori Tier 1 corrente; i due casi-test (global, elemento via param by-ref) non li richiedono. |

**Sotto-suddivisione TDD step 13:** **13-1** core return-by-ref (`FnDecl.by_ref` + `ReturnRef` + `AssignRefCall` + deref contesto-valore) — TDD da `$x=1; function &f(){global $x; return $x;} $y=&f(); $y=99; echo $x;` → `99`, più elemento-via-param-byref (`99`) e contesto valore (`echo f()`→`5`, `$y=f()`→copia); **13-2** diagnostica (i due Notice via canale `diags`).

**Step 13 IMPLEMENTATO (sessione 2026-06-14, TDD, zero D-NEW):** +7 test (213→220), oracle-verificato, clippy pulito.
- **13-1 `b6c76ee`** (core): `FnDecl.by_ref` (lowering legge `func.ampersand`, rimosso il reject). Dentro una fn by-ref, `return <lvalue>` → nuovo `StmtKind::ReturnRef(Place)` che promuove il place a cella condivisa (riusa 11d/12) e ritorna `Zval::Ref(cell)`. Call-site `$y=&f()` → nuovo `ExprKind::AssignRefCall{target,call}`: `assign_ref_call` chiama **raw** (`eval_call_for_ref`, no deref) e aliasa la cella; risultato non-Ref → cella fresca. Contesto valore (`$y=f()`, `echo f()`): `eval(Call)` deref-a il risultato della user-fn. Lowering: predicato `is_returnable_lvalue` + flag `fn_by_ref` nel Lowerer. +4 test (99, 99, 5, 1).
- **13-2 `87f676d`** (diagnostica): runtime field `fn_returns_ref` (save/restore in `call_user_fn` da `FnDecl.by_ref`). Un `StmtKind::Return` (non ReturnRef) dentro fn by-ref → Notice "Only variable references should be returned by reference" (copre `return;` e `return <nonplace>`). `assign_ref_call` → Notice "Only variables should be assigned by reference" quando il callee NON è by-ref (callee by-ref che ritorna non-place ha già emesso il suo Notice — oracle F). +3 test (canale `diags`).
- **Scope-out confermati (D-13.7):** `static $x`, return-by-ref di proprietà (no OOP), return-by-ref in `foreach`. Bonus: il modello regge anche `$x = &$GLOBALS['y']` (da step 12) senza modifiche.

### Scope-out espliciti (oltre Tier 1)

| Fuori scope | Perché | Cosa richiederebbe |
|---|---|---|
| Return by-reference (`function &f()`) | ~~Raro nel corpus Tier 1~~ **→ implementato in step 13** (vedi sezione Step 13). | — |
| `static $x` in funzione | Stato persistente cross-call; serve per i contatori return-by-ref. | Slot persistente per (funzione, nome), inizializzato una volta. |
| GC ciclico | Con element-ref i cicli diventano possibili (`$a[0]=&$a`); leak accettato (D-R15/D-G6). | `Rc` → servirebbe weak/cycle-collector. |

### Suddivisione in sotto-step (proposta per la sessione dedicata)

- **11a** ✅ (`cb403bc`) — `Binding` enum + read/write-through + `$b = &$a` + `unset` (D-R1..R5, D-R8, D-R9).
- **11b** ✅ (`06ddf17`) — parametri by-ref `f(&$x)` (D-R6).
- **11c** ✅ (`81ae800`) — builtin by-ref: `array_push`/`sort`/`array_pop`/`array_shift` (D-R7).
- **11d** ⏳ (design sopra) — element-ref + foreach-by-ref via `Zval::Ref` (D-R10..R15), 4 sotto-step TDD.

### Primo move della sessione dedicata

`superpowers:brainstorming` breve per validare D-R1 vs alternative (è una scelta
architetturale), poi `superpowers:test-driven-development` partendo da 11a. Verificare
ogni semantica contro l'oracle `/tmp/php-src/sapi/cli/php` come fatto allo step 10.

## Bug-class eliminate gratis dal target

1. Buffer overflow (bounds check), 2. use-after-free (ownership), 3. leak su error path
(RAII/Drop), 4. corruzioni della HashTable C (collezioni sicure), 5. race su stato
globale (Send/Sync), 6. errori di refcount manuale (Rc).

## Cose esplicitamente NON portate (Tier 1)

OOP (classi, interfacce, traits, enum, closures, generators, fibers), exceptions
user-level (`try/catch`), riferimenti `&` *dentro array* + foreach-by-ref + return-by-ref
(le reference a livello di variabile sono lo step 11, vedi sezione dedicata),
include/require, namespace, eval,
superglobals web ($_GET…), resources, INI system (default hardcoded: display_errors=1,
precision=14, serialize_precision=-1), opcache/JIT, ZTS.

## Punti di review per l'umano

1. **D-G12** (float formatting): se il differential mostra divergenze sistematiche sulla
   modalità precision=14, si porta `zend_gcvt` fedelmente (~150 LOC). Accettato?
2. **D-G8**: se mago non copre un costrutto 8.5 usato dai test, fallback = skip-list,
   non patch a mago. Accettato?
3. Ordine warning vs output bufferizzato: assumiamo stdout unbuffered interleaved
   (CLI default). Se i .phpt rivelano differenze, si adegua.
