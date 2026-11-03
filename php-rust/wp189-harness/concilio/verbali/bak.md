# Verbale sedia BAK — S-189 (bozza indipendente, FASE 1)
Generato con assistenza AI (Claude Fable 5).
Lente: VM di produzione (V8/HotSpot) — run queue condivisa, head-of-line, convoglio sul lock, calore delle cache, saturazione.

## VERDETTI
- **Q1 ROTTA: CONCORDO con (A)**, coda condivisa FIFO. (B) «primo libero + fallback round-robin» è una coda condivisa fatta male: sotto
  saturazione (tutti occupati) ricade ESATTAMENTE nel difetto; (C) lascia in produzione uno stallo che non è probabilistico ma DETERMINISTICO (R1).
  Nessun server di produzione (accept condiviso di `php -S`/FPM, run queue di V8/Node cluster, HotSpot thread pool) assegna il lavoro a un
  thread che non ha chiesto lavoro: il round-robin cieco è l'anomalia, la coda è la norma. Non è una leva di prestazioni e non va venduta così (R4).
- **Q2 CONTRATTO: CONCORDO CON EMENDAMENTI su (i)** — Σcount == nreq, insieme ⊆ {0..W-1}, mappa ADVISORY a W>1, W=1 unico modo census.
  Emendamento: per `footprint(W)` (design78.md:103-104, «≥100 richieste PER worker») la riga `worker_dispatch thr= count=` diventa il
  TESTIMONE: `min(count) ≥ soglia dichiarata` o run VOID (R5). Knob `rr`: NO. Barriera `__census_self`: accettata SOLO con timeout (R6).
- **Q3 FORMA E GATE: CONCORDO CON EMENDAMENTI.** Il dente a due curl concorrenti è una GARA (R2): sostituirlo col dente sequenziale a
  L=W+1 loopback dalla stessa richiesta. Soglia 0,97× vs il FILE t1: NON significativa così com'è (R3): il confronto vale solo pin vecchio
  vs pin nuovo nella STESSA finestra, interfogliati, solo braccio AX. Cella Site-Health sotto `ab`: scope creep per il record, ammessa come
  smoke ADVISORY (R7). «Bounded + 503» del WP-77.5: SUPERATO per nome, con apertura nominata «backpressure via accept» (R8).
  Timeout nel `rx.await`: NO (R9). Reserve worker / rilevamento self-connection: NO, invenzione (R10).

## ROTTA (una riga)
A con dente sequenziale deterministico, A/B throughput AX-vecchio/AX-nuovo stessa finestra (non vs t1), censimento con testimone di distribuzione e barriera a timeout; zero timeout nel handler, zero worker di riserva.

## REFUTAZIONI ED EMENDAMENTI
- **R1 — Il dossier SOTTOSTIMA il difetto: non «1/W», ma ⌊L/W⌋ stalli DETERMINISTICI.** `dispatch` (`worker_pool.rs:691-694`) avanza
  `next_worker` a ogni dispatch; su server IDLE una richiesta sul worker k che fa L loopback SEQUENZIALI manda l'i-esimo a k+i: l'i=W
  (e 2W, 3W…) colpisce k stesso. Site Health page-cache + loopback + REST = 5 loopback in una richiesta (dossier §2 D/E/F) ⇒ a W=4 UNO
  stalla SEMPRE; i 38 s del 02/11 (10+10+5+5+5 = 35 s + lavoro) dicono che TUTTI hanno stallato: era W=1 o wrapper senza timeout (§3.39).
  Viceversa il singolo loopback su server idle a W≥2 non stalla MAI con rr (k+1 ≠ k): la frase «con probabilità 1/W» va riscritta in
  §3.9 (h) così: «idle: stalli = ⌊L/W⌋ per richiesta; sotto traffico concorrente: probabilità ≈1/W per loopback». Fatto, non opinione.
- **R2 — Il dente `gate-loopback` di §4 (dossier :61-65) è una GARA, non un controllo positivo.** A (d=600) e B (d=300) partono come due
  processi: se B arriva PRIMA (B→k, A→k+1, B-inner→k+2 a t≈300 OK, A-inner→k+3≡k a t≈600: k è LIBERO da t≈310) ⇒ A=OK anche sul pin
  vecchio ⇒ «3 ripetizioni esito identico» fallisce per caso, non per difetto. Emendamento: `loop.php` fa L=W+1 curl SEQUENZIALI a
  `pong.php` (timeout 3 s) e stampa l'esito di ciascuno; pin vecchio W=3 ⇒ 1°,2°,3° OK e 4° ERR28 (3000±100 ms) — deterministico senza
  concorrenza né ordine; pin nuovo ⇒ 4/4 OK; W=1 entrambi ⇒ 1° ERR28 (fedeltà); oracle `PHP_CLI_SERVER_WORKERS=3` 4/4 OK, senza ERR28.
  Il verdetto legge il LOG richieste del server (S-188, una riga per richiesta, `main.rs:263-265`) e pretende l'ordine atteso, altrimenti
  ripetizione VOID (non FAIL). Costo: zero righe in più del dente originale.
