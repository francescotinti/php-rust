# TEAM-SONDA (Gregg · Pedersen · Leijen · web-runtime) — riconciliazione S-189 fase 2
Generato con assistenza AI (Claude Fable 5). Fonti vincolanti: verbali/{gregg,pedersen,leijen,web-runtime}.md.
Verifica del relatore (Read, `worker_pool.rs:412-415`): la riga `tag=worker_dispatch thr= count= arm=union` è un `eprintln!` sotto
`#[cfg(not(feature = "mem-census"))]`, emessa a teardown da OGNI worker dopo il ritorno di `worker_loop` (count=0 incluso) ⇒ DISPONIBILE sul pin di
release via SIGTERM+join: Gregg R3 («solo nei binari census») è REFUTATA dal codice; restano le sue condizioni (stderr catturato, SIGTERM, mai SIGKILL prima del drain).

## CONVERGENZE (n/4)
- **Q1 rotta (A) coda condivisa FIFO: 4/4** (B refutata: flag «libero» = TOCTOU, cieca sotto saturazione; C refutata: difetto di produzione §2 B/C).
- **Census a W=1 unico modo (KB-78-3): 4/4**; knob `rr` respinto 4/4; mappa per-thread a W>1 ADVISORY 4/4; `depth_max`/`inflight_max` e i lettori
  `measure78.sh:318,404`, `verdict81.sh` INVARIATI (QUEUE_DEPTH è già globale, Gregg R7; con dec nella sezione critica di `pop` diventa ≡ `len()`, Pedersen R7).
- **Timeout nel `rx.await` del handler: NO 4/4** (fedeltà a `php -S`; §3.38 è la sede). «Bounded + 503» WP-77.5: SUPERATO nella forma 503 4/4, con
  motivazione CORRETTA (Leijen R7, Gregg R10, wr R3): il backlog del kernel è BOUNDED e tiene byte nel kernel, phpr accetta senza limite e legge sorgente
  e corpo sul thread tokio PRIMA del dispatch ⇒ apertura UNICA per nome in §3.9: «backpressure-ad-attesa (Semaphore K=64·W all'ingresso del handler, prima
  di `route`) + corpo letto prima del dispatch: residenza ∝ connessioni, oracle ≤ W corpi» — fuori perimetro S-189.
