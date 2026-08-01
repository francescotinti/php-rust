# NEXT_SESSION_WORDPRESS.md — S-83.0: DENTI DI VERBALE + MISURE (verdict83 PASS, VC CHIUSA ~63×, partizione-per-TIPO) → WP-84

**Ultima sessione**: S-83.0 (2026-08-01, commit 050c891…857da59) — gli 8
punti del Concilio WP-84 eseguiti INTEGRALI: A-MS17 VmGate al PRIMO commit
(KS-MS-84-1), A-MS22, A-TH23/24/25, A-DS23, A-PP22/23, A-SK29/30, A-AH33/34,
A-DL20/21, fixture F14b/F15b/path415B/reg-only, A-DS22 (comparatore era
DOPPIO-cieco: metodi+const pool — rafforzato), slope in FORMA NUOVA
(min-of-R R=9 + probe per-request __reqns), **campagna-2 VERDICT83 PASS**
(`wp83-harness/MEASURE83_RESULTS.md`), **partizione-per-TIPO
UnitSlot{main, includes} IMPLEMENTATA** (main esente per corsia, F15/F15b
FLIPPATI verdi). Tre denti nuovi hanno MORSO al primo giro (A-DS23,
A-DS22, A-AH32 → emendato PRUNE-ONLY, giudizio WP-85). Dettaglio:
`sessions/WP_SESSION_83.md`.

## Stato gate

- **phpr (CLI, parità release)**: **7a6104575134de27** (stash additivo
  `phpr-wp83`; bit mossi da A-MS17/A-MS22/partizione A-MS24) — corpus
  Zend per NOME: **1418 IDENTICO** + refl 290 IDENTICO (verificato DUE
  volte: p1a e p6, `wp83-battery-out/` fuori repo); workspace 1655/1655.
- **php-server**: battery-83pre 15/15 a 925da3b + equivalenza LEGALE
  925da3b→e5f2a5e (prima uscita reale di battery-equivalence.sh, per
  NOME, ledgerata); matrix-final a **e5f2a5e**; campagna-2 stesso-chain
  senza commit intermedi. Gate lever: pins/fixtures/fixtures2 PASS con
  F15/F15b FLIPPATI (main_evicted==0 con macchina evizione VIVA).
- **Misure**: `wp83-harness/MEASURE83_RESULTS.md` + verdict83.out — VC
  **PASS verdict-grade** (slope 110,0 vs 6920,0 µs/req, min-of-R risolto)
  · VR algebra ESATTA (budget scorporato **19,69MB ×W**; one-time 1°
  lower 7,00MB=35,6%; condivisibile ~69%) · VA split (+4 reg / ≤+52
  include-HIT upper) · VW **2×len FALSIFICATO** al confine (esito
  nominato). Void: 1 quarantena, 21 slot-run, 108 file (lock-cmp bite).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (invariate)

1. **Output capture BEFORE request_end()**. 2. **Isolamento = semantica
FPM** (A-DS2). 3. **RetainSet thread-affine PER-RICHIESTA, sigillato**;
porta vm_new/park_main = token `VmGate` (A-MS17, rustc giudice). 4.
**Panic = FAIL-FAST**. 5. **Un body ≠ oracolo sulla leva ⇒ git revert,
mai fix-forward** (KS-DS-80-3, mai innescata). 6. **Mai rm di raw:
quarantena + manifest** (KS-AH-83-2).

## §WP-84 (prossima sessione) — aperture S-83.0 (per NOME) + ripresa ROADMAP

1. **A-PP18** (ledger drained esatto): PRIMA di ogni riconciliazione
   Δglobal a W>1 (A-PP24 — in S-83.0 nessuna tentata, vincolo onorato
   dall'ordine).
2. **A-BB34 pin identità peak** (232±1MB W=10) da rieseguire su matrix
   fresca post-partizione + VP formale della campagna WP-84.
3. **VW modello nuovo**: nominare il SITO della 4ª copia (a len=415:
   a_bytes = 2×len + 2×(len+1), a_calls=4) e pinnare il modello a soglia.
4. **Emendamento lock-cmp PRUNE-ONLY** (A-AH32): giudizio del Concilio
   WP-85 (verbale in wp85-harness quando convocato).
5. Registry condivisa read-only (condivisibile ~69% MISURATO): SOLO con
   A-BB35 budget ex-ante + costo sync sul path HIT + riapertura ESPLICITA
   KH81-3 (KH84-4).
6. Poi: **ripresa ROADMAP da [[php-rust-todo-master]]**. Deferred
   invariati: A-TH4; A-AH5/A-BB4 superglobali axum (KS-DS-82-3); ORM/hk
   perf solo build-adiacente (KS-AH-83-4).

**Kill-switch di rotta**: tabella WP-84 (31 KS, verbale
`wp84-harness/COUNCIL_WP84_REVIEWS.md`) TUTTI onorati o chiusi in S-83.0
(KS-MS-84-1 ✓ primo commit; KH84-1 ✓ split; KH84-3 ✓ dente cast;
KS-SK-84-1/KS-AH-84-3 ✓ equivalenza per NOME esercitata; KS-AH-84-1 ✓ il
dente ha MORSO (emendamento dichiarato); KB-84-1 ✓ forma cambiata, mai
rieseguito N-doppio; KB-84-2 ✓ fixture 415B (falsificazione onesta);
KB-84-3 ✓ fixture di controllo; KL-84-1/2 ✓ scorporo+scomposizione;
KS-DS-84-1 ✓ pin-HIT; KS-DS-84-2 ✓ A-DS22; KS-DS-84-3/4 ✓ F15b verde +
tripwire cablato; KG-84-1 ✓ void per-manifest; KG-84-2 ✓ refusal nel
base_run; KG-84-3 ✓ forma per-request presente; KS-PP-84-2 ✓ delta
enumerazione in a_pp20; KS-PP-84-3/KL-84-3 ✓ cifre per-ARM/etichette.
KS-PP-84-1 non innescato (nessun nuovo segmento twin-pair).

**NON riproporre**: tutti i NON-riproporre WP-83 restano; in più — slope
su coppie a fattore 3 "chiudibile per N" (KB-84-1: SOLO forma
per-request/min-of-R); byte-cmp del lock su manifest divergenti per
dev-dep (forma = PRUNE-ONLY, salvo diverso giudizio WP-85); seconda
equivalenza sulla stessa battery rev (ledger morde: si rifà la battery).

---
**Chiusura**: 2026-08-01. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
