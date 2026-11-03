# Verbale sedia STOGOV — S-189 (bozza indipendente, FASE 1)
Generato con assistenza AI (Claude Fable 5).
Lente: semantica Zend / SAPI / fedeltà all'oracle (`php -S` cli-server, php-fpm). Fonti lette riga per riga:
`sapi/cli/php_cli_server.c` 385-393, 780-790, 2388-2410, 2495-2600, 2700-2790; `main/fastcgi.c` 1385-1405;
`ext/standard/http_fopen_wrapper.c` 460-495; `sapi/cli/php_cli.c` 116-120, 1334-1336; `main/main.c` 840, 2631-2635;
`Zend/zend_execute_API.c` 1606-1618; Rust `file.rs` 493-560, `curl.rs` 381-386/543-548, `worker_pool.rs` 504-660, `main.rs` 405-440.

## VERDETTI
- **Q1 ROTTA: CONCORDO CON EMENDAMENTI su (A)** coda condivisa FIFO. È la semantica dell'oracle (backlog del kernel +
  `accept` da ogni processo libero: cli-server 2713-2716, fpm 1396-1398), NON un'ottimizzazione. (B) è più debole dell'accept
  condiviso sotto saturazione; (C) lascia in produzione un comportamento che l'oracle a N worker NON ha. Emendamenti R1-R4.
- **Q2 CONTRATTO DI MISURA: CONCORDO con (i)**: invariante Σcount == nreq, insieme ⊆ {0..W-1}, mappa ADVISORY a W>1, W=1 unico
  modo di census (KB-78-3). Knob `rr` (ii): MI OPPONGO (due semantiche nel binario di parità = due SAPI). Barriera
  `__census_self`: accettata con R7 (timeout di strumento → abort, mai attesa infinita).
- **Q3 FORMA E GATE: CONCORDO CON EMENDAMENTI** (R5, R6). Il dente basta come accettazione SEMANTICA; la soglia throughput
  è giusta ma non è di mia lente. «Bounded + 503» del WP-77.5: **SUPERATO**, con motivazione corretta (R8). Timeout nel
  `rx.await` del handler: **NO, è un'INVENZIONE** (R9). W=1 DEVE continuare a stallare (R3).

## ROTTA (una riga)
GO su (A): la FIFO condivisa è la trascrizione userland del backlog del kernel di `php -S`/fpm; W=1 stalla per fedeltà, nessun
timeout lato server, §3.38 (timer di esecuzione) resta la sola cura fedele allo stallo, §3.39 a catalogo senza bloccare il gate.

## REFUTAZIONI ED EMENDAMENTI
**R1 — Il dossier §3 è incompleto: il MASTER di `php -S` serve anch'esso.** `php_cli_server_startup_workers` (2503-2545) forka
N figli e RITORNA nel master, che prosegue in `php_cli_server_poller_ctor` + `poller_add(server_sock)` (2583-2586) ed entra nello
stesso event loop; nessun ramo `getpid()==master ⇒ non accettare` (l'unico uso del master è il `waitpid` del dtor, 2388-2410, e il
prefisso `[pid]` del log, 787). ⇒ `PHP_CLI_SERVER_WORKERS=N` = **N+1 acceptor**. Emendamento al dente §4: il braccio oracle «W=3»
va lanciato con `PHP_CLI_SERVER_WORKERS=2` (3 acceptor) o dichiarato «4 acceptor»; il braccio «senza» (1 processo) è corretto.
Nota a §3 del dossier e a §3.9 (h): «N worker» → «N figli + master».

**R2 — Backlog del kernel ≡ FIFO userland: CONFERMO, con due proprietà da dichiarare.** (a) Ogni acceptor prende la connessione
PIÙ VECCHIA pendente; un processo che esegue uno script non entra nel poller (loop 2770-2777 è single-thread per processo) ⇒ non
accetta: identico a un worker fermo in `execute_request`. (b) **Esecuzione fantasma**: se il client interno scade (errno 28) la
connessione resta nel backlog con la richiesta già scritta; il primo acceptor libero la esegue comunque e scrive su socket morto
(oracle) / `response_tx.send(...).ok()` su oneshot abbandonato (`worker_pool.rs:620`, `main.rs:437-440` mai raggiunto). Stessa
semantica, inclusi gli effetti collaterali di un POST eseguito a client sparito. Il dente deve DRENARE o riavviare fra le
ripetizioni: col round-robin il fantasma consuma un indice e cambia la mappa k della prova successiva (il «restart» di §4 è
OBBLIGATORIO, non prudenziale).