- **Throughput: RIMISURARE 4/4** («per costruzione» vale per CPU/req a W=1, non per il wall a C=4/8: Pedersen, wr R7). Footprint: NON rimisurare 4/4 (W=1 è l'identità, Leijen R3/R6).
- **Il dente deve VEDERE la collisione 3/4** (Pedersen R8, Leijen R5, wr R4: impronta di dispatch e/o orfani), Gregg contrario solo sulla disponibilità
  della riga (refutata). Firma per richiesta, non «A rossa» (Gregg R2, wr R6): 2/4 esplicito, 0 contrari. **`std::sync::Barrier` vietata 4/4** (KS-PP-6).
  Heap-neutralità (Leijen R1/R2): 0 contrari, `VecDeque::with_capacity(W)` senza `shrink_to_fit`. Dispatch ≠ accept (wr R1): DICHIARATO («FIFO di dispatch»), 0 contrari.
- **Slope ×W a client sequenziale irriproducibile dai pin ≥ s189** (Pedersen R4, Leijen R3, wr R7a: `notify_one` sveglia un parcheggiato non specificato):
  KB-87-3/KL-87-2/A-PP49 e il budget ×W di A-DL31 versionati «pin ≤ s188»; da s189 il budget ×W è LIMITE SUPERIORE, rimisurabile solo con C=W e `inflight_max==W` DICHIARATO.

## Q1 / Q2 / Q3 — verdetto del team
- **Q1: (A), 4/4, CON EMENDAMENTI** (sotto): la scelta A/B non è discriminata dal dente (Gregg R4b): la differenza vive solo sotto saturazione, nel p99.
- **Q2: (i) rafforzata.** Invariante = **W righe `worker_dispatch` con thr-set ESATTO {0..W-1}** (riga per worker a teardown, non proprietà del dispatcher;
  Pedersen R1, verificato) **∧ Σcount == nreq + orphan** (wr R4) **∧ orphan == 0 in campagna census**; `count == nreq/W` SOLO per pin ≤ s188, ADVISORY altrove; il «⊆» del dossier è respinto.
  **`__census_self`** — voto fase 1: 3 (Gregg, Leijen, web-runtime: barriera SOLO con pre-check `census_outstanding_now()==0` nel handler + `wait_timeout` 5 s
  ⇒ panic dentro `ProbeWindow`/abort) contro 1 (Pedersen: nessuna barriera in S-189, rifiuto in-band a W>1). **Posizione del team in fase 2: RIFIUTO IN-BAND in
  S-189, barriera rinviata a design189.** Motivo: le tre sedie pro-barriera dichiarano esse stesse il self-census a W>1 advisory o no-op (Gregg R8 `grade=advisory`,
  Leijen Q2 «W=1 unico modo», wr R5 «a W=1 il rendezvous è un no-op») e nessuna ha refutato Pedersen R5(c): i W task gang passano per OUTSTANDING ⇒
  `inflight_max==W` ⇒ ogni run gang a W>1 è VOID da KS-PP-83-3 e dai giudici esistenti. Una barriera che nessuna campagna valida può usare è superficie di
  deadlock senza potere. Atto S-189: `main.rs:396-397` se `worker_count()>1` ⇒ corpo `census-self: refused w=W (W=1 only, KB-78-3/A-PP28)` senza dispatch;
  commento `:392-394` cancellato. design189 (sessione che ne consuma l'esito) eredita l'UNIONE delle condizioni: pre-check `OUTSTANDING==0` Acquire
  immediatamente prima dei W push (Leijen R4, Pedersen R6.1), Mutex+Condvar con `wait_timeout ≤ -m` del driver e generazione azzerata (Pedersen R6.2, wr R5),
  `WorkerTask.probe` che inc/dec OUTSTANDING ma salta `note_outstanding` (Pedersen R6.3), `wait()` PRIMA di `ProbeWindow::arm()` (Leijen R4), VSELF ESATTO
  con W `thr=` distinti e heap distinti (Pedersen R6.4, KL-189-2), parser a finestra perché `census_line` è atomica per RIGA (Pedersen R6).
- **Q3: dente + throughput A/B interallacciato + tripwire W=1.** Dente: come in FETTA 0. Throughput: 4/4 rimisurare; sul COMPARATORE il team adotta Gregg R5
  (A/B `AXold, AXnew, O4` interallacciati nella STESSA finestra, R=3, N=150, C∈{1,4,8}, canonica `AXnew/AXold` per cella, banda C=4/8 [0,97;1,05], C=1 [0,95;1,05],
  spread ≤5 % per braccio, mediana fuori con un ciclo dentro ⇒ cella VOID; >1,05 = REPERTO da attribuire prima di promuovere; warm-up `-n 20 -c W`) perché gli spread
  di t1 (0,4–3,8 %) rendono «0,97× del file t1» un falso kill probabile e una regressione del 2 % invisibile; `O4/AX` resta companion di continuità col t1
  (±10 %, advisory); p99 companion con direzione dichiarata (C=8 nuovo ≤ vecchio, >1,10× = REPERTO); user/req e cpu/req registrati. A/P a W=1: NON ri-registrare,
  record t3 resta la cifra; tripwire non-regressione `Aold, Anew` × {front, login} × R=3 × K=30, `|Anew/Aold − 1| ≤ 2 %`, mai pubblicato come cifra (Gregg R9).
  KS-189-2 del dossier è SOSTITUITO da KT-5. Residuo dichiarato (Pedersen): a W=1 l'interna in stallo viene eseguita dopo il timeout PHP (zombie, come `php -S`).

## TESTO DELLE LETTERE EMENDATE
Regola comune: verdetti d'archivio (`verdict86-90.sh`, `va-census-template.sh`, `measure90-campaign.sh:296-303`) MAI editati; ogni lettera versionata «testo precedente valido per pin round-robin ≤ s188 (97b58fd225ca7289)»; pin ≥ s189 = template di verdetto NUOVO.
- **A-PP36** — old: «dispatch-count per-thread in-band» → new: «dispatch-count per-thread in-band a teardown su OGNI arm (`worker_pool.rs:412-415`), con
  `orphan=<n>` (contatore degli `Err` di `response_tx.send`, Relaxed) sulla riga di teardown e in `shutdown()`; l'atteso è Σcount == nreq + orphan con set esatto
  {0..W-1}; la ripartizione per-thread è «atteso» solo sui pin round-robin ≤ s188».
- **KS-PP-87-3** — old: «cifra net da run senza dispatch-count==atteso ⇒ envelope» → new: «cifra net da run senza dispatch-count in-band con W righe, set esatto,
  Σcount == nreq + orphan e orphan == 0 ⇒ envelope; dai pin ≥ s189 la mappa per-thread non è condizione (ADVISORY)».
- **A-PP37** — old: «verdict esige la MAPPA per-thread == attesa (non la sola somma); ghost next_worker dichiarato» → new: «verdict esige W righe con set esatto
  e Σcount == nreq + orphan; la MAPPA == nreq/W è attesa SOLO su pin round-robin ≤ s188; la coda condivisa non ha slot: `push` chiusa restituisce il task ⇒ nessun
  ghost di dispatch (clausola CHIUSA, commento `:724-729` cancellato a saldo righe); il ghost dei watermark `note_*` (A-MS16) RESTA; run con dispatch Err ⇒ VOID».
- **A-PP42 / KS-PP-89-2** — old: «VDISP = insieme esatto {0..W-1} e count==nreq/W; mai assumere il round-robin in silenzio» → new: «VDISP esige W righe
  `worker_dispatch` con thr-set ESATTO {0..W-1} (invariata: testimone «W worker vivi e joinati», A-MS7) e Σcount == nreq + orphan (+1 per `/__census_self` sullo
  slope a W=1); `count == nreq/W` SOLO per pin ≤ s188, ADVISORY altrove; Σ≠atteso ⇒ FAIL (phantom), set≠esatto ⇒ FAIL (worker perso); il dispatcher in vigore
  (round-robin | FIFO condivisa) va DICHIARATO nel raw, mai assunto».
- **KH78-2 / KB-78-2 / A-BB8** — old: «profondità coda mpsc >1 in misura ⇒ VOID; depth campionata a ogni dispatch» → new: «profondità QUEUE_DEPTH (globale:
  somma delle W code sui pin ≤ s188, `len()` della coda unica dai pin ≥ s189; a W=1 la stessa grandezza) >1 in misura ⇒ VOID; depth campionata a ogni dispatch;
  semantica «FIFO di dispatch, non di accept» (wr R1); test unitario `push N senza consumer ⇒ len()==N==QUEUE_DEPTH` in dispatch.rs».

## FETTA 0 — il dente e le sue prove
Coreografia (Gregg R1 + Pedersen R8 + Leijen R5 + wr R4/R6), porta DEDICATA ≠ 8080/8199, `127.0.0.1` ovunque, processo FRESCO per ripetizione, readiness su
file STATICO (`Routed::Ready`, non avanza `next_worker` ⇒ slot iniziale s=0 provato): A = `loop.php?d=800` a t=0; B = `loop.php?d=300` a t=+150 ms lanciata
SOLO dopo che il lanciatore ha VISTO la connessione di A `ESTABLISHED` (`lsof -iTCP:PORT -sTCP:ESTABLISHED`), mai `sleep` cieco; fixture con `curl` interna a
`pong.php`, `CURLOPT_TIMEOUT 3`, stampa `t0=<microtime> tc=<microtime> OK|ERR errno= ms=`; curl di shell `-m 15`, nessun `--retry`. Il giudice VERIFICA
`A.t0 < B.t0 < B.tc < A.tc` su ogni ripetizione. Chiusura: SIGTERM, attesa ≤ 30 s, stderr catturato, lettura delle W righe `worker_dispatch`.
| braccio | A | B | impronta `thr:count` | orphan |
|---|---|---|---|---|
| pin vecchio W=3 (controllo POSITIVO) | `ERR errno=28 ms∈[2800;3300]` | `OK ms<700` | ESATTA `{0:2,1:1,2:1}` (A' sul worker di A) | n/d (riga assente sul pin) |
| pin nuovo W=3 | `OK ms<1000` | `OK ms<1000` | Σ==4, set esatto {0,1,2}, mappa advisory | **0** (KT-4) |
| pin vecchio W=1 · pin nuovo W=1 · oracle senza workers (fedeltà) | `ERR28` | `ERR28`, wall esterno ≈ 6–7 s | `{0:4}` (phpr) | 2 attesi sul nuovo (A', B') |
| oracle `PHP_CLI_SERVER_WORKERS=3` | `OK` | `OK` | n/d | n/d |
| braccio NEGATIVO: A sola, pin vecchio W=3 | `OK` | — | `{0:1,1:1,2:0}` | n/d |
Nota: la cella «B=OK a W=1» di wr R6 è CORRETTA in `B=ERR28` (il worker che esegue B dovrebbe servire B': stallo certo, dossier §1; Gregg R2 concorde).
`errno` confrontato col token dell'oracle (libcurl vero), non il solo «ERR». 3 ripetizioni con restart, esito identico.
Evidenza per ripetizione (`<rep>.evidence`, fuori repo): sha256[0:16] del binario e W; pid in LISTEN == pid spawnato; porta libera PRIMA del boot e vuota dopo
kill+wait; log del server con conteggio ESATTO = sonde di readiness + 4 e URI ⊆ {loop.php, pong.php, sonda} (una riga in più ⇒ VOID, lezione t1); sul pin vecchio
la riga `pong.php` di A' DOPO l'ERR di A (prova fisica della coda sul worker di A); output delle 4 fixture con t0/tc/ms/errno; le W righe `worker_dispatch` e
`orphan=`. Il dente NON discrimina (A) da (B): va scritto nel criterio (Gregg R4b). Test unitari pretesi (dispatch.rs): N produttori/M consumatori; drena-poi-esce;
push dopo close restituisce il task; **falsificatore W=2 lungo+corto+corto** (round-robin: terzo attende il lungo; FIFO: terzo→thr=1; bound = timeout nel
messaggio, Pedersen R9); wakeup perso (push senza waiter, consumer dopo, Leijen R9a); W consumer occupati ⇒ (W+1)-esimo resta in coda (Leijen R9b); ciclo sul
predicato per spurious wakeup, mai `.await` sotto il Mutex (commento vincolante), panic di un consumer non avvelena gli altri — tutto SENZA `sleep` (wr R9).

## DISSENSI A REGISTRO
- **Leijen, Gregg, web-runtime**: barriera `__census_self` guardata (pre-check outstanding==0 + timeout+abort) già in S-189, non solo in design189. Il team
  la rinvia (Q2); il dissenso resta sotto i tre nomi (Leijen: con la barriera `dispatched` per thread cresce di 1 e ARRIVALS di W come oggi).
- **Pedersen, Leijen, web-runtime** (fase 1): soglia «0,97× del file t1» accettata; il team la sostituisce con l'A/B interallacciato (Gregg R5), rumore fra
  finestre non refutato e file mai difeso come comparatore: superamento, non dissenso attivo.
- **Gregg R3** (riga solo nei binari census): refutato dal codice; il log `PHPR_LOG=debug` per dispatch resta opzionale, mai nel binario di misura.
  **web-runtime R6** («B=OK a W=1»): corretto dal team; l'esito per-richiesta fail-closed che chiedeva è adottato.

## KILL-SWITCH DEL TEAM
| # | condizione (meccanica) | atto | sedia |
|---|---|---|---|
| KT-1 | pin vecchio W=3 con sfasamento verificato: firma `A=ERR28∈[2800;3300] ∧ B=OK<700` ∧ impronta `{0:2,1:1,2:1}` NON ottenute 3/3 | dente NULLO, nessuna promozione | Gregg·Pedersen·Leijen (KS-189-1) |
| KT-2 | cella della tabella per braccio×richiesta violata, errno≠28, ordine t0/tc non provato, log con righe/URI oltre le attese | ripetizione VOID; 2 VOID consecutive ⇒ coreografia riprogettata | Gregg·web-runtime |
| KT-3 | braccio negativo NON OK, o errno phpr ≠ errno oracle a W=1 | dente NULLO fino a cura di fixture/facciata | Gregg |
| KT-4 | pid LISTEN ≠ spawnato o porta non libera ⇒ run NULLA; pin nuovo W=3 non verde 3/3 o `orphan>0` su un worker in una ripetizione | cura NON promossa | Gregg·web-runtime |
| KT-5 | throughput A/B interallacciato: cella C∈{4,8} fuori [0,97;1,05] o C=1 fuori [0,95;1,05] con 3 cicli concordi; `ab -k` su un solo lato | cura NON promossa (branch); confronto VOID | Gregg·web-runtime (sostituisce KS-189-2) |
| KT-6 | tripwire W=1 `\|Anew/Aold−1\|>2 %` su front o login; p99 C=8 >1,10× o cpu/req fuori ±2 % | HALT; REPERTO, promozione sospesa finché attribuito | Gregg |
| KT-7 | verdetto d'archivio editato per far passare un raw di pin ≥ s189; test census esistente emendato senza voce nel verbale | verdetto NULLO, incidente (= KS-189-3) | Pedersen·Gregg |
| KT-8 | claim per-thread (mappa, composizione warm, budget b/worker) da raw `w>1` su pin ≥ s189 senza protocollo C=W dichiarato | claim ADVISORY, mai attribuzione | Pedersen |
| KT-9 | `__census_self` servito a `w>1` senza design189 firmato; raw con `census-self REFUSED`; `inflight_max>1` con righe `peak_self`; rendezvous scaduto | raw/campagna VOID; nessuna riga `peak_self` citabile | Pedersen·Leijen·web-runtime |
| KT-10 | commento ghost `:724-729` o commento «W sequential calls» `:392-394` sopravvivono alla cura | token-presenza: review FAIL (classe KS-PP-87-1) | Pedersen |
| KT-11 | dispatch.rs promosso senza falsificatore W=2, wakeup-perso, (W+1)-esimo; test o dente con `sleep` come sincronizzazione; `std::sync::Barrier` | cura NON promossa / test respinto | Pedersen·Leijen·web-runtime |
| KT-12 | cifra RSS/peak letta nel gate-loopback o nel throughput W=4 citata come footprint | cifra NULLA: footprint solo dal gemello census W=1 | Leijen |
| KT-13 | riga 1 del NEXT o lettera dichiara «503 superato» senza §3.9 (corpo prima del dispatch + backpressure-ad-attesa) e §3.39 (`default_socket_timeout` wrapper `http://`) aperte per nome | promozione sospesa; la dichiarazione decade | Leijen·web-runtime |
| KT-14 | server del dente non uscito 30 s dopo SIGTERM | SIGKILL + braccio `shutdown-hang` (registrato); due ⇒ incidente, riconvocazione | web-runtime |
| KT-15 | `battery61-axum` < 6/6 o G-APERTURA-2 FAIL dopo la cura | HALT (= KS-189-4) | Gregg (dossier) |