- **R3 — La soglia «mediane ≥ 0,97 × s188-throughput-verdetto-t1.out» non misura la cura.** Spread AX in t1: 0,4–2,8 % (login C=4 2,8 %,
  `s188-throughput-verdetto-t1.out`, blocco AX); classe di core 2,09–2,11 GHz dichiarata PER boot: un drift termico/di finestra di 2–3 %
  è dentro la storia del progetto senza toccare una riga. Confrontare una finestra nuova con un file vecchio confonde la cura col clima.
  Emendamento: catena `AX-old, AX-new, AX-old, AX-new…` R=3 nella STESSA finestra, SOLO braccio AX (O4 è già di record e non cambia),
  celle C=4 e C=8 su front e login (gli estremi: 300 ms e 115 ms CPU/req), rapporto new/old per cella ≥ 0,97 E spread ≤ 5 %; C=1 ±5 %.
  Stima di costo: AX per ciclo ≈ 5 min (t1: front C1 150/3,2 ≈ 47 s è la cella più lunga) ⇒ 2 bracci × 3 cicli ≈ 30 min, non 2 h.
- **R4 — Attesa quantificata (per non benedire a posteriori):** lock+unlock non conteso ≈ 50–100 ns, sezione critica ≈ push_back; il
  dispatch gira a ≤ 35/s (login C=8: 33,1 req/s) ⇒ utilizzazione del mutex ≈ 3·10⁻⁵, convoglio IMPOSSIBILE a queste frequenze; wakeup
  Condvar ≈ 10–30 µs contro 30–90 ms di t/req (t1 C=4/C=8). Mediane: Δ atteso entro il rumore (±1 %). C8/C4 oggi = 1,00–1,01 su 5 URL e
  C4/C1 = 3,96–4,10: NESSUNA perdita da head-of-line con durate omogenee per cella ⇒ anche la coda del tail a C=8 resta uguale (8 richieste
  identiche, 4 worker: la FIFO condivisa e due-per-coda producono lo stesso ordine di servizio). Chi si aspetta un «guadagno» dalla coda
  prende un verdetto VOID per rumore come conferma: la misura è un NON-REGRESSION gate e va scritta così nel criterio.
- **R5 — §4 «il worker caldo serve di più»: FALSO come proprietà di disegno, irrilevante per il record.** `std::sync::Condvar::notify_one`
  non specifica l'ordine di risveglio; su macOS (psynch) è ≈FIFO: sveglia il worker FERMO DA PIÙ TEMPO = il più FREDDO. Il calore L1/L2 tra
  richieste da 100–300 ms di CPU con working set da MB è nullo in ogni caso; l'unico calore che conta è `UNIT_CACHE` thread_local
  (`vm/mod.rs:16108-16113`): un worker freddo paga la compilazione (canale a3) UNA volta, poi è caldo per sempre. Sotto FIFO a C=1 i 4 worker
  si alternano come oggi col rr (4 cache piene, stessa memoria); sotto LIFO (Go/crossbeam) uno servirebbe tutto. Emendamento: la politica
  è OSSERVATA, non progettata: la distribuzione `worker_dispatch thr= count=` (riga su OGNI braccio, A-PP39, `worker_pool.rs:412-415`) va
  nel verdetto a C=1; `footprint(W)` esige `min(count) ≥ 100` (il «≥100 per worker» di design78 non è più garantito dal rr). Nessun codice
  per forzare LIFO: sarebbe invenzione e romperebbe A-PP42 oltre il necessario.
- **R6 — Barriera `__census_self` (dossier :57-59): corretta nell'idea, pericolosa nella forma.** `std::sync::Barrier` non ha timeout: se un
  worker è occupato (loopback in corso, richiesta tardiva), W−1 worker restano ALLA BARRIERA e il pool INTERO è fermo finché l'ultimo non
  arriva — un hang silenzioso dell'intera misura. Emendamento: latch su `Mutex<usize>+Condvar` con `wait_timeout` (30 s), scaduto ⇒ riga
  `tag=census_self_incomplete arrived=<n> w=<W>` + abort (classe A-MS31: strumento, non misurando). La precondizione «pool quiescente»
  diventa VERIFICATA dall'esito, non assunta.
