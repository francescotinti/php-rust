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

## ⚖️ Concilio WP-78 (verbali VINCOLANTI): `wp78-harness/COUNCIL_WP78_REVIEWS.md`
S-78.0 chiusa; kill-switch di misura (KB-78-*, KL-78-*, KG-78.*, KH78-*)
ATTIVI per la fase misura. ⚖️ Concilio WP-79 (su S-78.0 + programma misura):
`wp79-harness/COUNCIL_WP79_REVIEWS.md` — da convocare/leggere in apertura.

## §WP-78 (prossima sessione) — FASE MISURA (protocollo: `wp78-harness/design78.md`)

Ordine (dal design78, TUTTO già deliberato — non rinegoziare):
1. **KH78-1 PRIMA di tutto**: creare+eseguire il gate CONCORRENTE (richieste
   parallele su N worker, fixture stateful); panic/divergenza ⇒ HALT.
2. Gate della build da misurare: feature-matrix + run-gate (hash a verbale).
3. Tier-0 baseline (Axum runtime senza pool, riferimento WP-77.1).
4. Census per-fase `--workers 1` (compile/run/shutdown SEPARATI — KB-78-1):
   qui si giudicano **A-BB1** (≤+2% alloc, denominatore A-BG4: stesso binario
   `--cli-server` vs `--axum`) e **A-DL1** (frammentazione).
5. Probe amplificazione RSS(N)vs(2N) + linearità footprint(W) (A-BG5/A-DL2).
6. R-G4 CPU attribution SOLO se commissionato (contatori owner-level, mai
   solo `sample`).

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
