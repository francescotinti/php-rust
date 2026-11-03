# Verbale sedia HEJLSBERG — S-189 (bozza indipendente, FASE 1)
Generato con assistenza AI (Claude Fable 5).
Lente: ingegneria del runtime, costo delle primitive, matrice cfg, forma dell'API, igiene delle dipendenze.
Letto: dossier intero; `worker_pool.rs` 1-760 e 1780-2068; `main.rs` 200-440; `Cargo.toml`; `ci.yml` 77-135;
`loc_dente.rs:108`; `measure90-campaign.sh` 290-306; `php_cli_server.c:1369`; lettera WP-77.5 conv. #2.

## VERDETTI
- **Q1 ROTTA: CONCORDO CON EMENDAMENTI — A.** Coda condivisa FIFO, nella forma emendata R1-R8. B rifiutata (R12): «primo
  worker libero» è un'euristica con la stessa finestra di stallo, non una semantica. C rifiutata: il difetto è di produzione.
- **Q2 CONTRATTO: CONCORDO CON EMENDAMENTI — (i).** Σcount == nreq ∧ thr ⊆ {0..W-1}; mappa a W>1 ADVISORY; census solo a W=1
  (KB-78-3). Knob `rr`: NO (due vie al cap, settima riga di matrice cfg). Barriera per `__census_self`: SÌ nel principio, NO nella
  forma `static OnceLock<Barrier>` (R6): per-pool, in `WorkerPoolContext`.
- **Q3 FORMA E GATE: CONCORDO CON EMENDAMENTI.** Il dente `gate-loopback` basta per la CORRETTEZZA; il throughput (KS-189-2)
  resta come controllo di non-regressione, non come misura (R4-R5: il costo della coda è ≤10⁻⁴ della richiesta, ma l'ordine di
  risveglio cambia la geografia delle cache per-thread). «Bounded+503»: NÉ superata NÉ implementata ora — riaperta per nome con
  la cornice corretta (R9). Timeout su `rx.await`: NO (R10).

## ROTTA (una riga)
A: `dispatch.rs` con `SharedQueue<T>` generico su std `Mutex`+`Condvar` (`wait_while`), W = `handles.len()`, barriera per-pool
nel `WorkerPoolContext`, zero crate nuove, cap LOC fermo a 2074, dente gate-loopback + KS-189-2, «limite di accettazione» aperto per nome.

## REFUTAZIONI ED EMENDAMENTI
- **R1 — generico, non concreto.** `pub(crate) struct SharedQueue<T>`: due istanze monomorfe (`WorkerTask` + il tipo dei test),
  costo di compilazione nullo. La forma concreta obbligherebbe i test «N produttori × M consumatori, ogni id una volta sola» a
  costruire un `WorkerTask` (meta + oneshot) per id: il test perde la sua leggibilità ed è il motivo stesso del file nuovo.
  `assert_impl_all!(SharedQueue<WorkerTask>: Send, Sync)` va in `worker_pool.rs` accanto a :170, NON in `dispatch.rs` (che non deve
  conoscere `WorkerTask`: solo `std`, nessun `use tokio`). Dichiarazione `#[cfg(feature = "axum-server")] mod dispatch;` in
  `main.rs:208-209`: senza cfg la corsia default `ci.yml:81` lo vede morto e `-D warnings` è rossa (bin crate: `pub` in mod privato
  non salva dal dead_code).
- **R2 — lo scarto di `std::mpsc` è motivato male (dossier §4).** `Arc<Mutex<Receiver>>` è CORRETTO: il consumatore bloccato in
  `recv` tiene il lock, gli altri si accodano sul Mutex, prende il task e rilascia; FIFO; chiusura = drop del `Sender` → `Err`
  dopo il drain. E `len()` non serve a nessuno: QUEUE_DEPTH è già un contatore atomico separato (:710 inc, :549 dec). La ragione
  VERA per Condvar: `close()` esplicito e testabile senza passare dal drop del pool, nessun lock tenuto durante l'attesa, e il
  test «close sveglia il parcheggiato» non dipende dall'ordine di acquisizione del Mutex. La sintesi corregga la riga del dossier.
- **R3 — forma del pop.** `cvar.wait_while(guard, |s| s.q.is_empty() && !s.closed)` al posto del loop manuale: spurious wakeup
  gestiti dalla std, −6 righe, invariante leggibile. `notify_one` DENTRO il lock dopo `push_back` (ordine push→notify
  inattaccabile). Poison: `unwrap_or_else(PoisonError::into_inner)` come `main.rs:295` (idioma già in casa, A-PP9 copre il resto).
- **R4 — costo.** Mutex+Condvar non conteso 50-200 ns per op; conteso (≤8 produttori tokio + W consumatori) qualche µs; richiesta
  10-50 ms: rapporto ≤10⁻⁴. Nessuna leva e nessun rischio di prestazioni per costruzione. Rifiuto `parking_lot` (nel lock via
  tokio, ma sarebbe una riga di dipendenza diretta nuova) e `crossbeam-channel` (non nel lock: `Cargo.lock` ha solo deque/epoch/
  queue/utils transitivi).
- **R5 — ordine di risveglio.** `notify_one` sveglia un thread NON specificato (futex/ulock: nessuna FIFO garantita). Conseguenza
  da scrivere: a W>1 la mappa per-thread NON è deterministica e la unit cache per-thread si scalda in funzione del carico (LIFO
  concentra: meno RSS, più hit; FIFO spande: come il round-robin odierno). Nessun contratto sull'ordine; il budget fisico ×W
  (A-DL31) resta un LIMITE SUPERIORE; identico all'`accept()` dell'oracle. È l'unica ragione per cui KS-189-2 ha senso.
