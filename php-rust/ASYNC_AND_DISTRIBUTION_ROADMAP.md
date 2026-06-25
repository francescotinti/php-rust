# Roadmap Specifica: Il Futuro Asincrono e Single-Binary di PHP in Rust

Questo documento esplora tecnicamente come trasformare l'attuale VM in un runtime nativamente asincrono (capace di milioni di connessioni) e come impacchettarlo in un singolo binario autosufficiente, portando PHP nell'era di Deno e Go.

---

## 🌩️ Parte 1: Asincronia Nativa e Multi-Threading (Integrazione Tokio)

L'obiettivo è abbandonare il modello bloccante (I/O bound) storico di PHP. Poiché avete già implementato la sospensione dello stack tramite Fiber e Generatori (GEN-4) gestiti sull'heap, l'infrastruttura di base è pronta.

### Fase 1.1: Il Motore Asincrono (Tokio)
Invece di eseguire la VM su un thread nativo standard, il runtime andrà inizializzato all'interno di un event loop di `tokio`. 
- **Modifica Architetturale**: Il blocco `Vm::run()` dovrà diventare `async fn run()`, oppure dovrà potersi interfacciare con l'Executor di Tokio per cedere il controllo quando incontra I/O.

### Fase 1.2: Riscrivere l'I/O in chiave Non-Blocking
Oggi, se un built-in PHP come `fread()` o `file_get_contents()` legge da un socket, l'intero thread del sistema operativo si ferma.
- **La Soluzione Rust**: I built-in della libreria standard (`crates/php-builtins`) dovranno usare `tokio::net` e `tokio::fs`. 
- **Il Meccanismo**: Quando PHP chiama `stream_socket_recvfrom()`, il built-in Rust inizia un'operazione asincrona. Usa l'infrastruttura Fiber della VM per **parcheggiare il frame corrente** e cede il controllo a Tokio (`.await`). Il thread di Rust è ora libero di servire altre migliaia di richieste PHP. Quando il pacchetto di rete arriva, Tokio risveglia il Fiber e lo script PHP riprende esattamente da dove si era fermato, *senza che lo sviluppatore PHP debba scrivere codice asincrono*.

### Fase 1.3: Concorrenza Multi-Thread (Share-Nothing)
Sfruttando il paradigma *Shared-Nothing* di PHP (dove le variabili non sono condivise tra richieste), possiamo lanciare **migliaia di istanze della struct `Vm`** su diversi thread worker di Tokio.
- Invece di avere processi PHP-FPM da 30-50MB l'uno, avremo migliaia di struct `Vm` che pesano pochi KB in RAM. Questo è il segreto per reggere milioni di WebSockets concorrenti.

---

## 📦 Parte 2: Distribuzione Single Binary (L'Effetto Deno/Go)

L'obiettivo è eliminare PHP-FPM, Nginx, e i moduli `.so`. Lo sviluppatore scarica un singolo eseguibile `php-rust` ed ha tutto ciò che gli serve.

### Fase 2.1: Web Server Integrato (Hyper/Axum)
Il runtime binario includerà nativamente un web server ad altissime prestazioni scritto in Rust (es. `hyper` o `axum`).
- Eseguendo `php-rust serve public/index.php`, il binario apre la porta 8080.
- **Ciclo di Vita della Request**: Quando arriva una richiesta HTTP, Axum preleva un'istanza "calda" di `Vm` da un pool, popola gli array superglobali (`$_GET`, `$_SERVER`), ed esegue il bytecode. Finito lo script, la risposta viene inviata e la `Vm` viene riciclata (svuotando lo stato locale). Non c'è alcun overhead di boot del processo (FastCGI).

### Fase 2.2: Precompilazione in Memoria (Bytecode Caching)
Per polverizzare i tempi di latenza, il binario leggerà tutti i file `.php` all'avvio, li passerà attraverso `compile.rs`, e manterrà il `Module` (le istruzioni Bytecode) sempre residente in memoria RAM (l'equivalente di *OPcache* preload, ma nativo e immutabile). 
- Le richieste HTTP non faranno mai parsing del file system: salteranno direttamente all'esecuzione del bytecode.

