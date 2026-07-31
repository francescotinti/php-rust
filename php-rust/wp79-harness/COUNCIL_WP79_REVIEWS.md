# COUNCIL_WP79_REVIEWS.md — Concilio a 9 sedie su S-78.0 (sanatoria) + programma fase misura WP-78

**Data**: 2026-07-31 (chiusura S-78.0)
**Oggetto**: revisione di S-78.0 (commit 31201ba…f0bf015) + giudizio di eseguibilità del protocollo di misura (`wp78-harness/design78.md`)
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto NEXT_SESSION, WP_SESSION_78, design78.md, il proprio verbale WP-78 (verifica onorato/scoperto) e il codice del proprio perimetro.
**Status**: verbali VINCOLANTI per la fase misura WP-78 (blocco ⚖️ in NEXT_SESSION).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO CON EMENDAMENTI — nessuna opposizione.**
**L'opposizione Klabnik di WP-78 è SOLLEVATA**: G-APERTURA-2 è ora falsificabile e ha MORSO (bug body-raddoppiato trovato dal primo assert a contenuto assoluto). La sanatoria S-78.0 è giudicata REALE su tutti i 9 punti; il verdetto duro è sul **programma misura**: per Gregg «design78 è NON ESEGUIBILE così com'è» — tre strumenti richiesti dai kill-switch non esistono nel codice.

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **Purga doc INCOMPLETA — la mina A-DS1 sopravvive in 7+ punti** (Matsakis c.6, Hejlsberg c.7, Stogov c.2): worker_pool.rs:66/:77 «persistent Vm», :119/:172/:180 «object table, class cache persist» (la RetainSet parcheggia SOLO moduli unit), e — peggio — vm/mod.rs:3417-3420 doc di `request_end()` dice ancora «preserving persistent data (statics, …)»: statics dichiarati persistenti NEL CRATE RUNTIME. → A-MS6/A-AH8/A-DS6.
2. **Worker senza join: le stats a exit firmerebbero thread uccisi** (Matsakis c.7, Leijen c.3): i JoinHandle sono buttati (worker_pool.rs:103-105); `MIMALLOC_SHOW_STATS`/`time -l` a exit leggono uno stato non-deterministico. → A-MS7/A-DL6 [KS-MS-5, KL-78-4]. Correlato: **un panic Rust nel worker uccide il thread PER SEMPRE** — 1/N richieste morte in silenzio, nessun respawn/catch (Pedersen c.6) → A-PP9 [KS-PP-6].
3. **I kill-switch di misura sono INOSSERVABILI senza contatori** (Hoare c.9, Bak c.3-4, Leijen c.5-6, Gregg c.2-3-6, Stogov c.6): depth coda (KH78-2/KB-78-2), per-worker richieste/taglia RetainSet (A-BG5/KG-78.C), used_n post-request_end (KS-DS-78-2), contatori per-fase (A-BB2). NESSUNO esiste nel codice; «un KS senza osservabile è un gate che benedice» (WP-72). → la fase misura DEVE aprirsi con uno step di STRUMENTAZIONE feature-gated con controlli positivi, e i verdetti footprint escono SOLO dal gemello non strumentato. → A-BB7/A-BB8/A-BG7/A-BG11/A-DL7 [KB-78-5/6, KL-78-5, KG-79.A].
4. **La parity dei corpi fatal è AFFERMATA, mai cmp'd** (Hoare c.8, Pedersen c.6, Stogov c.3): compile fatal ha `Stack trace:` in coda, runtime fatal NO; parse error un terzo formato; il test usa `contains("Fatal error:")` — stessa classe del `head -1` refutato dalla Lezione 1 di S-78.0. L'oracolo per undefined function stampa `Uncaught Error: …`. → fixture fatal in run-gate con cmp full-body + formato unificato PRIMA del freeze del contratto HTTP → A-TH8/A-PP10/A-DS7 [KH79-2, KS-DS-78-5].
5. **Il grep-gate KS-PP-2 ha buchi**: non ricorsivo e solo php-server (Hoare c.7a), cieco interprocedurale (Hoare c.7b, Klabnik c.6), si disarmerebbe in silenzio quando A-TH4 cambierà la firma (Hoare c.7c → KH79-1), buco same-line (Pedersen c.4), marker allow abusabile ovunque (Pedersen c.3 → A-PP6 [KS-PP-4]). Contenimento stdout⊆rendered vero oggi ai siti di scrittura ma NON gateato (Pedersen c.2 → A-PP8 [KS-PP-5]).
6. **Identità del gate non chiusa** (Klabnik c.2, Hejlsberg c.4): l'hash è del file a riposo — nessuna prova che la build abbia scritto $BIN (CARGO_TARGET_DIR esterno) né che il PID sulla porta sia $SRV. → A-SK8/A-AH7 [KS-SK-79.1, KS-AH-78-5].
7. **Vietato il test a sola auto-uguaglianza** (Klabnik c.3, Stogov): `gate_apertura2_two_requests…` asserisce solo b1==b2 — un body raddoppiato lo passerebbe OGGI. Regola: ogni assert di parità convive con un assert assoluto/oracolo. → A-SK7 [KS-SK-79.2, KS-DS-78-5].
8. **La superficie RetainSet non è esercitata da NESSUN gate** (Stogov c.5c): nessuna fixture con include/require — il parking nella FrozenVec (unico meccanismo di persistenza, append-only, superficie di KL-78-1) è invisibile ai gate. → A-DS8 [KS-DS-78-4]; + static in metodo/ereditati (A-DS9); divergenza html_errors da registrare (A-DS10).
9. **Protocollo di misura da emendare prima di eseguirlo** (Gregg c.4-5, Leijen c.2, Bak c.3): warm-up «prime 10» arbitrario e INCOMPATIBILE con le metriche di picco processo-lifetime (KG-79.C); denominatore A/B = SOMMA di {tokio+canale+pool+registry-once+retention}, non "overhead del pool", e il ramo cli-server NON ha shutdown pulito (KG-79.B); mancano i punti di snapshot vmmap (A-DL8); Tier-0 senza configurazione eseguibile (KG-79.D); KH78-2 va riformulato a meccanismo (A-BG9). k della linearità = registry+RetainSet+stack (N copie Registry per N worker, Leijen c.4).
10. **KH78-1 resta bloccante e ora ha un CONTRATTO scritto** (Klabnik A-SK6, Hoare c.6): W≥2, ≥3×W richieste concorrenti, ogni body cmp vs expected, grep panic sul log, PASS solo con comando+hash [KS-SK-79.3].

### Ordine vincolante di apertura prossima sessione (S-78.1 "hardening pre-census", poi misura)

