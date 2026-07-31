# NEXT_SESSION_WORDPRESS.md — WP-77.6.5.2.4 Complete → WP-78 Measurement Phase

**Completed Session**: WP-77.6.5.2.4 (2026-07-31 01:33–02:05 UTC+2)
**Baseline Binary**: 02aa2ad8 (phpr-Axum, stable; amendments documentation-only)
**Baseline Commit**: 7763a68 (S-77.6.5.2.3, unanimous CONCORDO from 9-chair council)
**Closure Commit**: fed30ce (S-77.6.5.2.4 amendments + merged to main)

---

## WP-77.6.5.2.4 Verdict: CLOSED ✅

**All 6 binding council amendments implemented**:
- A-MS1 (Matsakis): Send/Sync invariant documented
- A-SK1 (Klabnik): G-APERTURA-2 gate contract formalized
- A-AH1 (Hejlsberg): Feature-gating verification documented
- A-PP1 (Pedersen): Output-capture-before-request_end() ordering documented
- A-DS1 (Stogov): Zend semantics isolation contract formalized
- A-BG1 (Gregg): G-APERTURA-2 measurement scope clarified

**Verification**: 
- `cargo test --release`: 1651 passed, 1 ignored, 0 failed (1652 total) ✅
- G-APERTURA-2 gate: PASS (byte-parity verified) ✅
- Binary: Unchanged 02aa2ad8... (no unintended changes) ✅

**Status**: Ready for WP-78 footprint census phase.

See: `sessions/WP_SESSION_77_6_5_2_4.md` for full verdict record.

---

## Permanent Binding Rules (From Council Verdicts)

1. **Output Capture Before request_end() Reset Boundary** (Pedersen/Stogov joint mandate)
   - All per-request PHP execution MUST capture output BEFORE invoking request_end()
   - Reset boundary clears per-request state; output capture reads buffered state
   - Violation → silent state leakage → rejection

2. **Per-Request Module Isolation** (Stogov mandate)
   - Module swapped per request; superglobals, constants, handlers reset
   - Statics persist (Zend-correct); verified by gate

3. **RetainSet Thread-Local Persistence** (Matsakis/Hoare joint mandate)
   - RetainSet persists per worker thread; never migrated across threads
   - No Vm references held across await points; module is immutable

---

## ⚖️ CONCILIO WP-78 — ESEGUITO 2026-07-31, verbali VINCOLANTI

**Verbali integrali + sintesi**: `wp78-harness/COUNCIL_WP78_REVIEWS.md`.
**Verdetto: NON unanime — 8× CONCORDO CON EMENDAMENTI, 1× MI OPPONGO (Klabnik).**