**R3 — Keep-alive: nessun impatto sulla delibera.** La cli-server scrive `Connection: close` su OGNI risposta
(`php_cli_server.c:393`): una connessione = una richiesta ⇒ coda di connessioni ≡ coda di richieste. In axum hyper tiene la
connessione e serializza le richieste di una stessa connessione; la seconda entra nel pool come task nuovo. I loopback di WordPress
(`Requests` → curl) aprono un handle per chiamata: equivalente. Resta residuo §3.9 (b), già a catalogo: nessun emendamento.
**W=1 deve stallare: SÌ.** `php -S` a 1 processo stalla fino al timeout del client (identico); Site Health segnala «loopback
fallito», ed è ciò che vede l'utente PHP. Qualunque «cura» a W=1 (richiesta interna inline, worker extra) sarebbe un SAPI che
l'oracle non ha. Esito fedele: `WP_Error` dopo il timeout del client, come in tabella §2.

**R4 — §3.38 è la vera variabile dello stallo, e sul Mac di misura conta il WALL clock.** `HARDCODED_INI` «max_execution_time=0»
vale SOLO per `cli_sapi_module` (`php_cli.c:1334-1336`): sotto `php -S` vige il php.ini (30, `main.c:840`), armato a ogni richiesta
(`main.c:2631-2635`). Su Apple Silicon ≥ macOS 14 Zend arma `ITIMER_REAL` (`zend_execute_API.c:1610-1614`, SIGALRM), su Linux
`ITIMER_PROF`/timer CPU: il tempo d'attesa sul socket CONTA solo sul Mac. ⇒ L'oracle `php -S` tronca la richiesta ESTERNA dell'editor
temi (timeout 100 s, §2 B) a 30 s di parete con banner 200, prima che il loopback scada; phpr (nessun orologio) aspetta i 100 s e
poi reverte la modifica. Due esiti diversi dallo stesso difetto: §3.38 va legato a §3.9 (h) nel catalogo e il dente (3 s) va tenuto
SOTTO 30 s (lo è). Emendamento §3.38: aggiungere la nota «macOS arm64: wall clock; Linux: CPU — la fixture busy33 è neutra, un
loopback NO».

**R5 — §3.39 (wrapper `http://` senza timeout): testo proposto e NON bloccante per il gate.** Oracle: `timeout` del contesto o
`FG(default_socket_timeout)` (60 s) applicato a connect E lettura (`http_fopen_wrapper.c:460-495`,
`PHP_STREAM_OPTION_READ_TIMEOUT`). phpr: `parse_http_options` (`file.rs:493-531`) legge method/content/header/ignore_errors/
max_redirects e IGNORA `timeout`; l'`Agent` di `http_get` (553-557) non imposta `timeout_global` ⇒ `file_get_contents('http://…')`
attende a oltranza; `stream_context_create(['http'=>['timeout'=>n]])` è silenziosamente ignorato (viola correct-or-absent: va
almeno un `TODO(port)` esplicito). Voce §3.39: «🟡 wrapper `http://`: nessun timeout; PHP onora `timeout` di contesto o
`default_socket_timeout`=60 s su connect+read. Effetto: un loopback via stream (non via curl) stalla senza fine a W=1; in WordPress
i loopback passano da `Requests`→curl con `CURLOPT_TIMEOUT` (`curl.rs:381-386`, `timeout_global` 543-548) ⇒ NON coinvolti».
Il gate usa curl ⇒ CONFERMO: §3.39 non va curato prima di fidarsi del gate. Cura (fuori perimetro S-189): leggere `timeout`
(double, secondi) e in difetto `ini default_socket_timeout` → `timeout_global`; verificare che `ini_get('default_socket_timeout')`
riporti 60.

