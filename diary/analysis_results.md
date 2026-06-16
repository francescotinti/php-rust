# Analisi del Progetto: PHP-Rust

Dopo aver analizzato la documentazione del progetto (inclusi `EXPERIMENT_PLAN.md`, il diario degli step, la tabella di mapping e le divergenze note), emerge che l'architettura attuale è un'eccellente e pragmatica reimplementazione di PHP. Sfrutta `mago` come front-end per aggirare il parsing complesso, e traduce l'AST in un HIR (High-Level Intermediate Representation) su cui opera un valutatore *tree-walking*.

L'approccio guidato dai test `.phpt` (Oracle-driven) ha garantito una fedeltà semantica impressionante (zero mismatch su oltre 37.000 casi per gli operatori). 

Tuttavia, considerando gli obiettivi di lungo termine e i limiti architetturali di un interprete *tree-walking*, ecco le **migliorie principali** che si possono applicare al progetto, suddivise per categoria:

## 1. Miglioramenti Architetturali e di Stabilità

### A. Prevenzione dello Stack Overflow (Call Stack esplicito)
**Problema:** Nel file `NEXT-backlog-scan.md` viene evidenziato che la ricorsione profonda nel valutatore *tree-walking* manda in crash il processo Rust (`SIGABRT`), interrompendo l'intero batch del `phpt-runner`. Rust non ha una protezione nativa contro lo stack overflow.
**Soluzione:** 
- Introdurre un limite di profondità di esecuzione (es. `self.depth += 1`) nell'`Evaluator`. Raggiunto il limite, sollevare un `PhpError::Error` ("Maximum function nesting level reached") per evitare il crash del processo host.
- *Evoluzione futura:* Implementare l'evaluator con un loop iterativo e uno stack esplicito allocato sull'heap, slegando l'esecuzione dal call stack nativo di Rust.

### B. Gestione della Memoria e Cicli (Garbage Collection)
**Problema:** L'implementazione attuale si affida a `Rc` e `RefCell` con semantica COW (Copy-On-Write). Come notato nelle decisioni (`D-R15`), i riferimenti circolari (es. `$a[0] = &$a;` o le referenze negli oggetti in Tier 2) causano *memory leak* poiché non c'è un ciclo-collettore.
**Soluzione:**
- Per un processo a singola richiesta CLI può essere accettabile, ma per un processo residente (il target web menzionato nell'EXPERIMENT_PLAN) diventerà un problema. 
- Introdurre un semplice tracciatore di cicli, oppure integrare un crate come `bacon-rajan-cc` (che offre reference counting con cycle collection) per il tipo `Zval::Ref` e `Zval::Object`.

## 2. Ottimizzazioni Prestazionali e di Memoria

### A. Ottimizzazione della dimensione di `Zval` (Nan-Boxing / Tagged Pointers)
**Problema:** L'enum `Zval` in Rust probabilmente occupa più dei 16 byte originali dello Zend Engine, a causa dell'overhead del tag dell'enum e del padding per i tipi più grandi (come i `f64` o i puntatori `Rc`). Un `Zval` più grande comporta un maggiore utilizzo di cache miss.
**Soluzione:**
- Sfruttare tecniche come il **Nan-Boxing** (usare i bit inutilizzati di un `f64` NaN per codificare i puntatori e i tag) o usare `union` e `NonZeroU64` in combinazione con un tag separato, per riportare la dimensione di `Zval` a 8 o 16 byte garantiti, migliorando drasticamente la cache locality durante l'iterazione degli array e lo scope locale.

### B. Dal Tree-Walking al Bytecode
**Problema:** Valutare l'albero (HIR) con `match` ad ogni step è intrinsecamente più lento della virtual machine a opcode dello Zend Engine (a causa del branch prediction e della località dei dati).
**Soluzione:** 
- Come ipotizzato nel piano originale, ora che la semantica è "inchiodata" e testata, si può introdurre uno step di **compilazione HIR -> Bytecode**. L'`Evaluator` diventerebbe una loop instruction dispatch (es. un grande `match` o computed goto se supportato in Rust nightly via inline assembly) operante su un array lineare di opcode.

## 3. Fedeltà ed Esperienza Utente (DevEx)

### A. Stack Trace accurati per le Eccezioni
**Problema:** La divergenza `D-NEW-7` nota che un *fatal error* dentro una funzione utente mostra solo `#0 {main}` invece dell'intero call stack.
**Soluzione:** 
- L'`Evaluator` deve mantenere un tracciato esplicito delle chiamate di funzione attive (un `Vec<FrameInfo>` con nome file, linea e nome funzione chiamante). Quando si crea un'eccezione o un errore fatale, questo stack deve essere formattato correttamente per riprodurre il trace completo (`#0 ...: f(...)`).

### B. Isolamento del `phpt-runner`
**Problema:** Attualmente il runner gira nello stesso processo. Un crash (come lo stack overflow descritto in `NEXT-backlog-scan.md`) fa abortire l'intera suite di test.
**Soluzione:** 
- Eseguire ogni test in un thread isolato usando `std::panic::catch_unwind` (se non è un abort) oppure fare spawn di child-process separati per eseguire ogni script. Questo renderà l'esecuzione dei 6172 test molto più resiliente.

## 4. Completamento Funzionalità (Migliorie Funzionali)

- **Corretto completamento di `yield`:** Aggiungere il supporto per il `throw()` dentro i generatori e la gestione del `finally` in caso di sospensione (attualmente definiti come scope-out in `D-GEN-4`).
- **Supporto `mbstring`:** È segnalato come "BLOCCATO" in backlog. Sfruttare l'ecosistema Rust (es. `encoding_rs` e stringhe mutabili) per fornire l'estensione senza dipendere dal binario Oracle locale.
- **Supporto Attributes e Reflection:** Costruire una semplice API di Reflection (esaminando l'HIR e le tabelle delle classi/funzioni salvate nell'Evaluator) permetterà di estrarre e usare gli attributi (es. `#[A(x: 1)]`), che sono al momento bloccati.
