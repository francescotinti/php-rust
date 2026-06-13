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

### Scope-out espliciti (sotto-step futuri)

| Fuori scope (per ora) | Perché | Cosa richiederebbe |
|---|---|---|
| Reference **dentro** array (`$arr[0] = &$x`) | Una cella-reference dovrebbe vivere come elemento di `PhpArray`, quindi servirebbe `Zval::Ref` (variant), con blast radius su ops/convert/dump/builtin. | Promuovere `Ref` a variant di `Zval` + deref pervasivo + annotazione `&` in var_dump/print_r. |
| `foreach ($a as &$v)` | Idem: gli elementi dell'array iterato devono diventare reference (`lower.rs:720`). | Dipende dal punto sopra. |
| Return by-reference (`function &f()`) | Raro nel corpus Tier 1 (`lower.rs:329`). | Modello di reference già pronto + propagazione nel return. |
| GC ciclico | Con reference **solo a livello di variabile** non si creano cicli (`$a=&$b;$b=&$a;` condividono una cella, nessun ciclo). Il rischio leak-on-cycle arriva **solo** con le reference dentro container (sopra). Coerente con D-G6. | `Rc` → ciclo possibile solo via element-ref; allora servirà GC o weak. |

### Suddivisione in sotto-step (proposta per la sessione dedicata)

- **11a** — `Slot` enum + read/write-through + `$b = &$a` + `unset` (D-R1..R5, D-R8, D-R9). TDD da `$b=&$a; $a=5; echo $b;` → `5`.
- **11b** — parametri by-ref `f(&$x)` (D-R6). TDD da `function inc(&$x){$x++;} $n=1; inc($n); echo $n;` → `2`.
- **11c** — builtin by-ref: `array_push`, poi `sort`/`array_pop`/`array_shift` (D-R7).
- **11d** *(opzionale, separato)* — element-ref / foreach-by-ref (richiede `Zval::Ref`).

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
