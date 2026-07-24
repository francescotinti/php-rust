# Analisi e Suggerimenti per il Porting PHP -> Rust (Iterazione 4)

Questo documento contiene un'analisi aggiornata della base di codice del progetto `php-rust` a seguito degli ultimi estesi sviluppi sulla Virtual Machine (file `vm.rs` e `compile.rs`), insieme alle raccomandazioni architetturali prioritarie.

## 1. Analisi dello Stato Attuale (Sviluppi Recenti)

Il progetto ha superato la fase di "Vertical Proof Slice" raggiungendo una maturità notevole nella sua nuova architettura:

- **Maturazione della Bytecode VM**: La transizione verso la VM a bytecode (`vm.rs`) ha fatto passi da gigante. Ora supporta nativamente costrutti complessi come loop (`foreach` by-ref e by-val), unwinding delle eccezioni (`try`/`catch`/`finally`), closure, first-class callables e la risoluzione OOP (inclusi i recursion guard per i magic methods).
- **Generatori e Fiber Nativi (GEN-4)**: L'abbandono di `corosensei` è un traguardo fondamentale. `yield`, `yield from` e `Fiber::suspend` ora parcheggiano i frame in memoria sull'heap Rust (nella struct `Vm`) senza manipolare lo stack nativo, garantendo sicurezza, conformità ed eliminando blocchi legati a codice unsafe o coroutine di sistema.
- **Arricchimento dei Built-in**: La libreria standard `crates/php-builtins` è cresciuta notevolmente, coprendo gran parte delle funzioni base per array (`array_slice`, `array_splice`, `array_merge`), I/O, stringhe e math. La semantica di type-juggling di PHP continua ad essere rispettata con estrema fedeltà.

## 2. Opportunità di Miglioramento (Priorità per il Prossimo Futuro)

Dall'ispezione della base di codice aggiornata, ecco i suggerimenti per portare la VM verso la feature-parity totale e un grado di pulizia "production-ready":

### A. Completamento della "Feature Parity" in `compile.rs`
Sebbene `vm.rs` sia in grado di eseguire opcodes complessi, il compilatore `compile.rs` blocca ancora alcune funzionalità del linguaggio, impedendo la dismissione del vecchio evaluatore AST (`eval.rs`). È necessario implementare la compilazione per i seguenti `CompileError::Unsupported`:
- **Argument Unpacking e Named Arguments**: Supportare l'operatore di spread (`...$args`) negli array e nelle chiamate a funzione, così come gli argomenti nominati (es. `foo(name: "val")`) per i metodi e le funzioni.
- **Riferimenti Dinamici**: Implementare le reference dinamiche a classi (`new $className`), la risoluzione ritardata (`self`, `parent` in contesti particolari) e la scrittura corretta nell'array globale (`$GLOBALS`).
- **Scritture Complesse su Array**: Sistemare la semantica di append `[]` in contesti di read e i reference assignment derivanti da non-call.

**Azione Consigliata:** Risolvere i gap mappando i rimanenti HIR nodes verso istruzioni della Bytecode VM, per poter infine deprecare ed eliminare il modulo `eval.rs`.

### B. Rimozione del Boilerplate nei Built-in (Procedural Macro)
In `crates/php-builtins/src` si nota ancora un esteso uso di estrazione e validazione manuale degli argomenti (es. `arg1(args, "nome")`, o conversioni manuali seguite da controlli di arità). Questo approccio genera rumore e aumenta la probabilità di produrre errori non in linea con i messaggi standard di PHP 8+.
- **Azione Consigliata:** Sviluppare finalmente la **Procedural Macro** (es. `#[php_builtin]`) ipotizzata nelle analisi precedenti. Questa macro dovrà auto-generare il boilerplate per:
  1. Parsing posizionale o nominale della slice `&[Zval]`.
  2. Coercizione silente o validazione tipizzata in linea con la semantica PHP (generando in automatico corretti `TypeError` o `ValueError`).

### C. Hardening della VM (Sostituzione degli `.expect()`)
Un'analisi di `vm.rs` rivela l'uso di numerosi metodi `.expect("...")` durante l'esecuzione del bytecode (es. `.expect("Dup on empty stack")`, `.expect("Binary rhs")`). 
Sebbene il bytecode prodotto da `compile.rs` sia "trusted" e garantisca in via teorica la bilanciatura dello stack, una VM robusta non dovrebbe **mai** causare un "panic" del processo host.
- **Azione Consigliata:** Sostituire progressivamente le assunzioni basate su `expect()` e `unwrap()` all'interno del loop di esecuzione con un costrutto di fallimento gestito (es. restituendo un errore interno fatale o un `Result<T, VmPanicError>`). Questo aumenta la resilienza della VM contro potenziali bug del compilatore.

### D. Ottimizzazioni Architetturali Future (Tier 3)
Con la struttura di base solida, il prossimo passo sarà colmare il gap prestazionale rispetto al runtime Zend in C:
- **Inline Caching:** Introdurre meccanismi di inline caching nei call-site dei metodi o durante l'accesso alle proprietà (`PropGet` / `PropSet`) per ammortizzare i costi delle lookup in hash table o risoluzione OOP dinamica.
- **String e Array Interning:** Condividere e internare le `PhpStr` ricorrenti (es. chiavi di array molto usate o nomi di metodi) per ridurre il footprint in memoria e accelerare le comparazioni in `php_types`.

## Conclusione
Il lavoro svolto ha portato il porting in Rust di PHP in una fase eccellente: il design "flat array" della Bytecode VM combinato con generatori implementati in user-space tramite salvataggio del frame su heap Rust è la direzione architetturale corretta. Concentrandosi sul completamento del mapping in `compile.rs` e sull'introduzione della procedural macro per i built-ins, il progetto potrà considerarsi strutturalmente vicino a una vera alternativa al runtime ufficiale.