### Fase 2.3: Inclusione Asset e Standard Library
Tramite macro Rust come `include_bytes!` o librerie come `rust-embed`, è possibile "iniettare" file `.php` nativi (la libreria standard, polyfill o interi framework) direttamente dentro il file binario al momento della compilazione (compile-time di Rust).
- Questo apre la strada a un comando `php-rust build my_app/`: il compilatore prende la tua app Laravel/Symfony, precompila il bytecode, lo fonde con il runtime Rust e sputa fuori un file `.exe` / ELF da 20MB. Lo metti sul server, lo avvii e regge 50.000 request/sec da solo.

---

## 🗺️ Roadmap Esecutiva (Come arrivarci)

Questa roadmap si incastra *dopo* aver completato la copertura del linguaggio (list, costanti, unpacking):

1. **Proof of Concept I/O**: Sostituire la chiamata bloccante in `file_get_contents` (in `php-builtins`) con una rudimentale implementazione asincrona agganciata a un mini loop Tokio, giusto per dimostrare che un Fiber PHP può aspettare la rete senza bloccare il thread Rust.
2. **Server HTTP Mock**: Scrivere un file `crates/php-cli/src/server.rs` che usa `hyper` per accettare una request HTTP semplice, iniettare "Hello World" in un mock di `$_GET` ed eseguire la VM, restituendo l'output come response HTTP.
3. **Pool di VM**: Strutturare un `VmPool` thread-safe. Il server HTTP pesca una VM libera per ogni richiesta concorrente.
4. **Standalone App Builder**: Creare la CLI che accetta l'argomento `build` e usa il crate `tar` o `include_dir` per inglobare una cartella utente in un binario finale distribuibile.

---

# Appendice (2026-06-25): stato reale del codice + correzione architetturale

> Audit del documento sopra contro il codice reale (`crates/`). La visione è giusta e la fondazione esiste davvero, ma **un punto va corretto** o si finisce in un vicolo cieco. Riferimento per quando arriverà il momento dell'async.

## A. Cosa è già presente / riciclabile (verificato nel codice)
- **Sospensione dei frame: ESISTE.** `enum RunExit { Returned, Yielded, Suspended }` (vm/mod.rs). Il dispatch loop sa già **parcheggiare un frame sull'heap** (`self.generators`/`self.fibers` in vm/coroutines.rs) e restituire `Suspended` al chiamante. È **esattamente** il primitivo per l'I/O async (vedi C). Fondazione genuina.
- **Server HTTP: già abbozzato.** `crates/php-server` (axum + tokio "full"), `src/main.rs`. MA è un mock: **ricompila il file a ogni request** (`run_source_with` = lower+compile+run) dentro `tokio::task::spawn_blocking` (un thread bloccante per request). Niente preload, niente fiber-async.
- **Preload bytecode + single-binary (Parte 2): validi, costo basso.** Il `Module` è già immutabile e riusabile → preload banale. `rust-embed`/`include_dir` per embeddare prelude/stdlib/app sono win immediati.

## B. L'errore di fondo da correggere (il punto cruciale)
Il documento propone «`Vm::run()` diventa `async fn`» e «migliaia di `Vm` su worker thread di Tokio». **Non funziona così**, per un vincolo ignorato:

→ **`Zval` usa `Rc<RefCell<...>>`** (php-types/src/zval.rs) ⇒ `Vm`/`Zval` sono **`!Send`**.
- Non puoi fare `async fn run()` con `.await` che attraversa thread (il future conterrebbe `Rc` → non `Send` → non compila su runtime work-stealing).
- Non puoi spostare una `Vm` tra i worker thread di Tokio (default = work-stealing).

Due vie reali:
- **(A) THREAD-PER-CORE** ✅ — un runtime Tokio `current_thread` **per thread OS**, ogni `Vm` *inchiodata* al suo thread, mai migrata. Gli `Rc` restano validi (single-thread per runtime). È il modello dei server Rust ad altissime prestazioni (glommio/monoio, arbiter di actix). **Preserva tutto il design `Rc` — nessun rewrite.**
- (B) Rewrite a `Arc<Mutex>` ❌ — enorme, uccide le performance single-thread, combatte il design. Da evitare.

