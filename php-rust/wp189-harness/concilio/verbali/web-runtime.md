# Verbale sedia cooptata WEB-RUNTIME — S-189 (bozza indipendente, FASE 1)

Generato con assistenza AI (Claude Fable 5). Lente: accept loop, backpressure, shutdown, timeout, worker bloccanti dietro un fronte async.
Letto: dossier intero; `main.rs` 208-714, `worker_pool.rs` 325-760, `axum_web.rs` 46, 155-320, `Cargo.toml`; lock: axum 0.7.9, hyper 1.11.0,
hyper-util 0.1.20, tokio 1.53.1; WP-77.5 §18-33/229-245; `GIUDIZIO_C_AXUM.md` 1-40; `s188-throughput.sh` 34, 92-104.

## VERDETTI
- **Q1 — CONCORDO CON EMENDAMENTI su (A)**. La coda condivisa è l'unico disegno che riproduce l'osservabile dell'oracle (backlog = coda, worker
  occupato non accetta). (B) è refutata: un flag «libero» letto fuori dal lock è TOCTOU e sotto saturazione degrada al round-robin cieco, cioè al
  difetto stesso. (C) è refutata: il difetto è di produzione (dossier §2 B/C: revert di modifiche e rollback di aggiornamenti).
- **Q2 — CONCORDO su (i)** con un termine in più: l'invariante è Σcount == nreq + orfani (R4), insieme ⊆ {0..W-1}, mappa W>1 ADVISORY, census solo
  a W=1. (ii) knob `rr` respinto (due vie al cap). Barriera `__census_self`: accettata SOLO con rendezvous a timeout (R5), mai `std::sync::Barrier`.
- **Q3 — CONCORDO CON EMENDAMENTI**: il dente `gate-loopback` basta come accettazione se dichiara l'esito PER richiesta (R6) e legge gli orfani
  (R4). Throughput: rimisurare (R7), non «per costruzione». Timeout nel `rx.await`: **NO** (R2). «Bounded + 503»: **superato nella forma 503**,
  riaperto per nome come backpressure-ad-attesa fuori scopo S-189 (R3).

## ROTTA (una riga)
GO su (A): `SharedQueue` Mutex+Condvar senza handle tokio, nessun timeout nel handler, contatore orfani in-band, rendezvous census a timeout, dente
per-richiesta + rimisura throughput R=3; «503» chiuso per nome, backpressure-ad-attesa e §3.39 aperti per nome.

