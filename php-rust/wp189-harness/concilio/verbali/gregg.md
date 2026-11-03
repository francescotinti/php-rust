# Verbale sedia GREGG — S-189 (bozza indipendente, FASE 1)
Generato con assistenza AI (Claude Fable 5). Lente: metodologia di misura e attribuzione. Mandato: refutare, mai benedire.
Fonti lette: `COUNCIL_DOSSIER.md`; `wp188-harness/s188-throughput-verdetto-t1.out`; `s188-criterio-throughput.md`; `s188-criterio-server-pair.md` p.11–13;
`revisione-s188-t1.md`; `wp78-harness/gate-axum/run-gate.sh:1-120`; `measure78.sh:310-325,400-410`; `COUNCIL_WP77.5_REVIEWS.md:225-227`;
`crates/php-server/src/worker_pool.rs:542-650, 690-746`; `crates/php-server/src/main.rs:389-440`.

## VERDETTI
- **Q1 ROTTA: CONCORDO CON EMENDAMENTI su (A)**. Dalla mia lente A e B sono INDISTINGUIBILI dal dente proposto (R4): la scelta A/B spetta alle sedie
  di meccanismo e fedeltà; io pongo le condizioni perché la promozione sia misurabile. (C) respinta: lascia un difetto di produzione non coperto da alcun dente.
- **Q2 CONTRATTO: CONCORDO con (i)** (Σcount == nreq, insieme ⊆ {0..W-1}, mappa W>1 ADVISORY, W=1 unico modo di census). Knob `rr` (ii): MI OPPONGO
  (seconda via al cap, una matrice cfg in più per salvare un invariante che vale solo a W=1 dove le due code coincidono). Barriera `__census_self`: accettata
  SOLO con timeout e abort (R8). `depth_max`/`inflight_max`: il loro significato NON cambia (R7).
- **Q3 FORMA E GATE: CONCORDO CON EMENDAMENTI**. Il dente `gate-loopback` come scritto nel dossier NON è un controllo positivo deterministico (R1):
  senza sfasamento il rosso sul pin vecchio è un lancio di moneta. Soglia throughput «0,97× del t1»: MI OPPONGO, alternativa = A/B interallacciato nella stessa
  finestra (R5). p99: sì, companion (R6). A/P a W=1: NON ri-registrare, ma tripwire di non-regressione obbligatorio (R9). «Bounded + 503»: dichiarare
  SUPERATO e aprire per nome «backpressure sotto overload» (R10). Timeout nel `rx.await`: no (fedeltà), ma il curl ESTERNO del dente deve averne uno (R2).

## ROTTA (una riga)
A, promossa SOLO se: dente sfasato con firma esatta e log-gate (R1–R3) rosso 3/3 sul pin vecchio e verde 3/3 sul nuovo; throughput A/B interallacciato
vecchio/nuovo entro [0,97; 1,05] con p99 registrato; tripwire A(W=1) nuovo/vecchio entro ±2 %; lettere 4/8/9/10/11 emendate per (i).