1. **Doc purge residua** (A-MS6/A-AH8/A-DS6: worker_pool.rs:66,77,119,172,180; vm/mod.rs:447,452,3417-3420) + gate KS-MS-2 eseguibile con esca positiva (A-MS5) [KS-MS-4].
2. **Lifecycle worker**: JoinHandle conservati + `shutdown()` (drop sender + join) chiamato dopo il drain (A-MS7/A-DL6) + politica panic dichiarata e gateata (A-PP9) [KS-MS-5, KL-78-4, KS-PP-6].
3. **Gate hardening**: grep-gate ricorsivo, pattern `request_end\(`, same-line fix, marker vincolato per forma+censimento (A-TH7/A-PP6/A-PP7) [KH79-1, KS-PP-4]; run-gate: PID==SRV, mtime/hash pre-post build, BIN da cargo metadata, expected rigenerati dall'oracolo con comando a commit (A-SK8/A-AH7) [KS-SK-79.1, KS-AH-78-5]; assert assoluto in gate_apertura2 (A-SK7) [KS-SK-79.2]; git rev in ogni verdict (A-SK9); `._*` fuori dalle fixtures (A-SK10); warnings fatali anche sui target test (A-AH6); nm esteso a axum|tokio|hyper|tower (A-AH7).
4. **Contratto fatal**: formato unificato + fixture fatal compile E runtime in run-gate con cmp full-body vs oracolo 8.5.7 + `exit(code)` a contratto (A-TH8/A-PP10/A-DS7) [KH79-2, KS-DS-78-5]; divergenza error-page in PHPR_DIVERGENCES_FROM_PHP.md (A-DS10); gate contenimento stdout⊆rendered (A-PP8) [KS-PP-5].
5. **Fixture estese**: include/require con `RetainSet::len()` stabile post-warm-up (A-DS8) [KS-DS-78-4]; static in metodo + ereditati (A-DS9).
6. **Strumentazione census** (feature-gated, controlli positivi obbligatori, verdetti footprint SOLO dal gemello non strumentato): contatori per-fase (A-BB7/A-BG11), per-worker richieste+RetainSet len+used_n accessor (A-BG7/A-DL7 + Stogov c.6), depth canale con burst positivo (A-BB8/A-TH9) [KB-78-5/6, KL-78-5, KG-79.A].
7. **design78 emendato**: snapshot vmmap dichiarati (A-DL8), warm-up ridefinito (A-BG8) [KG-79.C], A/B verbalizzato con lifecycle cli-server verificato (A-BG10) [KG-79.B], KH78-2 a meccanismo + spread definito (A-BG9), Tier-0 config tracciata o esce (KG-79.D), micro-igiene path caldo (A-BB9: file_s nel branch, clone evitabile).
8. **KH78-1** secondo contratto A-SK6 [KS-SK-79.3].
9. Solo DOPO: Tier-0 + census (A-BB1/A-DL1 si giudicano qui). Al primo rebuild phpr: audit `git diff Cargo.lock` = solo static_assertions, poi corpus per NOME (A-AH9) [KS-AH-78-4].

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH79-1 | A-TH4 atterra senza aggiornare il pattern del grep-gate nello stesso commit | gate invalido, reject |
| KH79-2 / KS-DS-78-5 | claim di parity su corpi d'errore senza cmp full-body vs oracolo | respinto d'ufficio / HALT freeze |
| KS-MS-4 | ricomparsa di "RetainSet is Send" / "persistent Vm" nei sorgenti | gate FAIL |
| KS-MS-5 / KL-78-4 | cifra residente/stats a exit senza join dei worker | cifra respinta / stats ADVISORY |
| KS-SK-79.1 / KS-AH-78-5 | PID sulla porta ≠ $SRV, o $BIN non aggiornato dalla build (mtime stale) | run del gate NULLA |
| KS-SK-79.2 | nuovo test/gate a sola auto-uguaglianza | gate invalido, PASS non dichiarabile |
| KS-SK-79.3 | census avviato senza verdict KH78-1 conforme A-SK6 | misure NULLE |
| KS-AH-78-4 | primo rebuild phpr con delta Cargo.lock ≠ sola static_assertions | HALT, audit |
| KB-78-5 / KL-78-5 | cifra footprint/peak da build strumentata | cifra NULLA (vale il gemello) |
| KB-78-6 / KG-79.A | census senza controlli positivi dei contatori visti | run VOID |
| KS-PP-4 | occorrenze marker allow difformi dal censimento | gate FAIL |
| KS-PP-5 | scrittura stdout senza tandem rendered in php-runtime | reject (decade body=rendered) |
| KS-PP-6 | worker morto (panic) durante un run di misura | run VOID, HALT freeze |
| KS-DS-78-4 | RetainSet::len() cresce oltre le unit distinte post-warm-up | FATAL, riapertura C-leak |
| KG-79.B | A/B con un braccio senza stats a uscita pulita | coppia VOID |
| KG-79.C | "warm-up escluso" su metrica di picco processo-lifetime | cifra respinta |
| KG-79.D | "Tier-0" senza configurazione eseguibile tracciata | baseline nulla |

---

## VERBALI INTEGRALI

### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

**Constatazioni (verificate sul codice, albero corrente)**

1. **A-TH1 onorato integralmente**: `RequestHandler`/`execute()` non esistono più in `crates/php-server/src/worker_pool.rs` (417 righe, nessuna occorrenza); l'unico entry-point è `execute_with_retain` (worker_pool.rs:205) con capture PRIMA del reset (`mem::take(&mut vm.rendered)` riga 288, `request_end()` riga 292). Nessun `unsafe` in worker_pool.rs né main.rs: safe-only confermato.
2. **A-TH2→A-MS2/A-MS3 onorati**: SAFETY riscritti con l'invariante vero (worker_pool.rs:15-24, 142-144); `assert_not_impl_any!(RetainSet: Send, Sync)` vive in php-runtime (vm/mod.rs:435) con clausola KS-MS-1 nel commento; controllo positivo `assert_impl_all!(WorkerTask: Send)` (worker_pool.rs:64).
3. **A-TH3 onorato**: main.rs:5-11 dichiara esplicitamente il rigetto WP-77.2 dell'architettura N=1; nessun residuo `Vm::reset()`.
4. **A-TH5 onorato nel codice**: ogni fatal → 500 (worker_pool.rs:225, 229, 239, 296-300) — KS-M3 riconciliato per fix, non per rettifica; test 393-411 col worker che sopravvive.
5. **A-TH6 onorato**: ci.yml:36-45 esegue la terna con `-D warnings` scopato via `cargo rustc` + `cargo test -p php-server --features axum-server`.
6. **KH78-1 correttamente BLOCCANTE**: primo atto in NEXT_SESSION §WP-78 e design78.md §Ordine punto 1; non ancora creato ma dichiarato come residuo. Conforme.
7. **REFUTO — il grep-gate KS-PP-2 ha tre buchi lessicali** (`wp78-harness/gate-capture-order.sh`): (a) scansiona solo `crates/php-server/src/*.rs` NON ricorsivo e non copre gli altri call-site di `request_end()` (php-cli SAPI, harness) — il verbale diceva "in nessun call-site"; (b) è cieco interprocedurale: un helper `fn take_output(&mut Vm)` che legge `.rendered`, chiamato dopo `request_end()`, passa pulito; (c) il pattern di armamento `request_end\(\)` si DISARMERÀ silenziosamente quando A-TH4 cambierà la firma in `request_end(self)` (WP-79) — il gate morirebbe esattamente nel commit che più lo richiede. Il self-test positivo (righe 32-43) c'è ed è corretto, ma copre solo il caso intra-funzione.
8. **REFUTO — il claim "corpo fatal byte-parity CLI" (verbale S-78 punto 9) è UNGATED**: il path runtime (worker_pool.rs:273) rende `Fatal error: … on line N` SENZA coda `Stack trace:` mentre il path compile (riga 222) la include; il test asserisce solo `contains("Fatal error:")` (riga 401) e run-gate.sh confronta con l'oracolo solo fixture hello/stateful, mai fatal. È la stessa classe d'errore del body raddoppiato: parity affermata senza confronto a contenuto assoluto.
9. **REFUTO — KH78-2/KB-78-2 (depth coda >1 ⇒ VOID) è inosservabile così com'è**: il canale resta `unbounded_channel` (worker_pool.rs:99) e NON esiste alcun contatore di profondità nel codice. "Chiuso-sequenziale per costruzione" senza probe è il claim che WP-72 vieta (contatore tutto-zero = controllo fallito): il kill-switch non può mai scattare.
10. Minore: `WorkerPoolContext` vuoto-TODO e parametro `_context` (worker_pool.rs:34-45, 131) — l'underscore elude lessicalmente `-D warnings`; deferito A-AH5, accettabile ma va nominato.

