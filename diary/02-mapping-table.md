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

### Scope-out espliciti (oltre Tier 1)

| Fuori scope | Perché | Cosa richiederebbe |
|---|---|---|
| Return by-reference (`function &f()`) | Raro nel corpus Tier 1 (`lower.rs:329`). | Modello di reference già pronto (D-R10) + propagazione nel return path. |
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
