# Analisi dei Vincoli Architetturali: Safe Rust vs C (23 Luglio 2026)

Oltre all'impatto dell'architettura della Virtual Machine (Stack vs Registri), i profili prestazionali mostrano altre criticità evidenti, in particolare l'enorme **footprint di memoria (12.0x rispetto all'oracolo)** e l'overhead di CPU nascosto.

Queste frizioni sono il costo diretto dell'implementazione di un linguaggio altamente dinamico e "sporco" come PHP all'interno del paradigma **Safe Rust**. Ecco le cause strutturali che stanno frenando `phpr`:

## 1. Il Problema della Memoria (Footprint 12.0x)

*   **La tassa di `Rc` (Reference Counting):**
    In C (Zend Engine), il reference counting è "intrusivo": la VM usa un singolo `uint32_t` (4 byte) nascosto nell'intestazione di ogni oggetto. In Rust, per avere reference counting in modo sicuro, si utilizza `Rc<T>`. Un puntatore `Rc` alloca spazio aggiuntivo per tracciare due contatori a 64-bit (Strong count e Weak count = 16 byte extra). Questo significa che ogni stringa, array o oggetto in `phpr` porta con sé una zavorra di decine di byte in più solo per gestire il proprio ciclo di vita.
*   **La tassa del `RefCell` (Mutabilità Condivisa):**
    PHP muta continuamente stato condiviso (es. scope globale, variabili passate per riferimento). Per replicare questo comportamento in Safe Rust evitando il controllo dei borrow a compile-time, si usa l'accoppiata `Rc<RefCell<T>>`. Un `RefCell` aggiunge ulteriore overhead in memoria per mantenere lo stato interno dei prestiti (borrow state), aumentando ulteriormente l'impronta di ogni Zval complesso.
*   **Leak Fisiologico dell'AST?**
    Zend Engine elabora il file PHP, crea l'Albero Sintattico (AST), emette il bytecode e distrugge istantaneamente l'AST. Se `phpr` mantiene in memoria l'AST generato dal parser (`mago`) o la rappresentazione intermedia (HIR) per tutta la durata dell'esecuzione della suite di WordPress, questo spiegherebbe da solo una larga fetta dei picchi multi-GB.
*   **Mancanza di Allocazione ad Arena (Bump Allocation):**
    Zend alloca quasi tutta la memoria temporanea di una richiesta HTTP in una singola Arena continua, senza metadata (no header di blocco per ogni stringa). `mimalloc` è un eccellente general-purpose allocator, ma paga comunque un overhead di frammentazione e metadati per i milioni di micro-allocazioni generate dall'interprete.

## 2. Il Problema della CPU (L'Overhead di "Safety")

*   **I controlli a runtime del `RefCell`:**
    Il `RefCell` non consuma solo memoria, ma brucia cicli di CPU. Ogni volta che la VM esegue un `.borrow()` o un `.borrow_mut()`, Rust inietta a runtime un controllo: incrementa i contatori interni, verifica la presenza di alias multipli e piazza un branch (un `if`) per sollevare un `panic` in caso di violazione. Zend Engine, in puro C, sovrascrive semplicemente i puntatori. Moltiplicato per milioni di operazioni, questo overhead pesa sul `run_loop`.
*   **Bounds Checking (Controllo dei limiti in memoria):**
    Quando si accede a un indice di un array o di una slice (`buf[i]`), il compilatore Rust inietta automaticamente dei check sui limiti di memoria per prevenire buffer overflow, a meno che non possa provare matematicamente che l'indice è sicuro. Zend Engine accede direttamente agli indirizzi di memoria calcolati. In loop serrati o operazioni massive sulle stringhe, questi branch imprevisti rallentano l'esecuzione e inquinano la branch prediction.
*   **Strutture dati "Generaliste" vs "Specializzate":**
    La `HashTable` di Zend Engine è uno dei pezzi di ingegneria C più complessi in circolazione: funge contemporaneamente da dizionario e lista concatenata, garantendo un packing della memoria ineguagliabile e una perfetta *cache locality*. Pur usando ottime librerie come `IndexMap` o `hashbrown`, `phpr` si affida a strutture *general-purpose* che non possono replicare la densità in memoria delle strutture ibride di PHP progettate su misura.

## Sintesi

Se il passaggio ai **Registri** rappresenterà la cura per l'overhead della CPU e la mole di bytecode, il **Footprint a 12.0x** è il prezzo dell'utilizzo di pattern tipici del *Safe Rust* (`Rc`, `RefCell`, Enum pesanti). 

Per abbattere concretamente questo 12x, il team dovrà valutare se rinunciare all'AST subito dopo la fase di *lowering*, o se, in futuro, sostituire `Rc<RefCell<T>>` con costrutti personalizzati (eventualmente appoggiandosi a blocchi `unsafe` molto circoscritti) per eguagliare la densità del C.
