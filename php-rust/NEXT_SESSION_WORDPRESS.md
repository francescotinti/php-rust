# NEXT_SESSION_WORDPRESS.md — S-80.0 identity & channel repair 8/8 → WP-81 (LEVA A-BB6)

**Ultima sessione**: S-80.0 (2026-07-31 pomeriggio/sera, commit
a1dee58…7ddb6bc) — ordine vincolante Concilio WP-81 eseguito sui punti 1-8:
identità GIT/tree chiusa (quintetto + tree=clean + archivio per-run + git
match nel driver), canale census riparato (drain ogni uscita, a1≤a FATAL,
RAII !Send con token LIFO, OUTSTANDING dec-post-send, tripwire d==0 con
negativi), sigillo A-MS3 nel type system, gate per-FILE, **misura di
riferimento R=3 spread 0,0%** (`wp80-harness/MEASURE80_RESULTS.md` —
sostituisce design79 §1), **DR-1 chiuso a verdetto MACCHINA** (IC
epoch-guarded, Program graph immutabile), design79 emendato nello stesso
commit della misura. Il punto 9 (leva) è per contratto "SOLO POI" = QUESTA
sessione. Dettaglio: `sessions/WP_SESSION_80.md`.

## Stato gate

- **phpr (CLI, parità release)**: **ef90cb19b0cf93ea** (stash additivo
  `phpr-wp80`; bit mossi dai fix warning A-AH15) — corpus Zend per NOME:
  1418 IDENTICO al set GATE72 + refl 290 IDENTICO
  (`wp80-harness/evidence/`); workspace 1652/0.
- **php-server QUINTETTO (git 6910767, feature-matrix.log `tree=clean`,
  archivio `wp78-harness/matrix-archive/`)**: union **5260f50b991d8cb7** ·
  census **5c9c6eec481d5133** · census-axum-only 4c6264 · axum-only
  29a62b00 · default 2d5257e3. Battery COMPLETA PASS (sorgenti crates ==
  6910767): run-gate · census-twin (marker PER-FILE 39 siti/7 file) ·
  concurrent overlap≥2 (esca ramo FAIL inclusa) · worker-panic 3 fasi +
  near-miss ARMATA · stdout-tandem 6/6 · capture-order (+censimento
  post-request_end==2) · doc-purge 10 pattern ·
  **DR-1 `wp80-harness/gate-dr1-module-immut.sh` PASS**.
- **Misura di riferimento** (KG-81-1 rispettato: raw committati):
  `wp80-harness/MEASURE80_RESULTS.md` — hello axum steady: a=80.476
  call/13.015.960 B, a1=74.288/10.825.612 (IDENTICO ovunque), a3=0,
  inflight_max=1 su 2.860 righe, idle drift=0 su entrambi gli arm (60s).
- Driver measure78.sh: ENFORCE quintetto+git; census W>1 rifiutato;
  boot-probe fuori canale; tripwire a3_trip/inflight; idle su entrambi gli
  arm (sommario tail-3).
- GATE72 CLI (corpus 1418 · refl 290 · ORM 3E/13F · hk 1665) resta la
  baseline trasversale.

## Permanent Binding Rules (invariate; enforcement S-80.0)

1. **Output capture BEFORE request_end()** — gate-capture-order (+ censimento
   letture post-reset pinnato, KS-PP-81-3) + stdout-tandem.
2. **Isolamento = semantica FPM** (A-DS2).
3. **RetainSet thread-affine e PER-RICHIESTA** — ora SIGILLATO:
   `execute_request` costruisce internamente, la forma `&RetainSet` è
   privata di modulo (KS-MS-81-3).
4. **Panic = FAIL-FAST via HOOK globale** + tripwire census d==0.

## ⚖️ Concilio WP-82 (2026-07-31, verbali VINCOLANTI): `wp82-harness/COUNCIL_WP82_REVIEWS.md`