## REFUTAZIONI ED EMENDAMENTI
**R1 — Ordine di dispatch ≠ ordine di accept: dichiararlo, non è un difetto.** `main.rs:700` costruisce `Runtime::new()` = flavour multi_thread
(N = CPU); `main.rs:560` `axum::serve` accetta in un task e spawna un task per connessione; il handler fa `route(..).await` (`axum_web.rs:194`) che
include `to_bytes(body).await` (`:234`) PRIMA di `dispatch` (`main.rs:426`). Due connessioni accettate c1<c2 possono quindi dispacciare c2<c1
(POST con body in volo perde la gara contro un GET). L'oracle serve nell'ordine del backlog. Finestra di riordino: µs/ms di parsing, mai i 300 ms
del dente ⇒ la coreografia A/B del controllo positivo (4 dispatch mod 3, invariante all'offset dei probe precedenti) NON è toccata; il contratto
di misura deve però scrivere «FIFO di dispatch, non di accept». Keep-alive: hyper h1 serve le richieste di una connessione in sequenza, ogni
chiamata del service = un `dispatch` separato: una connessione keep-alive NON pinna un worker (nell'oracle a fork il processo che accetta tiene la
connessione per la sua vita). `s188-throughput.sh:92-94` usa `ab` senza `-k`: equivalenza salva; `-k` su un solo lato ⇒ confronto VOID (KW-189-3).

**R2 — Timeout nel handler: NO.** `main.rs:432` `rx.await` senza timeout è fedele: `php -S` non ha timeout del server, governa il lato PHP
(`CURLOPT_TIMEOUT`, `default_socket_timeout`, `max_execution_time` §3.38). Un `tokio::time::timeout` produrrebbe un 5xx che PHP non produce e NON
libera nulla: il worker è un thread OS che non si cancella, il task continua, la risposta finirebbe orfana. Invenzione pura, respinta. Il vero
buco è a monte: il wrapper `http://` senza `default_socket_timeout` (dossier §1, §3.39): un loopback con `file_get_contents` verso un worker occupato
blocca PER SEMPRE (PHP: 60 s) e a W=1 il server è morto definitivamente. Fuori delibera ma da aprire per nome PRIMA della promozione della cura
(KW-189-4): la coda condivisa riduce la probabilità dello stallo a W>1, non la sua durata.

**R3 — «Bounded + 503» (WP-77.5 W-77.5.2) è superato nella forma, non nel merito.** Onestà: hyper/axum NON limitano le connessioni (`axum::serve`
0.7 accetta finché ci sono fd; `tower`/`ConcurrencyLimit` non sono montati, `main.rs:527-542`); l'unica bound oggi è il body cap 512 MiB
(`axum_web.rs:46`) PER task già in coda. La coda `SharedQueue` resta unbounded: sotto pool lento la memoria cresce di (body+header) per richiesta —
peggio di oggi? No: oggi le W code unbounded sommano uguale; non è una regressione, è un debito ereditato. L'oracle ha un backlog BOUNDED
(`listen(SOMAXCONN)`) la cui saturazione si osserva come ATTESA/ritrasmissione SYN, mai come 503: un 503 è un'invenzione. Via corretta e fedele:
backpressure-ad-attesa = `tokio::sync::Semaphore(K)` acquisito all'INGRESSO del handler (prima di `route`, così il body non viene letto in attesa),
K = 64·W default, knob `--queue-cap`; a C=8/W=4 (in volo ≤ 8 ≪ 256) non scatta mai. Non è precondizione di S-189: apertura per nome (§3.9 nuova
lettera), «503» dichiarato chiuso nel verbale di sintesi.

**R4 — Disconnessione del client durante lo stallo: esecuzione dell'orfano = fedeltà, silenzio = no.** hyper 1.x h1 rileva l'EOF a metà messaggio
e chiude la connessione: il future del handler viene droppato e con lui `rx`; il worker esegue comunque il task (anche l'oracle esegue lo script
fino alla prima scrittura: `ignore_user_abort` semantics) e `task.response_tx.send(response).ok()` (`worker_pool.rs:620`) inghiotte l'`Err`.
Lavoro sprecato accettabile; il silenzio no: è esattamente il «dispatch fantasma» di A-PP36/KS-PP-87-3 (un dispatch senza riga di risposta). Emenda:
`static ORPHANED: AtomicU64` incrementato sull'`Err` di `send` (Relaxed, nessun I/O nella finestra, WP-64) e stampato come `orphan=<n>` sulla riga
`tag=worker_dispatch` di teardown (`worker_pool.rs:413-415, 447-451`) e in `shutdown()` (`:670`). Invariante Q2: Σcount == nreq + orphan.
Nel dente a W=1 l'orfano atteso è esattamente 1 (l'interna di A); a W=3 col pin nuovo DEVE essere 0 (KW-189-1).

**R5 — Barriera `__census_self`: `std::sync::Barrier` è vietata, serve un rendezvous a timeout.** Precondizione «pool quiescente» (`main.rs:293-295`)
è una dichiarazione, non una prova: se un worker è bloccato (loopback, §3.39) W−1 worker si parcheggiano in `wait()` senza timeout, il handler
attende W oneshot senza timeout (R2) e il processo è un deadlock SILENZIOSO — lo stato che KS-PP-6 bandisce. Emenda: Mutex+Condvar con
`wait_timeout(5 s)`; allo scadere la sonda fa `panic!` DENTRO la `ProbeWindow` (`worker_pool.rs:589-603`) ⇒ abort attribuito allo strumento
(A-MS31/A-MS34), rc≠0, campagna VOID per costruzione (KW-189-2). La sola campagna census resta a W=1 (KB-78-3): a W=1 il rendezvous è un no-op.

**R6 — Il dente deve dichiarare l'esito per richiesta.** Dossier §4 «entrambi W=1: ERR28» è ambiguo: a W=1 (pin vecchio, nuovo e oracle senza
workers) A = ERR28 ∈ [2500;4000] ms e B = OK (B entra in coda dietro A, ~3,3 s, poi la sua interna trova il worker libero); a W=3 pin nuovo e oracle
a 3 worker A = OK e B = OK < 1000 ms; a W=3 pin vecchio A = ERR28, B = OK. Un parser che legga «ERR28 ovunque» a W=1 dichiara PASS un braccio
in cui anche B fallisce (sintomo di un bug diverso). Tabella attesa per braccio × richiesta nel criterio, verdetto fail-closed su ogni cella.