**R6 — Il dente deve registrare il contratto della misura `ms=N`.** `loop.php` misura il solo tempo della curl: a W=1 l'atteso è
≈ 3000 ms (CURLOPT_TIMEOUT 3), non `usleep+3000`; finestra [2500;4000] coerente SOLO se `ms` esclude la `usleep`. Dichiararlo nel
file del dente. Aggiungere al braccio oracle W=1 il controllo che la risposta esterna sia 200 (nessun fatal di §3.38 entro 3,6 s).

**R7 — Barriera `__census_self`: solo sotto `#[cfg(feature = "mem-census")]`, con scadenza.** Un worker fermo in `pop_blocking`
non ascolta altri canali; se la precondizione «pool quiescente» (`main.rs` 293-295) è violata la barriera attende per sempre e il
census si blocca in silenzio. Emendamento: `Barrier` con `wait` a tempo (istrumento) o verifica `OUTSTANDING == W` prima del wait;
violazione ⇒ `abort` con riga «instrument, not worker» (stile A-MS31, `worker_pool.rs:598-603`).

**R8 — «Bounded + 503» (WP-77.5): superato, ma la motivazione del dossier è falsa.** Il backlog del kernel NON è «unbounded»: è
limitato da `listen(2)`/somaxconn (macOS 128); al pieno il kernel scarta il SYN (ritardo di connect / `ECONNREFUSED`), e l'oracle
non emette MAI 503 (né cli-server né fpm, che ha `listen.backlog`). Un 503 di coda piena sarebbe un'invenzione. Dichiarare:
«coda interna unbounded (oggi) + backlog di hyper»: divergenza di SOVRACCARICO, non di semantica; voce a catalogo in §3.9.

**R9 — Timeout nel `rx.await` del handler: INVENZIONE, da respingere.** `php -S` non ha alcun timeout lato server oltre il timer
di esecuzione (§3.38, esito 200 + banner, mai 5xx) e il timeout del CLIENT. Un timeout sul handler (`main.rs:432`) risponderebbe
5xx mentre il thread del worker resta bloccato in ureq (nessuna cancellazione possibile): il client vedrebbe «libero» un worker
ancora occupato, `OUTSTANDING` divergerebbe dal filo (A-TH9) e si introdurrebbe uno status che l'oracle non produce. Cura fedele:
§3.38 (timer di esecuzione, fatal 200 + banner). Nessuna riga in `main.rs`.

**R10 — Lettere §6: 3 (A-DS2/KS-DS-78-1) INVARIATA** — la coda muove `WorkerTask`, la Vm nasce e muore sul thread (`worker_loop`
541-575, 619-620); FIFO non cambia l'isolamento. **1 (WP-77.5) DA DICHIARARE SUPERATA** per R8. **4, 8, 9, 10, 11 DA EMENDARE** come
in Q2 (i). Residuo dossier §1: «probabilità 1/W» è deterministica data la sequenza d'arrivo; dire «1 worker su W».

## KILL-SWITCH
| # | condizione (meccanica) | atto |
|---|---|---|
| KS-DS-189-1 | braccio oracle del dente lanciato con `PHP_CLI_SERVER_WORKERS=3` e presentato come «3 worker» | verdetto oracle VOID (R1: sono 4 acceptor) |
| KS-DS-189-2 | braccio W=1 (phpr o oracle) NON stalla (`OK` invece di `ERR errno=28`) | HALT: fedeltà rotta o fixture sbagliata |
| KS-DS-189-3 | qualunque diff a `main.rs` 420-440 che aggiunga timeout/`select` sul `rx.await` | REJECT d'ufficio (R9) |
| KS-DS-189-4 | fixture static-counter G-APERTURA-2 (KS-DS-78-1) stampa ≠1 dopo la cura | FATAL, nessuna promozione |
| KS-DS-189-5 | ripetizione del dente senza restart/drain fra le prove | prova VOID (R2 b) |
| KS-DS-189-6 | barriera `__census_self` senza scadenza o compilata fuori da `mem-census` | REJECT (R7) |
| KS-DS-189-7 | §3.39 scritta senza «curl non coinvolto» o cura di §3.39 messa a prerequisito del gate | emendare prima della firma (R5) |
| KS-DS-189-8 | un 503/504 di coda piena o d'attesa introdotto nel server | REJECT: status mai prodotto dall'oracle (R8, R9) |