(compilato in chiusura S-80.0 — vedi verbale per emendamenti e kill-switch
vincolanti sul programma leva)

## §WP-81 (prossima sessione) — LEVA A-BB6

Contratto = `wp79-harness/design79.md` EMENDATO S-80.0.7 (+ emendamenti
Concilio WP-82). Ordine:
1. Pre-flight skill + battery verde sul tree di apertura.
2. Implementazione leva (unit cache TL estesa al MAIN): probe
   canonicalize+stat+fp su `meta.path`, `MAIN_CHAIN_FP` COMPUTATO
   (KS-AH-81-2), campo `main_program: Option<Rc<Program>>`, pin coppia
   clone-on-stack (KS-PP-81-2), put DOPO link_fatal_check (§6), contatori
   §8 (`main_probe/main_hit/main_put/main_impure_skip/main_probe_fail`),
   M-68.5 riscritto same-commit + doc-purge (A-DS11), probe on/off UN
   parametro grep-gated (A-TH14), bump atteso A-DS8 1→2 PER NOME.
3. Strumenti §11 PRIMA dell'A/B: walk Program + regola Rc-shared +
   controllo positivo taglia nota (KL-81-2), riga matrix per mem-census,
   `stranded_bytes_dropped` + probe twin (KL-81-1), fixture autoload-run
   (KB-81-3), budget ×W ex-ante (A-MS12).
4. Fixture F1-F12 + F-probe + F-oneshot TUTTE VERDI (KS-DS-80-2;
   KS-DS-81-1/2 = respinta/revert) + battery su HIT full-body vs ORACOLO
   (KS-PP-80-3) + gate_stateful/A-DS9/include_gate/G-APERTURA-2 su HIT.
5. A/B design79 §11: retained ×W → churn R≥3 (predizioni §10: a_calls
   <4.000, KS-AH-80-4 v2 ≥90% su UNA quantità, resid/b invarianti,
   KG-81-3) → footprint twin V2+peak W=num_cpus con floor numerici →
   CPU slope due-N → corpus per NOME + workspace + battery + quintetto.
6. Revert policy KS-DS-80-3: un body ≠ oracolo ⇒ git revert, mai
   fix-forward.

**Kill-switch di rotta (attivi)**: tabella WP-81 (KH81-* · KS-MS-81-* ·
KS-SK-81-* · KS-AH-81-* · KB-81-* · KS-PP-81-* · KL-81-* · KS-DS-81-* ·
KG-81-*, verbale WP-81 §Kill-switch) + eventuali NUOVI WP-82 + mappa
design79 (KH80-* · KB-80-* · KS-*-80-*) + storici (KG-78.D · KS-PP-3 ·
KS-SK-79.2 · KS-AH-78-1). KH81-1 è SODDISFATTO (OUTSTANDING atterrato):
il claim closed-sequential è verdict-grade.

**NON riproporre**: N=1 Vm persistente (KS-DS-78-3); RetainSet condiviso/
Send/persistente o ri-esposto fuori dal sigillo; seconda cache Module
separata dalla unit cache (A-DS5); chiave (path,mtime) senza fingerprint
contenuto (KH80-1); "matches CLI" come contratto server; auto-uguaglianza
come controllo positivo; claim parity sui corpi parse-error (3.9);
MAIN_CHAIN_FP letterale hand-maintained (KS-AH-81-2); somme a_calls+
a1_calls (KS-SK-81-4); estrapolazione % WP da include_heavy (KB-81-4);
cifra retained da walker senza regola Rc-shared+controllo positivo
(KL-81-2).

**Deferiti WP-82+**: A-TH4 (request_end(self)→CapturedOutput); A-AH5/A-BB4
(superglobali reali axum — dichiarare l'asimmetria in ogni confronto
censuscli↔axum, KS-DS-81-3); registry condivisa read-only
(k=12,4-13,1M/worker); spread 12% picco W=10 (metrica di verdetto A/B).

---
**Chiusura**: 2026-07-31. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
