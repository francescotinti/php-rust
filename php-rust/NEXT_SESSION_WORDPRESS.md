# NEXT_SESSION_WORDPRESS.md — S-79.0 hardening 8/8 + design79 A-BB6 → WP-80

**Ultima sessione**: S-79.0 (2026-07-31 sera, commit ff22a2e…36d62df) —
ordine vincolante Concilio WP-80 eseguito 8/8, poi design leva A-BB6
(`wp79-harness/design79.md` = CONTRATTO d'implementazione). 🔵 **Split
census a1/a2/a3 operativo e già decisivo**: su include_gate a1 (prelude) =
10,83MB su 13,36MB (~81%) — il "99% fase a" era prelude, MA la forma fedele
(unit cache TL estesa alla UNIT MAIN, che contiene il prelude compilato)
salta a1+a2 insieme su HIT. Stranded keys chiuse (supersede-per-path);
overlap concorrente PROVATO (0,92s W=4 vs 3,43s W=1); panic-hook abort
globale (dispatcher incluso); riga census-cli sul path cli-server (A-BB1
giudicabile). Dettaglio: `sessions/WP_SESSION_79.md`.

## Stato gate

- **phpr (CLI, parità release)**: **a76acc1668d03c86** (stash additivo
  `phpr-wp79`) — corpus Zend per NOME PASS: 1418 fail, set IDENTICO a
  GATE72; workspace 1652/0/1 (1651 baseline + test supersede).
- **php-server QUATERNA (git 36d62df, feature-matrix.log della stessa
  build)**: union **0405077667de3f2d** · census **80f88ea7efcb3c4a** ·
  axum-only d010e291 · default 8d6c7778. Battery COMPLETA sul tree finale:
  run-gate union PASS · **census-twin PASS** (marker 18/11/2 + full-body
  vs oracolo sul binario census) · worker-panic PASS (3 fasi, abort 134
  worker E dispatcher) · concurrent PASS (overlap 0,917s + discriminatore
  W=1 3,429s) · doc-purge 9/9 · capture-order (1/1 + posizione) ·
  stdout-tandem 6/6 · php-server test 9/9 + 14/14.
- **Driver misura ENFORCE** (measure78.sh): hash==riga matrix per modo
  (census/censuscli→bin[census], altri→bin[union]) o rifiuta; righe==N;
  depth>1 ⇒ exit≠0; probe idle; modo nuovo `censuscli`.
- GATE72 CLI (corpus 1418 · refl 290 · ORM 3E/13F · hk 1665) resta la
  baseline trasversale.

## Permanent Binding Rules (invariate; enforcement aggiornato S-79.0)

1. **Output capture BEFORE request_end()** — gate-capture-order (ricorsivo +
   marker per FORMA/CENSIMENTO/POSIZIONE) + stdout-tandem (verbi estesi,
   tandem-di-scrittura, NSITES==6).
2. **Isolamento = semantica FPM** (A-DS2): statics/closure/static props
   muoiono col Vm.
3. **RetainSet thread-affine e PER-RICHIESTA** (KS-DS-78-4) — test tutti a
   forma worker_loop da S-79.0.6.
4. **Panic = FAIL-FAST via HOOK globale** (A-PP9+A-PP4): abort su OGNI
   thread del server, dispatcher incluso; gate-worker-panic 3 fasi.

## §WP-80 (prossima sessione) — FASE MISURA pre-A/B poi IMPLEMENTAZIONE A-BB6

Contratto = `wp79-harness/design79.md` (vincoli Council WP-80 già mappati).
⚖️ Il Concilio WP-81 sul S-79.0 è in `wp81-harness/COUNCIL_WP81_REVIEWS.md`
(vincolante: leggerne la sintesi PRIMA di iniziare).

1. **Misura census di riferimento R≥3** (KB-80-1/A-BG13): modi `census` E
   `censuscli`, fixture hello + include_gate + **include_heavy**, driver
   ENFORCE attivo; citare ASSOLUTI a1/a2/a3/resid; finestra idle nel
   verbale. (Le cifre S-79.0 sono smoke R=1, dichiarate tali.)
2. **DR-1 (bloccante)**: audit interior-mutability del grafo Module
   (PropIc/MethodIc — design79 §5); esito (a) assert o (b) fixture F7 verde.
3. Implementazione leva per design79 §2-8 (chiave/fp §3, pin §4, fixture
   F1-F9 §9 TUTTE verdi prima dell'A/B, contatori main_probe/hit/put/
   impure_skip + bump marker census-twin nello stesso commit).
4. **A/B design79 §11**: coppia churn+retained (retained profondo del main
   cached misurato PRIMA, ×W) + peak W=num_cpus (>2% ⇒ non passa) + CPU +
   corpus per NOME + battery completa. Predizioni §10 dichiarate PRIMA.
5. Su qualunque body ≠ oracolo: **REVERT, mai fix-forward** (KS-DS-80-3).

**Kill-switch di rotta (attivi)**: tutti quelli della mappa design79
§Kill-switch (KH80-1/2/3/4 · KB-80-1..5 · KS-SK-80-1..4 · KS-AH-80-1..4 ·
KS-MS-80-1..3 · KS-PP-80-1..3 · KL-80-1..3 · KS-DS-80-1..3 · KG-80-1..3) +
gli storici KG-78.D · KB-78-5/KL-78-5 · KB-78-3/KG-78.A · KH78-2/KB-78-2 ·
KS-PP-3 · KS-SK-79.2 · KS-AH-78-1 (ora ENFORCED dal driver).

**NON riproporre**: N=1 Vm persistente (KS-DS-78-3); RetainSet condiviso/
Send/persistente; seconda cache Module separata dalla unit cache (A-DS5);
chiave (path,mtime) senza fingerprint contenuto (KH80-1); "matches CLI"
come contratto server; auto-uguaglianza come controllo positivo; claim
parity sui corpi parse-error (3.9); verdetti depth-based senza il fix
inc-before-send (chiuso, ma il claim retroattivo su run S-78.1 resta
ADVISORY).

**Deferiti WP-81+**: A-TH4 (request_end(self)→CapturedOutput, gate ORM/hk);
A-AH5/A-BB4 (backpressure, superglobali reali da metadata HTTP); registry
condivisa read-only tra worker (k=12,4-13,1M/worker); spread 12% picco W=10
(È metrica di verdetto per A-BB6: entra nell'A/B §11.3).

---
**Chiusura**: 2026-07-31. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
