# NEXT_SESSION_WORDPRESS.md — S-78.1 hardening 8/8 + FASE MISURA ESEGUITA → WP-79

**Ultima sessione**: S-78.1 (2026-07-31 pomeriggio, commit 146a4c1…352b63c) —
ordine vincolante Concilio WP-79 eseguito 8/8 + fase misura design78 completa.
🔴 **LEAK REALE trovato dal gate A-DS8 e CHIUSO** (KS-DS-78-4: il RetainSet
per-worker accumulava 1 unit parcheggiata per include per richiesta —
append-only e write-only; fix = RetainSet PER-RICHIESTA, la macchineria del
main; l'opcache vero è la unit cache thread-local). 📊 **Census**: fase
a (lower+compile) = **99% dell'alloc/req** (80k call / 13MB per richiesta di
ricompilazione) — il dato frequenza×taglia che sblocca il design A-BB6.
Dettaglio: `sessions/WP_SESSION_78_1.md` + `wp78-harness/MEASURE78_RESULTS.md`.

## Stato gate

- **phpr (CLI, parità release)**: **ef980f8a** (stash additivo `phpr-wp78`) —
  corpus Zend per NOME PASS: 1418 fail, set IDENTICO a GATE72; workspace
  1651/0/1; Cargo.lock: delta dalla baseline = sola static_assertions (A-AH9 ✓).
- **php-server union (gemello misura)**: **5fdc971680d5e6a2** (git 275c518) —
  run-gate PASS (esteso: include/static_methods/fatal full-body vs oracolo) ·
  gate-worker-panic PASS · **KH78-1 concorrente PASS (W=4, 24 req)** ·
  feature-matrix PASS (default 6a738782 0 net-sym / axum-only f7f6ab17 /
  union-rustc bfd83eed; -D warnings anche sui test target). Census build
  8f4146db (SOLO contatori, KB-78-5).
- **Gate eseguibili (repo `wp78-harness/`)**: run-gate · gate-capture-order
  (ricorsivo, marker censiti 1/1) · gate-feature-matrix · gate-doc-purge
  (7 pattern, esca 7/7; in CI) · gate-stdout-tandem (A-PP8; 4 siti) ·
  gate-worker-panic (A-PP9) · gate-concurrent (KH78-1) · measure78.sh
  (driver misura, KG-78.D).
- **Misure di riferimento server** (MEASURE78_RESULTS.md): Tier-0 10,7MB ·
  axum W1 peak 59,7MB / V2 18,0M · cli-server 54,0MB / 13,2M · delta/req = 0
  (prova regina N vs 2N) · linearità W: k=12,4-13,1M/worker · used_n=0 ·
  depth=1 ovunque.
- GATE72 CLI (corpus 1418 · refl 290 · ORM 3E/13F · hk 1665) resta la
  baseline trasversale; corpus riconfermato oggi su ef980f8a.

## Permanent Binding Rules (invariate + 1 nuova da S-78.1)

1. **Output capture BEFORE request_end()** — enforcement: gate-capture-order
   (ricorsivo, pattern `request_end\(`, KH79-1) + A-PP3. Body HTTP = SOLO
   `vm.rendered` (gate-stdout-tandem custodisce il contenimento, KS-PP-5).
2. **Isolamento = semantica FPM** (A-DS2): statics/closure/static props
   muoiono col Vm; request_end non li tocca (KS-DS-78-1/-3).
3. **RetainSet thread-affine e PER-RICHIESTA** (A-MS2/A-MS3 + S-78.1.5):
   `!Send+!Sync` per costruzione (KS-MS-1) + pin request-lifetime — un
   RetainSet a vita più lunga della richiesta è il leak KS-DS-78-4
   (gate: include_units_pinned_per_request_not_leaked + pattern doc vietati).
4. **Politica panic worker = FAIL-FAST** (A-PP9/KS-PP-6): panic ⇒ abort del
   processo (mai thread morto in silenzio); gate-worker-panic.

