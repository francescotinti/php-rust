# DOSSIER — Concilio S-189 «coda condivisa per il pool di `php-server --axum` contro i loopback bloccanti di WordPress» (richiesto dall'utente, 2026-11-03)

Generato con assistenza AI (Claude Fable 5). Calco del dossier S-186 (`wp186-harness/COUNCIL_DOSSIER.md`).
Fonti: `PHPR_DIVERGENCES_FROM_PHP.md` §3.9 (h), `wp188-harness/revisione-s188-t2-axum.md` (rilievo #2), `sessions/WP_SESSION_188.md`,
`wp188-harness/s188-wp-server-pair-verdetto-t3.out`, `s188-throughput-verdetto-t1.out`, concili WP-77.5 / 78 / 79 / 81 / 82 / 83 / 87 / 88 / 89 / 90 / S-186,
sorgente WordPress `~/Claude/wpdev/src`, sorgente PHP 8.5.7 `sapi/cli/php_cli_server.c`, `main/fastcgi.c`.
Titolo di convocazione: REGOLE §7 — richiesta esplicita dell'utente (03/11 16:50) e cambio di rotta: la cura tocca la macchineria di misura per-thread (A-PP36).
**Mandato delle sedie: REFUTARE, mai benedire.** Il candidato tecnico in §4 è una proposta del relatore, non una delibera.

## 1. IL FATTO (verificato sul codice, riga per riga)
- `php-server --axum` ha **una coda per worker**: `WorkerPool { senders: Vec<mpsc::UnboundedSender<WorkerTask>>, next_worker: AtomicUsize, handles }`
  (`crates/php-server/src/worker_pool.rs` 333-342). `dispatch` sceglie `next_worker.fetch_add(1) % W` (690-694) **senza sapere chi è occupato**; census
  QUEUE_DEPTH/OUTSTANDING/ARRIVALS incrementati PRIMA della send (705-719), undo su send fallita (736-741), ghost `next_worker` dichiarato (A-PP37, 724-729).
- Il worker è un thread OS (`std::thread::spawn`, 379-479) che fa `rx.blocking_recv()` (542) ed esegue la richiesta sul proprio thread
  (`execute_request(&reg, &task.meta)` 606/611; `response_tx.send(` 620; OUTSTANDING −1 dopo la send, A-TH9, 630-640). Il handler tokio (`main.rs` 251-434)
  fa `route` → `oneshot` → `pool.dispatch` → `rx.await` **senza timeout** (432).
- Il client HTTP di phpr gira **sul thread del worker** ed è bloccante: `curl_*` è una facciata su `ureq` sincrono (`php-builtins/src/curl.rs`, timeout solo se
  `CURLOPT_TIMEOUT(_MS)`/`CONNECTTIMEOUT` > 0, 543-548); il wrapper stream `http://` (`php-builtins/src/file.rs` 493-531 `parse_http_options`, 553-557 `http_get`)
  **non imposta alcun timeout** (PHP: `default_socket_timeout` = 60 s) — reperto COLLATERALE, da catalogo (§3.39), non oggetto di delibera.
- Conseguenza: una pagina PHP che chiama lo stesso server (loopback) tiene occupato il worker k; tokio accetta la connessione interna e la accoda al worker
  `k+1+j` (j = richieste arrivate nel frattempo). Con probabilità **1/W** è il worker k stesso ⇒ **stallo fino al timeout lato PHP**. Con `--workers 1` lo
  stallo è CERTO. Nessun furto di lavoro, nessun flag «occupato», nessun 503, nessun timeout nel handler.

## 2. CHI LO ESERCITA IN WORDPRESS (grep su `~/Claude/wpdev/src`; esito del timeout = `WP_Error`, non fatal)
| flusso | file:riga | bloccante | timeout | conseguenza del timeout |
|---|---|---|---|---|
| A `spawn_cron` → `wp-cron.php` | `wp-includes/cron.php:982-997` (hook `init` e, da WP 6.9, `shutdown` 1018-1028) | **NO** (`'blocking' => false`) | 0,01 s | nessuna: innocuo |
| B editor temi/plugin, sanity check | `wp-admin/includes/file.php:587, 616` (`wp_edit_theme_plugin_file`) | sì, ×2 | **100 s** | **reverte la modifica** («Unable to communicate back with site…», 592-594) |
| C `WP_Automatic_Updater::has_fatal_error` | `class-wp-automatic-updater.php:1807` | sì | 50 s | può far **rollback** di un aggiornamento riuscito |
| D Site Health `can_perform_loopback` | `class-wp-site-health.php:3302` | sì | 10 s | test «critico» |
| E Site Health REST availability | `:2233` | sì | 10 s | idem |
| F Site Health page-cache | `:3633` ×3 | sì | 5 s ×3 | idem (P(≥1 colpo) = 1−(1−1/W)³) |
| G cron settimanale `wp_cron_scheduled_check` | `:3409/3418` | sì | 5 s | idem |
| H https detection | `wp-includes/https-detection.php:113/124` ×2 | sì | 5 s | idem |
| I–K update-core / multisito upgrade / install | `update-core.php:1629` · `admin.php:84` · `upgrade.php:541` | sì | 60 / 120 / 5 s | flusso rallentato |
`rest_do_request`/`rest_ensure_request`, preload dell'editor e `async_direct_test` sono IN-PROCESS (nessun HTTP). Il cron iniziale da 38 s del 02/11 era D+E+F.
Oggi le misure S-188 aggirano il difetto con `WP_BENCH_DISABLE_CRON` e `--workers ≥ 4`: il difetto è di PRODUZIONE (admin, Site Health, aggiornamenti), non di misura.

## 3. IL RIFERIMENTO DI FEDELTÀ
`php -S` con `PHP_CLI_SERVER_WORKERS=N` (`sapi/cli/php_cli_server.c`): socket creato una volta e reso non bloccante PRIMA del fork (2564-2580: «the first
process would successfully accept the connection but the others would block»), `php_cli_server_startup_workers` forka N figli (2503-2545), ogni figlio fa
`accept` sullo STESSO socket (2713-2716, loop 2770-2777): il backlog del kernel è **una coda condivisa**, un worker occupato non accetta mai; a 1 worker il
loopback stalla fino al timeout del client (identico a phpr). php-fpm: figli in `accept(listen_socket)` (`main/fastcgi.c:1397`), stesso disegno.
⇒ La coda condivisa è la SEMANTICA dell'oracle, non un'ottimizzazione; «W=1 stalla» è comportamento corretto da conservare.

## 4. IL CANDIDATO TECNICO DEL RELATORE (da refutare)
- Nuovo file `crates/php-server/src/dispatch.rs` (worker_pool.rs è a 2066/2074): `SharedQueue<T> { Mutex<State<T>>, Condvar }`, `State { VecDeque<T>, closed }`;
  `new() -> Arc<Self>` · `push(t) -> Result<(), T>` (Err(t) sse chiusa; `notify_one`) · `pop_blocking() -> Option<T>` (None sse chiusa E vuota: drena poi esce;
  loop sul Condvar) · `close()` (`notify_all`) · `len()`; lock `unwrap_or_else(|e| e.into_inner())` (un panic di worker già aborta il processo, A-PP9 469-474);
  `assert_impl_all!(SharedQueue<WorkerTask>: Send, Sync)`. Nessuna dipendenza nuova (Cargo.lock non ha crossbeam-channel/flume/async-channel).
  Scartati: `Arc<Mutex<std::mpsc::Receiver>>` (blocca col lock preso, niente `len`), `tokio::sync::Mutex<Receiver>` (i worker sono thread nudi senza runtime),
  `crossbeam-channel` (dipendenza per 60 righe).
- worker_pool.rs, saldo ≈ −4 righe: `senders`+`next_worker` → `queue: Arc<SharedQueue<WorkerTask>>` + `workers: usize`; `worker_loop` 542 →
  `while let Some(mut task) = queue.pop_blocking()`, **righe 543-648 INTATTE** (pin A-PP23 `execute_request(&reg, &task.meta)` prima di `response_tx.send(`,
  letture post-end = 2, `dispatched` per thread e riga `tag=worker_dispatch thr= count=` conservati); `dispatch`: via l'indice e il commento ghost, blocco census
  e undo INVARIATI (inc-before-push, undo su `Err`); `shutdown`: `queue.close()` al posto di `drop(senders)`.
- `__census_self` (solo mem-census; oggi W curl sequenziali coprono i W worker SOLO col round-robin, `main.rs` 389-397, `measure90-campaign.sh` 296-303):
  `static CENSUS_SELF_BARRIER: OnceLock<Barrier>` con W; il ramo self (588-604) fa `.wait()` prima di emettere; il handler spedisce W task in UNA richiesta e
  concatena i corpi ⇒ ogni worker libero ne prende esattamente uno (precondizione dichiarata: pool quiescente, già garantita a 293-295).
- Knob round-robin: **NON tenerlo** (due vie al cap + riga di matrice cfg). Ogni campagna census è a W=1 (KB-78-3, A-PP28) dove FIFO condivisa ≡ round-robin.
- Dente deterministico `wp189-harness/gate-loopback/gate-loopback.sh` + fixture `loop.php` (`usleep(d)` poi `curl` a `pong.php` con `CURLOPT_TIMEOUT 3`; stampa
  `OK ms=N` / `ERR errno=28 ms=N` — curl e NON `file_get_contents`, che non ha timeout) e `pong.php`. Due curl concorrenti A (`d=600`) e B (`d=300`) con
  W=3: round-robin ⇒ A→k, B→k+1, interna di B→k+2 (OK), interna di A→k+3≡k ⇒ **A = ERR28 ≥ 2500 ms** (controllo POSITIVO sul pin vecchio 97b58fd225ca7289);
  coda condivisa ⇒ entrambe OK < 1000 ms. Bracci: pin vecchio W=3 · nuovo W=3 · entrambi W=1 (ERR28 in [2500;4000]: fedeltà) · oracle `php -S` con
  `PHP_CLI_SERVER_WORKERS=3` (OK) e senza (ERR28). 3 ripetizioni con restart, esito identico richiesto. rc 0/1, verdetto fuori repo.
- Test unitari in dispatch.rs: N produttori/M consumatori ogni id una volta sola; drena-poi-esce; push dopo close restituisce il task; close sveglia il consumer
  parcheggiato. Test esistenti che restano validi INVARIATI: `census_queue_depth_burst_positive` (1801, W=1), `census_queue_depth_no_underflow_inc_before_send`
  (1841), tripwire 1911/1919, `shutdown_joins_all_workers_after_drain` (2029).

## 5. I NUMERI (S-188, pin server s188 `97b58fd225ca7289`, 0 VOID)
A/P = `--axum --workers 1` / modo nativo: front 1,00 · post 1,00 · feed 1,00 · login 1,00 · rest 1,00 (t3). A/O: 9,61 · 9,81 · 10,58 · 13,07 · 9,80.
Throughput `ab` O4/AX (oracle 4 worker / axum 4 worker): C=1 8,39–12,52 · C=4 8,42–12,00 · C=8 10,31–14,47; AX scala 3,96–4,10× C1→C4 e satura a C8 (1,00).
Il divario con PHP è il MOTORE (A/P = 1,00): la coda non è una leva di prestazioni e non viene presentata come tale.

## 6. LETTERE GIÀ ACQUISITE (non ri-litigare; dire per ognuna se INVARIATA o DA EMENDARE)
| # | lettera | testo (una riga) | stato col candidato |
|---|---|---|---|
| 1 | WP-77.5 conv. #2 (`COUNCIL_WP77.5_REVIEWS.md:18-33`) | worker-attore stile php-fpm, N thread OS proprietari della Vm, «mpsc bounded + oneshot, 503 su canale pieno» | thread-attore INVARIATO; «bounded + 503» MAI implementato (oggi unbounded): da dichiarare superato o aprire per nome (Q3) |
| 2 | A-MS2/A-MS3 (`COUNCIL_WP78_REVIEWS.md:40,130-131`) | RetainSet `!Send+!Sync` thread-affine; `assert_impl_all!(WorkerTask: Send)` | INVARIATA (la coda muove solo `WorkerTask`) |
| 3 | A-DS2 / KS-DS-78-1 (`:291,65`) | statics/closure_statics/static_props per-richiesta come FPM; richiesta N≥2 stampa ≠1 ⇒ FATAL | INVARIATA |
| 4 | KH78-2 / KB-78-2 (`:60,104,218`) + A-BB8 (`design78.md:121`) | profondità coda mpsc >1 in misura ⇒ VOID; depth campionata a ogni dispatch | semantica da RI-DEFINIRE: una coda sola (a W=1 identica) |
| 5 | A-TH9 (`COUNCIL_WP79_REVIEWS.md:85`), KH81-1/2 (`COUNCIL_WP81_REVIEWS.md:60-66`) | depth = semantica CODA; tripwire d==0 ⇒ abort; IN_FLIGHT separato | compatibile (più pulita con una coda) |
| 6 | A-TH15 (`wp82…:168-172`), A-TH22 (`wp83…:166`), KS-PP-83-3 | OUTSTANDING inc-before-send/dec-after-send; `inflight_max==1` | INVARIATA |
| 7 | A-BB17 / KB-81-1 (`wp81…:99,182,320,327`) | righe split/resid DEFINITE solo a W=1 | INVARIATA |
| 8 | **A-PP36** (`wp87…:323`) · **KS-PP-87-3** (`:161`) | dispatch-count per-thread in-band; «cifra net da run senza dispatch-count==atteso ⇒ envelope» | il contatore RESTA; «atteso» a W>1 diventa Σ==nreq: DA EMENDARE |
| 9 | **A-PP37** (`wp88…:131`) | «verdict esige la MAPPA per-thread == attesa (non la sola somma); ghost next_worker dichiarato» | DA EMENDARE: mappa W>1 non deterministica, ghost sparisce |
| 10 | **A-PP42 / KS-PP-89-2** (`wp90…:154`), `design89.md:64,81` | VDISP = insieme esatto {0..W-1} e count==nreq/W; «mai assumere il round-robin in silenzio» | DA EMENDARE (⊆ {0..W-1}, Σ==nreq) |
| 11 | protocollo `__census_self` (`main.rs` 389-397) | W chiamate sequenziali coprono i W worker | DA RI-DISEGNARE (barriera, §4) |
Parser che leggono la mappa: `wp86…/verdict86.sh:66`, `va-census-template.sh:17`, `verdict87.sh:69`, `verdict88.sh:109`, `verdict89.sh:163`, `verdict90.sh:216,223`;
lettori di `depth_max=`/`inflight_max=`: `measure78.sh:318,404`, `verdict81.sh:28,99,245`. Precedente: S-186 ha già declassato il census per-thread a calibrazione.

## 7. LE DOMANDE (verdetto separato per ciascuna: CONCORDO / CONCORDO CON EMENDAMENTI / MI OPPONGO, con alternativa scelta)
**Q1 ROTTA (GO/NO-GO)**: (A) coda condivisa FIFO come in §4 · (B) code per-worker + dispatch al primo worker LIBERO con fallback round-robin (meno righe,
ma sotto saturazione resta cieca: più debole dell'accept condiviso) · (C) status quo dichiarato (`--workers ≥ 4`, cron spento in misura, §3.9 (h) permanente,
il difetto resta in produzione). Attesa del relatore: A.
**Q2 CONTRATTO DI MISURA**: come si emendano le lettere 4, 8, 9, 10, 11: (i) invariante = Σcount == nreq e insieme ⊆ {0..W-1}; la mappa a W>1 è ADVISORY;
W=1 resta l'unico modo di census (KB-78-3) · (ii) knob `rr` per le campagne (due vie al cap, matrice cfg) · (iii) altro. Barriera per `__census_self`: accettata
o alternativa (si ricordi che un worker fermo in `pop_blocking` non può ascoltare un secondo canale).
**Q3 FORMA E GATE**: il dente `gate-loopback` (controllo positivo rosso sul pin vecchio, verde sul nuovo, fedeltà a W=1, oracle) basta come accettazione?
Soglia throughput pre-registrata: per URL, mediane AX a C=4 e C=8 ≥ 0,97 × `s188-throughput-verdetto-t1.out`, C=1 entro ±5 %, spread ≤ 5 %, R=3 (o: non
rimisurare e dichiarare «A/P = 1,00 per costruzione a W=1»?). «Bounded + 503» del WP-77.5: superato (il backlog del kernel è unbounded) o apertura per nome?
Un timeout nel `rx.await` del handler: sì/no (fedeltà: `php -S` non ne ha; `max_execution_time` è §3.38).
**Kill-switch proposti** (condizione meccanica → atto): KS-189-1 controllo positivo NON rosso sul pin vecchio ⇒ dente nullo, nessuna promozione ·
KS-189-2 throughput AX C=4 o C=8 < 0,97× del t1 su una URL ⇒ cura NON promossa (resta in branch) · KS-189-3 test census esistente emendato senza voce nel
verbale ⇒ incidente · KS-189-4 `battery61-axum` < 6/6 o G-APERTURA-2 FAIL dopo la cura ⇒ HALT.

## 8. FORMA DEI VERBALI (fase 1, indipendenti) → `verbali/<cognome>.md` ≤ 120 righe
`## VERDETTI` (Q1/Q2/Q3 con alternativa) · `## ROTTA (una riga)` · `## REFUTAZIONI ED EMENDAMENTI` R1..Rn con riga di codice o documento ·
`## KILL-SWITCH` (condizione → atto) · ricevuta in chat ≤ 80 parole. Fase 2: tre team (`team-meccanismo` Hoare·Matsakis·Hejlsberg·Bak; `team-sonda`
Gregg·Pedersen·Leijen·web-runtime; `team-governo` Klabnik·Stogov), poi la sintesi `COUNCIL_WP189_REVIEWS.md`. Le scritture su riga 1 del NEXT, sulle
lettere e qualsiasi riga di codice entrano SOLO con la firma dell'utente.
