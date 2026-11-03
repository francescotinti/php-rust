# CONCILIO S-189 — «coda condivisa per il pool di `php-server --axum` contro i loopback bloccanti di WordPress» (richiesto dall'utente 2026-11-03)
Generato con assistenza AI (Claude Fable 5). Fase 1: 10 verbali INDIPENDENTI in `verbali/` (9 sedie + sedia cooptata web-runtime, precedente WP-77.5; fonte
VINCOLANTE) · Fase 2: 3 team (`team-meccanismo` Hoare·Matsakis·Hejlsberg·Bak, `team-sonda` Gregg·Pedersen·Leijen·web-runtime, `team-governo`
Klabnik·Stogov) · Questa sintesi recepisce; i dissensi restano a registro col nome della sedia. Dossier: `COUNCIL_DOSSIER.md`.
**Le scritture che toccano riga 1 del NEXT, le lettere dei concili precedenti, `gaps/VETI.md` e QUALSIASI riga di codice entrano SOLO con la firma dell'utente.**

## §FONDAMENTALI (apertura obbligata)
Oggetto: il pool `--axum` ha una coda per worker e un dispatcher round-robin cieco; un loopback bloccante di WordPress (editor temi 100 s con REVERT della
modifica, auto-updater 50 s con ROLLBACK, Site Health 5–10 s, cron settimanale, update-core) cade sul worker che lo attende e stalla fino al timeout PHP.
`php -S` a N worker e php-fpm hanno l'accept condiviso sul socket: la coda condivisa è la SEMANTICA dell'oracle, non un'ottimizzazione e non una leva
(A/P = 1,00 resta il metro del motore). **Correzioni al dossier accolte dal concilio** (nessuna sedia ha benedetto, Klabnik R13 soddisfatto): (1) il difetto è
SOTTOSTIMATO: su server idle L loopback sequenziali da una richiesta producono ⌊L/W⌋ stalli DETERMINISTICI, non «probabilità 1/W» (Bak R1; Site Health ne fa 5
⇒ a W=4 uno stalla sempre); (2) `PHP_CLI_SERVER_WORKERS=N` = **N+1 acceptor**, il master accetta anch'esso (Stogov R1, `php_cli_server.c` 2503-2586); (3) il
backlog del kernel NON è unbounded: `listen(SOMAXCONN)` (`php_cli_server.c:1369`), oltre SYN scartati, MAI 503 (Hejlsberg R9, Stogov R8, Leijen R7); (4) lo
scarto di `Arc<Mutex<std::mpsc::Receiver>>` era motivato male: è corretto e FIFO; la ragione vera del Condvar è `close()` esplicito e testabile e nessun lock
tenuto durante l'attesa (Hejlsberg R2); (5) `static OnceLock<Barrier>` è RIFIUTATO: il binario di test costruisce più pool (Matsakis R3, Hejlsberg R6); (6) il
dente «due curl concorrenti» del dossier è una GARA sull'ordine di arrivo, non un controllo positivo (Gregg R1, Bak R2, Hoare R6); (7) l'insieme delle righe
`worker_dispatch` resta ESATTO {0..W-1}, non «⊆» (Pedersen R1, Matsakis); (8) «il worker caldo serve di più» cade: `notify_one` non specifica l'ordine, su macOS
sveglia il più freddo (Bak R5, Hejlsberg R5); (9) la riga `tag=worker_dispatch … arm=union` è un `eprintln!` incondizionato nelle build non mem-census
(`worker_pool.rs:412-415`, verificato dal relatore e da team-sonda): leggibile sul pin di release via SIGTERM — Gregg R3 refutata dal codice.

