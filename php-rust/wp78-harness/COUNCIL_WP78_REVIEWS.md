# COUNCIL_WP78_REVIEWS.md — Concilio a 9 sedie su S-77.6.5.2.4 + programma WP-78

**Data**: 2026-07-31 (mattina)
**Oggetto**: revisione di chiusura S-77.6.5.2.4 (implementazione 6 emendamenti Council, commit fed30ce) + giudizio sullo scope WP-78 (fase misura footprint post-Axum)
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto NEXT_SESSION, WP_SESSION_77_6_5_2_4, i verbali precedenti (COUNCIL_WP77_6_5_2_3_REVIEWS) e il codice del proprio perimetro (main.rs, worker_pool.rs; Matsakis/Stogov anche vm/mod.rs).
**Status**: verbali VINCOLANTI per il design di WP-78 (blocco ⚖️ in NEXT_SESSION §WP-78).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: NON unanime — 8× CONCORDO CON EMENDAMENTI, 1× MI OPPONGO (Klabnik).**

La chiusura di S-77.6.5.2.4 ("6/6 amendments IMPLEMENTED") è REFUTATA su tre emendamenti e sul gate:

1. **A-MS1 è FALSO** (Matsakis, con mea culpa: l'errore era nel suo emendamento originale, trascritto fedelmente). `RetainSet(elsa::FrozenVec<Rc<Module>>)` è `!Send + !Sync` — il commento SAFETY afferma il contrario del vero. La sicurezza reale poggia proprio sulla thread-affinità enforced dal compilatore. Va riscritto + static assert (A-MS2/A-MS3).
2. **A-DS1 è NON conforme** (Stogov). La matrice "statics persist — matches PHP CLI behavior" è sbagliata due volte: (a) un web server deve matchare PHP-FPM (statics reinizializzati a ogni richiesta), non la CLI; (b) il codice OGGI non fa persistere gli static — sono campi del Vm che muore per-richiesta, e `request_end()` (vm/mod.rs:3418-3526) non tocca né statics né closure_statics né static_props. L'isolamento avviene per morte del Vm, non per reset. La matrice documenta una semantica che se "rispettata" da un futuro Vm persistente introdurrebbe il bug.
3. **A-AH1 è NON implementato** (Hejlsberg): i comandi bash nel commento sono semanticamente sbagliati (`--features axum-server` fa l'UNIONE con `default=["cli-server"]`, non un build solo-axum); "codegen bloat ≤500 bytes" mai misurato; la matrice reale è una terna, non una coppia. Chiusura "IMPLEMENTED" a verbale = chiusura falsa.
4. **Il PASS di G-APERTURA-2 è VACUO** (Klabnik, MI OPPONGO; confermato indipendentemente da Hejlsberg): `wp77-harness/gate-axum/` è VUOTA (nessuno script eseguibile); `cargo test --release` **non compila una sola riga** del codice sotto revisione (worker_pool.rs e axum_handler sono feature-gated fuori dai default — prova: il binding `reg` inutilizzato non ha mai prodotto warning); il check "binario invariato 02aa2ad8" è sul binario SBAGLIATO (02aa2ad8 = phpr CLI, trivialmente invariato; il binario pertinente php-server, sha256 10729e94…, è stato ricompilato in sessione senza hash pre/post a verbale). Nessuna richiesta HTTP eseguita in S-77.6.5.2.4.

**Refutazioni convergenti sul codice** (≥3 sedie indipendenti ciascuna):
- **Legacy `execute()` viola la regola permanente** capture-before-request_end nello stesso file che la documenta (righe 330 vs 332-334), e la sua nota deprecation punta a `execute_with_vm`, funzione inesistente → RIMUOVERE, non deprecare (Hoare A-TH1, Pedersen A-PP2, Matsakis A-MS4, Hejlsberg A-AH5).
- **Doc di crate STALE**: main.rs righe 5-11 descrive ancora "Thread-local Vm N=1 + Vm::reset()" — l'architettura RESPINTA in WP-77.2 (Hoare A-TH3, Leijen, Gregg A-BG2).
- **Confine HTTP incoerente**: fatal runtime → 200 OK, errore di compilazione → 500 con formato corpo diverso; FPM risponde 500 pre-header (Hoare A-TH5, Pedersen A-PP4, Stogov A-DS4).
- **Body morto**: `meta.body` allocato e trasportato via canale ma mai passato alla VM (`request_start(None, &[])`) (Bak A-BB5); `reg` inutilizzato in worker_loop con commento che promette un'ottimizzazione inesistente (Bak, Pedersen A-PP5, Leijen, Hejlsberg, Klabnik).

**Refutazioni convergenti sul programma WP-78** (la misura così scritta emetterebbe verdetti nulli):
- **`--workers` non esiste** (hardcoded num_cpus): N=100 richieste round-robin su N core = ~100/N per worker → steady-state per-worker MAI raggiunto, footprint retained ×N non confrontabile col CLI (Bak A-BB3, Leijen A-DL2, Gregg A-BG3/rilievo 3).
- **Compile-side non separato**: lower_source+compile_program a OGNI richiesta; un numero unico alloc/req mescola compile e run — lezione WP-59 (compile-side = 65% non censito). Verdetto A-BB1 emesso su totale solo = VOID (Bak A-BB2, Hejlsberg A-AH4, Gregg A-BG4, Stogov A-DS5).
- **Denominatore A-BB1 indefinito**: confronto onesto = stesso binario, `--cli-server` vs `--axum`, stessa fixture, stesso N (Gregg A-BG4).
- **Shutdown pulito assente**: `/usr/bin/time -l` e MIMALLOC_SHOW_STATS riportano solo a exit; kill -9 = zero stats → serve SIGTERM/endpoint di uscita (Leijen A-DL3).
- **Regole storiche di misura assenti dal piano**: PURGE_DELAY=0, vmmap Physical footprint, divieto RSS ps, coppie build-adiacenti, R≥3, warm-up escluso (Leijen A-DL3, Gregg A-BG3).
- **Verbale .3 contiene un numero mai misurato**: "alloc overhead ≤+1.5% misurato steady-state" (sezione Bak) è una predizione travestita da misura → rietichettare (Gregg rilievo 2).
- **Gate mai concorrente**: KM-77-2 sollevato senza gate che morde; G-APERTURA-2 è sequenziale → serve gate con richieste concorrenti PRIMA del census (Hoare KH78-1).
- **Gate cieco allo stato**: hello-world a 2 richieste non vede la semantica statics → fixture STATEFUL ≥3 richieste (static $n, static property, static in closure, define()) (Stogov A-DS3).

### Ordine vincolante di apertura WP-78 (S-78.0 "sanatoria", TUTTO pre-misura)

1. **A-SK2** — `wp78-harness/gate-axum/run-gate.sh` ESEGUIBILE (build `-p php-server --features axum-server`, shasum del binario php-server, server up, 2 POST sequenziali, `cmp` byte, exit code). Il PASS si dichiara SOLO citando comando+hash. [KS-SK-78.1]
2. **A-MS2/A-MS3** — riscrivere i SAFETY comment con l'invariante vero (`!Send+!Sync`, thread-affine per costruzione) + `assert_not_impl_any!(RetainSet: Send, Sync)` e controllo positivo `assert_impl_all!(WorkerTask: Send)`.
3. **A-DS2/A-DS3** — matrice FPM corretta (meccanismo reale per riga: Vm drop vs request_end vs mass-teardown) + fixture stateful nel gate. [KS-DS-78-1]
4. **A-TH1/A-PP2/A-MS4** — rimozione integrale legacy `execute()` + grep-gate: nessuna lettura di rendered/stdout dopo request_end() in nessun call-site. [KS-PP-2]
5. **A-TH3/A-BG2/A-DL5** — purge doc N=1 da main.rs/worker_pool.rs; riga "Boot variance: Controlled" rimossa; `reg` morto sanato.
6. **A-AH2/A-AH3/A-SK4** — `gate-feature-matrix.sh` (terna di build con `-D warnings`, `nm/strings` per assenza simboli axum nel default) + step CI `--no-default-features --features axum-server`. [KS-AH-78-2]
7. **A-BB3/A-DL2** — flag `--workers N`; **A-BB5** — body morto risolto (collegato o rimosso dal Meta); **A-DL3** — shutdown pulito + protocollo di misura scritto in design78.md (A-BG3).
8. **A-PP3** — test capture-before-reset con controllo positivo (buffer azzerati post-request_end).
9. Solo DOPO: Tier-0 + census con misura PER-FASE (compile/run/shutdown separati), `--workers 1`, R≥3, denominatore A-BG4. **A-PP4/A-TH5/A-DS4** (fatal→500) prima del freeze del contratto HTTP.

**A-TH4** (request_end(self) -> CapturedOutput, tocca API Vm ⇒ gate ORM/hk) e **A-BB6/A-AH4-cache** (Module cache keyed by path, frequenza×taglia prima) restano per WP-79+.

### Kill-switch consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KS-SK-78.1 | run-gate.sh assente o non deterministico all'apertura WP-78 | HALT fase misura |
| KS-SK-78.2 | cmp dei due body ≠ 0 | REJECT S-77.6.5.2.3 |
| KS-SK-78.3 / KS-AH-78-2 | build feature axum con `-D warnings` fallisce (oggi: `reg` morto) | HALT / gate rosso |
| KS-AH-78-1 | misura senza log gate-feature-matrix (binario non identificato) | misure NULLE |
| KH78-1 | nessun gate concorrente prima del census; panic/divergenza sotto concorrenza | HALT |
| KH78-2 / KB-78-2 | profondità coda mpsc >1 durante run di misura | run VOID |
| KH78-3 / KS-PP-2 | grep-gate trova capture post-request_end in qualunque sito | reject immediato |
| KS-PP-1 | test A-PP3 fallisce (corpo vuoto 2ª richiesta / buffer non azzerati) | merge respinto |
| KS-PP-3 | WP-78 tocca il path request_* senza rilancio G-APERTURA-2 per NOME | gate invalido |
| KS-MS-1/2/3 | RetainSet diventa Send/Sync senza delibera; "RetainSet is Send" ricompare; RetainSet condiviso fra thread | build break / gate FAIL / REJECT |
| KS-DS-78-1 | fixture static-counter: richiesta N≥2 stampa ≠1 | FATAL, blocca merge |
| KS-DS-78-2 | residuo RetainSet post-request_end ≠ firma WP-72 (used_n=0) | FATAL |
| KS-DS-78-3 | proposta Vm persistente senza A-DS3 verde | respinta d'ufficio |
| KB-78-1 | compile >50% alloc/req e report non per-fase | verdetto A-BB1 VOID |
| KB-78-3 / KG-78.A | run con workers≠1 o senza R≥3 per-RetainSet o senza protocollo A-BG3 | VOID |
| KB-78-4 / KG-78.B | spread oltre ±2% build-adiacente | run da indagare, nessuna media |
| KG-78.C / KL-78-1/2 | delta per-richiesta RSS strutturalmente >0 a steady-state; footprint(W) non lineare | HALT, riapertura C-leak |
| KG-78.D / KL-78-3 | cifra citata senza run tracciato / senza PURGE_DELAY=0 / via ps RSS | cifra respinta d'ufficio |
| A-SK5 | "binary baseline" per lavoro Axum riferito a phpr anziché php-server | annotazione vacua, da correggere |

---

## VERBALI INTEGRALI

### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

La chiusura formale di S-77.6.5.2.4 è accettabile (binario 02aa2ad8 invariato, 1651/0, gate PASS), ma il mio mandato è refutare, e la lettura del codice refuta tre affermazioni del verbale precedente.

**Constatazioni**

1. Il drop-order RetainSet/Vm è l'unico invariante davvero garantito dal type system: `vm` prende `&retain` in prestito, quindi borrowck impone che Vm muoia prima di RetainSet. Bene.
2. Tutto il resto — ordine Stogov (start→run→shutdown→end) e capture-before-reset — vive **solo in commenti**. Nessun typestate, nessun assert.
3. **Il codice deprecato `RequestHandler::execute()` (worker_pool.rs:277-338) VIOLA la regola permanente Pedersen/Stogov**: cattura `vm.rendered`/`vm.stdout` **DOPO** `vm.request_end()` (righe 330 vs 332-334), a dieci righe dal commento SAFETY che dichiara la violazione motivo di reiezione. Inoltre la sua nota deprecation rimanda a `execute_with_vm`, funzione **inesistente**. È dead code mai chiamato, ma un contratto "permanente" contraddetto nello stesso file non è un contratto.
4. **Il commento A-MS1 afferma "RetainSet is Send + Sync" senza prova**: nessuno static assert, e l'architettura non lo richiede nemmeno (RetainSet nasce e muore dentro `worker_loop`, mai attraversa thread). Se è falso (probabile, se contiene Rc/RefCell), abbiamo firmato un invariante falso; se è vero, va asserito a compile-time. Un SAFETY comment non verificabile è peggio di nessun commento.
5. **Documentazione stantia contraddittoria**: main.rs:7-10 descrive ancora "Thread-local Vm (N=1 reuse) + Vm::reset()" — l'architettura **RESPINTA** in WP-77.2 — e worker_pool.rs:4 dice "each owns 1 persistent Vm". Se la documentazione è "parte del contratto" (Lezione 2 della sessione), il contratto attuale descrive due architetture, una delle quali bocciata.
6. Il verbale KS-M3 dichiara "HTTP 500 response sent" sul fatal; `execute_with_retain` ritorna **StatusCode::OK** anche con fatal (riga 271). Il gate hello-world non morde su questo percorso. Nessun `unsafe` nascosto: safe-only confermato.

**Emendamenti**

- **A-TH1** (WP-78): eliminare `execute()` deprecato (o riscriverlo conforme); aggiungere grep-gate: nessun sito cattura output dopo `request_end()`.
- **A-TH2** (WP-78): sostituire il claim Send+Sync con l'invariante vero ("RetainSet mai attraversa il confine di thread") oppure provarlo con `const _: () = { fn a<T: Send + Sync>() {}; ... }`.
- **A-TH3** (WP-78): purgare da main.rs/worker_pool.rs ogni residuo dell'architettura N=1 respinta.
- **A-TH4** (successiva a WP-78, tocca API Vm ⇒ gate ORM/hk): rendere A-PP1 vero per costruzione — `request_end(self) -> CapturedOutput` che consuma il Vm e restituisce l'output, rendendo irrappresentabile la cattura post-reset.
- **A-TH5** (WP-78): riconciliare fatal→200 con il verbale KS-M3 (fix a 500 o rettifica del verbale).
- **A-TH6** (WP-78): A-AH1 mandava un workflow CI; un commento con snippet bash non è CI. Implementare il dual-build check o descope formale.

**Kill-switch**

- **KH78-1**: KM-77-2 ("no RefCell panic concorrente") è stato sollevato senza un gate che morde — G-APERTURA-2 è sequenziale. Prima del censimento footprint: gate con richieste **concorrenti** su N worker; un panic o divergenza ⇒ HALT.
- **KH78-2**: la coda mpsc è **unbounded** — misure footprint sotto carico > capacità sono invalide per costruzione; se il footprint cresce con la profondità coda ⇒ HALT misura, bound della coda prima.
- **KH78-3**: se un grep-gate trova cattura output post-`request_end()` in qualsiasi call-site ⇒ reject immediato (regola permanente).

**Su WP-78**: approvo lo scope di misura (Tier-0, censimento, A-BB1, A-DL1) **a condizione** che KH78-1 preceda la misura: non si misura il footprint di una superficie concorrente mai gateata.

### 2. Matsakis — ownership/aliasing/Send-Sync — CONCORDO CON EMENDAMENTI

**1. A-MS1: la dichiarazione è FALSA — e l'errore è mio, transcritto fedelmente.**

Ho verificato la definizione: `pub struct RetainSet(elsa::FrozenVec<Rc<Module>>)` (`php-rust/crates/php-runtime/src/vm/mod.rs:426`). Quindi:

- **`RetainSet: !Send`** — contiene `Rc<Module>` (refcount non-atomico, `Rc` è `!Send` per definizione std).
- **`RetainSet: !Sync`** — doppia ragione: `Rc` è `!Sync`, e `elsa::FrozenVec` usa `UnsafeCell` senza sincronizzazione (la variante thread-safe è `elsa::sync::FrozenVec`, non usata).

Il commento in `worker_pool.rs:14` e `:129` («RetainSet is Send + Sync: refcount-only fields») afferma esattamente il **contrario** del vero, e contraddice l'header dello stesso file (riga 6: «!Send pinned ownership»; riga 113: «never Send/migrated»). La causalità è invertita: i «refcount-only fields» sono `Rc`, cioè precisamente il motivo per cui NON è Send/Sync.

**La dichiarazione non è nemmeno necessaria.** La sicurezza dell'architettura poggia sull'opposto: `retain` è costruito DENTRO `worker_loop` (riga 132), mai attraversa il canale; l'unica cosa che migra è `WorkerTask` (String/Vec/oneshot::Sender — Send), e il bound `Send` della closure di `thread::spawn` è già verificato dal compilatore. L'invariante vero — thread-affinità — è **enforced dal type system oggi**, gratis, proprio perché RetainSet è `!Send`. Il commento inoltre parla di «thread-local storage» (è uno stack local, non TLS) e di «no references across await» in un loop che non ha await (thread OS bloccante, `blocking_recv`).

Mea culpa: il testo falso proviene verbatim dal mio M-MS1 in COUNCIL_WP77_6_5_2_3 (righe 92-96). L'implementatore ha eseguito fedelmente un emendamento sbagliato. Un commento SAFETY che afferma il falso è peggio di nessun commento: autorizza il refactor che ucciderebbe l'invariante (es. condividere un RetainSet fra worker «tanto è Sync»).

**2. Reperto collaterale (perimetro Pedersen/Stogov, lo segnalo):** il legacy `RequestHandler::execute()` (worker_pool.rs:277-338, deprecato ma `pub` e compilato) chiama `request_end()` (riga 330) **PRIMA** della cattura output (righe 332-334) — violazione della PERMANENT RULE nello stesso file che la documenta. Va sanato o rimosso.

**3. WP-78:** dallo scranno ownership, nessuna obiezione allo scope di misura (Tier-0, censimento pool-vs-fresh, A-BB1, A-DL1): i worker possiedono ciascuno il proprio RetainSet, nessun aliasing cross-thread. Condizione: A-MS2/A-MS3 in apertura WP-78, prima di qualunque esperimento di scaling del pool.

**Emendamenti (BINDING):**

- **A-MS2** (WP-78 apertura): riscrivere i commenti SAFETY in `worker_pool.rs` (header righe 13-18 e righe 129-131) con l'invariante VERO: «RetainSet è `!Send + !Sync` (Rc + FrozenVec non-sync); la sicurezza deriva dalla costruzione thread-affine dentro `worker_loop`; solo `WorkerTask: Send` attraversa il canale; il compilatore rifiuta ogni migrazione». Il verbale S-77.6.5.2.3 M-MS1 è emendato di conseguenza.
- **A-MS3** (WP-78 apertura): l'invariante esce dal commento ed entra nel type system — in `php-runtime` (o test di `php-server`): `static_assertions::assert_not_impl_any!(RetainSet: Send, Sync);` più **controllo positivo** `assert_impl_all!(WorkerTask: Send)` (lezione WP-72: contatore tutto-zero = controllo fallito). In alternativa senza dipendenza: test trybuild compile-fail.
- **A-MS4** (WP-78): rimuovere `RequestHandler::execute()` legacy, o riordinarlo conforme alla permanent rule. Codice morto che viola una regola vincolante non può convivere con la regola.

**Kill-switch:**

- **KS-MS-1**: se una modifica futura rende `RetainSet` Send o Sync, l'assert A-MS3 rompe la build → rimozione dell'assert SOLO previa delibera del Concilio (mai in-sessione).
- **KS-MS-2**: grep-gate CI: zero occorrenze di «RetainSet is Send» nell'albero dopo A-MS2; occorrenza > 0 → gate FAIL.
- **KS-MS-3**: qualunque esperimento WP-78 con RetainSet condiviso fra thread (Arc, elsa::sync) è REJECT automatico senza nuova review L1.

### 3. Klabnik — spec/testabilità/gate — MI OPPONGO

(sulla dichiarazione "A-SK1 implementato" e sul PASS di G-APERTURA-2; nessuna obiezione ai contenuti dei doc-comment in sé, che sono corretti e utili)

**Motivazione (fatti verificati su disco, non opinioni)**

1. **Il gate non esiste in forma eseguibile.** `wp77-harness/gate-axum/` è VUOTA (verificato con ls). L'unica incarnazione di G-APERTURA-2 è il doc-comment in `crates/php-server/src/main.rs` (righe 13-20). Un doc-comment non può fallire; il mio A-SK1 chiedeva la formalizzazione del *contratto*, ma un contratto senza comando che lo verifichi è una promessa, non un gate. Il target ammesso era "lib.rs **o tests/integration/**": la directory `tests/` del crate non esiste affatto.

2. **Il PASS dichiarato è vacuo rispetto al gate.** Il session file fonde due cose: "cargo test --release: 1651/1/0" e "G-APERTURA-2: PASS (byte-parity verified)". Ma `Cargo.toml` di php-server ha `default = ["cli-server"]` e `worker_pool.rs` è interamente sotto `#[cfg(feature = "axum-server")]`: **il comando di verifica citato non compila nemmeno il codice del gate**. Prova indiretta: `worker_loop` contiene `let reg = php_builtins::registry();` mai usato — un binding che produrrebbe warning `unused` a ogni build con la feature attiva; il fatto che nessuno l'abbia notato conferma che il path Axum non passa dalla suite. Nessuna richiesta HTTP è stata eseguita in S-77.6.5.2.4: il byte-parity poggia solo sull'osservazione 45851 della sessione *precedente*, non riproducibile oggi con un comando.

3. **Il check di invarianza binaria è sul binario sbagliato.** Ho verificato gli hash: `02aa2ad8…` è **phpr** (CLI, mtime 01:20), che non contiene né main.rs né worker_pool.rs — la sua invarianza dopo edit a php-server è trivialmente vera e non dimostra nulla. Il binario pertinente, `php-server` (sha256 `10729e94…`, 22 occorrenze "axum" nelle strings), ha mtime 01:54, *dentro* la finestra di sessione: è stato ricompilato e nessuno ne ha registrato l'hash pre/post.

4. **A-AH1 (Hejlsberg) parimenti degradato**: il target verbalizzato era "GitHub Actions workflow"; `ci.yml` builda e testa solo default features — nessun job con `--features axum-server`, nessun assert di non-sovrapposizione. Anche qui: snippet bash in un doc-comment, non un check che possa fallire.

**Emendamenti (BINDING)**

- **A-SK2** (WP-78, *primo atto, prima di qualunque misura*): creare `wp77-harness/gate-axum/run-gate.sh` eseguibile: build `-p php-server --features axum-server`, registra `shasum` del binario, avvia il server, 2 richieste POST sequenziali (script hello-world), `cmp` byte dei body, exit 0/≠0. Il PASS di G-APERTURA-2 si dichiara SOLO citando questo comando e l'hash.
- **A-SK3** (WP-78): integration test `crates/php-server/tests/gate_apertura2.rs` sotto la feature: due chiamate a `execute_with_retain` sulla stessa RetainSet, assert di uguaglianza byte. È la forma minima del gate che vive in `cargo test`.
- **A-SK4** (WP-78): aggiungere a `ci.yml` uno step `cargo build --release -p php-server --features axum-server` con `-D warnings` (chiude davvero A-AH1 e stana il `reg` morto).
- **A-SK5** (registri): ogni futura citazione di "binary baseline" per lavoro Axum deve riferirsi a php-server, non a phpr; la riga "unchanged 02aa2ad8" di S-77.6.5.2.4 va annotata come vacua nel session file.

**Kill-switch proposti (misurabili)**

- **KS-SK-78.1**: se all'apertura di WP-78 `run-gate.sh` manca o non produce exit code deterministico → HALT della fase misura (misurare un'architettura non gateata è misurare l'ignoto).
- **KS-SK-78.2**: `cmp` dei due body ≠ 0 → REJECT S-77.6.5.2.3 (il kill-switch documentato diventa finalmente azionabile).
- **KS-SK-78.3**: build con feature + `-D warnings` fallisce → il path Axum non è in disciplina di compilazione; HALT.

**Su WP-78 (scope)**: lo scope di misura (Tier-0, census RetainSet-pool vs fresh-Vm, A-BB1 ≤+2%, A-DL1 frammentazione) è ben posto e lo approvo, **condizionato** ad A-SK2/KS-SK-78.1 come precondizione d'apertura: senza gate eseguibile, ogni numero di WP-78 sarebbe attribuito a un binario di cui non sappiamo dichiarare né l'identità né la correttezza.

### 4. Hejlsberg — compilazione incrementale/feature-gating/igiene di build — CONCORDO CON EMENDAMENTI

(vincolanti, pre-misura WP-78)

**Refutazioni di merito**

1. **A-AH1 NON è implementato: resta APERTO.** Il mio emendamento chiedeva una *verifica* del feature-gating; è stato chiuso come commento in `main.rs` (righe 30-38) con comandi bash mai eseguiti. Peggio: i comandi sono *semanticamente sbagliati*. `cargo build --features axum-server` NON produce un binario "solo axum" — cargo fa l'unione con `default = ["cli-server"]`. La matrice reale è una TERNA (default / `--no-default-features --features axum-server` / unione) e il commento ne descrive una coppia inesistente. L'asserzione "codegen bloat ≤500 bytes" è infalsificabile: mai misurata. "Binario invariato 02aa2ad8" è vero per costruzione (nessuna ricompilazione eseguita), non è una verifica. Marcare A-AH1 "IMPLEMENTED" nel verbale di sessione è una chiusura falsa.

2. **L'osservazione di sessione "workflows non esiste" è FALSA — ma il buco è reale.** `.github/workflows/ci.yml` esiste e gira build+test su default features. Conseguenza strutturale: il gate 1651/1/0 **non compila una sola riga** di `worker_pool.rs` né di `axum_handler` (entrambi `cfg(feature = "axum-server")`). Prova interna: `worker_loop` riga 125 contiene `let reg = php_builtins::registry();` mai usata — un warning che nessun gate ha mai visto perché quel path non entra mai in compilazione. Il "PASS" citato a copertura di S-77.6.5.2.4 copre lo zero per cento del codice sotto revisione.

3. **Ricompilazione integrale per richiesta.** `execute_with_retain` esegue `lower_source` + `compile_program` a OGNI richiesta, nessuna cache di Module. Accettabile *transitoriamente* per WP-78 — ma la lezione WP-59 (compile-side = 65% del footprint non censito) dice che il census confonderà compile-side e request-side. Il giudizio A-BB1 (≤+2% alloc vs CLI) rischia di essere emesso su un rumore dominante.

4. **Igiene/dedup**: `execute()` legacy deprecato duplica ~60 righe di `execute_with_retain`; `WorkerPoolContext` è uno struct vuoto TODO; campo `context` e `_context` inutilizzati. Codice morto invisibile al gate (punto 2) che divergerà silenziosamente.

**Emendamenti**

- **A-AH2** (WP-78, apertura, PRIMA di ogni misura): `wp78-harness/gate-feature-matrix.sh` che *esegue* — con log in harness — le tre configurazioni con `-D warnings` su php-server, verifica assenza di simboli axum nel binario default (`nm`/`strings`), e registra hash+feature-set del binario che verrà misurato. Correggere il commento A-AH1 con la terna reale. Senza questo, "02aa2ad8" resta un artefatto non identificato.
- **A-AH3** (WP-78): aggiungere a `ci.yml` uno step `cargo check -p php-server --no-default-features --features axum-server` + `--all-features`, con warnings fatali. Costo: secondi. Chiude il buco "gate che non morde" — regola WP-72 già a verbale: via appena attivata = superficie nuova da gateare.
- **A-AH4** (WP-78): il census DEVE attribuire separatamente compile-side (lower+compile per-request) e request-side. Nessuna Module-cache ora (cambierebbe la superficie prima della misura); se il costo lo giustifica, cache keyed by (path, hash sorgente) pianificata in WP-79.
- **A-AH5** (WP-79): rimuovere `execute()` legacy, `WorkerPoolContext` TODO, variabili morte; dedup del blocco fatal-rendering condiviso.

**Kill-switch**

- **KS-AH-78-1**: prima misura WP-78 senza log di `gate-feature-matrix.sh` in harness ⇒ misure NULLE (binario non identificato).
- **KS-AH-78-2**: un solo warning da `cargo check --no-default-features --features axum-server` ⇒ gate rosso (oggi ne esiste già almeno uno: `reg` inutilizzata).
- **KS-AH-78-3**: se compile-side > 30% dell'alloc per-request nel census, il verdetto A-BB1 è NON GIUDICABILE — sospenderlo finché la cache Module (o l'attribuzione separata) non isola il canale.

### 5. Bak — alloc-rate/code-cache/path caldi — CONCORDO CON EMENDAMENTI

Su S-77.6.5.2.4 in sé (documentation-only, binario 02aa2ad8 invariato, 1651/1/0) nulla da refutare. Ma il programma WP-78, così scritto, **non può emettere un verdetto valido su A-BB1**: la riga "verify ≤+2% alloc overhead vs CLI baseline" misura la cosa sbagliata su questo codice.

**Analisi dei quattro punti caldi**

1. **Ricompilazione per-richiesta** (`worker_pool.rs:198,211`: `lower_source` + `compile_program` in `execute_with_retain`) — INQUINA A-BB1. Ogni richiesta rialloca Program+Module interi; il beneficio dichiarato del RetainSet persistente ("class cache, interning persist") è in gran parte negato perché il Module rinasce a ogni giro. Un numero unico alloc/req mescola fase-compile e fase-run: se il ≤+2% passa, passa per caso; se fallisce, non sappiamo quale motore incolpare. Lezione WP-59/64: il compile-side era il 65% non-censito — non ripetiamo l'errore.
2. **`mpsc::unbounded_channel`** — NON inquina se e solo se il carico è chiuso-sequenziale (depth≤1); con carico concorrente il picco RSS misura la coda (body+source Vec per task accodato), non il motore. Debito da registrare.
3. **`to_bytes(usize::MAX)`** (`main.rs:102`) — non inquina il workload hello-world; debito DoS/memoria da registrare. Aggravante: **`meta.body` non arriva mai alla VM** (`vm.request_start(None, &[])`, riga 229) — il body è allocato, copiato via canale e buttato. Alloc morta contata come "overhead Axum".
4. **Round-robin `AtomicUsize` + `blocking_recv`** — head-of-line: inquina solo le latenze, non l'alloc. MA il round-robin inquina il **footprint**: default `num_workers=num_cpus` (main.rs:160, hardcoded `0`, **nessun flag `--workers`**) ⇒ N RetainSet vivi, e N=100 richieste sequenziali distribuite su N core danno ~100/N richieste per worker — lo steady-state per-RetainSet (regola R≥3 per-worker) non viene mai raggiunto e il footprint censito è N× quello confrontabile col CLI.

**Emendamenti**

- **A-BB2 (WP-78, vincolante)**: A-BB1 si misura **per-fase**: contatori alloc separati per (a) lower+compile, (b) run, (c) shutdown/end — predizione-misurata col contatore del canale PRIMA (regola WP-45..48). Il verdetto ≤+2% si emette su (b)+(c) E sul totale, riportati separatamente; un numero solo = verdetto nullo.
- **A-BB3 (WP-78, vincolante)**: aggiungere `--workers N` a main.rs; tutte le misure A-BB1/footprint girano con `--workers 1`, carico chiuso-sequenziale, steady-state R≥3 sul singolo RetainSet. Documentare che il default num_cpus moltiplica il footprint retained per N.
- **A-BB4 (WP-78, registrazione; fix WP-79+)**: registro debiti — (i) canale bounded + `try_send`→503; (ii) limite body + 413; (iii) coda per-worker osservabile o work-stealing contro head-of-line. Nessuno blocca WP-78 se A-BB3 è rispettato.
- **A-BB5 (WP-78, vincolante)**: risolvere il body morto PRIMA della misura: o si collega a `request_start` o si toglie da `WorkerHandlerMeta` per il censimento — altrimenti l'alloc del canale misura codice morto. (Minore, stesso commit: `reg` inutilizzato in `worker_loop:125`.)
- **A-BB6 (post-censimento, non prima)**: cache Module keyed su path — è la leva ovvia, ma frequenza×taglia sul workload PRIMA di progettare (regola WP-57).

**Kill-switch proposti**

- **KB-78-1**: se la fase compile supera il **50% dell'alloc per-richiesta** sul workload A-BB1 e il report non è per-fase ⇒ verdetto A-BB1 VOID.
- **KB-78-2**: profondità coda mpsc >1 in qualunque campione della run di misura ⇒ run VOID (inquinamento da backpressure assente).
- **KB-78-3**: run A-BB1/footprint con workers≠1 o senza R≥3 per-RetainSet ⇒ VOID.
- **KB-78-4**: se con `--workers 1` due richieste identiche divergono in alloc-count oltre lo spread della coppia build-adiacente (±2%) ⇒ sospetto leak/warm-up non dichiarato: fermare il censimento, aprire attribuzione stile WP-71.

### 6. Pedersen — disciplina di confine per-richiesta, lifecycle — CONCORDO CON EMENDAMENTI

Verifica sul codice, non sui verbali.

**(1) execute_with_retain() rispetta A-PP1.** Ordine verificato: fatal renderizzato in `vm.rendered` (r.247-252) → `request_shutdown()` (r.256) → capture di `rendered`+`stdout` (r.263-265) → `request_end()` (r.269). Anche il percorso Exit passa da shutdown prima della capture (coerente con Zend: le shutdown function girano dopo exit). L'ordine c'è. Ma l'"implementazione" dell'emendamento è un commento: nessun test morde se qualcuno inverte le righe 263-269. Una regola permanente affidata a un commento è una regola volontaria.

**(2) La legacy execute() VIOLA la regola permanente.** Righe 328-334: `request_shutdown()` → `request_end()` → *poi* capture di `rendered`/`stdout`. Esattamente l'ordine vietato. Aggravanti: (a) la nota di deprecazione indirizza a `execute_with_vm`, funzione che **non esiste** (si chiama `execute_with_retain`) — il copia-incollatore futuro non ha nemmeno un riferimento corretto; (b) se `request_end()` azzera i buffer, quella funzione restituisce corpo vuoto: non è "legacy", è rotta. Codice morto che viola una binding rule non si deprecata, si rimuove.

**(3) Fatal/exit: ordine ok, ma vedi (4).**

**(4) Il confine HTTP NON riflette il confine PHP.** Fatal runtime → `StatusCode::OK` (r.271, incondizionato) con corpo byte-parity; errore di compilazione → `500` con corpo **non**-parity (`"PHP Fatal: …"` vs `"Fatal error: …"`, r.201). php-fpm risponde 500 su fatal a headers non inviati. Oggi abbiamo l'incoerenza peggiore: stessa classe di errore, due status e due formati di corpo a seconda della fase in cui muore.

**Minore:** `worker_loop` r.125 dichiara `let reg = registry()` mai usato, col commento "once for all requests", mentre `execute_with_retain` richiama `registry()` per-request (r.193): il commento promette un'ottimizzazione che non esiste.

**Emendamenti:**

- **A-PP2** (WP-77.6.5.2.5 o primo atto WP-78, PRIMA della misura): rimuovere integralmente la legacy `execute()` (r.274-338). Non deprecare: eliminare. Correggere contestualmente la doc `execute_with_vm`→`execute_with_retain` ovunque residui.
- **A-PP3** (WP-78): la regola permanente deve avere un gate che morde: unit test con output bufferizzato (`ob_start` senza flush esplicito), due richieste sequenziali sullo stesso RetainSet, assert corpo non-vuoto e byte-identico; **più controllo positivo** — assert che `vm.rendered`/`vm.stdout` risultino azzerati *dopo* `request_end()`, dimostrando che la capture post-reset perderebbe i byte (contatore tutto-zero senza controllo positivo = gate finto).
- **A-PP4** (WP-78, prima del freeze del contratto Axum): unificare il confine HTTP — decisione esplicita: fatal (compile E runtime) → 500 con corpo byte-parity `"Fatal error: …"`, oppure 200-always dichiarato e gateato. Oggi il contratto è accidentale.
- **A-PP5** (contestuale ad A-PP2, doc-only): sanare `worker_loop` — rimuovere il `reg` inutilizzato o passarlo davvero a `execute_with_retain`; il commento deve descrivere il codice che c'è.

**Kill-switch proposti:**

- **KS-PP-1**: il test A-PP3 fallisce (corpo vuoto su seconda richiesta, o buffer non azzerati post-`request_end`) → merge respinto, nessuna misura WP-78 valida su quel binario.
- **KS-PP-2**: grep-gate CI sul crate php-server: qualunque funzione in cui una lettura di `vm.rendered`/`vm.stdout` segue lessicalmente `request_end()` → reject automatico (la disciplina di confine si chiude su TUTTI i verbi, non per fix puntuali).
- **KS-PP-3**: se WP-78 tocca qualunque riga del path `request_start/shutdown/end`, G-APERTURA-2 va rilanciato per NOME dei test, non per conteggio.

**Su WP-78:** scope di misura condivisibile, ma A-PP2/A-PP5 vanno chiusi *prima* del Tier-0 baseline (codice morto nel crate non altera i numeri ma altera il binario che li firma), e A-PP4 prima di dichiarare "contratto HTTP stabile".

### 7. Leijen — allocatore mimalloc/footprint fisico — CONCORDO CON EMENDAMENTI

Sui 6 emendamenti di S-77.6.5.2.4: implementazione verificata nel codice (A-MS1 in worker_pool.rs:129-131, A-PP1 in :258-262, A-DS1 in :184-188, A-SK1/A-BG1/A-AH1 in main.rs:13-38), binario invariato — coerente con "documentation-only". Ma una sessione documentation-only che lascia in piedi documentazione FALSA è mezza fallita: l'header di main.rs (righe 5-11) descrive ancora "thread-local Vm (N=1 reuse pattern)" e "Vm::reset() clears state between requests" — l'architettura N=1 è stata RESPINTA in WP-77.2 e Vm::reset() esplicitamente non implementato. Contraddice i commenti approvati venti righe sotto.

Sul programma WP-78 rispetto ad A-DL1: lo scope come scritto in NEXT_SESSION ("Fragmentation analysis: vmmap + mimalloc stats") è un titolo, non un protocollo. Quattro difetti:

1. **Termine lineare nei core invisibile.** `WorkerPool::new(pool_context, 0)` in main.rs:160 è cablato a num_cpus, senza flag. Il RetainSet per-worker persiste "object table, class cache, string interning pool" (worker_loop): il residente cresce ~linearmente con W, e il round-robin spalma N=100 richieste su W worker (≈100/W ciascuno) — il piano "N=100 steady-state" senza distinguere per-worker da aggregato sottostima la ritenzione per-worker e non falsifica la linearità. Nota: la stima "~50 bytes" del verbale precedente vale per la struct, non per il retained set che cresce con le richieste.

2. **Transiente vs residente non distinti.** La ricompilazione del Module a ogni richiesta (lower_source+compile_program, worker_pool.rs:198-217) è churn puro che mimalloc deve riassorbire: il peak fisico lo cattura, il residente no. Servono ENTRAMBI, con metodo dichiarato. Aggravante: `/usr/bin/time -l` e MIMALLOC_SHOW_STATS riportano solo a EXIT del processo — il server non ha un percorso di uscita pulita; kill -9 = zero stats.

3. **Regole storiche assenti dal piano.** Nessuna menzione di /usr/bin/time -l, vmmap Physical footprint, MIMALLOC_PURGE_DELAY=0, divieto di RSS ps. Non date per scontato ciò che WP-52 ha pagato per imparare.

4. **Global allocator: OK con caveat.** `#[global_allocator]` in main.rs:43-44 è process-wide: copre worker thread e runtime tokio, le stats mimalloc sono valide per tutto l'heap. Caveat: la comparazione col baseline CLI vale solo a parità di versione mimalloc nel Cargo.lock e in coppia build-adiacente stessa-sera (regola WP-57/65). Difetto collaterale: in worker_loop `let reg = registry()` (riga 125) è inutilizzato e registry() viene richiamato per-richiesta in execute_with_retain (riga 193): se non è lazy-static cached, inquina la misura del churn.

**Emendamenti:**

- **A-DL2 (WP-78, BINDING)**: flag `--workers N` in php-server; misurare W=1 e W=num_cpus con ≥100 richieste PER worker; riportare residente per-worker e verificare footprint(W) ≈ base + W·k. Senza questo, A-DL1 non è eseguibile correttamente.
- **A-DL3 (WP-78, BINDING)**: protocollo scritto in design78.md: peak = /usr/bin/time -l (richiede shutdown pulito — aggiungere SIGTERM/endpoint di uscita), residente = vmmap Physical footprint con MIMALLOC_PURGE_DELAY=0, stats mimalloc via MIMALLOC_SHOW_STATS=1 a uscita pulita; RSS ps vietato. Distinzione esplicita transiente(compile-churn)/residente.
- **A-DL4 (WP-78, BINDING)**: firma intera-esatta di crescita per-worker: contatore entries RetainSet (stringhe internate, class cache) per richiesta, R≥3 batch, con controllo POSITIVO (fixture-leak che il contatore DEVE vedere — lezione WP-71/72).
- **A-DL5 (pre-flight WP-78)**: correggere l'header stale di main.rs; verificare che registry() sia cached (rimuovere il binding morto riga 125).

**Kill-switch (strutturali, mai pin numerici):**

- **KL-78-1**: a W fisso, Δ vmmap Physical footprint per-richiesta a steady-state non converge a ~0 (oltre rumore ±2%) su R≥3 batch → HALT WP-78, riaprire C-leak.
- **KL-78-2**: footprint(W) non lineare in W o incremento per-worker che cresce coi batch → HALT.
- **KL-78-3**: qualunque cifra di footprint prodotta senza MIMALLOC_PURGE_DELAY=0 o via ps RSS → cifra NULLA, run da ripetere.

### 8. Stogov — semantica Zend engine — CONCORDO CON EMENDAMENTI

(vincolanti; A-DS1 dichiarato NON conforme e da rifare prima della chiusura WP-78)

**Refutazione del punto centrale.** La matrice A-DS1 in `worker_pool.rs:184-188` ("Persist cross-request: statics, closure_statics — matches PHP CLI behavior") è sbagliata due volte.

1. *Giustificazione sbagliata.* Un server HTTP non deve matchare la CLI: deve matchare PHP-FPM/mod_php, dove ogni richiesta è un request lifecycle completo (RINIT→RSHUTDOWN). In Zend gli static di function/method e le static properties di classe sono stato userland: vengono distrutti e reinizializzati a ogni richiesta (in 8.x via `static_variables_ptr` per-request); solo opcache persiste, e persiste bytecode/dati immutabili, MAI stato userland. `static $n=0; $n++;` deve stampare 1 a ogni richiesta. "Matches PHP CLI behavior" come contratto di un web server è la definizione di un bug, messa a verbale come invariante.

2. *Descrizione falsa del codice.* Ho verificato il runtime: `statics` (`vm/mod.rs:544`, `vec![None; module.static_count]`), `closure_statics` (2877) e `static_props` (543) sono campi del **Vm**, e il Vm è creato per-richiesta (`worker_pool.rs:225`) e muore a fine richiesta. Quindi oggi gli static NON persistono cross-request: il codice implementa de facto la semantica FPM corretta, e il doc-comment la contraddice. Peggio: il file si contraddice da solo — righe 118 e 224 dicono "request_end() resets ephemeral state (globals, **statics**, etc)", riga 187 dice che gli static persistono. Ho letto il corpo di `request_end()` (`vm/mod.rs:3418-3526`): non tocca né `statics` né `closure_statics` né `static_props`. Nessuna delle due frasi è vera: l'isolamento degli static avviene per **morte del Vm**, non per reset. La matrice è una mina: se WP-78+ rendesse il Vm persistente "rispettando il contratto documentato", introdurrebbe la divergenza da FPM che il gate hello-world a 2 richieste è strutturalmente cieco a vedere (nessuna fixture con stato).

**Punti confermati.** Ordine `request_shutdown()` prima di `request_end()`: rispettato (righe 256→269), con capture dell'output tra i due (A-PP1, corretto). `Exit` trattato come terminazione normale con shutdown functions eseguite: conforme a Zend. Fatal: shutdown eseguito, conforme. **Non** conforme: il path fatal ritorna `StatusCode::OK` (riga 271) — FPM risponde 500 quando gli header non sono ancora partiti. Noto anche che `meta.method/body` sono ignorati (superglobali non popolate): accettabile come stub dichiarato, non come parità.

**Emendamenti:**
- **A-DS2** (WP-78 apertura, bloccante): riscrivere la matrice A-DS1. Contratto = FPM: statics/closure_statics/static_props sono per-richiesta OGGI (per morte del Vm) e DEVONO restarlo anche se il Vm diventasse persistente. Eliminare "matches PHP CLI behavior". Ogni riga della matrice deve attribuire il MECCANISMO reale (Vm drop vs request_end() vs mass-teardown RetainSet), correggendo le righe 118/224.
- **A-DS3** (WP-78, prima del census): estendere G-APERTURA-2 con fixture STATEFUL a ≥3 richieste: `static $n` in funzione, static property di classe, `static` in closure, `define()`. Atteso: output identico a ogni richiesta (contatore sempre 1). Gate per nome.
- **A-DS4** (WP-78): fatal → HTTP 500 pre-header, con test.
- **A-DS5** (WP-78 scope): approvo il programma misura, con vincolo: Tier-0 deve separare compile-side da execute-side (il Module è ricompilato per-richiesta, riga 197-217 — lezione WP-59: il compile-side domina e falserebbe il census del pool).

**Kill-switch:**
- **KS-DS-78-1**: fixture static-counter, richiesta N≥2 stampa ≠1 ⇒ FATAL, blocca merge.
- **KS-DS-78-2**: dopo `request_end()`, residuo strutturale RetainSet ≠ firma WP-72 (used_n=0) ⇒ FATAL.
- **KS-DS-78-3**: qualunque proposta di Vm persistente senza A-DS3 verde prima ⇒ respinta d'ufficio.

### 9. Gregg — metodologia di misura/attribuzione — CONCORDO CON EMENDAMENTI

Ho letto NEXT_SESSION, WP_SESSION_77_6_5_2_4, il verbale .3 e il codice (main.rs, worker_pool.rs). A-BG1 risulta trascritto (main.rs righe 22-28): il testo è fedele al mio mandato. Ma la sessione .4 ha eletto la documentazione a contratto ("Documentation as Enforcement", Lezione 2): allora la giudico da contratto, e il contratto è incoerente.

**Rilievo 1 — doc di crate STALE che contraddice l'architettura approvata.** main.rs righe 5-11 descrivono ancora "Thread-local Vm (N=1 reuse pattern)" e "Vm::reset() clears state between requests": è l'architettura RESPINTA in WP-77.2. L'architettura approvata è Vm per-request + RetainSet persistente, senza Vm::reset(). Inoltre nel mio blocco A-BG1 è stata infilata la riga "Boot variance: Controlled via sequential request pattern" — claim non mio e non misurato: il pattern sequenziale non "controlla" la variance, la ignora.

**Rilievo 2 — il verbale .3 contiene un numero mai misurato.** La sezione Bak asserisce "Alloc overhead ≤+1.5% misurato steady-state" mentre lo stesso verbale rinvia OGNI misura a WP-78. È una predizione travestita da misura; regola di progetto: predizione-misurata = contatore del canale PRIMA. Va rietichettata.

**Rilievo 3 — "N=100 steady-state" è refutabile così com'è.** worker_pool.rs: dispatch round-robin su num_cpus worker → con N=100 richieste ogni worker ne riceve ~100/num_cpus (~8-12 su questa macchina). Lo steady-state per-worker NON è raggiunto; peggio, la RSS aggregata somma num_cpus arene fredde. Inoltre worker_loop non ha alcun contatore per-worker (né richieste servite né taglia RetainSet): attribuzione impossibile.

**Rilievo 4 — denominatore A-BB1 indefinito.** "≤+2% alloc vs CLI baseline": CLI = 1 processo/1 richiesta con compile incluso; il server ricompila il modulo A OGNI richiesta (Phase 1-2 di execute_with_retain) — quindi il confronto server-vs-CLI confonde compile-side (che da WP-59 è il 65% del footprint) con il costo del pool. Confronto onesto: stesso binario, stessa fixture, stesso N, `--cli-server` vs `--axum`.

**Rilievo 5 — R-G4 su macOS.** `sample` non vede nell'inlining (lezione WP-53/54). L'attribuzione CPU va fatta col metodo owner-level: contatori ns/evento instrumentati, su coppia build-adiacente strumentata/non-strumentata (lezione WP-33: un branch nel run_loop costa +2,9% — il probe non si misura addosso a sé stesso, lezione WP-64).

**Emendamenti**:

- **A-BG2** (WP-78, primo atto): correggere main.rs righe 5-11 (doc N=1/Vm::reset() → RetainSet persistente + Vm per-request) e rimuovere/riformulare la riga "Boot variance: Controlled". Doc-contratto incoerente = difetto, non cosmesi.
- **A-BG3** (WP-78, design78.md, PRIMA di ogni run): protocollo scritto — pool forzato a `num_workers=1` per lo steady-state per-worker (oppure N ≥ 100×num_cpus); warm-up escluso = prime 10 richieste; R≥3 run, spread accettato ±2%; peak fisico SOLO `/usr/bin/time -l` + vmmap Physical footprint, MAI RSS di ps; ogni A/B in coppia build-adiacente stessa-sera.
- **A-BG4** (WP-78): definire il denominatore A-BB1 = stesso binario, `--cli-server` vs `--axum`, stessa fixture, stesso N; rietichettare il "≤+1.5%" del verbale .3 come predizione da verificare, non baseline.
- **A-BG5** (WP-78): probe di amplificazione in modalità Axum: RSS(N) vs RSS(2N) con delta per-richiesta intero-esatto ≈0 (prova regina WP-71/72) + contatore per-worker richieste/taglia RetainSet feature-gated.
- **A-BG6** (WP-78+, se R-G4 commissionato): attribuzione CPU solo via contatori ns/evento owner-level; niente verdetti da solo `sample`.

**Kill-switch**:
- **KG-78.A**: run di misura senza protocollo A-BG3 scritto e con pool ≠ 1 worker (o N < 100/worker) → misura INVALIDA, nessun verdetto su A-BB1/A-DL1.
- **KG-78.B**: spread tra R≥3 run > ±2% → nessun numero entra nel verbale; si indaga la variance, non si media.
- **KG-78.C**: delta per-richiesta RSS(N)→RSS(2N) strutturalmente > 0 → leak RetainSet in modalità server: BLOCCO merge di ogni leva WP-78, riapertura C4.
- **KG-78.D**: qualunque cifra alloc/CPU citata in verbale senza run tracciato (comando+log) → cifra respinta d'ufficio.

---

**Chiusura concilio**: 2026-07-31. Verbali VINCOLANTI per il design WP-78 (S-78.0 sanatoria pre-misura obbligatoria).
