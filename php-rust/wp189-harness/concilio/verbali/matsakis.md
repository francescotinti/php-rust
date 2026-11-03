# Verbale sedia MATSAKIS — S-189 (bozza indipendente, FASE 1)

Generato con assistenza AI (Claude Fable 5).
Lente: ownership / aliasing / Send+Sync — cosa attraversa il confine di thread, cosa resta thread-affine, cosa il type system
prova DOPO la cura. Letto: dossier §1-8; `worker_pool.rs` 1-60, 119-175, 175-330, 330-760, 760-830, 1795-1921, 2024-2067;
`axum_web.rs` 1-60, 110-160, 500-522; `sapi.rs` 1-100; `main.rs` 240-440, 440-580; `COUNCIL_WP78_REVIEWS.md` 35-70, 125-135;
`measure90-campaign.sh` 288-310.

## VERDETTI

**Q1 ROTTA — CONCORDO CON EMENDAMENTI su (A).** (B) richiede un flag «occupato» per worker letto dal dispatcher: stato pubblicato
cross-thread con TOCTOU (due dispatch vedono lo stesso worker libero, uno aspetta mentre un altro worker è libero) — introduce
aliasing di stato che oggi NON esiste ed è più debole dell'accept condiviso. (C) lascia il difetto in produzione. (A) è l'unica
forma in cui il type system continua a provare tutto ciò che prova oggi (R1-R4) e in più prova «un task per worker» nel census
(R6). Emendamenti vincolanti: R2 (bound e controllo negativo), R3 (niente statics), R5 (critical section vuota), R6 (barriera
per-pool con scadenza).

**Q2 CONTRATTO DI MISURA — CONCORDO con (i), precisato.** Il contatore `dispatched` resta locale al thread (`u64` sullo stack di
`worker_loop`, 515/649, consegnato al teardown 403): nessuna condivisione nuova. Invariante emendato: l'INSIEME delle righe
`tag=worker_dispatch thr=` resta ESATTO = {0..W-1} (ogni thread emette la sua riga al teardown, 412-415/447-451, a prescindere dal
conteggio); i CONTEGGI sono ⊆ ℕ con Σ == nreq; a W=1 il singolo count == nreq resta esatto. (ii) knob `rr`: MI OPPONGO — due vie
al cap, e un knob che reintroduce l'indice `next_worker` riporta il ghost A-PP37. Barriera `__census_self`: accettata SOLO
per-pool (`Arc`, non `OnceLock` statico) e con scadenza (R6); un worker fermo in `pop_blocking` non ascolta un secondo canale,
quindi la barriera è l'unico mezzo che PROVA «W task → W thread distinti» (il worker che ha preso un task census è bloccato
alla barriera e non può prenderne un secondo). Lettere 4/8/9/10/11: DA EMENDARE come in R7; lettere 1-3, 5-7: INVARIATE.

**Q3 FORMA E GATE.** Dente gate-loopback: basta come accettazione DEL DIFETTO, non della cura: vanno aggiunti i due controlli di
tipo (R2) e il test «W task census → W `thr` distinti» (R6). Soglia throughput: C=4/C=8 ≥ 0,97× accettata; C=1 a W=4 NON è
confrontabile 1:1 col pin vecchio (R8: regime di località della unit-cache diverso) → ±5 % a C=1 ADVISORY, oppure braccio C=1 a
W=1 (dove FIFO ≡ round-robin per costruzione). «Bounded + 503» WP-77.5: SUPERATO (la VecDeque unbounded eredita la stessa
proprietà dell'`UnboundedSender` odierno, 380: nessuna regressione di memoria; il backlog del kernel rifiuta a livello TCP, non
503) → apertura per NOME «backpressure del pool» in §3.9, nessuna implementazione ora. Timeout nel `rx.await` (432): NO —
fedeltà, e l'ownership già regge la cancellazione: il client che chiude fa droppare il future e il `Receiver`; il worker fa
`send(response).ok()` (620) su Sender orfano, nessun leak, nessun panic (INVARIATO, da dichiarare nel verbale di sintesi).

## ROTTA (una riga)
(A) coda condivisa in `dispatch.rs` con `T: Send` esplicito e controllo negativo, `Arc` per-pool per coda E barriera (zero
statics), critical section = sole operazioni VecDeque, barriera census con pre-check `OUTSTANDING==0` e scadenza; righe 543-648
intatte.

## REFUTAZIONI ED EMENDAMENTI

