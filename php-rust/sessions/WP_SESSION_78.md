# WP_SESSION_78.md — S-78.0 SANATORIA (fixing assoluto post-Haiku)

**Data**: 2026-07-31 (mattina)
**Scope**: eseguire l'ordine vincolante S-78.0 del Concilio WP-78 (9 punti,
tutti pre-misura) + rettifiche documentali. Sessione dichiarata dall'utente
"di fixing assoluto": le sessioni WP-77.x avevano girato con modello
inferiore; ogni claim andava ri-verificato sul codice prima di agire.
**Commit**: 31201ba → 2652b97 → c6b960b → 024a86b → 84d968c → c3a9b40 →
af04e0b → 022c8d2 (tutti su main, pushati).
**Binari**: phpr **02aa2ad8 INVARIATO** (mai ricompilato in sessione);
php-server (union, deployed) **d98b531e6b5ee306** — hash stampato da
run-gate.sh (A-SK5: la baseline Axum è php-server, mai phpr).

---

## Verifica indipendente dei claim del Concilio (pre-azione)

Tutti CONFERMATI sul codice: `RetainSet(elsa::FrozenVec<Rc<Module>>)` →
`!Send+!Sync` (vm/mod.rs:426); `request_end()` (3418-3526) azzera
stdout/rendered/superglobals/constants ma NON tocca statics/closure_statics/
static_props (isolamento per morte del Vm); `registry()` NON è cached
(ricostruita per-richiesta nel vecchio codice); gate-axum vuoto; `cargo test`
default non compilava una riga di worker_pool.rs.

## S-78.0 eseguita (9/9 punti)

| # | Emendamenti | Esito |
|---|---|---|
| 1 | A-SK2 (+fixture A-DS3) | `wp78-harness/gate-axum/run-gate.sh` eseguibile: build con feature, shasum php-server, hello×2 + stateful×3 POST, cmp full-body vs oracolo. **PASS bin=d98b531e6b5ee306**. KS-SK-78.1 sollevato |
| 2 | A-MS2/A-MS3 | SAFETY riscritti (`!Send+!Sync`, thread-affine per costruzione); `assert_not_impl_any!(RetainSet: Send, Sync)` in php-runtime + controllo positivo `assert_impl_all!(WorkerTask: Send)` (static_assertions 1.1) |
| 3 | A-DS2 | Matrice FPM col MECCANISMO reale per riga (request_end vs morte Vm vs mass-teardown WP-72 vs arena RetainSet); corretto anche "persists: object table" (la RetainSet parcheggia SOLO moduli unit) |
| 4 | A-TH1/A-PP2/A-MS4 | Legacy `execute()` ELIMINATA (violava la regola permanente nello stesso file; deprecation → funzione inesistente); struct RequestHandler rimossa; `gate-capture-order.sh` (KS-PP-2) con self-test positivo. PASS |
| 5 | A-TH3/A-BG2/A-DL5 | Doc N=1/Vm::reset() purgata da main.rs+worker_pool.rs; riga "Boot variance" rimossa; `reg` sanato PER DAVVERO: registry costruita una volta per worker e passata `&Registry` (prima: ricostruita a ogni richiesta = churn puro nel census) |
| 6 | A-AH2/A-AH3/A-SK4 | `gate-feature-matrix.sh`: TERNA reale (default faacd79d / axum-only 5a293aa3 / union b5a19fc5), `-D warnings` scopato a php-server via `cargo rustc`, nm-assert 0 simboli axum nel default con controllo positivo union (267). CI: terna fatale + `cargo test -p php-server --features axum-server`. KS-AH-78-1/2 verdi |
| 7 | A-BB3/A-DL2/A-BB5/A-DL3 | Flag `--workers N`; body/method morti RIMOSSI dal Meta e dal handler (con loro sparisce anche `to_bytes(usize::MAX)`); shutdown pulito SIGTERM/SIGINT (`with_graceful_shutdown`) per time -l / MIMALLOC_SHOW_STATS; campo `context` morto rimosso |
| 8 | A-PP3/A-SK3 | 4 test cargo feature-gated in worker_pool.rs: byte-parity 2 req stessa RetainSet; static counter ×3 (KS-DS-78-1); ob_start capture + CONTROLLO POSITIVO (buffer vuoti post-request_end, marker grep-gate-allow esplicito); fatal→500 compile+runtime col worker che sopravvive. 4/4 PASS |
| 9 | A-PP4/A-TH5/A-DS4 | OGNI fatal → HTTP 500 con corpo byte-parity CLI (`\nFatal error: … Stack trace…`); prima: runtime fatal=200, compile fatal=500 con corpo non-parity |

