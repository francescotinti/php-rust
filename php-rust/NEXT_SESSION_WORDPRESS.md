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

## ⚖️ Concilio WP-81 ESEGUITO (2026-07-31, verbali VINCOLANTI): `wp81-harness/COUNCIL_WP81_REVIEWS.md`
**9× CONCORDO CON EMENDAMENTI, 0 opposizioni.** S-79.0 verificata NEL CODICE
da ogni sedia. MA: **identità ancora bucata sull'asse GIT/tree** (driver non
confronta il git del matrix; log sovrascritto; tree sporco non rilevato — 3
sedie), **design79 §1 = cifra senza run tracciato** (Gregg, ADVISORY
bloccante: i numeri citati non stanno in alcun raw del repo), **anti-wrap
vacuo in release** (Hoare: il test passa CON la regressione; dissenso
Matsakis/Klabnik registrato — prevale la dimostrazione), **off-by-one
righe==N** (boot-probe = 1 riga extra: il check VOID-erebbe ogni run
legittima), **A-MS3 non sigillato** (bloccante pre-leva), **pin del Program
mancante** (il RetainSet non può parcheggiarlo; walker retained salta le
entry Rc-shared ⇒ sottostima proprio il prelude condiviso), **early-return
fatal non drena lo split** (F8 produrrebbe esattamente la sequenza
inquinante). Refutazioni RESPINTE dal collegio: doppio-pass warning-free
(Hejlsberg), supersede-vs-opcache (Stogov), soglie overlap (Klabnik — ma il
claim si riduce a "≥2 worker").

## §WP-80 (prossima sessione) — S-80.0 "IDENTITY & CHANNEL REPAIR" poi misura poi leva

Ordine vincolante = verbale WP-81 §Sintesi (9 punti, non rinegoziare):
1. Identità GIT/tree nel driver+matrix (archivio per-run, tree pulito,
   match esatto) [KG-81-2, KS-AH-81-1, KS-SK-81-2]
2. Driver: fix off-by-one righe==N (A-BG19) · split rifiutato a W>1
   (A-BB17) · idle probe su censuscli + run idle ≥60s · etichetta gross
3. Canale: drain split su OGNI uscita + assert a1≤a + A3Window !Send/LIFO/
   tripwire + a1 RAII [KS-PP-81-1, KS-MS-81-2] · tripwire depth d==0⇒abort
   + test negativo + IN_FLIGHT o declassamento [KH81-1/2]
4. Sigillo A-MS3 (newtype/costruzione interna o grep-gate) [KS-MS-81-3]
5. Gate: -D warnings cross-crate census + quinta config pinnata/bandita
   [KS-AH-81-4] · pin marker per-FILE [KS-SK-81-3] · SAFETY catch_unwind
   corretta (A-TH11) · near-miss dispatcher (A-PP12) · esca ramo FAIL +
   body W=1 concurrent (A-SK9) · censimento letture post-request_end
   [KS-PP-81-3]
6. Misura riferimento R≥3 (census E censuscli; hello+include_gate+
   include_heavy) che SOSTITUISCE il §1 ADVISORY + floor non-compile
   ex-ante [KB-81-2, KG-81-1]
7. design79 emendato stesso commit: MAIN_CHAIN_FP computato (A-AH19/A-DS7)
   [KS-AH-81-2] · pin coppia (Module,Program) + DR-1 esteso al Program
   [KS-PP-81-2] · F10-F12 + F-probe-fail + F8b + link-fatal + main_probe==0
   one-shot [KS-DS-81-1/2] · KS-AH-80-4 su UNA quantità [KS-SK-81-4] ·
   predizioni resid+b [KG-81-3] · KS-MS-80-2/F6 su park-eventi · strumento
   retained con regola Rc-shared + controllo positivo [KL-81-2] · budget
   ×W ex-ante · supersede bytes+probe twin [KL-81-1] + filtro reg_mode
8. DR-1 (Module E Program): IC fuori dal Module o reset-on-hit con
   contatore; audit = check macchina, mai prosa/solo-F7 [KH81-3]
9. SOLO POI leva + fixture verdi + A/B (CPU slope due-N, peak W=num_cpus,
   floor numerici; body ≠ oracolo ⇒ REVERT, KS-DS-80-3)

**Kill-switch di rotta (attivi)**: la tabella NUOVA WP-81 (KH81-1..3 ·
KS-MS-81-1..3 · KS-SK-81-1..4 · KS-AH-81-1..4 · KB-81-1..5 · KS-PP-81-1..3 ·
KL-81-1..3 · KS-DS-81-1..3 · KG-81-1..3 — verbale §Kill-switch) + la mappa
design79 (KH80-* · KB-80-* · KS-SK-80-* · KS-AH-80-* · KS-MS-80-* ·
KS-PP-80-* · KL-80-* · KS-DS-80-* · KG-80-*) + gli storici KG-78.D ·
KB-78-5/KL-78-5 · KB-78-3/KG-78.A · KH78-2/KB-78-2 (ora ADVISORY per KH81-1
finché IN_FLIGHT non atterra) · KS-PP-3 · KS-SK-79.2 · KS-AH-78-1.

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