**Emendamenti (vincolanti)**

- **A-TH7 (pre-misura)**: gate-capture-order.sh: scan ricorsivo (`src/**/*.rs` + `tests/`), pattern d'armamento `request_end\(` (senza parentesi chiusa obbligatoria), e limite interprocedurale documentato nell'header dello script.
- **A-TH8 (pre-misura, prima del freeze del contratto HTTP)**: fixture fatal (compile E runtime) in run-gate.sh con `cmp` full-body contro l'oracolo CLI; risolvere la divergenza Stack-trace tra i due path.
- **A-TH9 (pre-misura, dentro KH78-1)**: contatore max-depth per worker stampato a shutdown, con controllo POSITIVO (run concorrente che DEVE mostrare depth>1); senza strumento, KH78-2 resta lettera morta.

**Kill-switch nuovi**

- **KH79-1**: se A-TH4 atterra senza aggiornare il pattern del grep-gate NELLO STESSO commit ⇒ gate invalido, reject (disarmo silenzioso = KS-PP-2 vacuo).
- **KH79-2**: `cmp` fatal-fixture ≠ 0 vs oracolo ⇒ contratto HTTP non congelabile, HALT sul punto 9.

### 2. Matsakis — ownership/aliasing/Send-Sync — CONCORDO CON EMENDAMENTI

Il nucleo del mio mandato S-78.0 è onorato (A-MS2/A-MS3/A-MS4 reali, KS-MS-1 nel type system), ma KS-MS-2 è onorato solo nello STATO, non nel MECCANISMO, e la purga documentale A-TH3 ha lasciato residui di "persistent Vm"/"object table" proprio nelle righe che il mio scranno deve sorvegliare.

**Constatazioni**

1. **A-MS2 ONORATO.** Il SAFETY dice ora il vero: worker_pool.rs:15-24 (`!Send+!Sync`, thread-affinità per costruzione, solo WorkerTask attraversa il canale) e :142-144 (costruzione dentro worker_loop, stack del worker). Coerente con la definizione `RetainSet(elsa::FrozenVec<Rc<Module>>)` (vm/mod.rs:427). Sparite le falsità TLS/await.
2. **A-MS3 ONORATO, posizionamento corretto.** `assert_not_impl_any!(RetainSet: Send, Sync)` a vm/mod.rs:435: top-level, nel crate che DEFINISCE il tipo, senza alcun `cfg` — compila in ogni build di php-runtime, quindi ogni consumer è protetto. Dep `static_assertions` incondizionata (workspace Cargo.toml:37, php-runtime:43, php-server:18). Il controllo positivo `assert_impl_all!(WorkerTask: Send)` (worker_pool.rs:64) è dentro il mod feature-gated ma FUORI da `cfg(test)`: compila in axum-only e union, entrambe in CI con -D warnings (ci.yml:38-40) — accettabile e di fatto più forte di un test-only.
3. **A-MS4 ONORATO.** La legacy `execute()` non esiste più; il grep-gate KS-PP-2 (`gate-capture-order.sh`) ha self-test positivo che morde.
4. **`std::mem::take(&mut vm.rendered)` (worker_pool.rs:288) è CORRETTO.** Move del `Vec<u8>` senza copia, borrow `&mut` sul campo chiuso prima di `request_end()` (:292); il buffer resta valido-vuoto per il reset. Nessun aliasing; il test positivo (:376-387) prova che il reset azzera davvero.
5. **KS-MS-2 SCOPERTO.** Verificato via search: **0 occorrenze** di "RetainSet is Send" in `crates/**/*.rs` — lo stato è pulito. Ma il kill-switch chiedeva un **grep-gate CI**: non esiste né in ci.yml né in gate-capture-order.sh né in gate-feature-matrix.sh. Per la Lezione 3 della stessa S-78.0 ("PASS senza comando che possa fallire = chiusura falsa"), KS-MS-2 oggi è una fotografia, non un gate.
6. **Residui di claim falsi di persistenza** (purga A-TH3/A-DS2 incompleta in worker_pool.rs): `:66` "each managing **1 persistent Vm**"; `:78` "Creates **persistent Vm** bound to RetainSet"; pseudo-codice `:85-87` con `task.handler` (campo inesistente); `:119` "arena for **object table, class cache**"; `:172` "per-thread persistent **object table**"; `:180` "(**object table, class cache** persist)"; `:185` "**objects** in RetainSet persist". E in php-runtime: vm/mod.rs:447 "persistent-Vm worker-pool integration", :452 "(repeated for persistent Vm)". La RetainSet parcheggia SOLO moduli unit (come dice correttamente :140-141 e :201-203): lo stesso file afferma due cose. Un futuro lettore che "rispetti" :180 reintrodurrebbe esattamente ciò che KS-DS-78-3 vieta.
7. **Lifecycle/ownership al shutdown**: `WorkerPool::new` scarta i JoinHandle (worker_pool.rs:103-105); il graceful shutdown (main.rs:171-188) drena Axum ma NON joina i worker → all'exit i thread sono uccisi senza eseguire il drop della RetainSet sul thread proprietario. Innocuo per la correttezza (!Send resta vero), ma `MIMALLOC_SHOW_STATS`/A-DL3 a exit conteranno le arene come vive: il census di WP-78 leggerebbe un finto residuo.

**Emendamenti**

- **A-MS5 (pre-misura)**: implementare KS-MS-2 come gate eseguibile (script o step CI: zero occorrenze di "RetainSet is Send" in `crates/**/*.rs`, con snippet-esca come controllo positivo, stile gate-capture-order.sh). Estenderlo a "persistent Vm" e "object table persist" nel perimetro php-server+vm/mod.rs (→ KS-MS-4).
- **A-MS6 (pre-misura, doc-only)**: purgare i residui della constatazione 6 (worker_pool.rs:66,78,85-87,119,172,180,185; vm/mod.rs:447,452) — lessico unico: "arena di moduli unit compilati".
- **A-MS7 (pre-misura, prima di qualunque cifra MIMALLOC_SHOW_STATS/vmmap a exit)**: WorkerPool conserva i JoinHandle ed espone `shutdown()` (drop dei sender + join) chiamato dopo il drain Axum: ogni RetainSet viene droppata sul SUO thread prima delle stats di exit.

**Kill-switch nuovi**

- **KS-MS-4**: ricomparsa di "RetainSet is Send" / "persistent Vm" nei sorgenti ⇒ gate FAIL (azionato dal gate A-MS5).
- **KS-MS-5**: qualunque cifra di residente/leak letta a exit su un binario SENZA join dei worker (A-MS7 assente) ⇒ cifra respinta d'ufficio — misurerebbe thread uccisi, non memoria del motore.

