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
| D-G14 | riferimenti `&$x` | differiti (variant `Ref(Rc<RefCell<Zval>>)` aggiunto quando serve); leak-on-cycle accettato e documentato | ROI: pochi .phpt Tier 1 li richiedono | confermato |
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

## Bug-class eliminate gratis dal target

1. Buffer overflow (bounds check), 2. use-after-free (ownership), 3. leak su error path
(RAII/Drop), 4. corruzioni della HashTable C (collezioni sicure), 5. race su stato
globale (Send/Sync), 6. errori di refcount manuale (Rc).

## Cose esplicitamente NON portate (Tier 1)

OOP (classi, interfacce, traits, enum, closures, generators, fibers), exceptions
user-level (`try/catch`), riferimenti `&`, include/require, namespace, eval,
superglobals web ($_GET…), resources, INI system (default hardcoded: display_errors=1,
precision=14, serialize_precision=-1), opcache/JIT, ZTS.

## Punti di review per l'umano

1. **D-G12** (float formatting): se il differential mostra divergenze sistematiche sulla
   modalità precision=14, si porta `zend_gcvt` fedelmente (~150 LOC). Accettato?
2. **D-G8**: se mago non copre un costrutto 8.5 usato dai test, fallback = skip-list,
   non patch a mago. Accettato?
3. Ordine warning vs output bufferizzato: assumiamo stdout unbuffered interleaved
   (CLI default). Se i .phpt rivelano differenze, si adegua.
