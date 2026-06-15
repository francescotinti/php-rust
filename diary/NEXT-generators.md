# NEXT — Generators (`yield`) — design pass / handoff

> Generato con assistenza AI (Claude Opus 4.8). Design pass per una **sessione
> dedicata**. Scelto data-driven (`NEXT-backlog-scan.md`: `yield` = 284 file, la
> lacuna di linguaggio più grande). È lo step **più difficile** finora: richiede
> **esecuzione sospendibile** in un interprete tree-walking `&mut self`. Leggere
> tutto prima di iniziare. Stato di partenza: steps 0–38 done, 686 test verdi.

## 1. Obiettivo e scope

Implementare i generatori PHP: una funzione che contiene `yield` (a qualunque
profondità) è un **generatore**; chiamarla NON esegue il corpo ma ritorna un
oggetto `Generator`. Il corpo gira pigramente, sospendendosi a ogni `yield`.

**In-scope (target sessione):** `yield`, `yield $k => $v`, `yield` nudo,
`yield from` (array + generatore + Traversable), `return` in generatore +
`getReturn()`, `send()`, metodi `Iterator` (`current`/`key`/`next`/`valid`/
`rewind`), `foreach` su generatore, `instanceof Generator/Iterator/Traversable`.

**Scope-out probabili (Decider):** `Generator::throw()`, eccezioni che attraversano
`yield`/`finally` dentro il generatore (interazione unwinding), generatori by-ref
(`function &gen()` + `yield`), `yield` in contesti esotici (espressione annidata
in operatori complessi), migliaia di generatori vivi (memoria stack).

## 2. Recon oracle (PHP 8.5.7, verificato — usare file temp, non `-r`)

| Caso | Codice | Output |
|---|---|---|
| basic + key auto | `function g(){yield 1;yield 2;yield 3;} foreach(g() as $k=>$v)echo "$k:$v ";` | `0:1 1:2 2:3` |
| key esplicita | `yield "a"=>1; yield "b"=>2;` | `a:1 b:2` |
| `yield from` | `function g(){yield 1; yield from [10,20]; yield 2;}` | `0:1 0:10 1:20 1:2` |
| return+getReturn | `function g(){yield 1; return 99;} $x=g(); foreach($x as $v)echo $v; echo $x->getReturn();` | `1` poi `99` |
| `send()` | `function g(){$a=yield 1; echo "got=$a\n"; $b=yield 2; ...}` `$g->current(); $g->send("X"); $g->send("Y")` | `1`,`got=X`,`2`,`got=Y` |
| metodi manuali | `$g->current(),$g->key(); $g->next(); $g->valid()` | come Iterator |
| `yield;` nudo | `function g(){yield;}` `$g->current()`/`->key()` | `NULL` / `int(0)` |
| instanceof | `$g instanceof Generator/Iterator/Traversable` | tutti `true` |

**Nota chiave su `yield from`** (key numbering): `yield from` **preserva le chiavi
interne** e NON le rinumera rispetto al generatore esterno → nell'esempio sopra
le chiavi sono `0:1 0:10 1:20 1:2` (il `10,20` riparte da 0). Il contatore
auto-key dell'esterno NON avanza durante un `yield from`. `getReturn()` di un
`yield from` su un generatore restituisce il valore di return del delegato (è il
valore dell'espressione `yield from`).

## 3. AST mago

`Expression::Yield(Yield)` con `enum Yield { Value(YieldValue), Pair(YieldPair),
From(YieldFrom) }` (`mago-syntax/src/ast/ast/yield.rs`):
- `YieldValue { value: Option<&Expression> }` — `yield $v` o `yield;` (None).
- `YieldPair { key, value }` — `yield $k => $v`.
- `YieldFrom { iterator: &Expression }` — `yield from $x`.

**Detezione generatore**: una funzione è un generatore sse il suo corpo contiene
un `yield`/`yield from` a qualunque profondità **ma non dentro una funzione/
closure annidata**. Servirà un walk (riusare `Node::children()` di mago o un
visitor sul corpo lowerato) che si ferma ai confini di funzione/closure.

## 4. Struttura attuale del valutatore e IL PROBLEMA

- `Evaluator` è un tree-walker ricorsivo: `fn eval(&mut self, e) -> Result<Zval>`
  e `fn exec_stmts(&mut self) -> Result<Flow>`. `enum Flow { Normal, Break(u32),
  Continue(u32), Return(Zval) }` (eval.rs:117).
- `Zval` usa `Rc<RefCell<...>>` → l'`Evaluator` è **`!Send`** (non attraversa
  thread).
