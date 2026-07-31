# NEXT_SESSION_WORDPRESS.md — S-78.0 sanatoria CHIUSA → WP-78 fase misura

**Ultima sessione**: S-78.0 SANATORIA (2026-07-31, commit 31201ba…022c8d2 +
chiusura) — tutti i 9 punti dell'ordine vincolante del Concilio WP-78
eseguiti; **bug reale trovato+fixato: body HTTP raddoppiato** (rendered+stdout;
rendered contiene già stdout — invisibile ai check di auto-uguaglianza e a
`head -1`). Dettaglio: `sessions/WP_SESSION_78.md`.

## Stato gate

- **phpr (CLI, parità release)**: **02aa2ad8** — NON ricompilato in S-78.0.
  ⚠️ Al primo rebuild l'hash CAMBIERÀ (dep compile-time `static_assertions`
  aggiunta a php-runtime): non è un allarme; a quel punto rifare il gate
  corpus per NOME e ri-verbalizzare la parità (stash additivo phpr-old-target).
- **php-server (Axum, union build)**: **d98b531e6b5ee306** — baseline Axum
  (A-SK5: MAI citare phpr per lavoro Axum). Hash stampato da run-gate.sh.
- **Suite**: 1651/0/1 (=1652) su albero 022c8d2; +4 test feature-gated
  (`cargo test -p php-server --features axum-server`).
- **Gate eseguibili (repo, `wp78-harness/`)**: `gate-axum/run-gate.sh`
  (G-APERTURA-2, PASS) · `gate-capture-order.sh` (KS-PP-2, PASS) ·
  `gate-feature-matrix.sh` (terna, PASS: default faacd79d / axum-only
  5a293aa3 / union b5a19fc5). CI estesa (terna -D warnings + test feature).
- **Harness**: canonico NEL REPO (`wp77-harness/`, `wp78-harness/`); la dir
  esterna `/Volumes/Extreme Pro/Claude/wp77-harness/` è archivio superseded.
- GATE72 (corpus 1418 · refl 290 · ORM 3E/13F · hk 1665) resta la baseline
  CLI: non toccato da S-78.0 (phpr invariato).

## Permanent Binding Rules (aggiornate dal Concilio WP-78)

1. **Output capture BEFORE request_end()** (Pedersen/Stogov) — enforcement:
   `gate-capture-order.sh` (KS-PP-2) + test A-PP3 con controllo positivo
   (KS-PP-1). Il body HTTP è SOLO `vm.rendered` (contiene già stdout).
2. **Isolamento = semantica FPM** (Stogov A-DS2): statics/closure_statics/
   static_props per-richiesta per MORTE del Vm (`request_end()` non li tocca);
   restano per-richiesta anche con un eventuale Vm persistente
   (KS-DS-78-1/-3). Gate: fixture stateful ×3 + test cargo.
3. **RetainSet thread-affine** (Matsakis A-MS2/A-MS3): `!Send+!Sync` per
   costruzione; static assert in php-runtime (KS-MS-1: rimozione solo per
   delibera Concilio); solo `WorkerTask: Send` attraversa il canale.

## ⚖️ Concilio WP-79 ESEGUITO (2026-07-31, verbali VINCOLANTI): `wp79-harness/COUNCIL_WP79_REVIEWS.md`
**9× CONCORDO CON EMENDAMENTI, 0 opposizioni — l'opposizione Klabnik di WP-78
è SOLLEVATA.** S-78.0 giudicata sanatoria REALE; ma il programma misura è
dichiarato **NON ESEGUIBILE così com'è** (Gregg): i kill-switch di misura non
hanno osservabili nel codice. La prossima sessione apre con **S-78.1
"hardening pre-census"** (ordine vincolante in §Sintesi del verbale WP-79),
POI esegue la misura. Kill-switch WP-78 + WP-79 tutti ATTIVI.

## §WP-78 (prossima sessione) — S-78.1 HARDENING → FASE MISURA

**S-78.1 (ordine vincolante, dal verbale WP-79 §Sintesi — non rinegoziare)**:
1. Doc purge residua (A-MS6/A-AH8/A-DS6: "persistent Vm"/"object table" in
   worker_pool.rs + vm/mod.rs:3417-3420) + gate KS-MS-2 eseguibile (A-MS5)
2. Lifecycle worker: JoinHandle+`shutdown()` join (A-MS7/A-DL6) + politica
   panic worker (A-PP9) [KS-MS-5, KL-78-4, KS-PP-6]
3. Gate hardening (A-TH7/A-PP6/A-PP7/A-SK7/A-SK8/A-SK9/A-SK10/A-AH6/A-AH7)
4. Contratto fatal full-body vs oracolo + formato unificato (A-TH8/A-PP10/
   A-DS7) [KH79-2, KS-DS-78-5] + divergenza html_errors (A-DS10) + A-PP8
5. Fixture estese: include/require + static metodo/ereditati (A-DS8/A-DS9)
6. Strumentazione census feature-gated con controlli positivi (A-BB7/A-BB8/
   A-BG7/A-DL7) — verdetti footprint SOLO dal gemello non strumentato
7. design78 emendato (A-DL8/A-BG8/A-BG9/A-BG10/A-BG11/A-BB9)
8. KH78-1 gate concorrente secondo contratto A-SK6 [KS-SK-79.3]

**Misura (solo dopo S-78.1 verde)**: Tier-0 (config tracciata o esce —
KG-79.D) → census per-fase `--workers 1` (A-BB1/A-DL1 si giudicano qui) →
probe amplificazione + linearità W → R-G4 solo se commissionato.
Al primo rebuild phpr: audit Cargo.lock = solo static_assertions (A-AH9)
[KS-AH-78-4], poi corpus per NOME.

**Kill-switch di rotta**: KG-78.D/KL-78-3 (nessuna cifra senza run tracciato /
PURGE_DELAY=0 / mai ps RSS) · KB-78-3/KG-78.A (workers≠1 o R<3 ⇒ VOID) ·
KH78-2/KB-78-2 (depth coda >1 ⇒ VOID) · KS-PP-3 (tocchi a request_* ⇒
G-APERTURA-2 per NOME).

**NON riproporre**: N=1 Vm persistente (respinto WP-77.2, KS-DS-78-3);
RetainSet condiviso/Send (KS-MS-3); cache Module PRIMA del censimento
(A-BB6); "matches CLI" come contratto server; check a auto-uguaglianza o
`head -1` come controllo positivo (lezione S-78.0).

**Deferiti WP-79+**: A-TH4 (request_end(self)→CapturedOutput, gate ORM/hk),
A-BB6 cache Module, A-AH5/A-BB4 (igiene, backpressure, limiti body,
superglobali reali da metadata HTTP).

---
**Chiusura**: 2026-07-31. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