### 3. Klabnik — spec/testabilità/gate — CONCORDO CON EMENDAMENTI (opposizione WP-78 SOLLEVATA)

La sanatoria S-78.0 ha reso G-APERTURA-2 falsificabile e il gate ha MORSO (bug body-raddoppiato): è la prova che il mio A-SK2 era necessario, non pedanteria. Restano due vettori di falso-PASS e un gate ancora auto-referenziale.

**Constatazioni**

1. A-SK2 ONORATO: `wp78-harness/gate-axum/run-gate.sh` eseguibile, readiness poll senza sleep cieco (:53-58), cmp full-body (:81, :95-101), exit 0/1 deterministico (:110, :113), verdict con hash+timestamp (:109). KS-SK-78.1 e KS-SK-78.2 finalmente azionabili. A-SK4 onorato e superato (`ci.yml:36-45`: terna `-D warnings` + `cargo test --features axum-server`). A-SK5 onorato (NEXT_SESSION:15-16, rettifiche a verbale). KS-SK-78.3 verde.
2. FALSO-PASS residuo #1 — identità del server non verificata: l'hash è calcolato sul file `$BIN` a riposo (run-gate.sh:43), ma (a) nulla prova che la build del passo 1 abbia scritto proprio in `$BIN` (dipende da CARGO_TARGET_DIR, esterno allo script che resetta PATH a :20); (b) se il bind fallisce per un server stale sfuggito al `pkill` pattern-dipendente (:47), il poll (:56) parla col processo VECCHIO e il PASS cita l'hash del nuovo. Nessun check PID-sulla-porta == `$SRV`.
3. FALSO-PASS residuo #2 — il test che porta il NOME del gate è ancora auto-uguaglianza: `gate_apertura2_two_requests_same_retainset_byte_parity` (worker_pool.rs:322-331) asserisce solo `b1==b2` + non-empty. Un body `hello axum\n` raddoppiato lo passerebbe OGGI. Il fix chiude la classe solo dove esiste l'assert assoluto (test 2-4: `b"1"`, `b"buffered-bytes"`, `b"alive"`; shell-gate :81/:96) — l'istanza è chiusa, la classe no, finché sopravvive UN assert di sola parità.
4. Il "corpo oracolo" è un letterale a mano (run-gate.sh:69-70), non output di phpr/php-cli sulla fixture: expected sbagliato + implementazione coerentemente sbagliata = PASS. Per queste fixture banali è tollerabile; come metodo no.
5. A-SK3, deviazione ACCETTATA: php-server è bin-only (nessuna lib target), `tests/` non potrebbe importare `execute_with_retain`; la sostanza è rispettata — compilato sotto feature, eseguito in CI (ci.yml:44-45), ed è stato l'assert a contenuto assoluto (`body == b"1"`) a trovare il bug. Condizione: se nasce una lib target, migrazione obbligatoria a `tests/`.
6. `gate-capture-order.sh`: self-test positivo presente (:32-43) — bene. Ma il PASS non cita l'identità dell'albero scansionato (nessun git rev nel verdetto: un "PASS" senza àncora), e chiude solo i verbi `.rendered`/`.stdout`: un accessor futuro (`vm.take_output()`) bypassa il grep. La chiusura per costruzione è A-TH4 — correttamente deferita a WP-79, ma va detto che fino ad allora il grep-gate è lessicale, non semantico.
7. KH78-1: correttamente BLOCCANTE (NEXT_SESSION:50-51 primo atto; design78.md §Gate concorrente; KS attivo con HALT). Difetto: nessun contratto scritto — rischio di gate concorrente debole scritto di fretta in apertura misura.
8. Igiene: `fixtures/` contiene AppleDouble `._hello.php`/`._gate_stateful.php` (junk macOS) nel harness canonico del repo.

**Emendamenti**

- **A-SK6** (WP-78, prima del census): contratto KH78-1 SCRITTO: W≥2 worker, ≥3×W richieste concorrenti su `gate_stateful.php`, OGNI body `cmp` vs expected (mai sola uguaglianza mutua), grep panic su server.log + exit code; PASS solo con comando+hash.
- **A-SK7** (subito): regola di gate — ogni assert di parità run-vs-run DEVE convivere nello stesso test con un assert a contenuto assoluto o oracolo; sanare `gate_apertura2` aggiungendo `assert_eq!(b1, b"hello axum\n")`.
- **A-SK8** (WP-78): run-gate.sh verifica identità: PID in ascolto sulla porta == `$SRV` (lsof), e prova che la build ha aggiornato `$BIN` (mtime/hash pre-post build); expected rigenerati una tantum dall'oracolo CLI, comando a commit.
- **A-SK9**: ogni verdict file (incluso capture-order) cita il git rev dell'albero.
- **A-SK10** (housekeeping): rimuovere i `._*.php` dalle fixtures.

**Kill-switch nuovi**

- **KS-SK-79.1**: PID sulla porta ≠ processo lanciato dal gate, o `$BIN` non aggiornato dalla build ⇒ run del gate NULLA.
- **KS-SK-79.2**: qualunque test/gate nuovo con sola auto-uguaglianza (nessun assert assoluto/oracolo) ⇒ gate invalido, PASS non dichiarabile.
- **KS-SK-79.3**: census avviato senza verdict KH78-1 conforme ad A-SK6 ⇒ misure NULLE.

### 4. Hejlsberg — compilazione incrementale/feature-gating/igiene di build — CONCORDO CON EMENDAMENTI

S-78.0 ha onorato la sostanza della mia sedia (A-AH2/A-AH3 implementati come gate che possono fallire, A-AH4 nel protocollo, KS-AH-78-1/2/3 attivi), ma la purga documentale dichiarata al punto 5 è INCOMPLETA e restano tre buchi di igiene di build da chiudere prima del census.

**Constatazioni**