## REFUTAZIONI ED EMENDAMENTI
- **R1 — La coreografia «due curl concorrenti» non è un controllo positivo** (dossier §4 r.62-63). Con W=3 e slot di partenza s qualsiasi (mod 3 l'analisi
  non dipende da s): se A è dispacciata prima, A→s, B→s+1, B'(t=300)→s+2 libero OK, A'(t=600)→s+3≡s = worker di A ⇒ A ERR28. Se invece B è dispacciata
  prima: B→s, A→s+1, B'→s+2 OK, A'→s+3≡s = worker di B, che a t=600 è GIÀ LIBERO (B ha finito a ~310 ms) ⇒ **entrambe OK sul pin vecchio**: KS-189-1
  dichiarerebbe «dente nullo» per caso. L'ordine di dispatch di due curl lanciate insieme dalla shell non è controllato: né l'ordine di `accept`, né — su runtime
  tokio multi-thread — l'ordine in cui due handler superano `route(...).await` (`main.rs:411`) e raggiungono `pool.dispatch` (`:426`). Emenda: sfasamento
  dichiarato: A (d=800) parte a t=0, B (d=300) a t=+150 ms, dopo che il lanciatore ha VISTO la connessione di A stabilita (`lsof -iTCP:PORT -sTCP:ESTABLISHED`
  o `curl --trace-time` con riga «Connected»), non un `sleep` cieco. Margini risultanti ≥ 300 ms fra ogni coppia di eventi (A 0 · B 150 · B' 450 · B fine ~460 ·
  A' 800). Ogni fixture stampa `t0=<microtime>` all'ingresso e `tc=<microtime>` prima della curl interna: l'ordine A.t0 < B.t0 < B.tc < A.tc va VERIFICATO dal
  giudice su ogni ripetizione, non supposto.
- **R2 — Firma esatta, non «A rossa»**. Un estraneo che si infila fra B e B' sposta gli slot: estraneo→s+2, B'→s = worker di A, A'→s+1 = worker di B ⇒ stallo
  incrociato, ENTRAMBE ERR28 (~3 s). Firma distinta dallo stallo semplice, quindi il giudice deve pretendere ESATTAMENTE: pin vecchio W=3 `A=ERR errno=28
  ms∈[2800;3300]` ∧ `B=OK ms<700`; nuovo W=3 entrambe `OK ms<1000`; W=1 (fedeltà, entrambi i binari e l'oracle a 1 worker) `A=ERR28` ∧ `B=ERR28`, con
  wall ESTERNO di B ≈ 6 s (B entra in coda dietro A, poi stalla a sua volta): il curl della shell DEVE avere `-m 15`, altrimenti a W=1 perde l'output della
  fixture e il braccio di fedeltà è cieco. `errno` confrontato col token dell'oracle (libcurl vero), non solo «ERR»: se la facciata ureq mappasse il timeout su
  un errno diverso il rosso sarebbe «per la ragione sbagliata» (`php-builtins/src/curl.rs:543-548`).
- **R3 — Evidenza minima per ripetizione** (file `<rep>.evidence`, fuori repo come il verdetto): (a) hash sha256[0:16] del binario e W, come
  `run-gate.sh:93-96`; (b) pid in LISTEN sulla porta == pid spawnato (KS-SK-79.1, `run-gate.sh:109-114`), porta DEDICATA ≠ 8080 e ≠ 8199 (gate-axum),
  verificata libera con `lsof` PRIMA del boot e verificata vuota dopo il kill+wait (riuso porta fra ripetizioni); (c) il log del server (`[code]: METHOD URI`,
  identico fra cli-server e axum, criterio p.13) con conteggio ESATTO = sonde di readiness + 4 (loop ×2, pong ×2) e insieme delle URI ⊆ {loop.php, pong.php,
  sonda}: una riga in più = ripetizione VOID (lezione t1: la scheda browser non la vede nessuna guardia di processo, `revisione-s188-t1.md` #1); (d) la riga
  `pong.php` di A' deve comparire DOPO l'ERR di A sul pin vecchio (il worker la serve a socket chiuso dopo lo stallo: è la prova fisica che era in coda sul
  worker di A); (e) l'output delle quattro fixture con t0/tc/ms/errno. La riga `tag=worker_dispatch thr= count=` (A-PP36) esiste solo nei binari census: NON è
  disponibile sul pin, non può essere prova del dente; se la si vuole, va aggiunta come riga di log `PHPR_LOG=debug` a ogni dispatch (`idx`, path) in un commit
  separato dalla cura, mai nel binario di misura. Readiness su file STATICO (`Routed::Ready`, `main.rs:412`), che non avanza `next_worker`.
- **R4 — Controlli di sanità del dente**: (a) braccio NEGATIVO: A da sola sul pin vecchio W=3 ⇒ OK (prova che loopback e fixture funzionano e che il rosso
  viene dallo slot, non da pong.php o dalla facciata); (b) il dente non discrimina (A) da (B): con «primo worker libero» B'→s+2 libero e A'→s+2 di nuovo libero
  ⇒ entrambe OK anche con (B). La differenza A/B vive solo sotto saturazione, come coda d'attesa e non come stallo: osservabile nel p99 a C=8 (R6), non in un
  pass/fail. Va scritto nel verbale, altrimenti un verde del dente verrà letto come prova della FIFO. Indirizzi: sempre `127.0.0.1` in fixture e shell, mai
  `localhost` (happy-eyeballs ::1 → errno di connessione, non 28). Nessun `--retry` di curl; `Connection: close` non serve (nessuna misura in gioco).
- **R5 — Soglia «0,97× del t1» indifendibile**: gli spread di t1 sono 0,4–2,8 % (AX) e 0,4–3,8 % (O4) GIÀ nella stessa finestra; fra giorni diversi (classe di
  core 2,09 GHz clampata, termica, Spotlight) una deriva ≥3 % su almeno una di 15 celle è attesa per solo rumore ⇒ falso kill probabile, e una regressione vera
  del 2 % passerebbe. Emenda: criterio pre-registrato con tre bracci interallacciati nella STESSA finestra, ordine `AXold, AXnew, O4` per ciclo, R=3, N=150,
  C∈{1,4,8}: canonica = `AXnew/AXold` req/s per cella (rapporto nello stesso ciclo, mediana dei 3); gate di cella spread ≤5 % su ciascun braccio; accettazione
  = ogni cella C=4/C=8 ∈ [0,97; 1,05] e C=1 ∈ [0,95; 1,05]; mediana fuori banda con un ciclo dentro ⇒ cella VOID (rimisura), non kill; >1,05 è un REPERTO da
  attribuire PRIMA di promuovere (una coda non deve regalare throughput: se lo fa, era il round-robin a sbilanciare 8 connessioni su 4 code). `O4/AX` resta
  companion di continuità col t1 (banda ±10 %, advisory). Warm-up: con FIFO + `notify_one` chi prende la richiesta a C=1 non è più deterministico; il warm-up
  `-n 20` va fatto a `-c W` (tutti occupati ⇒ tutte le unit cache per thread calde), non a C=1.
- **R6 — p99 sì, come companion con direzione dichiarata**: a parità di throughput (legge di Little: `t/req` medio identico) l'UNICO osservabile throughput-side
  della cura è la coda: registrare le righe `50%`/`90%`/`99%` di `ab` per cella; attesa pre-registrata: p99 C=8 nuovo ≤ vecchio; un peggioramento >10 % è
  REPERTO. Registrare anche user/req e cpu/req come già in t1 (sono il metro del motore e devono restare 1,00 ± rumore fra i due pin).
- **R7 — Il census conserva significato senza toccare i lettori**: `QUEUE_DEPTH` è GIÀ un contatore globale (inc al dispatch `worker_pool.rs:710`, dec al
  pickup `:549`), quindi `depth_max` misurava la somma delle code per-worker e ora misura la coda sola: a W=1 è la stessa grandezza, `measure78.sh:318`
  (`depth_max>1 ⇒ VOID`) e `:404` (`inflight_max==1`) restano validi senza emenda. `OUTSTANDING` inc-before-push/dec-after-send (`:713`, `:632`) invariato.
  A-PP37: il ghost sparisce per costruzione (`push` restituisce il task, nessun indice avanzato): da dichiarare CHIUSA, non emendata. Mappa per-thread: a W=1
  `{0: nreq}` byte-identica ⇒ `verdict86..90.sh` invariati; a W>1 Σ e ⊆ (Q2-i). Non aggiungere tripwire `len()==depth` (inc-before-push rende solo
  `depth ≥ len` vero sempre: zero potere).
- **R8 — Barriera `__census_self`**: corretta (un worker fermo alla barriera non può estrarre un secondo task), ma `std::sync::Barrier` non ha timeout: se la
  precondizione «pool quiescente» è violata il processo PENDE per sempre, lo stato «silently dead» che KS-PP-6 vieta. Emenda: contatore + `Condvar::wait_timeout`
  (5 s) ⇒ `eprintln` + `abort` (stile A-MS31 «instrument, not worker», `worker_pool.rs:599-602`). A W=1 la barriera è degenere e la riga è identica a oggi; a
  W>1 la riga porta `w=W tasks=W grade=advisory` perché UNA richiesta HTTP produce W ARRIVALS (`:718`) e `inflight_max=W`.
- **R9 — A/P a W=1: non ri-registrare, ma non «per costruzione» senza tripwire**. Ciò che cambia sul thread del worker è solo `blocking_recv` (park/unpark
  su semaforo tokio, con allocazione a blocchi di 32 slot per send) contro `Condvar::wait` + `VecDeque`: ordine 10²–10³ ns per richiesta contro 110–360 ms
  di user-CPU/req (t1): sotto il rumore di 2 ordini di grandezza, e nel dominio sys/idle, non user. Record t3 resta la cifra; però la dichiarazione va
  accompagnata da un tripwire che costa ~8 min: due bracci `Aold, Anew` (W=1) × {front, login} × R=3 × K=30 nella harness della coppia, `|Anew/Aold − 1| ≤ 2 %`
  (soglia VOID della coppia), etichettato «non-regressione», mai pubblicato come cifra.
- **R10 — «Bounded + 503» del WP-77.5 (`COUNCIL_WP77.5_REVIEWS.md:18-33`)**: SUPERATO, con precisione: il backlog del kernel di `php -S` NON è unbounded
  (SOMAXCONN; oltre, SYN scartato e il client attende/ritenta a livello TCP); phpr accetta tutto e tiene in RAM un `WorkerTask` con body per ogni richiesta in
  attesa. La differenza non è «503 o no» ma «chi tiene la coda sotto overload»: aprire per nome in §3.9 «backpressure: cap di coda N×W con pausa dell'accept,
  nessun 503», fuori da questa cura. Timeout nel `rx.await` del handler: no, fedeltà (`php -S` non ne ha; §3.38).

## KILL-SWITCH
| # | condizione (meccanica) | atto |
|---|---|---|
| KG-189-1 | pin vecchio W=3 con sfasamento R1: firma `A=ERR28 ∧ B=OK` NON ottenuta 3/3 | dente NULLO, nessuna promozione |
| KG-189-2 | firma diversa (B ERR, errno≠28, ms fuori banda) o log del server con righe/URI oltre le attese (R3c) | ripetizione VOID; 2 VOID consecutive ⇒ coreografia non idonea, si riprogetta |
| KG-189-3 | braccio negativo (A da sola, pin vecchio W=3) NON OK, oppure errno phpr ≠ errno oracle a W=1 | fixture o facciata rotte: dente NULLO fino a cura |
| KG-189-4 | pid in LISTEN ≠ pid spawnato, o porta non libera prima del boot, o nuovo W=3 non verde 3/3 | run NULLA |
| KG-189-5 | throughput A/B (R5): cella C∈{4,8} fuori [0,97; 1,05] o C=1 fuori [0,95; 1,05] con i 3 cicli concordi | cura NON promossa (resta in branch) |
| KG-189-6 | tripwire A(W=1) `|Anew/Aold − 1| > 2 %` su front o login | HALT: reperto da attribuire prima di qualsiasi pin |
| KG-189-7 | p99 C=8 nuovo > 1,10 × vecchio su una URL, o cpu/req nuovo/vecchio fuori ±2 % | REPERTO a verbale, promozione sospesa finché attribuito |
| KG-189-8 | barriera census senza timeout+abort nel diff, o test census esistente emendato senza voce nel verbale | incidente (= KS-189-3) |
| KG-189-9 | `battery61-axum` < 6/6 o G-APERTURA-2 FAIL (`run-gate.sh`) dopo la cura | HALT (= KS-189-4) |