## Q1 — ROTTA: **GO su (A) coda condivisa FIFO, 10/10** (tutti CONCORDO CON EMENDAMENTI, Bak e Leijen concordo pieno)
(B) «primo worker libero + fallback round-robin» REFUTATA 10/10: flag «occupato» letto fuori lock = TOCTOU, sotto saturazione ricade nel difetto, non è una
semantica. (C) status quo REFUTATA 10/10: difetto di PRODUZIONE (flussi B/C revertono modifiche e aggiornamenti); la batteria 61 non lo esercita, «6/6»
certificherebbe un difetto noto (Klabnik). **Forma vincolante del candidato (team-meccanismo E1–E5, E13, E15, E16)**: nuovo `dispatch.rs`
(`#[cfg(feature = "axum-server")] mod dispatch;`), `pub(crate) struct SharedQueue<T>` GENERICA con `impl<T: Send>` esplicito, zero `unsafe impl`, solo `std`
(né `WorkerTask` né tokio); `pop_blocking` `pub(super)` con `cvar.wait_while(g, |s| s.q.is_empty() && !s.closed)` (il sigillo contro risvegli persi/spuri e
furto), restituisce `T` a guard rilasciato, nessun `peek`/`try_pop`/`len()`; `push(T) -> Result<(), T>` con `notify_one` DOPO il rilascio del lock; `close()`
= flag sotto lock + `notify_all` (drena poi esce); sezione critica = SOLE operazioni `VecDeque` + flag, poison `unwrap_or_else(PoisonError::into_inner)`
tollerato solo a questa condizione (A-PP9); `VecDeque::with_capacity(W)` senza `shrink_to_fit` (Leijen R2). Sigilli in `worker_pool.rs:170`:
`assert_not_impl_any!(WorkerTask: Clone)` (garante «esattamente un worker» = proprietà, non canale) + `assert_impl_all!(SharedQueue<WorkerTask>: Send, Sync)`,
`assert_impl_all!(WorkerTask: Send)` RESTA (A-MS3); controllo NEGATIVO `assert_not_impl_any!(SharedQueue<Rc<()>>: Send, Sync)` in `dispatch.rs`.
`WorkerPool { queue: Arc<SharedQueue<WorkerTask>>, handles }`, `worker_count() = handles.len()` (nessun campo `workers`), la closure cattura `Arc<SharedQueue>`
MAI `Arc<WorkerPool>` (`try_unwrap` a `main.rs:573`); `worker_loop:542` → `queue.pop_blocking()`, **righe 543-648 INTATTE** (pin A-PP23, letture post-end 2,
`dispatched`); `dispatch`: via l'indice 691-694 e il paragrafo ghost A-PP37 724-735 (CANCELLATO, non aggiornato), blocco census inc-before-push e undo su `Err`
INVARIATI; `shutdown`: `queue.close()` poi join INVARIATO. **Contatore orfani** (web-runtime R4, team-sonda 4/4): `ORPHANED: AtomicU64` +1 sull'`Err` di
`response_tx.send` (Relaxed), stampato come `orphan=<n>` sulla riga di teardown e in `shutdown()`. Igiene: `Cargo.lock`/`[features]` IMMUTATI nel commit della
cura; cap `worker_pool.rs` 2074 FERMO (saldo da RICONTARE: il dossier dice 2066, Hejlsberg 2068); 6 corsie `-D warnings` verdi senza `#[allow]` nuovi.
Test in `dispatch.rs` su un tipo id, SENZA `sleep`: N produttori × M consumatori «ogni id una volta E totale == N·k»; «close con K in coda ⇒ K Some poi None»
con bound temporale; push dopo close restituisce il task; close sveglia il parcheggiato; wakeup perso (push senza waiter, consumer dopo); W consumer occupati
⇒ (W+1)-esimo resta in coda; panic di un consumer non avvelena gli altri. **Esclusi come INVENZIONE 10/10**: timeout nel `rx.await` del handler (il thread OS
non si cancella: comprerebbe un 5xx che PHP non emette e nessuna capacità), 503/504 di coda, worker di riserva, rilevamento self-connection, knob `rr`.
**Parità da dichiarare**: la richiesta interna orfana dopo ERR28 viene eseguita comunque al pickup (byte già nel socket): identico a `php -S`, POST inclusi.

