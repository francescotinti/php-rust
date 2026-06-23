# Piano di Refactoring per `vm.rs` (Modularizzazione Bytecode VM)

Questo documento descrive i passaggi per Claude AI al fine di eseguire il refactoring e la modularizzazione del file monolitico `crates/php-runtime/src/vm.rs` (attualmente ~10.000 righe) in moduli più piccoli e manutenibili, mantenendo la totale retrocompatibilità e il superamento dei test esistenti.

## 1. Obiettivo
Separare le diverse responsabilità della Virtual Machine (OOP, chiamate a funzioni, gestione array, coroutine/fiber, eccezioni) in file dedicati all'interno di una nuova cartella `vm/`, riducendo i conflitti di merge e migliorando la leggibilità.

## 2. Strategia di Suddivisione

Creare la directory `crates/php-runtime/src/vm/`. Spostare l'attuale `vm.rs` in `crates/php-runtime/src/vm/mod.rs`.

All'interno di `vm/`, creare i seguenti sottomoduli:
- `oop.rs`: Per istanziazione oggetti, chiamate a metodi magici (`__get`, `__set`, etc.), cast di oggetti (`object_cast`), e logica di esecuzione dei distruttori (`run_shutdown_destructors`).
- `calls.rs`: Per la risoluzione delle chiamate a funzione, metodi, closure (`invoke_value`, `CallValue`, gestione dell'arità) e spacchettamento dei reference.
- `arrays.rs`: Per le operazioni su array, lettura/scrittura di chiavi e logica dei path (`FetchDim`, `AssignPath`, `path_op`, `read_dim`, `unset_into`).
- `coroutines.rs`: Per la logica dei generatori (`yield`, `yield from`, parcheggio dei frame) e l'implementazione dei Fiber (GEN-4).
- `exceptions.rs`: Per l'unwinding dello stack (`unwind`), e la sintesi dei Throwable e dei fatal error interni.

## 3. Gestione della Visibilità in Rust

Poiché sposteremo le funzioni di `Vm` e `Frame` in sottomoduli (es. `impl<'m> super::Vm<'m> { ... }`), i sottomoduli avranno bisogno di accedere ai campi interni di `Vm` e `Frame`.

**Soluzione:** 
In `vm/mod.rs`, cambia la visibilità dei campi delle struct principali (`Vm`, `Frame`, `FiberState`, ecc.) da privati a `pub(super)` o `pub(crate)`. In questo modo i sottomoduli potranno accedere e modificare lo stato (come `self.frames`, `self.stack`, `self.diags`) senza dover scrivere getter/setter infiniti.

Esempio:
```rust
pub(super) struct Vm<'m> {
    pub(super) module: &'m Module,
    pub(super) frames: Vec<Frame<'m>>,
    // ...
}
```

## 4. Passaggi Operativi per l'Implementazione

**Fase 1: Preparazione**
1. Esegui `mkdir crates/php-runtime/src/vm`.
2. Esegui `mv crates/php-runtime/src/vm.rs crates/php-runtime/src/vm/mod.rs`.
3. In `crates/php-runtime/src/lib.rs` verifica che ci sia `pub mod vm;` (rimarrà inalterato in quanto punta ora alla cartella).

**Fase 2: Estrazione Moduli (Lavorare un modulo alla volta)**
Per evitare errori di compilazione titanici, estrai un modulo alla volta e assicurati che il codice compili eseguendo `cargo check -p php-runtime` prima di passare al successivo.
1. Crea `vm/exceptions.rs`, sposta il metodo `unwind` e le funzioni legate. In `vm/mod.rs` aggiungi `mod exceptions;`. Aggiungi `impl<'m> super::Vm<'m>` in `exceptions.rs`. Rendi i campi necessari `pub(super)`.
2. Crea `vm/coroutines.rs`. Sposta la logica di `Fiber` e `Generator`, inclusi i metodi di resume e sospensione.
3. Crea `vm/arrays.rs`. Sposta i complessi metodi helper per i path (`path_op`, `silent_get_path`, `field_set`).
4. Crea `vm/oop.rs`. Sposta `run_shutdown_destructors`, `object_cast`, e helper per i metodi magici.
5. Crea `vm/calls.rs`. Sposta `invoke_value` e i relativi setup per i frame chiamati.

**Fase 3: Refactoring del Loop Principale**
Il mega `match op` dentro `run_loop` in `vm/mod.rs` rimarrà il cuore del dispatch, ma il corpo dei match branch più complessi dovrà richiamare le funzioni estratte nei sottomoduli per mantenerlo leggibile.

**Fase 4: Verifica finale**
1. Esegui `cargo test -p php-runtime` per verificare l'assenza di regressioni.
2. Controlla che le macro e gli import (`use`) siano allineati in tutti i nuovi sottomoduli creati.

---
**Nota per Claude AI:** Questa base di codice fa un uso mirato di lifetimes (`'m`) per legare la VM al modulo caricato. Assicurati di mantenere rigorosamente questi lifetime in ogni blocco `impl<'m>` che vai a spostare nei nuovi file. Metti massima attenzione nel non causare memory leak o doppie esecuzioni di `__destruct` durante il refactoring dell'OOP.