- **R6 — `static OnceLock<Barrier>` RIFIUTATO.** I test creano più pool nello stesso processo (`new(ctx,1)` :1805/:1845,
  `new(ctx,2)` :2032): una barriera di processo dimensionata al primo W è sbagliata per gli altri e sotto mem-census blocca per
  sempre. Forma: `#[cfg(feature = "mem-census")] census_self: std::sync::OnceLock<std::sync::Barrier>` in `WorkerPoolContext`
  (:119-121, oggi vuota con TODO); `set(Barrier::new(num))` in `WorkerPool::new` dopo la risoluzione di W (:371-375), `expect("one
  context per pool")`; il worker la raggiunge da `_context` (:505) GIÀ passato — niente parametri cfg nelle chiamate (gli attributi
  sugli argomenti di chiamata non compilano), niente static. Compile-out pulito su tutte le corsie: campo cfg + `cfg_attr(not(
  mem-census), allow(unused_variables))` sul parametro, esattamente l'idioma di `worker_idx` (:507). `wait()` nel ramo :588
  PRIMA di `census_line`. Handler `main.rs:405-409`: W `dispatch` poi W `await`, corpi concatenati; la quiescenza resta
  precondizione dichiarata (una richiesta lunga RITARDA la barriera; un worker in stallo loopback la blocca: è la patologia stessa).
- **R7 — W.** Via il campo `workers: usize` proposto: `worker_count()` (:349-351) = `self.handles.len()`, una sola fonte di
  verità. `num_cpus::get()` ≥ 1 sempre ⇒ `Barrier::new(0)` irraggiungibile. Igiene FUTURA, non in questo arco (ricetta ferma in
  finestra di misura): `std::thread::available_parallelism()` toglie `num_cpus` da `Cargo.toml:17,30`.
- **R8 — `dispatched` e la riga per-thread.** Invariati (:543 conteggio al pickup; :413 e :447 le due emissioni). `worker_idx`
  diventa ORDINALE del thread, non indice di coda: una parola nel commento :509-514. Lettera 10: `count==nreq/W` →
  `Σ==nreq ∧ thr⊆{0..W-1}`. I parser verdict86..90 NON si toccano in questo arco: a W=1 la riga è identica byte per byte.
- **R9 — «bounded+503»: il dossier §7 dice «il backlog del kernel è unbounded»: FALSO.** `php -S` chiama
  `listen(retval, SOMAXCONN)` (`php_cli_server.c:1369`): backlog LIMITATO, oltre SYN scartati/RST = backpressure TCP, MAI un 503.
  Quindi: NON superata (l'accettazione illimitata di hyper + coda in memoria è una divergenza reale: l'oracle respinge a monte);
  NON implementata ora (`push → Err se len ≥ cap` confonde Closed e Full: servirebbe `enum PushError<T>{Closed(T),Full(T)}`, e
  un 503 che l'oracle non emette). Riaprire per NOME in §3.9 come «limite di accettazione N·k» via
  `tower::limit::ConcurrencyLimitLayer` (tower 0.4.13 già dipendenza, `Cargo.toml:15`; richiede la feature `limit`: quindi un
  commit suo, mai nel commit della cura).
- **R10 — timeout su `rx.await` (:432): NO.** Nota di tipo: la cancellazione lato client droppa `rx`, il task resta in coda e
  `send(response).ok()` (:620) scarta: comportamento identico a oggi e all'oracle (che serve sul socket morto).
- **R11 — cap LOC.** `worker_pool.rs` 2068/2074 (`loc_dente.rs:108`): 6 righe di margine. Il saldo «−4» del dossier non
  include R6 (campo + set + wait + commento ≈ +8): si paga compattando :724-735 (il paragrafo ghost A-PP37 sparisce, −6) e
  :687-689. Il cap NON sale nel commit della cura (REGOLE §8).
- **R12 — Q1-B in due righe.** Un flag «occupato» per worker letto dal dispatcher e scritto dal worker: fra la lettura e il
  push il worker può entrare nel loopback ⇒ lo stesso stallo con probabilità minore, non nulla; a saturazione degenera nel
  round-robin cieco. Non è una semantica: non può essere il riferimento di fedeltà.
- **Lettere (§6):** 1 INVARIATA nel thread-attore, «bounded+503» DA EMENDARE come R9 (non «superata») · 2-3-5-6-7 INVARIATE ·
  4 DA RI-DEFINIRE («una coda, depth = len della coda condivisa») · 8-9-10 DA EMENDARE come R8 · 11 DA RI-DISEGNARE come R6.

## KILL-SWITCH
| condizione | atto |
|---|---|
| KS-AH-189-1: una delle 6 corsie `-D warnings` (`ci.yml:81-85`, `:127-129`) rossa dopo la cura (dead-code da campo/param cfg) | cura non promossa; si corregge il cfg, MAI un `#[allow]` nuovo |
| KS-AH-189-2: `SharedQueue` o la barriera in forma di `static` di processo | rifiuto in revisione (R6) |
| KS-AH-189-3: test N×M / drena-poi-esce / close-sveglia assenti o scritti su `WorkerTask` invece che su un tipo id | `dispatch.rs` non entra |
| KS-AH-189-4: `Cargo.lock` o `[features]` cambiano nel commit della cura (crate nuova, feature `limit`, `num_cpus` via) | HALT: commit separato, fuori finestra |
| KS-AH-189-5: `loc_dente` rosso o cap di `worker_pool.rs` alzato nel commit della cura | HALT |
| KS-AH-189-6: «bounded+503» dichiarata «superata» nella sintesi senza riga nominata in §3.9 | voce di dissenso a verbale |