**R7 — Throughput: rimisurare, R=3, soglia 0,97 come proposta.** A W=1 la coda condivisa è identica per costruzione (A/P resta 1,00 senza rimisura:
concordo). A W=4 il cammino cambia primitiva: mpsc lock-free + wakeup per-worker → Mutex+Condvar con `notify_one` (un lock per push e per pop, µs
contro ~10 ms per richiesta: atteso invisibile). Atteso ≠ misurato: KS-189-2 resta. Due osservabili collaterali da DICHIARARE: (a) `notify_one`
sveglia un waiter arbitrario — a C=1 la cache unit thread-local si scalda su un solo worker e gli altri restano freddi (prima richiesta su un worker
freddo paga la compile), a C=4/8 tutti lavorano: la riga `count=` per thread può essere 0 per un worker a basso carico, coerente con ⊆ {0..W-1};
(b) il mutex condiviso è un punto di contesa di W+N thread: se a C=8 la mediana cala > 3 % su una URL, la prima ipotesi è il lock, non il motore.

**R8 — Shutdown: `pool.shutdown` È chiamato su SIGTERM, ma il drain è illimitato.** Percorso letto: segnale → `with_graceful_shutdown`
(`main.rs:549-562`) → axum 0.7.9 chiude il listener e attende la fine di TUTTE le connessioni in volo → `serve` ritorna → `Arc::try_unwrap(pool)` →
`shutdown()` (`:573-574`) → `drop(senders)`/`queue.close()` → drain → join (`worker_pool.rs:659-669`). Corretto e invariato col candidato (`close()`
+ `pop_blocking` «drena poi esce»). Rilievo fuori scopo: una richiesta in stallo tiene il processo vivo fino al timeout lato PHP (100 s flusso B,
per sempre con §3.39) mentre l'oracle su SIGTERM esce subito; `s188-throughput.sh:100-102` attende 30 s poi SIGKILL e marca PEAK-ADVISORY: giusto,
il dente gate-loopback deve avere lo stesso cap (KW-189-5). Con la coda condivisa gli orfani in coda alla chiusura vengono eseguiti nel drain:
costo ≤ 1 pong per orfano, accettabile, conteggiato da R4.

**R9 — Worker thread nudi e `std::sync::Condvar`: confermato, nessun handle tokio.** I worker sono `std::thread::spawn` (`worker_pool.rs:389`) fuori
dal runtime; oggi `blocking_recv()` (`:542`) funziona solo perché il thread NON è dentro un contesto tokio (panica altrimenti). Mutex+Condvar std
elimina anche quel vincolo implicito. Tre vincoli da pinnare nel test di `dispatch.rs`: (1) `pop_blocking` cicla sul predicato (spurious wakeup);
(2) il handler tokio fa `push` sotto un Mutex std: ammesso perché O(1) e mai `.await` sotto lock — un `assert`/commento vincolante nel file;
(3) poisoning: `unwrap_or_else(into_inner)` è coerente con l'abort hook (A-PP9) ma il test «panic di un consumer non avvelena gli altri» va scritto
senza `sleep`. `with_connect_info` (`main.rs:560`): invariato, il peer delle interne è 127.0.0.1:effimera, coerente con `$_SERVER` dell'oracle.

**Lettere (dossier §6)**: 1 DA EMENDARE («503» chiuso, backpressure-ad-attesa aperta, R3); 4 RI-DEFINITA (una coda, depth = len della coda);
8/9/10 DA EMENDARE con Σ == nreq + orphan e ⊆ {0..W-1} (R4); 11 RI-DISEGNATA con timeout (R5); 2/3/5/6/7 INVARIATE.

## KILL-SWITCH
| # | condizione (meccanica) | atto |
|---|---|---|
| KW-189-1 | dente gate-loopback, pin NUOVO, W=3: `orphan>0` su qualunque worker in una delle 3 ripetizioni | cura NON promossa (lo stallo è avvenuto anche se A/B leggono OK) |
| KW-189-2 | rendezvous `__census_self` scade (5 s) ⇒ abort rc≠0; oppure una campagna census emette righe `peak_self` con rc=0 ma `thr` distinti < W | campagna VOID; mai verdetto da un self-census parziale |
| KW-189-3 | `ab -k` (o qualsiasi keep-alive) presente su UN solo lato del confronto O4/AX | confronto throughput VOID (coda per-connessione ≠ coda per-richiesta) |
| KW-189-4 | §3.39 (`default_socket_timeout` sul wrapper `http://`) non aperta per nome nel NEXT alla promozione | promozione sospesa: la cura cambia la probabilità dello stallo, non la sua durata infinita |
| KW-189-5 | server del dente non uscito 30 s dopo SIGTERM | SIGKILL + braccio marcato `shutdown-hang` (registrato, non VOID); due bracci così ⇒ incidente, riconvocazione |
| KW-189-6 | test di `dispatch.rs` o del dente che usa `sleep` come sincronizzazione, o `std::sync::Barrier` nella sonda | test respinto in review (determinismo, KS-PP-6) |