## ⚖️ Concilio WP-80 ESEGUITO (2026-07-31, verbali VINCOLANTI): `wp80-harness/COUNCIL_WP80_REVIEWS.md`
**9× CONCORDO CON EMENDAMENTI, 0 opposizioni.** S-78.1 giudicata reale
(cifre = raw ×campione — Gregg; fix leak Zend-fedele — Stogov; rigore gate
mantenuto — Klabnik). MA: **bug reale nel contatore depth** (inc-DOPO-send,
3 sedie: fail-open, può sotto-contare — ogni claim depth-based è ADVISORY
finché non fixato) e **il "99% fase a" NON autorizza ancora il design A-BB6**
(3 sedie: va decomposto prelude/main/include — su hello è quasi tutto
PRELUDE). Delta A/B declassato: verdict-grade = **+4,8M residente** (il
+5,7MB peak usa il braccio vietato da KG-79.B). Forma fedele della leva
(Stogov A-DS5): **la STESSA unit cache TL estesa al main** (canonicalize+
stat+fingerprint), MAI una seconda cache, MAI mtime (6 sedie).

## §WP-79 (prossima sessione) — S-79.0 "PRE-LEVER HARDENING" poi design A-BB6

**S-79.0 (ordine vincolante, verbale WP-80 §Sintesi — non rinegoziare)**:
1. Fix depth inc-PRIMA-di-send + test anti-wrap + reset watermark nei test
   (A-TH1/A-BB13/A-MS1/A-SK8) [KH80-4]
2. Doc purge: frase request_end "cross-request survivor" + pattern nuovi
   (A-DS1); SAFETY inline su AssertUnwindSafe (A-MS4)
3. Census hardening: split a1/a2/a3 (prelude/main/include) + seconda fixture
   include-pesante + residuo Δglobale−(a+b+c) + controllo positivo canale c
   + righe-attese==N + finestra idle + census.inc R≥3
   (A-TH2/A-BB10..12/A-SK5/A-PP5/A-AH13/A-DL5/A-BG13/A-BG14)
4. Identità QUATERNA: census build nel feature-matrix+CI con hash nel log
   (A-AH10), gate equivalenza funzionale del gemello (A-AH11), driver
   ENFORCE hash+depth exit≠0 (A-SK7/A-BG15/A-AH12)
5. Gate hardening: concurrent con osservabile di sovrapposizione (A-SK1),
   stdout-tandem NSITES==4+verbi+tandem-scrittura (A-SK2/A-PP3), marker per
   posizione (A-PP1), worker-panic RC!=134 FAIL + N pinnato (A-SK4), panic
   dispatcher gateato (A-PP4)
6. Test in-cargo a forma worker_loop (A-MS2/A-TH5) + riga census sul path
   cli-server (A-BG16, prerequisito del verdetto A-BB1)
7. Stranded keys unit cache: supersede-per-path o cap + contatore (A-DS2)
   [KS-DS-80-1]
8. SOLO POI design A-BB6 coi vincoli consolidati (§Sintesi punto 8 del
   verbale: unit cache TL estesa al main, fingerprint, pin nel RetainSet
   della richiesta, immutabilità Module asserita, 6 fixture A-DS6,
   predizione ex-ante, A/B coppia churn/retained + peak W=num_cpus + CPU +
   corpus per NOME; body ≠ oracolo ⇒ REVERT) [KS-DS-80-2/-3, KG-80-1,
   KL-80-1/2, KS-AH-80-2/3/4]

**Kill-switch di rotta (attivi)**: KG-78.D (cifra senza run tracciato) ·
KB-78-5/KL-78-5 (cifre solo dal gemello) · KB-78-3/KG-78.A (workers≠1 o R<3)
· KH78-2/KB-78-2 (depth>1 ⇒ VOID) · KS-PP-3 (tocchi a request_* ⇒
G-APERTURA-2 per NOME) · KS-DS-78-4 (retain len oltre le unit distinte) ·
KS-SK-79.2 (mai test a sola auto-uguaglianza) · KS-AH-78-1 (misura senza
feature-matrix.log = nulla).

**NON riproporre**: N=1 Vm persistente (KS-DS-78-3); RetainSet
condiviso/Send/persistente (KS-MS-3, KS-DS-78-4); "matches CLI" come
contratto server; auto-uguaglianza o `head -1` come controllo positivo;
claim di parity sui corpi parse-error (KS-DS-78-5, divergenza registrata
PHPR_DIVERGENCES 3.9).

**Deferiti WP-80+**: A-TH4 (request_end(self)→CapturedOutput, gate ORM/hk);
A-AH5/A-BB4 (backpressure, limiti body, superglobali reali da metadata HTTP);
registry condivisa read-only tra worker (k=12,4-13,1M/worker); spread 12%
picco W=10 (solo se diventa metrica di verdetto).

---
**Chiusura**: 2026-07-31. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