1. **Scoping `-D warnings` a php-server: scelta GIUSTA, non un buco.** `cargo rustc -p php-server -- -D warnings` (gate-feature-matrix.sh:33, ci.yml:38-40) è il gate minimo corretto: estendere la fatalità a php-runtime/php-types avrebbe imposto una sessione di warning-fix sulla superficie CLI pre-misura (vietato: cambierebbe la superficie prima del census) o rotto il gate su debito fuori scope. Il fingerprint cargo include i flag extra, quindi niente falsi PASS da cache.
2. **Buco reale: i TEST non passano sotto `-D warnings`.** `cargo rustc` compila solo il target bin; i 4 gate-test di worker_pool.rs (righe 308-412) vengono compilati SOLO da `cargo test` (ci.yml:45), che non è fatale sui warning. Codice morto nei test drifterà invisibile — la stessa classe di guasto del `reg` mai visto.
3. **nm-assert: robusto nella direzione che conta, fail-closed nell'altra.** Nel default axum non è nemmeno una dep attiva (Cargo.toml:14 optional), quindi "0 simboli" è quasi tautologico ma resta tripwire valido contro una futura dep non-optional. Il falso negativo da LTO/strip è coperto dal controllo positivo union (gate-feature-matrix.sh:61-69): se un futuro `strip=true` azzera i simboli, l'union fallisce con "detector broken" — fallisce chiuso. Residuo: il grep conta solo `axum`, non `tokio|hyper|tower` (stessa classe di leak, non rilevata).
4. **Fragilità: BIN hard-coded** (gate-feature-matrix.sh:23, `$HOME/Claude/php-rust-output/release/php-server`): se CARGO_TARGET_DIR non è configurato nell'ambiente di run, cargo builda altrove e lo shasum firma un binario STALE — hash a verbale di un artefatto sbagliato, esattamente ciò che KS-AH-78-1 vieta.
5. **static_assertions → hash phpr: gestione dichiarata SUFFICIENTE con un pin mancante.** La dep è compile-time-only (zero codegen runtime); "gate corpus per NOME al primo rebuild + stash additivo" (NEXT_SESSION:11-14) è la ricetta consolidata e basta, perché non tocca ref/arg/reflection (niente obbligo ORM/hk). Manca però il pin che rende il rebuild AUDITABILE: il diff di Cargo.lock deve contenere SOLO l'entry static_assertions.
6. **A-AH4 correttamente nel protocollo**: design78.md:35-44 — contatori separati compile/run/shutdown, KB-78-1 (>50% ⇒ VOID) e KS-AH-78-3 (>30% ⇒ NON GIUDICABILE) entrambi presenti, cache Module esplicitamente post-censimento. Conforme al mio emendamento.
7. **Purga doc N=1 INCOMPLETA (contraddice il punto 5 del session log):** worker_pool.rs:66 "each managing 1 **persistent Vm** + RetainSet" e :77 "Creates **persistent Vm**" — è il lessico dell'architettura RESPINTA in WP-77.2. Inoltre :120, :172, :178 dicono ancora che la RetainSet persiste "**object table**, class cache" — l'esatto claim falsificato da A-DS2 (la RetainSet parcheggia SOLO moduli unit; object store svuotato dal mass-teardown, firma used_n=0). Lezione S-78.0 n.2: un commento falso è peggio di nessun commento — e qui contraddice la matrice corretta 20 righe sotto (:187-204). main.rs invece è pulito: la terna (24-35) descrive la matrice reale.
8. Igiene residua dichiarata e deferita (A-AH5, WP-79): WorkerPoolContext TODO vuoto (worker_pool.rs:34-45) — accetto il defer, ma il TODO "registry: Arc<…>" a :35 è ora FUORVIANTE (la registry è per-worker by design, :137).

**Emendamenti**

- **A-AH6** (pre-census): warnings fatali anche sul target test — `RUSTFLAGS` no; usare `cargo rustc -p php-server --features axum-server --profile test -- -D warnings` o equivalente `--tests`, in gate + CI.
- **A-AH7** (pre-census): gate-feature-matrix.sh deriva BIN da `cargo metadata` (target_directory) o asserisce mtime del binario > inizio build; estendere il grep nm a `axum|tokio|hyper|tower`.
- **A-AH8** (pre-census, 10 minuti): purga dei 5 residui doc di worker_pool.rs (:66, :77, :120, :172, :178) — è completamento del punto 5 già deliberato, non lavoro nuovo.
- **A-AH9** (al primo rebuild phpr): verbalizzare `git diff Cargo.lock` = solo static_assertions, poi corpus per NOME.

**Kill-switch nuovi**

- **KS-AH-78-4**: primo rebuild phpr con delta Cargo.lock diverso dalla sola static_assertions ⇒ HALT, audit del delta prima di ogni gate.
- **KS-AH-78-5**: hash a verbale da un BIN il cui mtime precede la build del gate ⇒ misura NULLA (binario stale).

### 5. Bak — alloc-rate/code-cache/path caldi — CONCORDO CON EMENDAMENTI