- **R7 — Cella loopback-pesante sotto `ab`: scope creep per il record** (il throughput è dell'engine, A/P=1,00). Smoke ADVISORY ammesso
  e utile: boot W=4 con cron ABILITATO, prima `GET /`, poi lettura del log: il pin vecchio mostra la catena D/E/F con un timeout (R1), il
  nuovo no. Una riga nel verbale, mai un gate.
- **R8 — «Bounded + 503» (WP-77.5): SUPERATO per nome, non in silenzio.** L'oracle non emette MAI 503 da sovraccarico: `listen(backlog)`
  del kernel è il limite e oltre il backlog i SYN vengono SCARTATI (il client vede connect timeout). Il 503 sarebbe una risposta che PHP non
  dà. Ma tokio accetta tutto con avidità: la coda condivisa è UNBOUNDED e sotto overload la memoria cresce dove `php -S` rifiuta a monte.
  Apertura nominata «backpressure via accept» (cap di in-flight che SOSPENDE l'accept, fedele al backlog), fuori S-189.
- **R9 — Timeout nel `rx.await` (`main.rs:432`): NO, e non solo per fedeltà.** Il worker è un thread OS non interrompibile: il timeout
  orfanerebbe il task (poi ESEGUITO lo stesso al pickup, `response_tx.send(...).ok()` a `worker_pool.rs:620`) e il worker resterebbe
  bloccato uguale. Comprerebbe un 5xx che l'oracle non emette e NESSUNA capacità. Stessa sorte per il ghost: la richiesta interna abbandonata
  da curl (ERR28) viene comunque eseguita da `php -S` (byte già nel socket) e da phpr: parità, da dichiarare in §3.9 (h).
- **R10 — Saturazione (W worker tutti in loopback): declare-only.** FPM risponde con `pm.max_children` + `request_terminate_timeout`
  (uccide il figlio), `php -S` con NIENTE: stalla fino al timeout del client. Un worker di riserva cambia la semantica di W (`w=<W>` in-band
  A-PP28, A/P «1,00 per costruzione a W=1») e il rilevamento self-connection è invenzione (WP può passare da un alias host). Niente avviso
  a startup (`php -S` non ne stampa). Riga in §3.9 (h): «a W worker, W loopback concorrenti stallano fino al timeout PHP: come l'oracle».
- **R11 — Hot path della coda, due righe di contratto:** `push` fa `notify_one` DOPO il rilascio del lock (evita il risveglio che si
  rischiaccia sul mutex); `len()` è esposto ma `QUEUE_DEPTH` RESTA l'unica fonte del census (inc-before-push, A-TH1): due fonti in un test
  = incidente. `pop_blocking` in loop sul predicato (risvegli spuri). Il `tokio::sync::Mutex` è giustamente scartato: il lock non attraversa
  alcun `.await`. Lettere 2, 3, 5, 6, 7: INVARIATE; 4, 8, 9, 10, 11: DA EMENDARE come in Q2.

## KILL-SWITCH
| # | condizione | atto |
|---|---|---|
| KB-189-1 | dente sequenziale (R2) sul pin vecchio W=3, L=4: il 4° loopback NON è ERR28 in [2900;3400] ms, o il log non mostra l'ordine atteso | dente nullo, nessuna promozione (ripetizione con ordine diverso = VOID, non FAIL) |
| KB-189-2 | throughput misurato contro il file t1 anziché AX-old/AX-new interfogliati nella stessa finestra | cifra ADVISORY, non è gate |
| KB-189-3 | new/old < 0,97 su una cella C=4/C=8 (front, login) o spread > 5 % | cura NON promossa (resta in branch); si ripete la finestra prima di indagare il codice |
| KB-189-4 | righe `worker_dispatch`: Σcount ≠ nreq, thr ∉ {0..W-1}, o `footprint(W)` con min(count) < 100 | run VOID; mai «assumere i worker caldi» |
| KB-189-5 | `__census_self` non ritorna entro 30 s | abort con riga `census_self_incomplete`; campagna VOID; un hang silenzioso = incidente |
| KB-189-6 | nel diff compare un timeout su `rx.await`, un worker di riserva, un rilevatore di self-connection o un knob `rr` | HALT: invenzione fuori oracle |
| KB-189-7 | un test census esistente (1801/1841/1911/1919/2029) emendato senza voce nel verbale, o `len()` usato come fonte di `QUEUE_DEPTH` | incidente |
| KB-189-8 | `battery61-axum` < 6/6 o G-APERTURA-2 FAIL dopo la cura | HALT |