## Q2 — CONTRATTO DI MISURA: **(i) rafforzata, 10/10 sull'invariante; `__census_self` = RIFIUTO IN-BAND in S-189, barriera RINVIATA a design189**
Invariante nuovo (team-sonda, team-meccanismo): **W righe `tag=worker_dispatch` con thr-set ESATTO {0..W-1}** (riga per worker a teardown, testimone «W worker
vivi e joinati», A-MS7) **∧ Σcount == nreq + orphan ∧ orphan == 0 in campagna census**; `count == nreq/W` vale SOLO per i pin round-robin ≤ s188
(`97b58fd225ca7289`); a W>1 la mappa è **non verdict-grade per nome, fail-closed** (formula Klabnik, «advisory» di Stogov/Hoare assorbita: un run a w≠1 non
entra in nessuna cifra); **W=1 resta l'UNICO modo di census** (KB-78-3, A-PP28), dove FIFO condivisa ≡ round-robin per identità: nessuna campagna a registro
cambia grado; `footprint(W)` esige `min(count) ≥ 100` (Bak). `depth_max`/`inflight_max` e i lettori `measure78.sh:318,404`, `verdict81.sh` INVARIATI
(QUEUE_DEPTH è già globale). **Verdetti d'archivio MAI editati** (`verdict86-90.sh`, `va-census-template.sh`, `measure90-campaign.sh:296-303`: giudicano pin
round-robin; pin ≥ s189 = template nuovo; Pedersen KS-PP-189-1, Hejlsberg R8, team-sonda). Le campagne ×W (m88, m90), la composizione warm per-worker e il
budget ×W di A-DL31 sono versionate «pin ≤ s188»; da s189 il budget ×W è LIMITE SUPERIORE, rimisurabile solo con C=W e `inflight_max==W` DICHIARATO.
Knob `rr`: **MI OPPONGO 10/10**. **`__census_self`**: voto di fase 1 8 (barriera SOLO guardata: pre-check `outstanding==0` in codice + latch a `wait_timeout`
+ abort, per-pool) contro 2 (Pedersen, Klabnik: nessuna barriera in S-189); in fase 2 **due team su tre** (sonda, governo) adottano il rifiuto in-band, perché
nessuna sedia ha refutato Pedersen R5(c): i W task gang passano per OUTSTANDING ⇒ `inflight_max == W` ⇒ ogni run gang a W>1 è VOID da KS-PP-83-3 e dai
giudici esistenti; le stesse sedie pro-barriera la dichiarano no-op/advisory a W>1; è apparato senza committente (timebox §1) e il saldo LOC non la copre
(Hejlsberg R11). **La sintesi recepisce**: `main.rs:396-397` se `worker_count() > 1` ⇒ corpo `census-self: refused w=W (W=1 only, KB-78-3/A-PP28)` senza
dispatch; commento `:392-394` cancellato. **design189** (apertura per NOME, sessione che ne consuma l'esito) eredita l'UNIONE delle condizioni: latch per-pool
in `WorkerPoolContext` sotto `mem-census` (mai static), pre-check `OUTSTANDING==0` Acquire immediatamente prima dei W push con HTTP 409 + `census-self REFUSED
outstanding=N`, `Mutex+Condvar` con `wait_timeout` ≤ `-m` del driver e generazione azzerata, `WorkerTask.probe` che inc/dec OUTSTANDING ma salta
`note_outstanding`, `wait()` PRIMA di `ProbeWindow::arm()` e fuori da ogni lock, VSELF ESATTO con W `thr=` e heap distinti, parser a finestra, scadenza ⇒
riga `tag=census_self_incomplete arrived=<n> w=<W>` + abort (A-MS31).

## Q3 — FORMA E GATE: **CONCORDO CON EMENDAMENTI 10/10** (dente riprogettato, throughput A/B stessa finestra, 503 superato PER NOME, nessun timeout)
- **Dente**: la sintesi adotta la forma SEQUENZIALE di Bak (team-meccanismo), deterministica senza concorrenza, offset o `lsof`: riproduce ⌊L/W⌋; la coreografia
  sfasata di team-sonda (A d=800 / B d=300 a +150 ms con prova ESTABLISHED) resta a registro come alternativa equivalente. La disciplina d'evidenza di
  team-sonda e team-governo si applica integralmente (FETTA 0).
- **Throughput**: A/B interallacciato AX-old/AX-new nella STESSA finestra (team-meccanismo, team-sonda; Klabnik «0,97× del file t1» = dissenso: spread di t1
  0,4–3,8 % e drift di finestra 2–3 % confonderebbero la cura col clima). Il throughput è gate di NON-REGRESSIONE, mai record: `PERF_MAP` NON si tocca.
- **A/P a W=1**: NON si ri-registra (t3 resta la cifra; FIFO ≡ round-robin a W=1 dichiarato per nome) ma con tripwire di non-regressione (Gregg R9).
- **«Bounded + 503» WP-77.5**: il 503 è SUPERATO per fedeltà (l'oracle non lo emette mai); la lettera NON si chiude in silenzio: apertura UNICA per nome in
  §3.9 (l) «coda interna unbounded + corpo letto sul thread tokio PRIMA del dispatch (residenza ∝ connessioni, oracle ≤ W corpi) + backpressure-ad-attesa»
  (forme a registro: `Semaphore(K=64·W)` all'ingresso del handler prima di `route`, web-runtime; `tower::limit::ConcurrencyLimitLayer` in commit suo, Hejlsberg;
  cap di in-flight che sospende l'accept, Bak). Nessuna implementazione in S-189.
- **§3.39** (wrapper `http://` senza timeout): voce a catalogo con «curl NON coinvolto» (Stogov); APERTA per nome con KS proprio PRIMA della promozione (Hoare
  R8/KS-TH-189-6: con la coda condivisa W pagine `file_get_contents` loopback simultanee = deadlock PERMANENTE dove PHP si sblocca a 60 s); non blocca il
  gate (che usa curl); cura fuori perimetro. **§3.38 ↔ §3.9 (h)** legate per nome: sotto `php -S` vige `max_execution_time=30` e su macOS arm64 Zend arma
  `ITIMER_REAL` (wall) ⇒ l'oracle tronca la richiesta esterna dell'editor (100 s) a 30 s con banner 200, phpr aspetta 100 s e reverte (Stogov R4).

## FETTA 0 — il dente `wp189-harness/gate-loopback/` (accetta il DIFETTO; la CURA è accettata dai sigilli di tipo e dai test)
`loop.php` (`L` e `port` in query string; `usleep` NON usata) fa **L = W+1 curl SEQUENZIALI** a `pong.php` con `CURLOPT_TIMEOUT 3`/`CONNECTTIMEOUT 3`
(curl, NON `file_get_contents`: §3.39) e stampa per ognuna `i=<n> OK|ERR errno=<e> ms=<N>` (`ms` = sola curl); `pong.php` = `echo "pong"`. Server FRESCO per
ripetizione (`next_worker` da 0, fantasmi drenati: Stogov R2), porta ≠ 8080/8199 scelta libera e passata in query string, docroot = sola cartella delle fixture,
`127.0.0.1` ovunque, UN probe di prontezza su file STATICO (`Routed::Ready`, non avanza il contatore), curl di shell `-m 30`, nessun `--retry`, chiusura con
SIGTERM (attesa ≤ 30 s, poi SIGKILL + braccio `shutdown-hang`), stderr catturato e lettura delle W righe `worker_dispatch` + `orphan=`.
| braccio | esiti attesi (L=W+1) | impronta teardown | log richieste |
|---|---|---|---|
| **pin vecchio** `97b58fd225ca7289` W=3 (CONTROLLO POSITIVO) | OK · OK · **ERR28 ms∈[2900;3400]** · OK (il 3°: i≡0 mod W colpisce k) | ESATTA `{0:2,1:2,2:1}`, Σ=5 (il fantasma conta al pickup) | 1+4 righe, `pong` del 3° DOPO l'ERR |
| **nuovo** W=3 | 4/4 OK, ms < 1000 | Σ=5, set esatto {0,1,2}, mappa advisory, **orphan=0** | 1+4 |
| pin vecchio W=1 · nuovo W=1 (fedeltà) | L=2: ERR28 · ERR28 | `{0:3}`, orphan=2 | 1+2 |
| oracle `php -S` **`PHP_CLI_SERVER_WORKERS=2`** (= 3 acceptor) | 4/4 OK, risposta esterna 200 | n/d | — |
| oracle 1 processo | L=2: ERR28 · ERR28, esterna 200 (< 30 s di §3.38) | n/d | — |
| braccio NEGATIVO: pin vecchio W=3, L=1 | OK (prova fixture e facciata) | `{0:1,1:1,2:0}` | 1+1 |
`errno` confrontato col token dell'oracle (libcurl vero). 3 ripetizioni per braccio, esito IDENTICO (qualsiasi difforme = FAIL, mai «2 su 3»); log con righe
oltre le attese (scheda browser, lezione t1) ⇒ ripetizione VOID e ripetuta (max 3); hash del pin == `PIN_ATTESO` altrimenti VOID (mai skip). Il dente va ROSSO
PRIMA sul tree (braccio new = build dall'artefatto cargo PRIMA del cablaggio) e VERDE dopo; riga PASS `old=<hash> new=<hash> git=<rev>`; `.out` fuori repo,
script+fixture in git. Il dente NON discrimina (A) da (B): scritto nel criterio. **Dente unitario in CI** `no_idle_worker_while_queue_nonempty`
(`worker_pool.rs` tests, pool fresco W=2, come 1801/1841): tre task spediti in ordine senza attendere — A = `usleep(1500000)`, B breve, C breve; asserzioni:
B e C rispondono PRIMA di A, C entro 1 000 ms, A entro 5 000 ms. Round-robin da 0: C→worker 0 dietro A ⇒ ROSSO (prova sul tree C2 nel `.out`); FIFO ⇒ VERDE.
**Smoke ADVISORY** (Bak R7): boot W=4 con cron ABILITATO, `GET /`, lettura del log: pin vecchio mostra la catena D/E/F con un timeout, nuovo no; una riga nel
verbale, mai gate.

## FETTA 1 — guardia throughput e tripwire (criterio proprio ≤10 righe, finestra concordata con l'utente, lock TOKEN s189, CI quieta)
Catena `AX-old, AX-new` per ciclo, R=3, N=150, warm-up `-n 20 -c W` (tutte le unit cache calde), stesse 5 URL e C∈{1,4,8} del t1, `ab` senza `-k` su
entrambi (keep-alive su un solo lato = VOID): canonica = `AXnew/AXold` req/s per cella nello stesso ciclo, mediana dei 3; spread ≤ 5 % per braccio; accettazione
C=4/C=8 ∈ [0,97; 1,05], C=1 ∈ [0,95; 1,05] (regime di località della unit cache diverso: Matsakis); mediana fuori banda con un ciclo dentro ⇒ cella VOID
(rimisura), non kill; > 1,05 = REPERTO da attribuire prima di promuovere; p99 companion (attesa: C=8 nuovo ≤ vecchio; > 1,10× = REPERTO); user/req e cpu/req
registrati (±2 %); `O4/AX` companion di continuità col t1 (±10 %, advisory); distribuzione `worker_dispatch` a C=1 nel verdetto. ≈ 30 min. **Tripwire W=1**
(≈ 8 min): `Aold, Anew` × {front, login} × R=3 × K=30 nella harness della coppia, `|Anew/Aold − 1| ≤ 2 %`, etichetta «non-regressione», mai cifra.

## LA FORMA (lettere emendate per nome; testi vincolanti di team-sonda; archivio intatto; ogni lettera versionata «testo precedente valido per pin ≤ s188»)
- **A-PP36**: «dispatch-count per-thread in-band a teardown su OGNI arm, con `orphan=<n>`; atteso = Σcount == nreq + orphan con set esatto {0..W-1}; la
  ripartizione per-thread è “atteso” solo sui pin round-robin ≤ s188».
- **KS-PP-87-3**: «cifra net da run senza dispatch-count in-band con W righe, set esatto, Σcount == nreq + orphan e orphan == 0 ⇒ envelope; dai pin ≥ s189
  la mappa per-thread non è condizione (non verdict-grade)».
- **A-PP37**: «verdict esige W righe con set esatto e Σcount == nreq + orphan; MAPPA == nreq/W attesa SOLO su pin ≤ s188; la coda condivisa non ha slot:
  `push` chiusa restituisce il task ⇒ nessun ghost di dispatch (clausola CHIUSA, commento cancellato a saldo righe); il ghost dei watermark `note_*` (A-MS16)
  RESTA; run con dispatch Err ⇒ VOID».
- **A-PP42 / KS-PP-89-2**: «VDISP esige W righe con thr-set ESATTO {0..W-1} e Σcount == nreq + orphan (+1 per `/__census_self` sullo slope a W=1);
  `count == nreq/W` SOLO per pin ≤ s188; Σ≠atteso ⇒ FAIL (phantom), set≠esatto ⇒ FAIL (worker perso); il dispatcher in vigore (round-robin | FIFO condivisa)
  va DICHIARATO nel raw, mai assunto».
- **KH78-2 / KB-78-2 / A-BB8**: «profondità QUEUE_DEPTH (globale: somma delle W code sui pin ≤ s188, coda unica dai pin ≥ s189; a W=1 la stessa grandezza)
  > 1 in misura ⇒ VOID; depth campionata a ogni dispatch; semantica “FIFO di dispatch, non di accept”».
- **WP-77.5 conv. #2**: nota a piè: «thread-attore INVARIATO; “bounded + 503” SUPERATA S-189 nella forma 503 (l'oracle non lo emette: `listen(SOMAXCONN)`,
  SYN scartati); residuo §3.9 (l) aperto per nome». **A-DL31, KB-87-3, KL-87-2, A-PP49**: «valide per pin ≤ s188; da s189 budget ×W = limite superiore».
- **§3.9 (h)** riscritta: «CURATA S-189 (pin s189): coda condivisa = accept condiviso di `php -S` (N figli + master)/php-fpm; dente `wp189-harness/gate-loopback`
  bilaterale con controllo positivo sul pin s188; su server idle il pin ≤ s188 stallava ⌊L/W⌋ loopback sequenziali per richiesta; a W=1 il loopback stalla fino
  al timeout del client COME l'oracle (conservato per nome); a W worker, W loopback concorrenti stallano come l'oracle; la richiesta orfana è eseguita comunque
  (parità); nessun timeout nel handler; legata a §3.38 (ITIMER_REAL wall su macOS)». **§3.9 (l)** nuova (testo Q3). **§3.39** nuova (testo Stogov R5).
  **§3.38** nota «macOS arm64: wall; Linux: CPU — busy33 neutra, un loopback NO». `GIUDIZIO_C_AXUM.md`: una riga.
- **Gate**: `gate-lever-pins.sh` + riga di ASSENZA `next_worker` in `worker_pool.rs` e assenza di `timeout(`/`503` nel handler di `main.rs`;
  `gate-capture-order.sh` scansiona `dispatch.rs` con atteso 0; commento 514 di `two_workers_no_request_bleed` emendato (copertura probabilistica), test intatto.
- **Sequenza (ogni commit col suo gate, build mai red, criterio PRIMA del codice)**: C0 docs (`s189-criterio-loopback.md` ≤10 righe + criterio guardia AX +
  §3.9 (h)(l) + §3.39 + nota §3.38 + `next-budget.sh` PASS) → C1 fixture + `gate-loopback.sh` (rosso GIUSTO sul pin s188, oracle `WORKERS=2`/1 processo,
  new = tree ROSSO) → C2 `dispatch.rs` + test coda (`cargo test -p php-server` ×3 feature, `loc_dente`, corsie `-D warnings`) → C3 cablaggio + rifiuto
  `__census_self` + orfani + dente unitario (rosso su C2 provato) + `gate-lever-pins`/`gate-capture-order`/`gate-doc-purge`/`gate-loopback` VERDE 3/3/
  `run-gate.sh` G-APERTURA-2 (static-counter = 1)/`battery61-axum` 6/6 → C4 lettere + sintesi + `GIUDIZIO_C_AXUM` → riga 1 NEXT + `gaps/VETI.md` (con firma)
  → C5 `scripts/pin-server.sh s189` + `PIN_REGISTRY` + `PIN_ATTESO` dei harness per nome → FETTA 1 (guardia AX + tripwire) ⇒ promozione = pin s189 registrato.
  **Merge a `main` resta decisione separata dell'utente.** Rollback: `git revert` di C3 E C2 insieme, prova = `gate-loopback.sh` di nuovo rosso sul tree.
  Apparato oltre ½ sessione a qualunque passo ⇒ incidente §1: si chiude al commit verde raggiunto.

## RIGA 1 DEL NEXT (testo per la firma; team-governo; aritmetica verificata: riga 1 oggi 600/600, file 7.853/8.000)
**Scarico deliberato per nome**: la clausola «KILL A REGISTRO: S-168 Σ mock 9,92<10 ⇒ R1 chiusa; R4 eseguita come “corpo del handler” (S-170) SENZA tappa
reale, sanata qui» (128 car.) migra in `gaps/VETI.md` con riga datata 2026-11-03 «scaricata dalla riga 1 per delibera S-189» (REGOLE §8: un kill resta
«fino al concilio seguente»; S-186 bis e S-189 lo sono). **ROTTA B MORTA resta verbatim.** Riga 1 proposta (598 caratteri):
```
# NEXT_SESSION — phpr: ORIZZONTE parità (≥1×) con l'oracle · TAPPA D (delibera utente S-186, 2026-10-22, verifica al concilio ~S-192): WP ≤1,5 (coppia notturna, mediana) · ORM ≤5 (coppia E5, al netto del pavimento sqlite3 comune) · micro ≤3× 6/6 · **ROTTA B MORTA (S-186 bis, 01/11, rc=4): mutante MUTO anche vs gemello Z (ΔM +2,18 s IN finestra [1,66;2,25]; builtin 67 % < 75 %) ⇒ strumento non provato, kill definitivo (criterio p.9 + firma (b): nessun altro riesame)** · **delibera S-189, 2026-11-03: `--axum` a CODA CONDIVISA (= accept `php -S`); W=1 stalla; gate-loopback rosso sul pin s188**.
```
Lettere emendate, barriera APERTA (design189), §3.9 (l), §3.39 vanno nel CORPO del NEXT a saldo (chiudi-sessione).

## ORDINE OBBLIGATO DEI PASSI
(0) **firma dell'utente** su questa sintesi: Q1 (A), Q2 (rifiuto in-band, barriera a design189), Q3 (dente sequenziale + A/B stessa finestra), riga 1 + scarico
S-168 → (1) C0 → (2) C1 (dente rosso giusto) → (3) C2 → (4) C3 (tutti i gate verdi) → (5) C4 → (6) riga 1 + VETI → (7) C5 pin s189 → (8) FETTA 1 ⇒ promozione.
Build, test e misure SOLO a macchina ferma su avviso dell'utente. Se un kill scatta: KS-189-1 ⇒ fermi a C1, nessun `.rs` scritto; KS-189-5/6 ⇒ C2–C4 restano in
`s186-firma` NON promossi, pin e `PIN_ATTESO` restano s188, §3.9 (h) «cura in branch, non promossa», kill in riga 1 (scarico per nome con firma); HALT ⇒ revert
C3+C2 insieme, docs C0/C1 restano (il criterio sopravvive al codice).

## KILL-SWITCH A REGISTRO (deduplicati dalle tre tabelle; il più stretto vince)
| # | condizione (meccanica) | atto | sedie |
|---|---|---|---|
| KS-189-1 | pin con hash == `PIN_ATTESO` W=3 L=4: il 3° loopback NON è ERR28 ∈ [2900;3400] ms, o OK·OK·ERR·OK non ottenuto 3/3, o impronta ≠ {0:2,1:2,2:1}/Σ≠5, o hash ≠ atteso | dente NULLO, nessuna promozione né merge | Bak, Hoare, Pedersen, Leijen, Klabnik, Gregg |
| KS-189-2 | ripetizione difforme (qualsiasi), errno ≠ 28, log con righe/URI oltre le attese, boot senza restart, esito «2 su 3» | FAIL del gate / ripetizione VOID; 2 VOID consecutive ⇒ coreografia riprogettata | Gregg, Klabnik, Stogov, web-runtime |
| KS-189-3 | braccio negativo (L=1) NON OK; braccio W=1 (phpr o oracle) NON stalla o esterna oracle ≠ 200; errno phpr ≠ errno oracle; oracle lanciato con `WORKERS=3` e detto «3 worker» | dente NULLO fino a cura / HALT / braccio oracle VOID | Gregg, Stogov |
| KS-189-4 | pin NUOVO W=3: non verde 3/3, `orphan>0` su un worker, pid LISTEN ≠ spawnato, porta non libera | cura NON promossa / run NULLA | Gregg, web-runtime |
| KS-189-5 | FETTA 1: cella C∈{4,8} fuori [0,97;1,05] o C=1 fuori [0,95;1,05] con 3 cicli concordi; `ab -k` su un solo lato; confronto fatto contro il file t1 come gate | cura NON promossa (resta in branch), pin s188; cifra ADVISORY | Bak, Gregg, web-runtime, Matsakis, Hejlsberg |
| KS-189-6 | tripwire W=1 `|Anew/Aold−1| > 2 %` su front o login; p99 C=8 > 1,10×; cpu/req fuori ±2 % | HALT / REPERTO: promozione sospesa finché attribuito | Gregg |
| KS-189-7 | `pop_blocking` senza predicato sotto lock (`wait` nudo, `if`), API con `&T`/guard, o sotto il lock una chiamata fuori da VecDeque/flag (`send`, `php_runtime`, tripwire, `census_line`, `wait`) | REJECT del commit | Hoare, Matsakis |
| KS-189-8 | mancano `impl<T: Send>`, `assert_not_impl_any!(SharedQueue<Rc<()>>)`, `assert_not_impl_any!(WorkerTask: Clone)`, `assert_impl_all!(SharedQueue<WorkerTask>)`, o sparisce :170; `unsafe impl`; coda/W in `static`/`OnceLock` di processo/`Box::leak`; closure che cattura `Arc<WorkerPool>` | build non promuovibile / HALT | Hoare, Matsakis, Hejlsberg |
| KS-189-9 | test E16 assenti o su `WorkerTask`, N×M senza «ogni id una volta E totale», close-K senza bound, `sleep` come sincronizzazione, `std::sync::Barrier`; dente unitario W=2 assente da C3 o senza prova del rosso su C2 | `dispatch.rs`/C3 non entrano; se in `main`: incidente | Hoare, Hejlsberg, Pedersen, Leijen, web-runtime, Klabnik |
| KS-189-10 | nel diff: timeout/`select` sul `rx.await`, 503/504, worker di riserva, self-connection, knob `rr`, barriera `__census_self` costruita in S-189 | HALT: invenzione fuori oracle / apparato oltre mandato, revert prima del pin | Bak, Stogov, Klabnik |
| KS-189-11 | `__census_self` servito a `w>1` senza design189 firmato; raw con `census-self REFUSED`/`inflight_max>1` con righe `peak_self`; claim per-thread da raw `w>1` su pin ≥ s189 senza protocollo C=W | raw/campagna VOID; claim ADVISORY, mai attribuzione | Pedersen, Leijen, web-runtime |
| KS-189-12 | righe `worker_dispatch` ≠ {0..W-1}, Σcount ≠ nreq + orphan (W>1), count ≠ nreq (W=1), `footprint(W)` con min(count) < 100 | run VOID | Matsakis, Bak |
| KS-189-13 | verdetto d'archivio editato per far passare un raw ≥ s189; test census 1801/1841/1911/1919/2029 emendato senza voce; commento ghost `:724-729` o «W sequential calls» `:392-394` sopravvivono; seconda fonte di QUEUE_DEPTH | verdetto NULLO, incidente / review FAIL (token-presenza) | Pedersen, Bak, Klabnik |
| KS-189-14 | riga 1 NEXT riscritta per far posto senza scarico deliberato, o senza «delibera S-189» a fine sessione; «503 superato» senza §3.9 (l) e §3.39 aperte per nome con KS proprio | incidente KS-K186-1 / promozione sospesa, la dichiarazione decade | Klabnik, Leijen, web-runtime, Hoare, Hejlsberg |
| KS-189-15 | una corsia `-D warnings` rossa; `Cargo.lock`/`[features]` cambiati nel commit della cura; `loc_dente` rosso o cap alzato; `battery61-axum` < 6/6; G-APERTURA-2 FAIL (static-counter ≠ 1); lever-pins/capture-order FAIL; server del dente non uscito 30 s dopo SIGTERM ×2 | HALT (mai `#[allow]` nuovo; commit separato fuori finestra) / incidente | Hejlsberg, Bak, Stogov, Klabnik, web-runtime |
| KS-189-16 | cifra RSS/peak letta nel gate-loopback o nel throughput W=4 citata come footprint; cifra «A/P per costruzione» non dichiarata per nome | cifra NULLA / ADVISORY | Leijen, Hoare |

## DISSENSI A REGISTRO
- **Hoare, Matsakis, Hejlsberg, Bak (team-meccanismo), Leijen, Gregg, web-runtime, Stogov** (fase 1, 8/10): barriera `__census_self` GUARDATA già in S-189
  (per-pool, pre-check `outstanding==0`, latch `wait_timeout` 5 s, abort). La sintesi la rinvia a design189 con l'unione delle loro condizioni (Q2); il
  dissenso resta sotto gli otto nomi; Leijen nota che con la barriera `dispatched` cresce di 1 per thread e ARRIVALS di W come oggi.
- **Klabnik**: throughput contro il file t1 (0,97×, KS-SK-189-6) e riga di guardia `[ "$W" = 1 ] || FAIL VDISP` nei sei verdict d'archivio (copia dichiarata);
  **Hoare**: t1 rimisurato contro le soglie pre-registrate. Superati dall'A/B stessa finestra e dall'archivio intatto; il «sì rimisurare» è onorato.
- **team-sonda**: dente sfasato con prova ESTABLISHED e firma `{0:2,1:1,2:1}`; la sintesi preferisce la forma sequenziale (nessuna dipendenza da tempi); le due
  forme sono equivalenti come controllo positivo e la sfasata resta disponibile se la sequenziale mostrasse un difetto in C1.
- **Hejlsberg**: `notify_one` DENTRO il lock (forma); lessico «superata» per bounded+503 (soddisfatto dall'apertura (l) per nome e da KS-189-14).
- **Bak**: timeout del latch 30 s (preso 5 s in design189, 30 s resta cap esterno della curl); `len()` esposta (tolta); «i 38 s del 02/11 = tutti e 5 i
  loopback stallati ⇒ W=1 o wrapper senza timeout» (inferenza non verificata). **Bak** scriveva «4° ERR28»: corretto in 3° dal suo stesso R1 (team-meccanismo).
- **web-runtime R6** «B=OK a W=1»: corretto (stallo certo). **Gregg R3** «riga solo nei binari census»: refutato dal codice. **Klabnik R8/R12**: «backlog
  unbounded» rettificato con il testo di Stogov; testo riga 1 da 247 car. refutato dall'aritmetica (598 proposti dal team-governo).
- **Stogov «advisory» vs Klabnik «fail-closed»** (Q2): assorbito nella formula «non verdict-grade per nome, fail-closed».
