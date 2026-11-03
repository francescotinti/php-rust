# Verbale sedia LEIJEN — S-189 (bozza indipendente, FASE 1)

Generato con assistenza AI (Claude Fable 5).
Lente: allocatore (mimalloc, heap thread-local) e footprint fisico. Letti: dossier intero; `worker_pool.rs` 119-175, 185-303,
400-486, 504-650, 659-746; `axum_web.rs` 1-60, 202-236; `main.rs` 314-369, 389-440, 469-575; `COUNCIL_WP81_REVIEWS.md` (A-BB17/KB-81-1);
`design78.md` 100-160; `WP_SESSION_188.md` 11, 17 (peak 334 · 335-349 MiB misurato sul braccio A = `--axum --workers 1`).

## VERDETTI

- **Q1 ROTTA: CONCORDO con (A)**, coda condivisa FIFO. Dal lato heap la cura è NEUTRA: non cambia chi alloca né chi libera
  (R1), e il contenitore della coda è più piccolo di oggi (R2). (B) non ha alcun vantaggio di footprint (tiene W canali tokio) e
  resta cieca sotto saturazione; (C) lascia in produzione un difetto che la mia lente non può assolvere.
- **Q2 CONTRATTO: CONCORDO CON EMENDAMENTI, via (i)**: Σcount == nreq e insieme ⊆ {0..W-1}; mappa per-thread ADVISORY a W>1;
  W=1 resta l'UNICO modo census (KB-78-3, A-BB17/KB-81-1 INVARIATE: a W=1 FIFO condivisa ≡ canale singolo, bit per bit — R3).
  Barriera per `__census_self`: ACCETTATA SOLO con la precondizione di quiescenza resa MECCANICA nel handler (R4: senza di essa
  la barriera crea un deadlock che la sola FIFO non ha). Knob `rr`: NO (nessun valore per il census, che è a W=1).
- **Q3 FORMA E GATE: CONCORDO CON EMENDAMENTI**: il dente `gate-loopback` basta come accettazione se legge anche la mappa
  `tag=worker_dispatch` a teardown (R5). Throughput: rimisurare (la soglia 0,97 è giusta; il lock per dispatch è trascurabile
  contro 115-351 ms CPU/req, ma «trascurabile» si misura, non si dichiara). Footprint: NON rimisurare (R6). «Bounded + 503»
  del WP-77.5: SUPERATO per la lunghezza della coda, ma la vera divergenza di footprint sta altrove e va aperta PER NOME
  (R7). Timeout nel `rx.await`: NO (nessun argomento di footprint, R8). Il gate NON registra il peak (R6).

## ROTTA (una riga)

GO su (A): heap-neutra per costruzione; barriera census solo con rifiuto meccanico a `outstanding≠0`; gate legge la mappa dispatch; il residuo di footprint vero (corpo letto prima del dispatch) va nel catalogo §3.9, non in questa cura.

## REFUTAZIONI ED EMENDAMENTI

- **R1 — «La coda condivisa cambia chi libera cosa»: REFUTATA.** Oggi: il `WorkerTask` nasce sul thread tokio (`main.rs:424`;
  `meta.source` da `std::fs::read` in `axum_web.rs:222`, il corpo da `to_bytes(...).to_vec()` in `axum_web.rs:233-234`,
  DUE copie momentanee sul thread tokio) e muore sul worker (`task.web.take()` 575, drop di `meta` a fine iterazione 648):
  free cross-thread. La risposta `Vec<u8>` nasce sul worker (`Vm::rendered`), attraversa la oneshot (620) e muore sul task
  axum dopo la scrittura socket: free cross-thread già DICHIARATA (A-PP-69, `main.rs:353-356`). In mimalloc un free
  cross-thread è un push atomico sulla `xthread_free` della pagina: la memoria resta ATTRIBUITA all'heap del thread che ha
  allocato finché quello non la ricicla (prossima alloc da quella pagina o `mi_collect`). Con la coda condivisa l'insieme
  {allocatore, liberatore} di OGNI oggetto per-richiesta è IDENTICO: solo il contenitore cambia. Lettere 2 e 7 INVARIATE.