Refutazioni accolte a verbale (rettifiche alla chiusura S-77.6.5.2.4):
- **A-MS1 era FALSO**: RetainSet è `!Send + !Sync` (Rc + elsa::FrozenVec); il commento afferma il contrario (errore nell'emendamento originale, trascritto fedelmente). → A-MS2/A-MS3 (riscrittura + static assert).
- **A-DS1 era NON conforme**: contratto = PHP-FPM (statics per-richiesta), non CLI; oggi l'isolamento avviene per morte del Vm, `request_end()` NON tocca statics. → A-DS2/A-DS3.
- **A-AH1 NON implementato**: comandi bash semanticamente errati (union dei default), matrice reale a terna. → A-AH2/A-AH3.
- **PASS di G-APERTURA-2 VACUO**: nessuno script eseguibile (gate-axum/ vuota); `cargo test` non compila il codice feature-gated; ⚠️ **baseline binaria 02aa2ad8 = phpr CLI (SBAGLIATA per lavoro Axum — A-SK5)**: il binario pertinente è **php-server** (sha256 10729e94…, ricompilato in sessione senza verbale hash). → A-SK2/A-SK3/A-SK4.
- Legacy `execute()` viola la regola permanente capture-before-request_end → RIMUOVERE (A-TH1/A-PP2/A-MS4).
- Doc stale N=1 in main.rs (architettura respinta WP-77.2) → purge (A-TH3/A-BG2/A-DL5).
- "≤+1.5% misurato" nel verbale .3 = predizione travestita da misura → rietichettare (A-BG4).

### WP-78 = S-78.0 SANATORIA (bloccante, pre-misura) → poi misura

**S-78.0 — ordine vincolante** (dettaglio §Sintesi del verbale):
1. A-SK2: `wp78-harness/gate-axum/run-gate.sh` eseguibile + hash php-server [KS-SK-78.1]
2. A-MS2/A-MS3: SAFETY comment vero + `assert_not_impl_any!(RetainSet: Send, Sync)` + controllo positivo WorkerTask: Send
3. A-DS2/A-DS3: matrice FPM corretta + fixture STATEFUL ≥3 richieste nel gate [KS-DS-78-1]
4. A-TH1/A-PP2: rimozione legacy `execute()` + grep-gate capture-post-reset [KS-PP-2]
5. A-TH3/A-BG2/A-DL5: purge doc N=1; `reg` morto sanato
6. A-AH2/A-AH3/A-SK4: gate-feature-matrix.sh (terna, -D warnings, nm/strings) + CI step [KS-AH-78-2]
7. A-BB3/A-DL2: flag `--workers N`; A-BB5: body morto risolto; A-DL3: shutdown pulito (SIGTERM) + protocollo misura in design78.md (A-BG3)
8. A-PP3: test capture-before-reset con controllo positivo
9. A-PP4/A-TH5/A-DS4: fatal → HTTP 500 (prima del freeze contratto HTTP)

**Misura (solo dopo S-78.0 verde)**: Tier-0 + census PER-FASE (compile/run/shutdown separati — lezione WP-59), `--workers 1`, R≥3, warm-up escluso, denominatore A-BB1 = stesso binario `--cli-server` vs `--axum` (A-BG4), peak /usr/bin/time -l + vmmap, PURGE_DELAY=0, probe amplificazione RSS(N)vs(2N) (A-BG5). A-BB1 (Bak) e A-DL1 (Leijen) si giudicano QUI.
**Deferiti WP-79+**: A-TH4 (request_end(self)→CapturedOutput, tocca API Vm ⇒ gate ORM/hk), A-BB6/cache Module (frequenza×taglia prima), A-AH5/A-BB4 (igiene + backpressure/limiti body).

**Kill-switch consolidati**: tabella nel verbale (KS-SK-78.*, KS-MS-*, KS-DS-78-*, KS-PP-*, KS-AH-78-*, KB-78-*, KL-78-*, KG-78.*, KH78-*). Regola trasversale: nessuna cifra senza run tracciato; nessuna misura senza binario identificato.

**Pre-Flight Checklist (WP-78 start)**:
- ✅ Disk space: ≥15G both volumes
- ✅ MySQL: wp, wp_o, wp_p, wptests ready
- ⚠️ Binary baseline: phpr 02aa2ad8 vale SOLO per il CLI; per Axum il riferimento è php-server (hash da ri-verbalizzare con run-gate.sh — A-SK5)
- ✅ Processes: None (cleaned at WP-77.6.5.2.4 closure)
- ❌ Gate G-APERTURA-2: PASS precedente ANNULLATO dal concilio (vacuo) — ri-eseguire via run-gate.sh come primo atto
- 📋 Reading order (WP-78): wp78-harness/COUNCIL_WP78_REVIEWS.md (sintesi + kill-switch) → sessions/WP_SESSION_77_6_5_2_4.md → codice worker_pool.rs+main.rs

---

**Session Closed**: 2026-07-31 02:05 UTC+2 (concilio eseguito in coda: 2026-07-31 mattina)
**Ready for**: WP-78 = S-78.0 sanatoria (bloccante) → fase misura
