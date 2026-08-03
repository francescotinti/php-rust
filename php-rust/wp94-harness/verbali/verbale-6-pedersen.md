# Verbale SEDIA 6 — Pedersen (Concilio WP-94)

Perimetro: confine per-richiesta, lifecycle harness, failpath. Mandato: REFUTARE.
Evidenza: `crates/php-server/src/main.rs` (246-299), `crates/php-server/src/worker_pool.rs`
(216-314, 595-615, 677-686), `wp91-harness/battery-91pre.sh` (57-117),
`wp83-harness/battery-equivalence.sh` (333-353), `sessions/WP_SESSION_92.md`;
mini-ledger eseguito: `/private/tmp/pp68-mini.sh` (T1-T5).

## VERDETTO: PASS CON RISERVE — nessuna refutazione capitale; 1 implicazione da declassare, 1 bug failpath, 2 dichiarazioni mancanti.

## Q1 — Finestra della riga unificata
(a) La richiesta che arriva DOPO il post-load ma PRIMA che il driver legga la riga è
**irrilevante PER COSTRUZIONE**: i byte del dump sono già scritti in `$PHPR_MEM_CENSUS`
dentro `peak_census_dump`, e la riga stampa quattro locali già caricati — un arrivo tardivo
può sporcare solo dump FUTURI. Lo dichiaro chiuso.
(b) **Buco residuo formale**: l'implicazione nel commento (main.rs:273) «pre==post==0 ∧
arr_pre==arr_post ⇒ window clean» è FALSA nel modello di memoria puro. Una richiesta
0→1→0 contenuta nella finestra: entrambi i load Acquire di OUTSTANDING possono
legittimamente leggere il PRIMO zero (la coherence non impone freschezza), e allora
nessun synchronizes-with col dec Release esiste; `census_arrivals_now` carica per giunta
**Relaxed** (worker_pool.rs:313) su inc Relaxed — arr_post può mancare l'arrivo. La coppia
può dichiarare pulita una finestra sporca. È schermata SOLO dal protocollo (driver
sequenziale: l'unico client è la richiesta census stessa, che risponde dal router PRIMA di
`dispatch` e non tocca i contatori).
(c) Universo del testimone = sole richieste DISPATCHED: 404, `/__reqns` e il census stesso
non toccano né OUTSTANDING né ARRIVALS — mutazione heap invisibile alla coppia.
(d) Il dec scatta dopo la send sulla ONESHOT, non dopo la write sul socket: il free del
body della richiesta PRECEDENTE (A-PP-69) può cadere dentro una finestra "pulita" anche
in sequenziale (scheduling del task axum). Già dichiarato — resta corretto dichiararlo.

## Q2 — A-PP-68, caso limite costruito
Bash esegue il trap pendente al completamento del figlio, PRIMA del ramo `then`: segnale
durante un gate ⇒ gate_in_flight ancora settato ⇒ mai una riga `none` per un segnale
arrivato IN un gate. Quella direzione regge. Ma: **deferred è DERIVATO, non misurato**
(`defer=[gif≠none]`, battery-91pre.sh:72-73) — porta zero bit oltre gate_in_flight.
Caso limite: segnale durante `say "== $label =="` (riga 109, gate settato ma MAI
lanciato) ⇒ riga `gate_in_flight=label deferred=1`: «il trap è corso a fine gate» è falso,
il gate non è mai partito. Secondo limite: segnale durante i denti di batteria non-gate
(grep judge_sha 146-149, comm/join matrix 153-190) ⇒ `none/0` etichetta come «fra i
gate» lavoro verdict-bearing in corso. Terzo: TERM pid-mirato durante parity-full — il
figlio corre a termine, attempt_epoch = tempo del trap, l'epoca del segnale è PERSA
(anche +20 min). Nessuno è VOID; tutti e tre vanno dichiarati.

## Q3 — reason=signal+head-moved: eseguito
Mini-ledger /tmp, denti v2 ESATTI di battery-equivalence (338-350): **T1 riga reale del
trap PASSA** — '+',':' non rompono la grammatica k=v (nessuno spazio nel valore);
**T3** i due rev si estraggono a macchina (`head-moved:([0-9a-f]+)-to-([0-9a-f]*)`,
hex senza collisioni con '-'/'+'); **T4** il negativo MORDE; **T5** l'adiacenza
`gate_in_flight=X deferred=D` è RIGIDA (ordine invertito ⇒ VOID: fail-closed, da
dichiarare). **BUG trovato (T2)**: se `git rev-parse` fallisce nel trap (volume esterno
staccato — rischio REALE su Extreme Pro) `$now` è vuoto ⇒
`reason=signal+head-moved:REV-to-` passa i denti e traveste un guasto git da head-move
con destinazione VUOTA.

## Q4 — Cardinalità A-PP-69
Per le risposte census la dichiarazione regge A FORTIORI: i formati grandi NON viaggiano
nel body HTTP (peak dump → file; self census → stderr via `census_line`/
`mi_bin_thread_rows`, worker_pool.rs:561-580); i body sono stringhe fisse minime, allocate
DOPO il post-load. Ma «un Vec per richiesta» è un **lower bound**: l'involucro risposta
(HeaderMap + insert x-phpr-pid, buffer di scrittura hyper, oneshot) libera anch'esso lato
axum dopo il dec — O(1) alloc per richiesta, non 1; e per le risposte PHP ordinarie
l'unico Vec ha taglia ILLIMITATA (l'intero output): il rischio è la magnitudine, non la
cardinalità.

## Emendamenti
- **A-PP-70**: il claim clean-window è verdict-grade SOLO con driver sequenziale
  dichiarato in-banda (altrimenti ADVISORY), o si passa a SeqCst sui sei op dei contatori;
  emendare il commento main.rs:273 di conseguenza.
- **A-PP-71**: `on_signal` sentinella su `$now` vuoto (`git-unavailable`), mai emettere
  `-to-` con destinazione vuota.
- **A-PP-72**: deferred= dichiarato DERIVATO per NOME; i denti non-gate della batteria
  ricevono etichetta propria (es. `GATE_IN_FLIGHT=tooth:matrix`).
- **A-PP-73**: A-PP-69 riformulato: residuo = involucro risposta, O(1) alloc, taglia ∝
  body; risposte census esenti per costruzione (canali file/stderr).

## KS
- **KS-PP-94-1**: una finestra dichiarata pulita senza driver sequenziale in-banda è ADVISORY.
- **KS-PP-94-2**: un campo derivato non è un secondo testimone.
- **KS-PP-94-3**: un valore composto con separatori resta grammaticale solo finché il
  sotto-valore non può essere vuoto o contenere il separatore.

## Refutazioni capitali: NO.