- `call_user_fn`/`run_user_fn_body` installano un frame locale (`self.locals`),
  eseguono `exec_stmts(&f.body)`, ripristinano. **Tutto sullo stack nativo Rust.**

**Il problema centrale**: `yield` deve **sospendere** l'esecuzione nel mezzo della
ricorsione di `eval()` (lo `yield` può essere dentro loop/if annidati), restituire
il valore al chiamante, e **riprendere** esattamente da lì (con un valore da
`send()`). Sospendere a metà di `eval()` significa **preservare lo stack nativo**
della ricorsione. Un tree-walker `&mut self` non può farlo nativamente.

## 5. Opzioni architetturali (ricerca crate fatta — vedi tabella)

| Opzione | Stackful? | Forza Send/'static? | Verdetto |
|---|---|---|---|
| **`corosensei` 0.3.4** (`ScopedCoroutine`) | sì | **no** (non-'static OK, `!Send` OK) | **CONSIGLIATA** |
| `generator` 0.8.9 (`LocalGenerator`) | sì | default sì; opt-out `LocalGenerator` | fallback |
| `genawaiter` | no (stackless) | — | **scarta** (non sospende dentro `eval()`) |
| `may`/`mco` | sì | sì `Send+'static` (fatale) | **scarta** |
| `std::ops::Coroutine`/`gen` | no (stackless) | — | **scarta** (nightly + stackless) |
| state-machine esplicita (no crate) | n/a | no | principled, ma rewrite enorme |

**Perché stackful è obbligatorio**: lo `yield` scatta in profondità dentro
`self.eval(...)`. Solo le coroutine **stackful** (corosensei/generator) possono
sospendere quello stack nativo. Tutte le stackless (genawaiter, `gen`/`Coroutine`
nativo) richiederebbero di riscrivere TUTTO il valutatore come async/state-machine
→ a quel punto conviene la opzione 6 (state-machine) direttamente.

**Il nodo del borrow (vale per OGNI opzione stackful)**: il driver (`$gen->next()`,
che gira con `&mut Evaluator`) e il corpo della coroutine (che fa `self.eval(...)`)
hanno bisogno **dello stesso `&mut Evaluator`**, mai simultaneamente (l'esclusività
è temporale: il driver non tocca `self` mentre la coroutine è viva e viceversa).
Il borrow-checker non vede questa esclusività attraverso il punto di sospensione.
**Soluzione**: NON catturare `&mut Evaluator` nella closure; passarlo **dentro a
`resume()`** a ogni ripresa come `*mut Evaluator`, e reborrow `unsafe { &mut *p }`
dentro. L'unsafe è confinato a 1-2 helper, giustificato dall'esclusività; guardia:
mai rientrare nello stesso generatore (creerebbe aliasing reale).

## 6. Architettura consigliata (corosensei) — dettaglio

**D-GEN-1 (motore): `corosensei::ScopedCoroutine`.** Aggiungere dep
`corosensei = "0.3"` a `php-runtime/Cargo.toml`.

**Oggetto `Generator`**: un nuovo `Zval::Object` di una classe engine `Generator`
(come `Closure`: NON nella user class table; istanza speciale), oppure una nuova
variante `Zval` dedicata. **Consigliato**: variante dedicata
`Zval::Generator(Rc<RefCell<GenState>>)` (come `Zval::Closure`), con
`is_instance_of` special-case per `Generator`/`Iterator`/`Traversable` (come fatto
per `Stringable` allo step 24). `GenState` tiene: la coroutine corosensei, lo
stato (NotStarted/Suspended/Running/Done), l'ultimo `(key, value)` correnti, il
contatore auto-key, il valore di `getReturn`, eventuale `send` pendente.

**Mapping semantico**:
- `Yielder::suspend(YieldOut{key,value})` ⇄ `yield` → restituisce il valore di
  `send()` (diventa il risultato dell'espressione `yield`).
- `CoroutineResult::Yield` ⇄ generatore ha prodotto un valore (Suspended).
- `CoroutineResult::Return` ⇄ corpo finito (Done) → valore per `getReturn()`.
- `current()`/`key()`: leggono l'ultimo `(key,value)`; avviano il generatore
  (prima `resume`) se NotStarted (PHP avvia pigro al primo accesso).
- `next()`: `resume` con `send=null`.
- `send($v)`: `resume` con `send=$v` (se NotStarted, PHP fa prima un avvio).
- `valid()`: stato != Done.
- `rewind()`: avvia se NotStarted; errore se già avanzato (PHP: "Cannot rewind a
  generator that was already run").
- `getReturn()`: solo dopo Done.
- auto-key: contatore che parte da 0, avanza a ogni `yield` SENZA key esplicita
  (anche `yield $k=>$v` con `$k` intero ≥ contatore aggiorna il contatore, come
  gli array). `yield from` NON avanza il contatore esterno.

**`yield` nel valutatore**: nuovo `ExprKind::Yield { key: Option<Expr>, value:
Option<Expr> }` e `ExprKind::YieldFrom(Expr)`. Quando `eval` incontra uno `yield`,
chiama `self.current_yielder.suspend(...)` (lo `Yielder` attivo è memorizzato
sull'`Evaluator`, salvato/ripristinato come `cur_this`). `yield from` = loop che
guida il sub-iteratore/sub-generatore e ri-sospende ogni elemento (preservando le
chiavi), poi il valore dell'espressione = `getReturn()` del delegato (se
generatore) o null (se array/Traversable).

**`foreach` su generatore/oggetto Iterator**: oggi `foreach` itera solo array
(eval.rs `exec_foreach*`). Serve estendere `foreach` a iterare un oggetto
`Traversable`: se è un `Zval::Generator` → guidarlo (rewind/valid/current/key/next);
se è un oggetto user che implementa `Iterator` → chiamare i suoi metodi. **Questo è
un companion necessario** (scope-out storico "foreach su oggetti/Generator" dello
step 20). Decidere se includerlo qui (D-GEN-2: consigliato sì, almeno per
Generator; Iterator user generico può essere sotto-step).

**Detezione generatore al lowering**: `FnDecl.is_generator: bool` calcolato con un
walk del corpo (fermandosi a funzioni/closure annidate). `call_user_fn`: se
`is_generator`, NON eseguire il corpo — costruire e ritornare un `Zval::Generator`
che cattura il frame (args già legati) e il corpo, da eseguire pigramente nella
coroutine.

## 7. Decisioni da prendere col Decider (default consigliati)

- **D-GEN-1** motore = `corosensei` `ScopedCoroutine` (vs `generator`
  `LocalGenerator`, vs state-machine). *Consigliato: corosensei.*
- **D-GEN-2** `foreach` su Generator incluso nello step (vs sotto-step a parte).
  *Consigliato: incluso (senza generatore-in-foreach i test base non girano).*
  `foreach` su Iterator user generico = sotto-step successivo.
- **D-GEN-3** rappresentazione = `Zval::Generator(Rc<RefCell<GenState>>)` variante
  dedicata (vs oggetto di classe prelude). *Consigliato: variante dedicata (come
  Closure), instanceof special-case.*
- **D-GEN-4** `Generator::throw()` + eccezioni-attraverso-yield = **scope-out**
  (interazione unwinding complessa). *Consigliato: scope-out, cataloghare.*
- **D-GEN-5** confine unsafe: un solo helper `resume_generator(&mut self, &mut
  GenState, send: Zval)` che fa il reborrow `*mut Evaluator`; documentare
  l'invariante di non-rientranza. *Consigliato: sì.*

## 8. Scaletta TDD (39-1 … 39-N)

1. **39-1 infra + basic `yield`**: dep corosensei; `ExprKind::Yield`; detezione
   `is_generator`; `Zval::Generator`+`GenState`; `call_user_fn` ritorna il
   generatore; `current`/`next`/`valid`/`key` minimi; `resume_generator` con
   l'unsafe reborrow. RED: `function g(){yield 1;yield 2;} $g=g(); echo
   $g->current(); $g->next(); echo $g->current();` → `12`.
2. **39-2 `foreach` su Generator** (D-GEN-2): estendere `exec_foreach` a guidare un
   `Zval::Generator`. RED: `foreach(g() as $k=>$v) echo "$k:$v ";`.
3. **39-3 key esplicita + auto-key** (`yield $k=>$v`, contatore).
4. **39-4 `send()`** (valore di ritorno di `suspend` → risultato dello `yield`).
5. **39-5 `return` + `getReturn()`** (CoroutineResult::Return).
6. **39-6 `yield from`** (array + generatore; preservazione chiavi; getReturn del
   delegato). 
7. **39-7 instanceof Generator/Iterator/Traversable + rewind semantics + var_dump**.
8. **39-8** corpus `Zend/tests/generators` + diary/divergences/README/memoria.

## 9. Rischi / note

- **Memoria stack per generatore**: corosensei alloca uno stack reale (~128KB
  default, configurabile) per generatore vivo. OK per centinaia; attenzione a
  test con migliaia di generatori (cataloghare se emergono).
- **Drop di generatore sospeso**: corosensei fa unwinding dello stack sospeso
  (mappa bene su `finally` PHP, ma vedi D-GEN-4 per eccezioni).
- **Re-entrancy guard**: mai chiamare `resume` su un generatore già `Running`
  (PHP: "Cannot resume an already running generator") — implementare il check,
  protegge anche l'invariante unsafe.
- **`!Send` / `Rc`**: corosensei `ScopedCoroutine` non forza `Send`/'static → OK.
  NON usare il `generator` crate con le alias di default (forzano `Send`).
- **Finding tooling pre-esistente**: un test in `Zend/tests` manda il tree-walker
  in **stack overflow** (vedi `NEXT-backlog-scan.md`); i generatori NON lo
  peggiorano ma il corpus `Zend/tests/generators` va girato per-file con
  `perl -e 'alarm N;exec @ARGV'` (su macOS non c'è `timeout`/`gtimeout`).

## 10. File e punti d'integrazione

- `php-runtime/Cargo.toml` — dep `corosensei`.
- `php-types/src/zval.rs` — `Zval::Generator(Rc<RefCell<GenState>>)`.
- `php-runtime/src/hir.rs` — `ExprKind::Yield`/`YieldFrom`; `FnDecl.is_generator`.
- `php-runtime/src/lower.rs` — lowering `Expression::Yield`; walk `is_generator`.
- `php-runtime/src/eval.rs` — `GenState`, `resume_generator` (unsafe reborrow),
  arm `ExprKind::Yield`/`YieldFrom`, `call_user_fn` generator-aware, metodi
  `Generator` (come `closure_method`), `exec_foreach` su Generator, instanceof
  special-case, `current_yielder` salvato/ripristinato.
- Test: `php-runtime/tests/eval.rs` (sezione Step 39). Corpus:
  `/tmp/php-src/Zend/tests/generators/`.

## Appendice — sketch del reborrow (D-GEN-5)

```rust
// In/out della coroutine
struct ResumeIn { sent: Zval, ev: *mut Evaluator }
enum YieldOut { Yielded { key: Zval, value: Zval } }

// Corpo (NON cattura &mut Evaluator; lo riceve a ogni resume):
let co = ScopedCoroutine::new(move |y: &Yielder<ResumeIn, YieldOut>, start: ResumeIn| -> Zval {
    let ev: &mut Evaluator = unsafe { &mut *start.ev };
    ev.set_yielder(y);                 // così l'arm Yield può chiamare y.suspend
    ev.run_generator_body(&body)        // ricorsione eval; su `yield` -> y.suspend(...)
});

// Driver (gira con &mut self):
fn resume_generator(&mut self, gs: &mut GenState, sent: Zval) -> Result<(), PhpError> {
    debug_assert!(gs.state != Running);  // re-entrancy guard (invariante unsafe)
    gs.state = Running;
    match gs.co.resume(ResumeIn { sent, ev: self as *mut _ }) {
        CoroutineResult::Yield(YieldOut::Yielded{key,value}) => { gs.cur=(key,value); gs.state=Suspended; }
        CoroutineResult::Return(ret) => { gs.ret=ret; gs.state=Done; }
    }
    Ok(())
}
```
L'`unsafe { &mut *start.ev }` è valido perché `self` (driver) non è in uso mentre
la coroutine gira (esclusività temporale) e la guardia impedisce la rientranza.
