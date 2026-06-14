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

### Step 17 — espansione builtin per frequenza (gruppi string/math/array) (design pass, sessione 2026-06-14)

> Stesso pattern di step 10: funzioni **pure**, ABI esistente `fn(&[Zval], &mut Ctx)`, **zero modifiche all'evaluator**, TDD isolato per gruppo, ognuna verificata byte-per-byte contro l'oracle PHP 8.5.7 (`php -n -r`). 24 builtin in 5 gruppi, scelti per frequenza d'uso nel corpus `/tmp/php-src/tests` + `Zend/tests` (vedi priorità in `php-rust-next-step7`). Niente by-ref (tutte by-value). Semantiche chiave verificate sull'oracle (recon di sessione): vedi note per gruppo.

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-17.1** | Gruppo case (`strtoupper`/`strtolower`/`ucfirst`/`lcfirst`/`ucwords`) | Trasformazione **ASCII-only** byte-wise (`A`-`Z`/`a`-`z`); i byte ≥ 0x80 restano intatti (no locale, no Unicode). `ucfirst`/`lcfirst` toccano solo il primo byte; `ucwords` capitalizza dopo ogni separatore (default `" \t\r\n\f\v"`). Stringa vuota → "". | Oracle: `strtoupper("héllo")`→`"HéLLO"` (é intatto). PHP CLI usa la C locale → ASCII-only. |
| **D-17.2** | Gruppo build (`str_repeat`/`str_pad`/`chr`/`ord`) | `str_repeat($s,$n)`: `$n<0` → `ValueError` ("must be greater than or equal to 0"), `$n==0`→"". `str_pad($s,$len,$pad=" ",$type=STR_PAD_RIGHT=1)`: `$len<=strlen`→`$s` invariato; pad vuoto→`ValueError` ("must not be empty"); type 0=LEFT,1=RIGHT,2=BOTH (both: extra a destra). `chr($n)`: `(($n % 256)+256)%256` → 1 byte. `ord($s)`: primo byte (0 se vuota). | Oracle-verificato. **Scope-out:** le Deprecation 8.5 di `chr` (fuori [0,255]) e `ord` (stringa vuota/multi-byte) — emesse dall'oracle ma rare; il valore di ritorno è comunque corretto. |
| **D-17.3** | Gruppo trim (`trim`/`ltrim`/`rtrim`) | Charlist default `" \t\n\r\0\x0B"`. Charlist esplicita supporta i **range** `a..z` (come PHP: `c1..c2` espande l'intervallo di byte). Trim toglie i byte in set da inizio (`l`)/fine (`r`)/entrambi. | Oracle: `trim("a1b2c","a..c")`→`"1b2"`. Il range è una feature reale di PHP, non un letterale. |
| **D-17.4** | Gruppo math (`intdiv`/`pow`/`sqrt`/`floor`/`ceil`/`round`) | `intdiv`: troncata verso zero; `/0`→`DivisionByZeroError`; `intdiv(PHP_INT_MIN,-1)`→`ArithmeticError`. `pow`: **int** se base+exp interi e exp≥0 (con overflow→float), altrimenti **float**. `sqrt`→float (neg→NAN). `floor`/`ceil`/`round`→**sempre float**; `round($v,$prec=0)` half-away-from-zero, precision negativa ok (`round(1234.5,-2)`→`1200.0`). | Oracle-verificato: `pow(2,3)`→`int(8)`, `pow(2,-1)`→`float(0.5)`, `round(2.5)`→`3.0`. |
| **D-17.5** | Gruppo array (`range`/`array_slice`/`array_reverse`/`array_unique`/`array_sum`) | `range($a,$b,$step=1)`: int/float/char auto-detect; float se uno qualunque di a/b/step è float; direzione da a vs b; `step==0`→`ValueError` ("cannot be 0"); range **crescente** con step<0→`ValueError` ("...for increasing ranges"); decrescente usa `|step|`; char-mode solo se a,b sono stringhe non-numeriche di 1 byte. `array_slice($a,$off,$len=null,$preserve=false)`: off/len negativi dall'estremità; reindex chiavi **int** se `!preserve` (string keys sempre preservate). `array_reverse($a,$preserve=false)`. `array_unique($a)`: SORT_STRING (confronto come stringa), tiene la **prima** occorrenza, preserva le chiavi. `array_sum`: somma (int o float; `[]`→`int(0)`). | Oracle-verificato: `range(5,1,-1)` valido (decrescente), `array_unique([1,"1",2])`→`[0=>1,2=>2]`, `array_sum([])`→`int(0)`. |
| **D-17.6** | Errori | Riusa `PhpError::{ValueError,DivisionByZeroError?,TypeError,ArgumentCountError}`. `intdiv` richiede `DivisionByZeroError`/`ArithmeticError`: se non esistono in `php-types`, aggiunta additiva (come step 10 fece per ValueError/ArgumentCountError). | Messaggi byte-esatti dall'oracle. |

**Sotto-suddivisione TDD step 17:** **17-1** case (5 fn) · **17-2** build (4 fn) · **17-3** trim (3 fn) · **17-4** math (6 fn) · **17-5** array (5 fn). Un commit TDD-isolato per gruppo, ognuno RED→GREEN, oracle-verificato. Test in `crates/php-builtins/tests/builtins.rs` (registry completa → `var_dump`/`echo`).

**Scope-out step 17:** Deprecation 8.5 di `chr`/`ord` (D-17.2); `str_word_count`/`wordwrap`/`number_format`/`array_map`/`array_filter` (quest'ultime due → closures, prossima feature); `range` con argomenti misti char↔numerici; mb_* (multibyte). 

**Step 17 IMPLEMENTATO (sessione 2026-06-14, TDD, zero D-NEW):** +20 test (244→264), oracle-verificato, clippy pulito. 24 builtin in 5 commit TDD-isolati, ABI `fn(&[Zval],&mut Ctx)` invariata, zero modifiche all'evaluator.
- **17-1 case** (`crates/php-builtins/src/string.rs`): `strtoupper`/`strtolower`/`ucfirst`/`lcfirst`/`ucwords`. ASCII-only (`make_ascii_*`), byte ≥0x80 intatti; helper `str_arg`. +2 test.
- **17-2 build** (string.rs): `str_repeat` (neg→`ValueError`), `str_pad` (left/right/both via `pad.iter().cycle()`, pad vuoto→`ValueError`, len≤strlen→invariato), `chr` (`rem_euclid(256)`), `ord` (primo byte, 0 se vuota). +5 test. NB le Deprecation 8.5 di chr/ord sono scope-out (valore corretto comunque).
- **17-3 trim** (string.rs): `trim`/`ltrim`/`rtrim` con driver `do_trim(left,right)` + `trim_mask` (256-bool) che espande i range `c1..c2` (php_charmask). Default `" \t\n\r\0\x0B"`. +2 test.
- **17-4 math** (`crates/php-builtins/src/math.rs`): `intdiv` (trunc verso zero; `/0`→`DivisionByZeroError`; `i64::MIN/-1`→`ArithmeticError`), `pow` (int^int≥0 con `checked_mul`→overflow promuove a float; altrimenti `powf`), `sqrt`/`floor`/`ceil` (sempre float), `round` (half-away-from-zero via `(x±0.5).floor/ceil`, precision anche negativa). Helper `as_double`/`double_arg`/`to_int_arg`. +5 test.
- **17-5 array** (`crates/php-builtins/src/array.rs`): `range` (int/float/char auto-detect; float se un operando è float; `emit_int_range` per int/char, count-based per float anti-drift; `step==0`/neg-su-crescente→`ValueError`), `array_slice` (offset/len negativi, `preserve_keys`), `array_reverse` (`preserve_keys`), `array_unique` (SORT_STRING via `to_zstr`, prima occorrenza, chiavi preservate), `array_sum` (accumulo `ops::add`, `[]`→`int(0)`). Helper `range_num`/`byte0`/`push_entry`. +6 test.
- **Registry** (`lib.rs`): +24 `add(...)`. **Costanti named non lowered** (es. `STR_PAD_LEFT`, `PHP_INT_MIN`): i test usano i valori literali (0/1/2) o le costruiscono (`-9223372036854775807 - 1`). Possibile step futuro: `ConstFetch` per costanti engine.

### Step 16 — `declare(strict_types=1)` (strict scalar typing) (design pass, sessione 2026-06-14)

> Complemento di step 14: chiude lo scope-out strict_types. Semantiche verificate sull'oracle PHP 8.5.7. In strict mode la coercizione scalare è **disattivata**: il tipo deve combaciare esatto, con l'**unica eccezione `int→float`** (widening). Risultati: `int←int` ok, `int←"5"` → TypeError (`int, string given`), `float←int` → ok (widen, niente errore), `int←5.0` → TypeError (`int, float given`), `float←float` ok, `string←int` → TypeError, `bool←int` → TypeError, `?int←null` ok, return `:int ← "5"` → TypeError. Messaggi TypeError **identici** al weak. `declare(strict_types=0)` → weak (default). Nota: oggi `declare(...)` non è gestito affatto (→ Unsupported), quindi questo step **sblocca anche il parsing di `declare`**.

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-16.1** | Parsing `declare` | Nuovo arm `Statement::Declare`: estrae l'item `strict_types` (valore intero literal) → `Program.strict: bool` (1→strict; 0/assente→weak). Il body (`DeclareBody::Statement`, per `strict_types` è un `;`→Nop) viene lowered; altri item (`ticks`/`encoding`) → no-op runtime. `declare(...){…}` colon-form → Unsupported (raro). | Prima `declare` cadeva su `_ => Unsupported`; ora gestito. Lo strict è un flag di programma (Tier 1 = single file). |
| **D-16.2** | Runtime flag | `Evaluator.strict` da `Program.strict`. `coerce_to_hint(value, hint, diags, strict)` ramifica sul flag. | Riusa interamente la pipeline di coercizione di step 14 (param/default/return). |
| **D-16.3** | Coercizione strict | `coerce_strict(value, hint)`: tipo esatto richiesto; UNICA conversione implicita `int→float` (`Long`→`Double`). `null` solo se nullable. Niente coercizione né deprecation. Mismatch → `None` → stesso `arg_type_error`/`return_type_error` del weak. | Verificato sull'oracle: widening int→float è l'unica eccezione in strict. |
| **D-16.4** | Scope-out | strict per-call-site cross-file (rilevante solo multi-file; PHP usa il `declare` del file *chiamante*), semantica reale di `ticks`/`encoding`, `declare` colon-form. | Tier 1 è single-file; gli altri declare non hanno effetto osservabile qui. |

**Sotto-suddivisione TDD step 16:** un solo sotto-step: parsing `declare` + flag strict + `coerce_strict`. Test: strict int←int ok, int←"5" fail, float←int widen ok, int←5.0 fail, string←int fail, ?int←null ok, return strict fail, + weak ancora coerce (regressione).

**Step 16 IMPLEMENTATO (sessione 2026-06-14, TDD, zero D-NEW) — commit `43ee473`:** +8 test (236→244), oracle-verificato, clippy pulito. Nuovo arm `Statement::Declare` (estrae `strict_types` → `Program.strict`; **fixa anche il fatto che `declare` prima era Unsupported**); `Evaluator.strict`. `coerce_to_hint` guadagna il parametro `strict`; `coerce_strict` richiede tipo esatto con unica eccezione `int→float` widening (niente coercizione/deprecation). Mismatch → stesso `TypeError` del weak. Applicato a param/default/return via la pipeline di step 14. `strict_types=0` → weak. Chiude lo scope-out strict_types di step 14. Scope-out residuo (D-16.4): strict per-call-site cross-file, `declare` colon-form, ticks/encoding reali.

### Step 15 — static variables (`static $x = init;`) (design pass, sessione 2026-06-14)

> L'utente ha scelto static vars dopo type-hint. Semantiche verificate sull'oracle PHP 8.5.7: `function f(){ static $n=0; $n++; echo $n; } f();f();f();` → `123` (init una volta, persiste cross-call); `static $a;` (no init) → `NULL` poi persiste; ricorsione `function f($d){ static $n=0; $n++; if($d>0) f($d-1); return $n; } f(3)` → `4` (cella **condivisa** tra i frame ricorsivi); isolamento per-funzione (`f`→1, `g`→101, `f`→2 = `11012`); init **non-costante** consentita (`static $x = strlen("ab")` → `2`, valutata alla prima call); `static $a, $b=5` multipli.

| ID | Tema | Decisione | Razionale |
|---|---|---|---|
| **D-15.1** | Modello persistenza | Store `statics: Vec<Option<Rc<RefCell<Zval>>>>` nell'Evaluator, indicizzato da uno `static_id` univoco globale assegnato a lowering; persiste per tutto il run. Ogni `static $x` distinto = un `id`. | La cella condivisa dà in un colpo: persistenza cross-call, sharing cross-ricorsione, init-una-volta. Riusa `Zval::Ref` di 11d (come `global`/`$GLOBALS`). |
| **D-15.2** | HIR | `StmtKind::StaticVar(Vec<StaticBinding>)`, `StaticBinding { slot: Slot, id: usize, init: Option<Expr> }`. `Program.static_count: usize` per dimensionare il Vec. | `id` flat-index → store O(1) senza HashMap. |
| **D-15.3** | Lowering | `Statement::Static` → per ogni `StaticItem`: slot locale (`slot_for`), `id = self.static_count` (poi `+= 1`), `init = Some(lower_expr(value))` per `Concrete` / `None` per `Abstract`. Lowerer field `static_count`; copiato in `Program.static_count` a fine lowering. | `id` univoco e stabile tra tutte le funzioni. |
| **D-15.4** | Eval | Per ogni binding: se `statics[id]` è `None` → valuta `init` (o `Null`) nel frame corrente, crea `Rc::new(RefCell)`, salva; poi `frame_mut!()[slot] = Zval::Ref(Rc::clone(cell))`. | Init solo alla prima esecuzione; re-alias su ogni call alla stessa cella persistente. |
| **D-15.5** | Init non-costante | `init` è un `Expr` generico, valutato alla prima esecuzione del `static` (PHP 8.3+). | Oracle: `strlen("ab")` → 2. |
| **D-15.6** | Scope-out | `static::` (late static binding, OOP), static closures (`static function/fn`), proprietà statiche di classe. | Richiedono OOP. |

**Sotto-suddivisione TDD step 15:** un solo sotto-step (contenuto): `StmtKind::StaticVar` + lowering + store persistente + eval. Test: accumulate (`123`), ricorsione condivisa (`4`), isolamento per-funzione, no-init→null+persist, init non-costante, item multipli.

**Step 15 IMPLEMENTATO (sessione 2026-06-14, TDD, zero D-NEW) — commit `4a48dc7`:** +6 test (230→236), oracle-verificato, clippy pulito. `StmtKind::StaticVar(Vec<StaticBinding{slot,id,init}>)` + `Program.static_count`. Lowering (`Statement::Static` → per `StaticItem::{Abstract,Concrete}`): slot locale + `id` univoco (`static_count++`) + init. Evaluator: `statics: Vec<Option<Rc<RefCell<Zval>>>>` dimensionato a `static_count`, persiste per tutto il run. Eval: prima esecuzione → valuta init (o Null), crea cella; ogni esecuzione → re-alias `frame_mut!()[slot] = Zval::Ref(clone(cell))`. Persistenza cross-call + sharing cross-ricorsione + init-una-volta in un solo meccanismo (riusa `Zval::Ref` di 11d). Scope-out (D-15.6): `static::`/static closures/proprietà statiche (OOP).

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

---

# Step 18 — Closures / callables (design pass)

Prima feature di "funzioni come valori": funzioni anonime (`function() use(...) {}`),
arrow function (`fn() => expr`), chiamata dinamica (`$f()`, `$a['k']()`, IIFE), callable
stringa (`$f = 'strlen'; $f(...)`), e i builtin higher-order **array_map / array_filter /
usort** (+ `is_callable`, `call_user_func[_array]`). Sblocca lo scope-out di step 10/17.
Inclusi tutti e 4 gli extra opzionali (var_dump esatto, first-class callable `strlen(...)`,
`call_user_func_array`, modi di `array_filter` → richiedono **ConstFetch**).

Semantica oracle-verificata 8.5.7 (`php -n -r`): `use($x)` cattura **by value alla
definizione**; `use(&$x)` by-ref; `fn()=>` auto-cattura **by value** (transitivo per arrow
annidate); `gettype` → `"object"`; var_dump/print_r → `Closure Object` con `name`/`file`/`line`;
dispatch `$f()`: Closure→invoca, stringa→user-fn poi builtin altrimenti `Call to undefined
function nope()`, altro→`Value of type int is not callable`; array_map preserva le chiavi
(single) e reindicizza (multi-array); array_filter senza callback = truthy, con callback
preserva le chiavi; usort in-place by-ref, reindicizza 0..n, ritorna `true`; troppi pochi
argomenti a una closure → `ArgumentCountError` fatale.

## Decisioni (D-18.x)

| ID | Costrutto | Scelta Rust | Razionale |
|---|---|---|---|
| D-18.1 | Valore closure | **`Zval::Closure(Rc<Closure>)`** variante dedicata (NO oggetto OOP) | Niente OOP ancora; anti-priming GoF (enum tipizzato > gerarchia). `gettype`→`"object"`, `error_type_name`→`"Closure"`. |
| D-18.2 | Storage funzioni anonime | tabella piatta **`Program.closures: Vec<FnDecl>`** + `ExprKind::Closure{fn_idx, captures}` | Riusa l'intera macchina `FnDecl`/`run_user_fn_body`. Annidamento → tabella piatta, `fn_idx` flat. Il valore `Closure` porta `captures: Vec<(u32 dst_slot, Zval)>` (auto-contenuto: nessun parallel-array col FnDecl). |
| D-18.3 | Cattura `use($a, &$b)` | by-val: `deref_clone` dello slot padre alla **creazione** (undef→Warning+Null); by-ref: condivide la cella (`Zval::Ref`) | Snapshot at-definition è la semantica PHP. Lo slot padre è risolto nello scope **chiamante** prima di installare lo scope della closure. |
| D-18.4 | Arrow `fn()=>expr` | auto-cattura **by value** dei free var presenti nello scope chiusura tramite **AST walk** ∩ slot già esistenti nello scope padre; body = `return <expr>` | Distingue var del padre (catturate) da nuovi local (write interni). Euristica "lo slot esiste già nel padre" ≈ semantica PHP at-definition; var usata-ma-non-ancora-definita → Null+Warning (raro, documentato). |
| D-18.5 | Chiamata dinamica | nuovo **`ExprKind::CallDynamic{callee, args}`**; metodo `call_value(&Zval, argv)` dispatcha Closure / stringa / errore | Copre `$f()`, `$a['k']()`, IIFE `(function(){})()`. Argomenti **by value** (by-ref ai dynamic call = scope-out). |
| D-18.6 | Builtin higher-order | **intercettati nell'evaluator** (non nella registry): array_map, array_filter, usort, is_callable, call_user_func[_array] | L'ABI builtin `fn(&[Zval],&mut Ctx)` non ha accesso all'evaluator per invocare la callback; infilare `&mut Evaluator` in `Ctx` litiga col borrow checker. Idiomatico: metodi dell'evaluator. `usort` prende arg0 by-ref (come `sort`). Bonus: funzionano anche con registry vuota → testabili in `eval.rs`. |
| D-18.7 | `ConstFetch` costanti named | arm di lowering `Expression::ConstantAccess` → sostituzione literal da **tabella costanti engine** (ARRAY_FILTER_USE_KEY=2/USE_BOTH=1, STR_PAD_LEFT/RIGHT/BOTH, PHP_INT_MAX/MIN/SIZE, PHP_FLOAT_*, PHP_EOL, SORT_*, COUNT_*, M_PI, true/false/null) | Sblocca i modi di `array_filter` e retro-sblocca l'ergonomia di TUTTI i builtin con flag (step 17). Backlog #3. Costante sconosciuta → resta Unsupported (no const utente). |
| D-18.8 | Type hint `callable` | accettato, **non enforced** (lower→`None`) | Coerente con D-14.1 (hint non scalari → nessuna coercizione). Funziona già "gratis". |
| D-18.9 | var_dump/print_r closure | formato 8.5 esatto `object(Closure)#N (3){name,file,line}` / `Closure Object(...)`; contatore object-id | Extra richiesto. `name` = `{closure:<file>:<line>}`. |
| D-18.10 | First-class callable `strlen(...)` | produce una `Closure` che incapsula un **nome** (`ClosureKind::Named`) | Extra richiesto (sugar 8.1). var_dump mostra `object(Closure)`. |

## Gruppi TDD

- **18-1**: infra `Zval::Closure` + `function(){} use(...)` (by-val/by-ref) + `$f()` (`CallDynamic`/`call_value`/`call_closure`) + IIFE + `gettype`="object". Arm `Zval::Closure` nei funnel `ops`/`convert`/`zval` (non esaustivi).
- **18-2**: arrow function + free-var walk + cattura by-value (incl. annidate).
- **18-3**: callable stringa + `is_callable` + `call_user_func` + `call_user_func_array` + conferma hint `callable`.
- **18-4**: `ConstFetch` + tabella costanti engine.
- **18-5**: `array_map` (single/multi/chiavi) + `array_filter` (con/senza callback + modi via ConstFetch) + `usort`.
- **18-6**: first-class callable `strlen(...)`.
- **18-7**: var_dump/print_r esatto per closure + docs/metrics.

## Scope-out (debito esplicito)

`Closure::bind`/`bindTo`/`call`/`fromCallable` e static closures (richiedono `$this`/OOP);
argomenti by-ref ai dynamic call (`$f(&$x)`); string-call di un builtin by-ref (`$f='sort'; $f($a)`);
spread `...$args` negli argomenti; callable array `[$obj,'m']`/`['Cls','m']` (OOP);
cattura by-value di var del padre usata-ma-non-ancora-definita testualmente (→ Null+Warning).

## STATO: IMPLEMENTATO (7 gruppi, +59 test 264→323, clippy pulito, zero D-NEW)

Tutti e 7 i gruppi TDD shippati come da design (design `d9c6fed`; 18-1 `9a556ff`,
18-2 `a899bd1`, 18-3 `f8a7a26`, 18-4 `c30263e`, 18-5 `15c2197`, 18-6 `cef7e5f`,
18-7 `732e6b7`). Nessuna D-decisione riaperta. Note di implementazione in
`diary/metrics.md` § "Step 18". Due divergenze note documentate nello scope-out di
18-7 (object-id non riciclati; first-class callable di builtin senza `parameter[]`).
D-18.8 confermata "gratis": il hint `callable` lowering→`None` passa il valore senza
enforcement. Object→string di una closure: PHP fa un fatal `Error`, il funnel
infallibile `to_zstr` emette invece un Warning + placeholder (edge non testato,
rivedere con OOP). **Terza divergenza var_dump (corpus):** PHP aggiunge `["static"]`
con le variabili catturate per le closure con `use`/arrow — omessa (richiede
recursion-guard per `use(&$f)`); dettaglio in `diary/metrics.md` § Step 18.

---

# Step 19 — OOP / classi (design pass)

Il blocco più grande di `unsupported` nel corpus (~5028 casi). Scope **Full Tier-1**
deciso col Decider (2026-06-14): classi, proprietà (default + visibility), metodi,
`__construct`, `$this`, `new`, semantica **handle**, read/write proprietà,
**ereditarietà** (`extends`/`parent::`/`self::`), **membri static**, **costanti di
classe**, **`instanceof`/interfaces**, **abstract/final**, **`__toString`**,
**`Closure::bind`/`bindTo`/`fromCallable`** + static closures, var_dump/print_r esatto
con **recursion-guard** (retro-sblocca anche `["static"]` delle closure dello step 18).
**Eccezioni (`try/catch/finally`/`throw` + Exception/Error) = step 20 separato**
(control-flow a sé, riusa le classi di qui).

Semantica oracle-verificata 8.5.7 (`php -n -r`): assegnare un oggetto copia
l'**handle** (mutazioni condivise, contrasta gli array COW); `var_dump` →
`object(C)#N (k) { ["p"]=>…, ["p":protected]=>…, ["p":"C":private]=>… }`; `gettype`
→ `"object"`; `$p instanceof C` → bool; proprietà non dichiarate sono dinamiche
(deprecation 8.2, ma supportate); `new C` senza `()` legale; `$this` fuori da metodo
→ Error; accesso a proprietà private/protected dall'esterno → Error.

## Decisioni (D-19.x)

| ID | Costrutto | Scelta Rust | Razionale |
|---|---|---|---|
| D-19.1 | Valore oggetto | **`Zval::Object(Rc<RefCell<Object>>)`** | Semantica handle: clone condivide l'`Rc`, mutazione via `RefCell` visibile a tutti. NON `Rc::make_mut` (≠ array COW). `gettype`→`"object"`, `error_type_name`→nome classe. |
| D-19.2 | Struct oggetto | `Object { class: ClassId, props: Props, id: u32 }` dove `Props` è una **mappa ordinata `Box<[u8]>→Zval`** (riusa il pattern `PhpArray`: Vec di entry + index, ordine di inserzione per var_dump) | Le proprietà PHP conservano l'ordine di dichiarazione/assegnazione; var_dump lo riflette. Oggetti = poche prop → struttura leggera. |
| D-19.3 | Class table | **`Program.classes: Vec<ClassDecl>`** hoisted al lowering (come `functions`/`closures`) + `name→ClassId` a runtime (case-insensitive) | Le classi sono visibili prima della decl (hoisting PHP, salvo `extends` di classe condizionale → scope-out). `ClassId = usize`. |
| D-19.4 | ClassDecl | `{ name, parent: Option<ClassId>, interfaces: Vec<ClassId>, props: Vec<PropDecl>, static_props: Vec<…>, methods: Vec<MethodDecl>, consts: Vec<(name,Expr)>, is_abstract, is_interface }` | Risoluzione `extends`/`implements` per nome→id al lowering (forward-ref ok: 2-pass). |
| D-19.5 | Metodo | `MethodDecl { fdecl: FnDecl, name, vis: Visibility, is_static, is_abstract, is_final, defining_class: ClassId }` con **slot riservato per `$this`** nel frame del metodo | Riusa interamente `FnDecl`/`run_user_fn_body`. `$this` è una var normale: il lowerer pre-registra lo slot `this` nello scope del metodo e lo memorizza; il dispatch lo lega all'handle. |
| D-19.6 | `new C(args)` | nuovo **`ExprKind::New { class: ClassRef, args: Vec<Expr> }`**; crea `Object` con prop default valutate per-istanza, poi chiama `__construct` se esiste | Default = `Expr` valutati al `new` (literali / `self::CONST`). `ClassRef` = nome literal (Tier-1) o `new $var`/`self`/`static` (D-19.16). |
| D-19.7 | Method call | nuovo **`ExprKind::MethodCall { object, method, args }`** (e `NullSafe`); risolve il metodo risalendo la catena `parent`, installa frame, lega `$this`, esegue | `$obj->m()`. Dispatch = `call_method(obj, class_start, name, argv)`. Metodo assente → `__call` (scope-out) o Error. |
| D-19.8 | Property read | **`ExprKind::PropGet { object, name }`** (+ `NullSafe`); legge dalla mappa prop dell'oggetto (no risalita: le prop ereditate sono già materializzate nell'istanza) | `$obj->p`, `$this->p`. Prop assente → Warning "Undefined property" + Null. Nome dinamico `$obj->$n` → scope-out parziale (literal-first). |
| D-19.9 | Property write | estendere **`PlaceStep` con `Prop(Box<[u8]>)`**; `place_cell`/`write_into`/navigazione entrano nel `RefCell` dell'oggetto (condiviso, **niente write-back COW**) | `$obj->p = v`, `$this->p = v`, compound/`++`/`??=`, `$obj->arr[] = v`, nested `$a->b->c`. Punto più delicato → gruppo 19-2 isolato. Prop inesistente in write → creata (dinamica). |
| D-19.10 | Ereditarietà | `extends` unico (PHP single-inheritance); prop ereditate copiate nella decl figlia al lowering (flatten), metodi risolti a runtime risalendo `parent` | Flatten prop = istanza self-contenuta; metodi via catena per supportare override + `parent::`. |
| D-19.11 | `parent::` / `self::` | `self` = classe **definente** il metodo corrente; `parent` = il suo `parent`; risolti via contesto runtime (`cur_class`/`cur_static_class`) | `parent::__construct()`, `self::method()`, `self::CONST`. |
| D-19.12 | `static::` (LSB) | late static binding minimale: `cur_static_class` = classe dell'oggetto/chiamata reale, propagata nelle call | `new static()`, `static::method()`. |
| D-19.13 | Visibility | enum `Visibility {Public, Protected, Private}`; **enforcement** all'accesso esterno (Error PHP-esatto); usata da var_dump (`:protected`, `:"C":private`) | Default `public`. Accesso da metodo della stessa classe (o discendente per protected) consentito. |
| D-19.14 | Static members | `static_props: Vec<(name, vis, cell: Rc<RefCell<Zval>>)>` per-classe nel runtime (persistono per il run, init una volta); `Class::$p`, `static::$p`, `self::$p` | Riusa il pattern `statics` dello step 15 (cella persistente). |
| D-19.15 | Class constants | `Class::CONST`, `self::CONST`, `parent::CONST`; tabella `consts` per-classe, valutate lazy/al primo accesso, risalita per ereditarietà | Default di prop possono riferirle (D-19.6). |
| D-19.16 | `instanceof` | operatore: `$x instanceof C` true se la classe di `$x` è `C`, un suo antenato, o un'interfaccia implementata (transitiva) | Mago: `instanceof` come binary/op dedicato → nuovo `ExprKind` o `BinOp`. |
| D-19.17 | interfaces / abstract / final | `interface` = ClassDecl con `is_interface` (solo costanti + metodi astratti); `implements` riempie `interfaces`; `abstract class`/`abstract function` non istanziabili/da implementare; `final` non overridabile/estendibile (enforcement) | Le interfacce partecipano a `instanceof`. |
| D-19.18 | `__toString` | object→string (echo, `.`, `(string)`, sprintf `%s`) chiama `__toString` se definito, altrimenti **Error** "Object of class C could not be converted to string" | Sostituisce il placeholder/Warning del funnel `to_zstr` (debito step 18 chiuso). Richiede che `to_zstr` possa rientrare nell'evaluator → gestito a livello evaluator, non in `convert.rs`. |
| D-19.19 | `Closure::bind`/`bindTo`/`fromCallable` + static closures | `Closure` acquisisce `bound_this: Option<Zval::Object>` + `scope: Option<ClassId>`; `$this` dentro la closure legato; `static function(){}` = nessun bind | Chiude lo scope-out dello step 18. `fromCallable` = wrap di callable in Closure. |
| D-19.20 | var_dump/print_r + recursion-guard | formato 8.5 esatto con annotazioni visibility + **guardia di ricorsione generale** (`*RECURSION*`) su oggetti/array già in corso di dump | Retro-sblocca `["static"]` delle closure catturanti (step 18). Set di puntatori "in-progress" durante il dump. |

## Gruppi TDD

- **19-1** Infra: `Zval::Object(Rc<RefCell<Object>>)` + `Object`/`Props` + `Program.classes`/`ClassDecl`/`MethodDecl` + lowering `class` (prop+metodi, hoisted, 2-pass) + `new C(args)` (`ExprKind::New`) + `__construct` + `$this` + `$obj->m()` (`ExprKind::MethodCall`) + prop read (`ExprKind::PropGet`) + `gettype`/`error_type_name`. Arm `Zval::Object` non-esaustivi in `ops`/`convert`/`zval`/var_dump.
- **19-2** Write-path proprietà: `PlaceStep::Prop` + `$obj->p = v`/`$this->p = v` + compound/`++`/`??=` + `$obj->arr[] = v` + nested `$a->b->c` + `isset`/`empty`/`unset` su proprietà.
- **19-3** Ereditarietà: `extends`, risoluzione metodi su catena, `parent::m()`, prop ereditate (flatten), `self::`, enforcement visibility public/protected/private.
- **19-4** Static + costanti: `static $prop`/`Class::$p`/`static::$p`/`self::$p`, `static::` LSB, `Class::m()` (static call), `const`, `Class::CONST`/`self::CONST`/`parent::CONST`.
- **19-5** `instanceof` + interfaces + abstract/final: `interface`/`implements`, `instanceof` transitivo, abstract non istanziabile, final non overridabile.
- **19-6** Magic `__toString` (object→string nei vari contesti) + `Closure::bind`/`bindTo`/`fromCallable` + static closures.
- **19-7** var_dump/print_r esatto per oggetti + recursion-guard generale (+ `["static"]` closure) + docs/metrics + validazione corpus.

## Scope-out (debito esplicito → futuri step)

`try/catch/finally`/`throw` + gerarchia Exception/Error built-in (**step 20**);
generators/`yield`, fibers; **traits** (`use` dentro classe); **enum** (puro/backed);
**anonymous class** (`new class {}`); namespace + `::class`; magic dinamici
`__get`/`__set`/`__isset`/`__unset`/`__call`/`__callStatic`/`__invoke`; `readonly`
enforcement; property hooks 8.4; clone/`__clone`; nomi membro dinamici complessi
(`$obj->{$expr}`, `$obj->$$x`); `Stringable`/`ArrayAccess`/`Iterator`/`Countable`
(interfacce magiche); `::class` su istanza; `get_class`/`get_object_vars`/altri
builtin di reflection (valutare a parte); covarianza/contravarianza tipi; GC ciclico
(handle + prop creano cicli → leak accettato come gli element-ref, D-R15).

## STATO: IMPLEMENTATO (7 gruppi, +54 test 323→377, clippy pulito, zero D-NEW)

Tutti e 7 i gruppi shippati come da design (design `43379f1`; 19-1 `92b8360`,
19-2 `ba0c2e4`, 19-3 `1b628cd`, 19-4 `bae3c7a`, 19-5 `1e07a13`, 19-6 `5d59ba3`,
19-7 = questo commit docs). Nessuna D-decisione riaperta.

- **19-1**: `Zval::Object(Rc<RefCell<Object>>)` (handle semantics) + `Object`/`Props`
  (php-types) + `Program.classes`/`ClassDecl`/`MethodDecl` + lowering classe (2-pass
  hoist) + `new` + `__construct` + `$this`=`ExprKind::This` + `$obj->m()` + prop read
  + write semplice (`PlaceStep::Prop`).
- **19-2**: write-path completo proprietà (compound, `++`=`IncDecPlace` che copre anche
  `$a[k]++`, `??=`, `$o->arr[]`, nested, isset/empty/unset).
- **19-3**: ereditarietà (`extends`, risoluzione metodi su catena, override, prop
  flatten parent-first), `parent::`/`self::` (=`ExprKind::StaticCall`+`ClassRef`,
  `cur_class`), enforcement visibility public/protected/private (read+write+metodi).
- **19-4**: costanti di classe (`Class::C`/`self::C`/`::class`), static props
  (cella persistente per-declaring-class, `Class::$p` read/write/compound/incdec),
  static method call, **late static binding** (`cur_static_class`, `new static`,
  `static::m()`, forwarding self/parent/static).
- **19-5**: `instanceof` (=`ExprKind::InstanceOf`, transitivo su catena+interfacce),
  `interface`/`implements`, abstract non istanziabile (fatal runtime), metodi
  abstract = solo firma (skip al lowering).
- **19-6**: `__toString` (helper `stringify` in echo/concat/`(string)`; chiude il
  debito step-18 di `to_zstr`), closure bind `$this` (`Closure.bound_this`,
  cattura alla creazione per closure/arrow non-static; `static fn` no-bind),
  `bindTo`/`call`/`Closure::bind`/`fromCallable`.
- **19-7**: var_dump/print_r esatti per oggetti con annotazioni visibility
  (`["p":protected]`, `["p":"C":private]`; print_r `[p:C:private]`) via
  `ObjectInfo`/`PropVis` portati nel valore (shape per-classe cache); **recursion
  guard generale** (`*RECURSION*`) su oggetti e array (fixa anche un loop latente
  su array auto-referenziali).

**Validazione corpus:** `/tmp/php-src/tests/classes` ora **57 pass / 45 fail / 181
skip** (102 runnable; prima dello step 19 erano ~tutti skip "unsupported").

**Scope-out (debito esplicito → futuri step):** `final` enforcement (extend/override
= fatal *compile-time* con formato diverso da "Uncaught"); `closure instanceof
Closure` (le closure non sono object table-backed); scope binding delle closure per
accesso a private (solo `$this` è legato); sprintf `%s` `__toString` (il builtin non
può rientrare nell'evaluator); closure `["static"]` in var_dump (il recursion-guard
ora c'è, ma le var catturate non sono ancora esposte); `__get`/`__set`/`__call` e gli
altri magic dinamici; traits; enum; anonymous class; nomi membro dinamici
(`$o->$n`); dynamic-prop deprecation 8.2; covarianza tipi. Eccezioni (`try/catch`/
`throw` + Exception/Error) = **step 20**.

---

## Step 20 — Eccezioni (`try`/`catch`/`finally` + `throw` + gerarchia Throwable) — DESIGN PASS

**Obiettivo:** control-flow di unwinding che riusa interamente le classi dello step
19. `throw <expr>`, `try { } catch (T $e) { } finally { }`, gerarchia built-in
`Throwable`/`Exception`/`Error` + sottoclassi SPL, accessor (`getMessage`/`getCode`/
`getPrevious`/`getLine`/`getFile`/`getTrace`/`getTraceAsString`), rendering "Uncaught".

### Oracle recon (PHP 8.5.7, `php -n -r`) — fatti verificati
- `getCode()` default `int(0)`; `getMessage()` default `string(0) ""`.
- `__construct(message="", code=0, previous=null)`; `getPrevious()` ritorna la prev.
- Ordine `finally`: `try`→`catch`→`finally`→codice dopo (es. `tcfafter`).
- `finally` con `return` **sovrascrive** il `return`/`throw` del try (`fin` vince).
- `finally` gira anche se il try ritorna (`return "t"` in fn → stampa `f` poi `t`).
- Gerarchia: `Error` NON è preso da `catch (Exception)`; `Throwable` prende tutto.
  `RuntimeException`/`InvalidArgumentException` → `instanceof Exception`+`Throwable`;
  `InvalidArgumentException instanceof LogicException` = true.
- Multi-catch `catch (A | B $e)`; catch senza variabile `catch (Exception)`.
- `throw` come **espressione** (`$x ?: throw new …`).
- Eccezione non catturata (exit 255):
  `\nFatal error: Uncaught <Class>: <msg> in <file>:<line>\nStack trace:\n#0 {main}\n  thrown in <file> on line <line>\n`
  La `<line>` è la linea di **creazione** (`new`), recuperabile via `getLine()`.
- Errori engine catturabili: `TypeError`/`DivisionByZeroError`/… sono Throwable.
- `var_dump` di un'eccezione espone 7 prop interne (message/string/code/file/line/
  trace/previous) con annotazioni `:protected` e `:"Exception":private` → **fuori
  scope step 20** (richiede le pseudo-prop interne `string`/`trace`; debito).

### Architettura (riusa step 19)
- **Classi built-in via PRELUDE PHP.** Un sorgente PHP statico (`PRELUDE_SRC` in
  `lower.rs`) definisce `interface Throwable`, `Exception`/`Error` (props
  `message`/`code`/`file`/`line` protected, `previous` private; `__construct` +
  accessor con corpi HIR reali) e le sottoclassi SPL (`LogicException`/
  `RuntimeException` + figlie, `TypeError`/`ValueError`/`ArithmeticError`/
  `DivisionByZeroError`/`ArgumentCountError`/`ErrorException`/`UnhandledMatchError`).
  Lowerato con un `Lowerer` usa-e-getta; le `ClassDecl` **owned** risultanti sono
  iniettate in testa a `Program.classes` (ids 0..N) PRIMA dell'hoisting delle classi
  utente. Così `extends Exception`, `instanceof`, `resolve_class_ref`, `collect_props`
  (init prop), `class_shape` (var_dump) e `parent::__construct` funzionano GRATIS con
  la macchina dello step 19 — zero dispatch speciale per i metodi.
- **Unwinding via `PhpError::Thrown(Zval)`.** Nuova variante che trasporta l'oggetto
  lanciato; si propaga da sola attraverso ogni `?` in `eval` (espressioni) ed
  `exec_stmt` (statement) — copre throw in profondità (throw-expr, throw dentro
  metodo/funzione). `PhpError` perde i derive `PartialEq, Eq` (Zval non è Eq: ha f64);
  nessuno li usava. `class_name()`/`message()` restano totali (sentinel per `Thrown`,
  mai usati su quel ramo: il rendering lo gestisce a parte).
- **HIR.** `StmtKind::Try { body, catches: Vec<CatchClause>, finally: Vec<Stmt> }`
  (`finally` vuoto = assente); `struct CatchClause { types: Vec<Box<[u8]>>, var:
  Option<Slot>, body }`; `ExprKind::Throw(Box<Expr>)`.
- **Lowering.** `Statement::Try` → estrae `block`/`catch_clauses`/`finally_clause`;
  ogni catch: `hint` (Identifier o `Union` → lista nomi), `variable` → slot opzionale.
  `Expression::Throw` → `ExprKind::Throw`.
- **Eval.** `exec_stmt` Try: esegue body; su `Err(Thrown)` prova i catch per
  `is_instance_of(obj_class, type_id)` (riusa step 19-5); lega `$e`, esegue il body del
  catch. `finally` gira SEMPRE (anche su `Err`/`Flow` non-Normal): se finally esce
  Normal propaga l'esito di try/catch, altrimenti il control-flow di finally vince.
  `ExprKind::Throw` → `Err(PhpError::Thrown(obj))`. `eval_new`: per gli oggetti
  Throwable setta `line` (linea del `new`) e `file` PRIMA del costruttore.
- **Rendering.** `render_fatal` ramo `Thrown`: classe da `obj.class_name`, msg/line da
  prop `message`/`line`.
- **20-3:** errori engine (`PhpError::TypeError(..)` ecc.) resi catturabili —
  al catch-site un errore non-`Thrown` è confrontato per `class_name` contro la
  gerarchia e, se preso, sintetizzato in un oggetto della classe corrispondente.
  Uncaught invariato (i 377 test restano verdi).

### Gruppi TDD
- **20-1:** prelude + throw stmt/expr + try/catch (single/multi/no-var) + accessor +
  uncaught rendering.
- **20-2:** finally (normal/return/throw/break + finally-overrides).
- **20-3:** engine errors catturabili + sottoclassi utente/`parent::__construct` +
  catene `getPrevious` + validazione corpus `Zend/tests/exceptions`.

### Scope-out (debito → futuri step)
`var_dump`/`print_r` esatto delle eccezioni (pseudo-prop interne `string`/`trace`);
stack-trace reale con frame (resta `#0 {main}`); `getTrace()` = `[]`;
`set_exception_handler`/`set_error_handler`; `finally` con eccezione che ne maschera
un'altra in modo annidato oltre il caso base; coercizione scalare dei param di
`__construct` (lasciati untyped); `DesiredException::__construct` con typed `?Throwable`.

### Step 20 IMPLEMENTATO (20-1 `7d575fd`, 20-2 `f12a2e5`, 20-3 `7d5e350`)
TDD, **+26 test (377→403)**, clippy pulito, zero D-NEW. 3 gruppi:
- **20-1** prelude (`PRELUDE_SRC` in lower.rs, lowerato da un `Lowerer` usa-e-getta,
  classi owned iniettate in testa a `Program.classes`; fix `hoist_classes` per
  offsettare gli id su `self.classes.len()`); `PhpError::Thrown(Zval)` (persi i derive
  `PartialEq/Eq`); `StmtKind::Try`+`CatchClause`, `ExprKind::Throw`; `handle_thrown`
  (match per `is_instance_of`, single/multi/no-var); `eval_new` setta `line`/`file`
  sui Throwable; `render_fatal` ramo `Thrown`. accessor (getMessage/Code/Previous/
  Line/File) = corpi HIR reali del prelude.
- **20-2** finally: la logica era già nel Try arm di 20-1 (finally gira sempre; il suo
  control-flow sovrascrive l'esito di try/catch). 8 test oracle (order, return,
  override, rethrow, break/continue in loop, return-inghiotte-throw).
- **20-3** errori engine catturabili (`handle_thrown` risolve `class_name()`→classe
  prelude e `synthesize_throwable` materializza l'oggetto solo se un catch lo lega);
  sottoclassi utente + `parent::__construct` + catene `getPrevious` + throw-espressione
  (tutto gratis da prelude+step19).

**Validazione corpus** `Zend/tests/{exceptions,throw,try}`: **36 pass / 52 fail / 91
skip** (179; prima ~tutti skip:unsupported). Fail residui = scope-out: stack-trace con
frame reali (`#0 foo(...)`; noi `#0 {main}`), foreach su oggetti/Generator, `stdClass`
non nel prelude, magic methods.