- **R2 — Contenitore della coda: saldo ≤ 0.** Oggi W canali `mpsc::unbounded` con blocchi da 32 slot × sizeof(WorkerTask)
  (`worker_pool.rs:380`), allocati dal produttore e liberati/riciclati dal consumatore (cross-thread). Domani UN `VecDeque`
  la cui capacità cresce fino al picco della coda (≈ C) e non torna indietro: a C=8 sono 8-16 slot, meno di un solo blocco
  tokio. `Mutex`/`Condvar` std non allocano per operazione. Emendamento: `SharedQueue::new` con `VecDeque::with_capacity(W)`
  e NESSUN `shrink_to_fit` (una capacità stabile è un `resid` stabile a W=1, A-BB11).
- **R3 — Attribuzione per-thread del mem-census: SOPRAVVIVE.** `memcensus_unitcache_main_rows(worker_idx)` e
  `mi_bin_thread_rows(worker_idx)` corrono SUL worker (`worker_pool.rs:452,458,595`) e leggono l'heap di default di quel
  thread: la coda non li tocca. A W=1 un produttore e un consumatore su una FIFO ≡ il canale di oggi; `LAST_S3`/`resid`
  (`worker_pool.rs:279-291`) vedono gli stessi contatori globali. Unica differenza dichiarabile: l'allocazione one-shot del
  deque al posto del primo blocco tokio, entrambe nel warm-up (resid(req1), A-BB19). A W>1 cambia QUALE worker si scalda:
  `notify_one` sveglia un thread in ordine non specificato ⇒ sotto carico sequenziale il lavoro si concentra su 1-2 worker
  «caldi» (unit cache e theap già popolati) invece di scaldarne W come il round-robin. Conseguenza: il budget ×W di
  A-DL31 diventa un LIMITE SUPERIORE (raggiunto solo a saturazione C ≥ W), non più il valore atteso. Da scrivere nella
  lettera A-DL31, non da misurare in S-189.
- **R4 — Barriera `__census_self` senza quiescenza meccanica = DEADLOCK.** Scenario: worker k esegue una pagina con
  loopback; arriva `__census_self` ⇒ W task in coda; i W−1 worker liberi li prendono e si parcheggiano in `Barrier::wait`;
  la richiesta interna di k entra DOPO i task census (FIFO) e nessuno può servirla: stallo fino al timeout PHP, poi k
  prende l'ultimo task, la barriera cede e la richiesta interna viene servita a cliente già sparito. Oggi (round-robin) lo
  stesso scenario costa al più una riga census emessa «dopo», mai uno stallo. Il footprint dei W task parcheggiati è nullo
  (path + `source: Vec::new()` + `web: None`, `main.rs:407`), non è questo il rischio. Emendamento: il handler legge
  `census_outstanding_now()` PRIMA di spingere (come fa `main.rs:357` per `/__census`); se ≠0 NON spinge e risponde
  `census-self REFUSED outstanding=N` senza righe. Inoltre `wait()` PRIMA di `ProbeWindow::arm()` (un wait non è strumento).
  Nota a favore: con la barriera ogni worker prende ESATTAMENTE un task census per chiamata, quindi `dispatched` per thread
  cresce di 1 come oggi e ARRIVALS cresce di W per campagna come oggi (W chiamate da 1 ⇒ 1 chiamata da W): le attese di
  A-PP36 sulla parte census NON cambiano.
