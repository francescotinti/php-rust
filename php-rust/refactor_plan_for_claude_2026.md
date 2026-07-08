# Piano di Refactoring Strategico per Claude AI (VM & Compile)

Questo documento contiene le direttive strategiche per smembrare i colli di bottiglia strutturali identificati nel progetto `php-rust-experiment`, in particolare `vm/mod.rs` e `compile.rs`. 

Claude, in quanto esecutore, segui scrupolosamente queste fasi passo dopo passo, validando ogni passaggio tramite il compilatore. L'obiettivo primario è **non alterare il comportamento logico**, ma solo ristrutturare il codice per renderlo manutenibile.

---

## Fase 1: Estrazione degli Host Builtins dalla VM (`vm/mod.rs`)

Il file `crates/php-runtime/src/vm/mod.rs` ha superato le 24.000 righe. La maggior parte di questo volume è dovuta a centinaia di metodi "host" (come `ho_proc_open`, `ho_file_get_contents`, ecc.) implementati direttamente all'interno di `impl Vm`.

**Passaggi operativi:**
1. Crea un nuovo file `crates/php-runtime/src/vm/host.rs`.
2. Nel file `vm/mod.rs`, in cima (accanto agli altri moduli `arrays`, `calls`, ecc.), aggiungi `mod host;`.
3. Crea un blocco `impl<'m> super::Vm<'m> { ... }` all'interno di `host.rs`. Assicurati di importare i tipi e le struct necessarie (es. `Zval`, `PhpError`, `Ctx` ecc.).
4. Sposta da `vm/mod.rs` a `vm/host.rs` la funzione gigante `dispatch_host_builtin` e **tutti** i metodi che iniziano con il prefisso `ho_` (es. `fn ho_strlen`, `fn ho_json_encode`, e così via).
5. **Risoluzione della visibilità:** Poiché queste funzioni manipoleranno campi interni della `Vm` o chiameranno altre funzioni di supporto definite in `vm/mod.rs`, dovrai rendere pubblici (o `pub(super)`) i metodi e campi chiamati all'interno di `vm/mod.rs`.
6. Esegui `cargo check -p php-runtime`. Ripristina eventuali import (`use`) e visibilità mancanti fino a che la compilazione non ha successo.

---

## Fase 2: Modularizzazione del Compilatore (`compile.rs`)

Il file `crates/php-runtime/src/compile.rs` (circa 5.300 righe) ha perso modularità. Dovrai modellarlo strutturalmente ad albero come è stato fatto in parte per `lower/`.

**Passaggi operativi:**
1. Esegui il setup della cartella:
   - `mkdir crates/php-runtime/src/compile`
   - `mv crates/php-runtime/src/compile.rs crates/php-runtime/src/compile/mod.rs`
   - Verifica che `crates/php-runtime/src/lib.rs` continui a far riferimento a `pub mod compile;` senza errori.
2. In `compile/mod.rs`, analizza le definizioni delle `struct` di contesto (come `FnCompiler<'a>`, `ProgramCtx<'a>`). Cambia la visibilità dei loro campi interni a `pub(super)` così da potervi accedere dai sottomoduli.
3. Estrai le funzioni logiche nei seguenti sottomoduli (creandoli e aggiungendoli con `mod xyz;` in `mod.rs`):
   - `compile/expr.rs`: sposta l'implementazione del metodo `expr` e gli helper direttamente correlati al parsing delle espressioni (ci sono funzioni da ~800 righe qui). Usa `impl<'a> super::FnCompiler<'a>`.
   - `compile/stmt.rs`: sposta il metodo `stmt`, `try_stmt`, loop e i vari costrutti di controllo di flusso.
   - `compile/class.rs`: sposta `compile_class`, `stub_class`, funzioni di utilità dei path delle classi (es. `find_const_decl`).
   - `compile/func.rs` (o `calls.rs`): sposta `compile_fndecl`, `compile_body`, `call()`, e la risoluzione delle chiamate a funzione.
4. Dopo la creazione di ogni sottomodulo, esegui `cargo check -p php-runtime` per verificare le reference. Mantieni rigorosamente i parametri lifetime `'a`.

---

## Fase 3 (Bonus): Ulteriore pulizia in `lower/mod.rs`

Se rimane spazio nel contesto della sessione, puoi aggredire `crates/php-runtime/src/lower/mod.rs` (che conta ancora 6.500 righe) estraendo ulteriori astrazioni:
1. Crea `crates/php-runtime/src/lower/stmt.rs` per ospitare la porzione di traduzione degli *statement* (che molto probabilmente ingolfa `mod.rs`).
2. Usa lo stesso paradigma del `pub(super)` sulle struct di stato in `mod.rs` e definisci i metodi in `impl<'a> super::StructLowering<'a> { ... }`.

---
## Checklist di Esecuzione & Verifica:
- [ ] L'astrazione logica del codice non è stata modificata (no bug-fixing durante questa fase, **solo spossamenti strutturali**).
- [ ] Nessun warning o errore introdotto (esegui costantemente `cargo check -p php-runtime`).
- [ ] Esegui i test prima del commit (`cargo test -p php-runtime` e `cargo test -p php-builtins`).
- [ ] Tutti i lifetime (`'m`, `'a`) sono intatti.