- **R1 (INVARIATA) `WebRequest: Send` è reale, non forzato.** `sapi.rs` 19-50: soli `Vec<u8>`, `String`, `u16`, `f64`,
  `Option<Vec<u8>>`, tuple — nessun `Rc`, nessun `unsafe impl`. Il PRIMO avvolgimento in `Rc` avviene sul worker
  (`axum_web.rs` 133 `Rc::new(w)` dentro `RequestScope::install`, chiamato a `worker_pool.rs` 575 DOPO il pickup); il
  thread-local `WEB_REQUEST: RefCell<Option<Rc<WebRequest>>>` (`sapi.rs` 52-53) non è toccato dalla cura. L'assert positivo
  170 copre già `Option<WebRequest>`.
- **R2 Bound da scrivere, non da sperare.** `Mutex<State<T>>: Sync ⇔ T: Send`; `Condvar: Send+Sync`; quindi
  `Arc<SharedQueue<T>>: Send ⇔ T: Send` (nessun `T: Sync` richiesto). Emendamento: `impl<T: Send> SharedQueue<T>` esplicito;
  `assert_impl_all!(SharedQueue<WorkerTask>: Send, Sync)` COME PROPOSTO ma NON al posto di 170 (A-MS3 resta); CONTROLLO
  NEGATIVO obbligatorio `assert_not_impl_any!(SharedQueue<std::rc::Rc<()>>: Send, Sync)` (lezione WP-72: un contatore
  tutto-positivo è un controllo fallito). Zero `unsafe impl` in `dispatch.rs`.
- **R3 `OnceLock<Barrier>` statico: REFUTATO.** Il binario di test costruisce PIÙ pool nello stesso processo
  (`WorkerPool::new(ctx, 1)` 1805/1845, `new(ctx, 2)` 2032, `axum_web.rs` 502), in thread paralleli: il primo pool semina W nel
  `OnceLock`, un pool diverso eredita una `Barrier(W)` sbagliata → deadlock (attesa di un partecipante inesistente) o passaggio
  anticipato. In produzione il pool è uno (`main.rs` 507) ma il type system non lo prova. Emendamento: campo
  `census_barrier: Arc<_>` costruito in `new` con `num` e clonato nella closure 389 (la `Barrier` è `Send+Sync`); stessa
  regola per la coda: `Arc<SharedQueue>` per-pool, MAI `static`/`&'static` (`Box::leak` farebbe trapelare una coda per pool
  di test). Il `worker_count()` (349-351) diventa `self.handles.len()`: nessun secondo contatore `workers` che possa divergere.
- **R4 Prova «ricevuto una volta sola»: non si perde, cambia garante.** Oggi `mut rx: UnboundedReceiver` è MOSSA nella closure
  (506): il tipo prova che un solo thread riceve da quel canale. Con `Mutex<VecDeque<T>>` il garante è `pop_front` sotto lock:
  il task è MOSSO fuori, due thread non possono estrarre lo stesso elemento. Ciò che si perde davvero è solo l'AFFINITÀ
  (task i → worker i mod W), che non era mai stata una prova di tipo ma un `AtomicUsize` (337, 691-694). Accettato.
- **R5 Lock avvelenato: `into_inner` è tollerabile SOLO se la critical section è vuota di codice utente.** Con A-PP9 (469-474)
  nessun panic di worker sopravvive, quindi un lock avvelenato è irraggiungibile; ma la proprietà regge solo se sotto il lock
  stanno SOLO `push_back`/`pop_front`/flag `closed`/`len` — mai `response_tx.send`, mai una chiamata in `php_runtime`, mai il
  `Barrier::wait`. Emendamento: `pop_blocking` restituisce il task DOPO aver rilasciato il guard; `wait` della barriera fuori
  da ogni lock (il ramo 588-604 gira già a guard rilasciato, 542 è fuori dal loop del Condvar).
- **R6 Barriera census senza scadenza = l'istrumento può stallare il pool.** Se un worker è bloccato in un loopback la cui
  richiesta interna è in coda DIETRO i W task census, i W−1 worker liberi aspettano alla barriera il worker bloccato, che
  aspetta una richiesta che nessuno può prendere → stallo fino al timeout PHP (identico al difetto che si cura, ma causato
  dalla sonda). La precondizione «pool quiescente» (293-295 dello script) è prosa. Emendamento: il handler
  (`main.rs` 396-409) PRIMA di spedire i W task legge `census_outstanding_now()==0` (318-320, già Acquire) e rifiuta con corpo
  `census-self REFUSED outstanding=N` (riga VOID per il giudice); la barriera usa `Condvar::wait_timeout` (5 s) su contatore
  per-pool, non `std::sync::Barrier` (senza timeout); scaduta → `census-self thr=k TIMEOUT` + abort come A-MS31 (istrumento,
  non misurando). Il test unitario: W=3, tre task census in una spedizione → tre `thr` distinti (prova positiva di R4+R6).