S-78.0 è una sanatoria reale (9/9 punti verificabili, e il primo gate a contenuto assoluto ha morso subito: body raddoppiato, `head -1` cieco — esattamente la classe di bug che l'auto-uguaglianza benedice). Il programma misura design78 recepisce il mio A-BB2 sulla carta, ma ha DUE buchi operativi che, se non chiusi, produrranno un census con verdetto A-BB1 VOID per costruzione.

**Constatazioni**

1. **Punti caldi sanati, verificati sul codice**: registry once/worker (worker_pool.rs:137, era churn per-richiesta); body/method morti rimossi — `WorkerHandlerMeta` è solo path+source (worker_pool.rs:51-54) e il handler non tocca più il body (main.rs:97-106), sparito `to_bytes(usize::MAX)`. A-BB3 implementato (`--workers N`, main.rs:228-241; doc footprint ×N a main.rs:150-152). A-BB5 ✓, A-BB3 ✓, A-BB4 registrato a debito WP-79+ ✓, KB-78-1/3/4 recepiti in design78 ✓.
2. **Alloc per-richiesta residue nel path caldo** (in ordine di peso atteso): (a) `lower_source`+`compile_program`+`vm_new` — Program+Module+Vm interi per richiesta (worker_pool.rs:218,234,248): dominante, è il canale che per predizione supererà il 50% e farà scattare KB-78-1 su qualunque report non per-fase; (b) `fs::read` source Vec (main.rs:86); (c) la path allocata TRE volte: `uri.path().to_string()` (main.rs:77), `request_path.clone()` (main.rs:104 — clone evitabile, è l'ultimo uso), `meta.path.clone().into_bytes()` (worker_pool.rs:210); (d) **`file_s` allocata incondizionatamente a ogni richiesta ma usata SOLO nel ramo Fatal** (worker_pool.rs:217) — alloc morta nel path felice, da spostare nel branch; (e) oneshot channel (main.rs:95) + nodo mpsc per send. La response è `mem::take` (worker_pool.rs:288): zero-copy, corretto.
3. **Buco 1 — i contatori A-BB2 non esistono nel codice**: design78 §attribuzione prescrive contatori (a)/(b)/(c) con predizione-misurata, ma in worker_pool.rs non c'è UNA riga di strumentazione census. Implementarli È lavoro legittimo della fase misura, MA l'"Ordine della fase misura" (design78) non ha uno step "costruire la strumentazione": salta da Tier-0 (step 2) a "Census per-fase" (step 3) come se i contatori piovessero. Senza quello step, o si misura un numero unico (⇒ KB-78-1 VOID) o si improvvisa la strumentazione fuori protocollo.
4. **Buco 2 — KB-78-2 è oggi un kill-switch che non può mordere**: depth coda >1 ⇒ VOID, ma con mpsc unbounded e zero contatori la depth non è osservata da nessuno. "Garantita dal client chiuso-sequenziale" non basta: un KS senza osservabile è un gate che benedice (lezione WP-72: contatore tutto-zero = controllo positivo fallito). Il canale unbounded in sé resta accettabile per `--workers 1` sequenziale (debito A-BB4 correttamente registrato) — purché la depth diventi osservabile.

**Emendamenti**

- **A-BB7 (vincolante, pre-census)**: inserire nell'ordine di misura lo step esplicito "strumentazione census": contatore alloc per-fase (wrapper sul global allocator o hook mimalloc) dietro feature dedicata; predizioni per canale dichiarate PRIMA della run (WP-45..48); coppia strumentata/non-strumentata build-adiacente (lezione WP-33: un branch nel path caldo non è gratis). Le cifre di ATTRIBUZIONE vengono dalla build strumentata; le cifre di FOOTPRINT/peak SOLO dalla non-strumentata.
- **A-BB8 (vincolante, rende KB-78-2 esecutivo)**: worker_loop logga la depth (`rx.len()` a ogni recv) sotto la feature census, con controllo POSITIVO: un burst di 3 richieste concorrenti deve far osservare depth>1 almeno una volta, altrimenti l'osservatore è rotto e KB-78-2 è vacuo.
- **A-BB9 (igiene, stesso commit della strumentazione)**: `file_s` nel ramo Fatal; eliminare il clone main.rs:104 (move); path passata una volta come bytes. Micro-alloc, ma sporcherebbero il canale (c) del census firmandosi come "overhead Axum".

**Kill-switch nuovi**: **KB-78-5** — cifra di footprint/peak citata da una build strumentata ⇒ respinta d'ufficio; **KB-78-6** — census eseguito senza controllo positivo A-BB8 verde ⇒ KB-78-2 dichiarato inosservato e run VOID.

### 6. Pedersen — disciplina di confine per-richiesta, lifecycle — CONCORDO CON EMENDAMENTI

S-78.0 onora la mia sezione (A-PP2/3/4/5 chiusi con gate che mordono; KS-PP-1/2/3 rispettati, G-APERTURA-2 rilanciato dopo il tocco al path di capture). Il bug del body raddoppiato conferma la mia tesi (l'auto-uguaglianza non è oracolo). Il freeze del contratto HTTP pre-misura è però CONDIZIONATO ai punti 2 e 6 sotto.

**Constatazioni**

1. **Ordine Stogov: rispettato in tutti i path con Vm.** worker_pool.rs: start :252 → run :256 → flush_diags :267 → render fatal :270-275 → shutdown :279 → CAPTURE :288 → end :292. Exit (:261) confluisce nel path comune (shutdown gira dopo exit — conforme Zend). I fatal di lower/compile (:218-231, :234-240) escono PRIMA di creare il Vm: nessun confine da resettare, corretto per costruzione.
2. **body = `mem::take(vm.rendered)` (:288) prima di `request_end()` (:292): conforme alla permanent rule.** Contenimento stdout⊆rendered verificato ai siti di scrittura runtime: ogni extend di stdout è in tandem con rendered (vm/mod.rs:5359-60, :5458-59, host.rs:6190-91, ini.rs:656-57), e rendered ha SOLO scritture extra (diagnostica/fatal: exceptions.rs:44,132,138; vm/mod.rs:824,5227). Oggi non esiste un caso in cui stdout contenga byte assenti da rendered — ma l'invariante NON è gateato, e il controllo positivo A-PP3 usa `!(rendered.is_empty() && stdout.is_empty())` (worker_pool.rs:377) che passa anche a stdout vuoto: non prova il contenimento.
3. **Marker `grep-gate-allow`: scappatoia reale.** gate-capture-order.sh:20 salta QUALUNQUE riga contenente la stringa, ovunque: `let out = vm.rendered.clone(); // grep-gate-allow: post-reset-emptiness-check` in codice di produzione passerebbe il gate. Il marker va vincolato per forma e censito per conteggio.
4. **Buco same-line nel detector.** `:22 /request_end\(\)/ { seen=1; next }`: una violazione sulla STESSA riga (`vm.request_end(); let o = vm.rendered.clone();`) è invisibile (`next` salta il resto della riga). Inoltre il gate lessicale è cieco all'indirezione (una futura `vm.take_output()` chiamata post-end non matcha `.rendered|.stdout`).
5. **Esclusione commenti: direzione sicura.** Righe interamente commentate non armano né violano (:19); un trailing comment che cita `request_end()` su riga di codice ARMA falsamente `seen` → falso POSITIVO (fail rumoroso), non nasconde violazioni. Il pericolo sta in (3)+(4), non nei commenti.
6. **Fatal→500: coperto per i fatal PHP, NON per i panic Rust.** Il test worker_pool.rs:393-411 prova che il worker sopravvive al fatal (richiesta successiva pulita, stessa RetainSet). Ma un panic in `vm.run()` uccide il thread (`std::thread::spawn` :103-105, nessun catch_unwind/respawn): 1/N delle richieste round-robin muore PER SEMPRE, in silenzio. Il confine HTTP non è "stabile per il freeze" finché il destino del worker su panic non è dichiarato. Minore: l'assert fatal usa `contains("Fatal error:")` (:401) — più debole del claim byte-parity (lezione S-78.0 applicata a metà); `exit($code)` → 200 con code scartato (:261) è conforme FPM ma non dichiarato nel contratto.

**Emendamenti**

- **A-PP6** (pre-freeze): vincolare il marker allow — il detect lo accetta SOLO su righe `assert!.*is_empty`; il gate censisce le occorrenze (attese: esattamente 1, worker_pool.rs:384) e una nuova occorrenza è FAIL.
- **A-PP7**: chiudere il buco same-line (normalizzare `;` in newline prima dell'awk, o non fare `next` sulla riga di request_end).
- **A-PP8** (pre-freeze del contratto body=rendered): gateare il contenimento — grep-gate in php-runtime "ogni scrittura a stdout ha tandem rendered" con self-test positivo, oppure test cargo multi-canale con assert `rendered ⊇ stdout` post-shutdown.
- **A-PP9** (prima del census): dichiarare e gateare il destino del worker su panic (catch_unwind→500, o respawn); un worker morto a metà run di misura falsifica il census in silenzio.
- **A-PP10**: assert fatal a contenuto assoluto (full-body vs oracolo CLI) + fixture `exit(code)` a contratto.

**Kill-switch nuovi**

- **KS-PP-4**: occorrenze del marker allow difformi dal censimento → gate FAIL.
- **KS-PP-5**: scrittura stdout senza tandem rendered in php-runtime → reject (decade il contratto body=rendered).
- **KS-PP-6**: worker thread morto (panic) durante un run di misura → run VOID, HALT del freeze.

### 7. Leijen — allocatore mimalloc/footprint fisico — CONCORDO CON EMENDAMENTI

S-78.0 onora A-DL2/A-DL3/A-DL5; A-DL4 è correttamente collocato in fase misura ma sotto-specificato; due difetti nuovi nel percorso di uscita e nel termine per-worker vanno chiusi prima del census.

**Constatazioni**

1. **A-DL2 ONORATO**: flag `--workers N` reale (main.rs:228-241; worker_pool.rs:90-95, 0=num_cpus documentato). A-DL5 ONORATO: header main.rs:1-11 riscritto (N=1/Vm::reset purgati); `reg` sanato PER DAVVERO — registry costruita una volta per worker (worker_pool.rs:137) e passata `&Registry` a execute_with_retain (:150, :205-208). KL-78-1/2/3 trascritti integri in design78.md:52-57 e nei kill-switch di NEXT_SESSION:60-63.
2. **A-DL3 SOSTANZIALMENTE ONORATO** (design78.md:27-33): peak=/usr/bin/time -l con uscita pulita, residente=vmmap Physical footprint con MIMALLOC_PURGE_DELAY=0, MIMALLOC_SHOW_STATS a exit, ps RSS VIETATO, coppie build-adiacenti stessa-sera con mimalloc pinnata nel Cargo.lock (:24-26), R≥3/±2%/warm-up escluso. Transiente/residente distinti come chiedevo (:29-31 e step 4 :73). Gap residuo: il protocollo NON dichiara QUANDO si scattano gli snapshot vmmap (KL-78-1 richiede Δ per-richiesta tra batch: servono punti di campionamento dichiarati — post-warm-up, post-batch k — altrimenti il Δ non è definito).
3. **Shutdown: pulito per tokio, NON per i worker.** `with_graceful_shutdown` (main.rs:171-184) fa tornare `serve`, il drop di Router/AppState chiude i sender mpsc e i worker_loop escono su `blocking_recv`→None — ma i JoinHandle sono BUTTATI (worker_pool.rs:103-105): nessun join. Il processo può fare exit mentre i worker stanno ancora droppando i RetainSet ⇒ MIMALLOC_SHOW_STATS stampa con heap di thread abbandonati in stato non-deterministico run-per-run (spread gonfiato, stats "in use" residue finte). Il residente a vmmap PRE-exit invece NON è inquinato: a steady-state i RetainSet vivi sono esattamente il segnale da misurare, non rumore.
4. **Termine per-worker k include N copie della Registry** (worker_pool.rs:137: una per worker; il TODO Arc in :35 resta deferito). Legittimo, ma la linearità A-DL2 `base + W·k` va verbalizzata con k = registry + RetainSet-steady + stack, altrimenti un domani lo sharing via Arc verrebbe letto come "de-leak del RetainSet".
5. **A-DL4 NON implementato — collocazione CORRETTA, disciplina MANCANTE.** Nessun contatore entries nel codice (giusto: in sanatoria sarebbe stato un altro "IMPLEMENTED" senza gate che morde). Ma design78.md:58-60 non dice: (a) che il contatore è un probe che cambia il binario ⇒ va feature-gated e i verdetti footprint si emettono dal gemello NON strumentato della coppia build-adiacente (lezione WP-64/WP-33: il probe non si misura addosso); (b) quale fixture-leak fa da controllo positivo (serve una fixture con include che minta 1 modulo/richiesta nella FrozenVec — l'hello-world può dare entries=0 e un contatore tutto-zero è un controllo FALLITO, WP-72); (c) la predizione del canale PRIMA della run (WP-45..48).
6. KH78-2/KB-78-2 ("depth coda >1 in un campione ⇒ VOID") è inosservabile: nessun contatore di profondità sulla mpsc unbounded. Con client chiuso-sequenziale depth≤1 vale per costruzione — va verbalizzato così, o il kill-switch è decorativo.

**Emendamenti**

- **A-DL6 (BINDING, pre-census)**: WorkerPool conserva i JoinHandle; allo shutdown, drop dei sender + join di TUTTI i worker prima del return da start_server. Solo così MIMALLOC_SHOW_STATS e time -l firmano uno stato deterministico.
- **A-DL7 (BINDING, prima dello step 5)**: contatore A-DL4 feature-gated (`retain-probe`); verdetti footprint SOLO dal gemello non strumentato; fixture positiva dichiarata (include che minta 1 entry/richiesta, amplificazione intera-esatta attesa); predizione a verbale PRIMA della run.
- **A-DL8**: design78 aggiunge i punti di snapshot vmmap (post-warm-up, dopo ogni batch R) e dichiara k = registry+RetainSet+stack per la linearità in W; depth≤1 dichiarata per costruzione dal client chiuso.

**Kill-switch nuovi**

- **KL-78-4**: stats mimalloc raccolte senza join dei worker ⇒ stats ADVISORY, nessun verdetto A-DL1 su di esse.
- **KL-78-5**: cifra footprint emessa da build con contatore A-DL4 attivo ⇒ cifra NULLA (vale solo il gemello non strumentato).

### 8. Stogov — semantica Zend engine — CONCORDO CON EMENDAMENTI

La sanatoria onora la sostanza del mio mandato: A-DS2 fedele nel file giusto, A-DS3 gateata e mordente (KS-DS-78-1 in cargo test con assert a contenuto assoluto — proprio quello che ha stanato il body doppio), A-DS4 eseguito, A-DS5 recepito in design78. KS-DS-78-3 rispettato (NEXT_SESSION §NON riproporre). Ma restano residui documentali della stessa classe della mina A-DS1, il contratto fatal ha ancora due formati e un assert debole, KS-DS-78-2 non è osservabile, e il canale RetainSet non è MAI esercitato da alcun gate.

**Constatazioni**

1. **A-DS2 FEDELE** (worker_pool.rs:187-204). Ri-verificato su vm/mod.rs: `request_end()` (3427-3534) azzera superglobals (3437), constants (3508), OB stack (3431), handlers, resource-id, enum_cache (3502); NON tocca statics/closure_statics/static_props → isolamento per morte del Vm, corretto. `RetainSet(FrozenVec<Rc<Module>>)` (vm/mod.rs:427) = solo bytecode: l'analogia opcache regge, static assert presente (435).
2. **Doc residua contraddittoria — la mina A-DS1 sopravvive in due punti**: worker_pool.rs:119 («server-lifetime arena for object table, class cache») e :180 («object table, class cache persist») contraddicono la matrice 15 righe sotto (:199-203: object store svuotato dal mass-teardown). Peggio: vm/mod.rs:3417-3420, doc di `request_end()`, dice ancora «preserving persistent data (**statics**, module code, class table)» e «prepare the Vm for the next request» — statics dichiarati "persistent" nel crate runtime, Vm che "si prepara" a una richiesta successiva che non vivrà mai. La sanatoria ha bonificato worker_pool.rs, non il runtime.
3. **Fatal→500: DUE formati di corpo restano**. Compile fatal (worker_pool.rs:221-225) appende `Stack trace:\n#0 {main}\n`; runtime fatal (:273) no; parse error (:228) `PHP Parse error: {e}` senza `in file on line`. Il test (:400-404) usa `contains("Fatal error:")` — stesso vizio del `head -1` refutato dalla Lezione 1 di S-78.0. "Byte-parity" è dichiarata, mai cmp'd contro l'oracolo (che per undefined function stampa `Uncaught Error: … Stack trace:\n#0 {main}\n  thrown in …`).
4. **html_errors**: il corpo CLI-style (html_errors=0, display sempre attivo) non è né FPM-dev (html_errors=On di default ⇒ `<br />`/`<b>`) né FPM-prod (display_errors=Off ⇒ 500 a corpo vuoto + log). Contratto transitorio legittimo finché il seeding web_request/INI è WP-79+, ma è una DIVERGENZA da dichiarare, non un dettaglio.
5. **Buchi fixture** (gate_stateful.php): (a) manca static in METODO di classe (storage op_array; semantica 8.1+: condiviso col metodo ereditato); (b) manca static property EREDITATA (slot condiviso col parent); (c) **nessuna fixture con include/require**: il parking nella RetainSet — l'UNICO meccanismo di persistenza dell'architettura — non è esercitato da alcun gate; FrozenVec è append-only, quindi la crescita per-include è esattamente la superficie di KL-78-1 e oggi nessun gate la vede.
6. **KS-DS-78-2 senza contatore**: va preteso come accessor feature-gated (`used_n()` dello store + `RetainSet::len()`), asserito (i) nel test manual-lifecycle A-PP3 dopo `request_end()` (lettura sanzionata come l'emptiness-check :383-387), (ii) per-worker nel gate concorrente KH78-1, (iii) riusato tale-quale per la firma A-DL4. Un canale, tre gate — PRIMA del census.

**Emendamenti**

- **A-DS6** (pre-misura): purge doc residua — worker_pool.rs:119/:180 e vm/mod.rs:3417-3420 (request_end NON "preserva statics": sono fuori dal suo scope perché muoiono col Vm).
- **A-DS7** (pre-freeze contratto HTTP): parity fatal FULL-BODY — fixture fatal compile E runtime in run-gate.sh con `cmp` vs oracolo 8.5.7; unificare i due formati; test cargo ad assert assoluto, mai `contains`.
- **A-DS8**: fixture con include/require nel gate stateful (≥3 richieste, parity + `RetainSet::len()` stabile post-warm-up).
- **A-DS9**: estendere gate_stateful con static-in-metodo (+ variante ereditata, semantica 8.1) e static prop ereditata.
- **A-DS10**: registrare la divergenza error-page (CLI-style vs FPM html_errors/display_errors) in PHPR_DIVERGENCES_FROM_PHP.md.

**Kill-switch nuovi**

- **KS-DS-78-4**: `RetainSet::len()` cresce oltre il set di unit distinte dopo warm-up ⇒ FATAL, riapertura C-leak.
- **KS-DS-78-5**: qualunque claim di parity su corpi d'errore senza cmp full-body vs oracolo ⇒ respinto d'ufficio (estensione vincolante della Lezione 1 S-78.0).

### 9. Gregg — metodologia di misura/attribuzione — CONCORDO CON EMENDAMENTI VINCOLANTI

S-78.0 SANATA (A-BG2/A-BG4-rietichettatura/KG-78.D onorati, cifre tracciate); il protocollo di misura design78 è invece NON ESEGUIBILE così com'è — tre strumenti richiesti dai kill-switch non esistono nel codice e due definizioni lasciano margini che produrranno cifre contestabili.

**Constatazioni**

1. **A-BG2 onorato** (main.rs:5-11): doc N=1/Vm::reset purgata, "Boot variance" rimossa. A-BG4 onorato come TESTO (design78.md:48-50). KG-78.D onorato in S-78.0: ogni hash del session file risale a comando tracciato (run-gate.sh:43, feature-matrix.log). Unica eccezione: "phpr 02aa2ad8 INVARIATO" (WP_SESSION_78:10) è asserito senza shasum eseguito in sessione — invarianza dedotta, non misurata (tollerabile, già disinnescata in NEXT_SESSION:11-14). [nota del segretario: lo shasum È stato eseguito in chiusura, output 02aa2ad8 a verbale]
2. **La mia richiesta "attribuzione impossibile senza contatori" (rilievo 3) è RINVIATA, non onorata.** worker_pool.rs:130-153: worker_loop non ha NESSUN contatore — né richieste servite per-worker, né taglia RetainSet per richiesta. A-BG5 chiedeva esplicitamente il "contatore per-worker richieste/taglia RetainSet feature-gated": design78.md:58-60 lo prescrive ma il codice non lo contiene. Senza di esso la "firma intera-esatta" (A-DL4) e KG-78.C sono INESEGUIBILI: il probe RSS(N)vs(2N) resterebbe address-level, non intero-esatto.
3. **KH78-2/KB-78-2 è un detector che non può sparare** (lezione WP-72): il canale è `mpsc::unbounded_channel` (worker_pool.rs:99) e nessun punto del codice campiona la profondità. "depth >1 in un campione ⇒ VOID" presuppone un campionatore che non esiste. Va sostituito dal MECCANISMO: client chiuso-sequenziale per costruzione (richiesta N+1 solo dopo body N) + assert sulla len del receiver.
4. **"Warm-up escluso: prime 10 richieste" (design78.md:21) è doppiamente difettoso.** (a) Il 10 è arbitrario: la finestra di regime si dichiara sugli EVENTI del workload (WP-69), qui l'evento strutturale è il plateau della RetainSet — che senza il contatore del punto 2 non è osservabile. (b) Per le metriche di PICCO processo-lifetime (`/usr/bin/time -l`, design78.md:29) il warm-up NON è escludibile per definizione: il picco include le prime 10 richieste. Il protocollo non dice come conciliare le due righe → cifra contestabile garantita.
5. **Denominatore A-BB1: ben definito come comando, non come ISOLAMENTO.** Il ramo `--cli-server` del binario union delega a `php_cli::server::serve` (main.rs:294): architettura diversa (niente pool, niente canale, forse registry per-richiesta, lifecycle proprio). Il delta misura quindi la SOMMA {tokio+canale+pool+registry-once+RetainSet-retention} — non "l'overhead del pool". Inoltre lo shutdown pulito SIGTERM (S-78.0.7) è implementato SOLO nel ramo axum (main.rs:171-188): se php_cli::server non esce pulito, `time -l`/MIMALLOC_SHOW_STATS sul braccio CLI = zero stats → A/B asimmetrico.
6. **Census per-fase senza strumento nominato** (design78.md:37-39): mimalloc stats sono process-global; i contatori per-fase (lower+compile / run / shutdown) non esistono in execute_with_retain (worker_pool.rs:205-302) e design78 non nomina né lo strumento né il suo costo proprio (WP-64: il probe non si misura addosso). "Tier-0 Axum senza pool" (design78.md:70) non ha configurazione eseguibile nel binario.

**Emendamenti**

- **A-BG7** (bloccante, pre-census): contatori feature-gated in worker_loop — richieste servite per-worker + entries RetainSet per richiesta — con controllo POSITIVO (fixture-leak che il contatore DEVE vedere) PRIMA di qualunque verdetto A-BG5/KG-78.C.
- **A-BG8**: riscrivere il warm-up: escludibile SOLO per contatori per-richiesta; per il picco time -l dichiararlo INCLUSO, e stimare il regime col metodo-delta RSS(N)vs(2N). Il "10" va sostituito da criterio strutturale o dichiarato scelta arbitraria a verbale.
- **A-BG9**: KH78-2 riformulato a meccanismo (client chiuso per costruzione + assert len canale); fissare N del census e definire lo spread ((max−min)/mediana, per metrica) PRIMA della run.
- **A-BG10**: prima dell'A/B A-BB1, verificare e verbalizzare il lifecycle del ramo cli-server (registry, shutdown pulito); dichiarare a verbale COSA il delta somma. Tier-0 esiste come config tracciata o esce dal protocollo.
- **A-BG11**: nominare lo strumento per-fase (contatori WP-45-style) e misurarne il costo su coppia strumentata/non-strumentata.

**Kill-switch**

- **KG-79.A**: census/probe eseguiti senza contatori A-BG7 compilati e senza controllo positivo visto ⇒ verdetti A-BB1/A-BG5 VOID.
- **KG-79.B**: A/B in cui un braccio non produce statistiche a uscita pulita ⇒ coppia VOID.
- **KG-79.C**: "warm-up escluso" dichiarato su metrica di picco processo-lifetime ⇒ cifra respinta d'ufficio.
- **KG-79.D**: "Tier-0" citato senza configurazione eseguibile tracciata ⇒ baseline nulla.

---

**Chiusura concilio**: 2026-07-31. Verbali VINCOLANTI: la prossima sessione apre con S-78.1 (hardening pre-census, ordine in sintesi) e SOLO DOPO esegue la misura.