## C. Il pattern async corretto (NON `async fn run`)
La VM resta **sincrona**. Un builtin di I/O:
1. avvia l'operazione, registra un future/waker legato al fiber corrente,
2. parcheggia il frame e ritorna `RunExit::Suspended`.
Un **driver async ESTERNO** (stesso thread, runtime `current_thread`) fa l'`.await` sul future e poi **risveglia il fiber** col risultato. Lo sviluppatore PHP scrive codice sincrono e ottiene async "gratis" — ma il meccanismo è questo (suspend+resume sui frame), NON rendere async il dispatch loop.

## D. Roadmap corretta (ancorata al codice)
| # | Cosa | Costo | Trigger / Note |
|---|---|---|---|
| 0 | Copertura linguaggio (include/eval/autoload) | in corso | Prerequisito: far girare app vere. Vedi [[php-rust-include-eval-handoff]]. |
| 1 | **Preload bytecode** in php-server | basso | Compila i `.php` UNA volta all'avvio, `Module` residenti; request → `run_module`. Oggi ricompila ogni volta. |
| 2 | **Single-binary** (rust-embed/include_dir) | basso | Prelude/stdlib (+app) embeddati; `php-rust build`. |
| 3 | **VmPool** (Vm calde riusate) | medio | La `Vm` va RESETTATA tra request (frames/statics/globals/output): share-nothing = svuotare stato locale. Tenere la `Vm` resettabile è l'unica cosa da non rompere fin da ora. |
| 4 | **I/O async** = thread-per-core + driver su `RunExit::Suspended` | ALTO | Richiede PRIMA il layer di rete/socket (oggi inesistente — vedi EXTENSIONS_ARCHITECTURE.md Appendice A) + reactor per-thread. Riusa la macchina Fiber. |
| 5 | **Cycle GC** (cycle collector) | ALTO | Gemello dell'async: server long-running ⇒ `Rc` non libera i cicli ⇒ leak. Prerequisito per un vero server persistente. |

## E. QUANDO introdurre tutto questo (i trigger, non le date)
- **ORA**: NIENTE async/server. Si fa solo copertura linguaggio. *Unica regola preventiva*: mantenere la `Vm` **resettabile** e non aumentare l'entanglement `Rc` inutilmente, per non chiudere la porta a thread-per-core/VmPool. Costo zero, va solo tenuto a mente.
- **Trigger Step 1-2 (preload + single-binary)**: quando un'app reale gira end-to-end — cioè DOPO include/require/autoload (Fasi 2-3) e una rotta "hello world" di un micro-framework / app multi-file. Prima non c'è nulla che valga la pena impacchettare. È il primo momento "demo al mondo".
- **Trigger Step 3 (VmPool)**: quando si fanno benchmark di throughput reali sul server e il boot-per-request diventa il collo di bottiglia. Richiede il preload (Step 1) prima.
- **Trigger Step 4 (I/O async)**: SOLO dopo che (a) esiste il layer di rete/socket E (b) ci sono builtin I/O bloccanti reali che ne beneficiano (driver DB, client HTTP) — cioè dopo la roadmap estensioni Tier 2/3. È il pezzo PIÙ tardivo: fare l'async prima di avere I/O da rendere async è lavoro sprecato.
- **Trigger Step 5 (Cycle GC)**: quando un processo server sotto carico sostenuto mostra crescita di memoria (leak osservabile), o quando si fa girare un framework reale che crea cicli. Differibile finché un workload reale non lo espone, ma è il gate per un server persistente "production".

**In sintesi sul timing**: l'asincronia è l'ULTIMO grande pezzo, non il prossimo. L'ordine di valore è: *linguaggio (include/eval/autoload) → estensioni I/O (rete, DB) → preload/single-binary per impacchettare → POI async I/O + cycle GC*. Anticiparlo significherebbe ottimizzare l'attesa di I/O che ancora non esistono. La cosa giusta da fare oggi è solo non chiudere le porte (Vm resettabile, design Rc intatto: thread-per-core lo ama).