- **R7 Lettere.** A-PP36/KS-PP-87-3: «atteso» = righe {0..W-1} esatte ∧ Σcount == nreq. A-PP37: il ghost `next_worker`
  SPARISCE col campo (724-729 da riscrivere: una `push` fallita è solo «coda chiusa», undo census 736-741 INVARIATO); la mappa
  per-thread a W>1 è ADVISORY. A-PP42/KS-PP-89-2: VDISP = righe esatte {0..W-1}, count==nreq/W SOLO a W=1. KH78-2/KB-78-2:
  «profondità della coda condivisa >1 ⇒ VOID» (a W=1 identica a oggi: `census_queue_depth_burst_positive` 1801 e
  `no_underflow` 1841 restano verdi INVARIATI). Lettera 11: riscritta come in R6. `two_workers_no_request_bleed` (500-522):
  resta VERO (il bleed è escluso dall'RAII di `RequestScope`, non dall'affinità) ma il commento 514 «on the same workers» va
  emendato: senza round-robin la copertura di ENTRAMBI i worker è probabilistica — dichiararlo, non riscrivere il test.
- **R8 Località della unit-cache (thread_local) cambia regime, non sicurezza.** `RetainSet` nasce sullo stack del worker
  (768) e `UNIT_CACHE` resta per-thread: nessun attraversamento nuovo. Ma `notify_one` sveglia un waiter ARBITRARIO: un client
  sequenziale a W=4 può colpire sempre lo stesso worker (una cache calda) o ruotare (quattro cache da scaldare) — a differenza
  del round-robin deterministico. Zero impatto sul census (W=1, KB-78-3); impatto dichiarato sull'arm C=1 del throughput (Q3).
- **R9 `Arc<WorkerPool>` e il join di uscita.** `main.rs` 573-577 esige `Arc::try_unwrap(pool)` riuscito per chiamare
  `shutdown()`. I worker devono catturare `Arc<SharedQueue>` (e `Arc` della barriera), MAI `Arc<WorkerPool>`: altrimenti il
  try_unwrap fallisce per sempre e le cifre di uscita restano ADVISORY in silenzio. `shutdown` (659-661): `queue.close()` al
  posto di `drop(senders)`, poi join INVARIATO; `shutdown_joins_all_workers_after_drain` (2029) è il controllo meccanico.

## KILL-SWITCH

| KS | condizione | atto |
|---|---|---|
| KS-MS-189-1 | `dispatch.rs` senza `impl<T: Send>` esplicito, o con un `unsafe impl Send/Sync` | build REJECT, nessun pin |
| KS-MS-189-2 | manca `assert_not_impl_any!(SharedQueue<Rc<()>>: Send, Sync)` o sparisce `assert_impl_all!(WorkerTask: Send)` (170) | A-MS3 violata, merge REJECT |
| KS-MS-189-3 | qualunque `static`/`OnceLock`/`Box::leak` che porti W, coda o barriera | REJECT (più pool per binario di test: 1805/1845/2032/`axum_web.rs` 502) |
| KS-MS-189-4 | la closure del worker cattura `Arc<WorkerPool>` (`try_unwrap` a `main.rs` 573 impossibile) | `shutdown_joins_all_workers_after_drain` rosso ⇒ HALT |
| KS-MS-189-5 | sotto il lock della coda compare una chiamata fuori da VecDeque/flag (`send`, `php_runtime`, `wait`) | REJECT |
| KS-MS-189-6 | `__census_self` senza pre-check `OUTSTANDING==0` o con barriera senza scadenza | righe census VOID, nessuna promozione |
| KS-MS-189-7 | righe `worker_dispatch` ≠ {0..W-1} o Σcount ≠ nreq (W>1), count ≠ nreq (W=1) | run VOID (A-PP36/A-PP42 emendate) |
| KS-MS-189-8 | ±5 % a C=1/W=4 trattato come verdetto senza braccio C=1/W=1 | cifra C=1 ADVISORY, non entra nel record |