- **R5 — Il dente deve VEDERE la collisione, non dedurla dal timeout.** Il binario di release stampa a teardown
  `tag=worker_dispatch thr=i count=n arm=union` (`worker_pool.rs:412-415`) e SIGTERM drena e joina (`main.rs:548-573`).
  Emendamento al gate: chiusura con SIGTERM e lettura della mappa: sul pin vecchio a W=3 con 4 dispatch la mappa è (2,1,1)
  con il count=2 sul worker della richiesta A ⇒ ERR28 attribuito alla collisione; sul pin nuovo Σ=4 e insieme ⊆{0,1,2}
  (mappa advisory, Q2). Un ERR28 senza count=2 sul pin vecchio è un rosso per ALTRA causa: controllo positivo nullo.
- **R6 — Footprint: niente rimisura, niente peak nel gate.** Il peak S-188 (334 · 335-349 MiB) è del braccio
  `--axum --workers 1`: a W=1 la cura è l'identità (R3) e la cifra non può muoversi. Il gate gira a W=3 su 4 richieste:
  un `time -l` lì misura il preludio, non la coda, e una cifra senza protocollo diventa una cifra citata (KB-78-5/KL-78-5).
- **R7 — «Bounded + 503 superato perché il backlog del kernel è unbounded»: REFUTATA nella motivazione.** Il backlog
  del kernel è BOUNDED (`listen(backlog)`) e tiene byte nei buffer socket del kernel, non nell'heap; `php -S` legge il corpo
  NEL figlio dopo l'accept, quindi tiene in heap al più W corpi. phpr legge sorgente e corpo sul thread tokio PRIMA del
  dispatch (`axum_web.rs:222,233`) e hyper accetta senza limite: residente ∝ connessioni accettate × ≤512 MiB
  (`BODY_CAP`, `axum_web.rs:46`), e una connessione abbandonata lascia il task in coda col corpo finché un worker non lo
  esegue a vuoto. La coda condivisa non peggiora né cura questo (è così anche oggi, per worker). Emendamento: dichiarare
  «bounded + 503» SUPERATO per la lunghezza della coda e aprire PER NOME in §3.9 il residuo «corpo letto prima del
  dispatch: residenza ∝ connessioni, oracle ≤ W»; fuori dal perimetro S-189.
- **R8 — Timeout nel `rx.await`: nessun argomento di heap.** Se il handler se ne va, il worker completa comunque e la
  `send` fallisce: la risposta muore sul worker (free locale, più pulito di oggi). Non libera né corpo né coda prima del
  tempo. Fedeltà decide: NO.
- **R9 — `dispatch.rs`: due test che la mia lente pretende.** (a) «push senza consumatori in attesa, consumer arriva dopo e
  prende» (controllo positivo contro il wakeup perso: `pop_blocking` deve ricontrollare il deque sotto lock prima di ogni
  `wait`); (b) «W consumer bloccati ciascuno con un item in mano ⇒ il (W+1)-esimo item resta in coda finché uno si libera»
  (è la proprietà che rende la barriera esclusiva, R4).

## KILL-SWITCH

| id | condizione (meccanica) | atto |
|---|---|---|
| KL-189-1 | `__census_self` spinto con `census_outstanding_now()≠0`, o raw di campagna contenente `census-self REFUSED` | run census VOID; nessuna riga `peak_self` citabile |
| KL-189-2 | risposta `__census_self` con meno di W `thr=` distinti, o righe `tag=peak_self` con heap pointer duplicato o count≠W | campagna VOID (estende il giudice che già rifiuta heap duplicati) |
| KL-189-3 | gate-loopback senza righe `tag=worker_dispatch` a teardown, o pin vecchio ERR28 senza count=2 sul worker di A | controllo positivo NULLO, dente non promosso (rafforza KS-189-1) |
| KL-189-4 | qualsiasi cifra di RSS/peak letta nel gate-loopback o nel throughput W=4 citata come footprint | cifra NULLA: footprint SOLO dal gemello census a W=1 (KB-78-3/5) |
| KL-189-5 | riga 1 del NEXT o lettera che dichiara «bounded + 503 superato» senza il residuo §3.9 di R7 aperto per nome | incidente di governo: la dichiarazione decade |