**Rettifiche documentali**: A-SK5 (blocco rettifica nel session file .4:
02aa2ad8 vacuo per lavoro Axum); verbale .3 rettificato (M-MS1 emendato,
"≤+1,5% misurato" → PREDIZIONE, PASS vacuo annotato); NEXT_SESSION Permanent
Binding Rules 2-3 riscritte (erano i claim falsificati); divergenza harness
risolta: **repo canonico** per wp77+/wp78-harness, dir esterna marcata
superseded (README_SUPERSEDED.md).
**design78.md**: protocollo misura completo scritto PRIMA di ogni run
(A-BG3/A-DL3/A-BG4/A-BG5 + tutti i kill-switch di misura).

## 🐛 BUG REALE trovato dai nuovi gate (non previsto dal Concilio)

**Il body HTTP era RADDOPPIATO**: `execute_with_retain` concatenava
`vm.rendered + vm.stdout`, ma `rendered` è il flusso CLI-fedele che CONTIENE
già stdout (con diagnostica inline). Ogni risposta usciva doppia. Invisibile
a TUTTI i "PASS" precedenti perché: (a) byte-parity di auto-uguaglianza
(doppio==doppio); (b) il check `head -1` del vecchio gate leggeva solo la
prima riga. Trovato dal primo assert a CONTENUTO ASSOLUTO (test cargo
`body == b"1"` → riceveva `"11"`). Fix: body = solo `vm.rendered`
(`mem::take`); run-gate.sh rafforzato a cmp full-body contro corpo oracolo.

## Verifica finale (tutta sull'albero 022c8d2)

- `cargo test --release --workspace`: **1651 passed / 0 failed / 1 ignored**
  (=1652, baseline rispettata; exit 0 verificato senza pipe)
- `cargo test -p php-server --features axum-server`: 4/4 gate test PASS
- `run-gate.sh`: **G-APERTURA-2 PASS bin=d98b531e6b5ee306**
- `gate-capture-order.sh`: PASS (self-test positivo attivo)
- `gate-feature-matrix.sh`: PASS (terna loggata in gate-axum/out/)

## ⭐ Lezioni

1. ⭐⭐ **L'auto-uguaglianza non è un oracolo**: un gate che confronta solo
   run-vs-run (byte-parity) benedice qualunque bug deterministico (body
   doppio). Il controllo positivo vero è il confronto col CONTENUTO ATTESO
   dall'oracolo, full-body — `head -1` è un controllo bucato.
2. ⭐⭐ **Un SAFETY comment falso è peggio di nessun commento**: A-MS1
   affermava l'esatto contrario del vero e autorizzava il refactor letale;
   l'invariante ora vive nel type system (static assert) dove non può mentire.
3. ⭐⭐ **"IMPLEMENTED" senza comando che possa fallire = chiusura falsa**:
   tre emendamenti "implementati" come commenti (A-SK1/A-AH1/A-DS1) sono
   stati refutati; da S-78.0 ogni PASS cita comando + hash.
4. ⭐ **`cargo test` default può coprire lo 0% del codice sotto revisione**
   (feature-gating): il warning `unused reg` mai visto era la prova. Ora la
   feature entra in CI sia a build (-D warnings) che a test.
5. ⭐ **I contatori di misura si proteggono PRIMA della misura**: registry
   ricostruita per-richiesta e body morto nel canale avrebbero firmato il
   census come "overhead Axum".

## Residui / NON fatto (dichiarato)

- **Fase misura WP-78 NON eseguita** (Tier-0, census per-fase, A-BB1, A-DL1,
  probe A-BG5): protocollo pronto in design78.md.
- **Gate concorrente KH78-1 NON ancora creato** (obbligatorio PRIMA del census).
- phpr non ricompilato: al primo rebuild l'hash cambierà per la dep
  compile-time static_assertions → rifare il gate corpus per NOME a quel punto
  (nessun cambio comportamentale atteso: 1651/0 già verdi sul nuovo sorgente).
- Deferiti WP-79+ (da Concilio): A-TH4 (`request_end(self)→CapturedOutput`),
  A-BB6 cache Module, A-AH5/A-BB4 (igiene residua, backpressure, limiti body,
  superglobali reali da metadata HTTP).
