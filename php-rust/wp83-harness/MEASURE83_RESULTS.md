# MEASURE83_RESULTS.md — misure WP-83 nelle FORME ordinate dal Concilio WP-84

**Cifre di misura** = verdetto SCRIPTATO dei raw (`wp83-harness/verdict83.sh`
→ `verdict83.out`, entrambi committati; fail-closed A-SK19). **Cifre
derivate** = righe `[derivata]` di verdict83.out (A-BG26). Documento
vincolato al corpus committato da `gate-measure-cifre.sh` (KG-83-3, corpus
esteso ai raw m83/83*).

## Identità

- Campagna-2 a git **e5f2a5e**, matrix stesso-chain senza commit intermedi
  (matrix-final PASS a e5f2a5e); battery-83pre PASS 15/15 a 925da3b,
  equivalenza LEGALE 925da3b→e5f2a5e via `battery-equivalence.sh`
  (delta = solo archivio matrix; per NOME, ledgerata — prima uscita reale
  della macchina A-SK30/A-AH34). Driver_sha con campaign script incluso
  (A-AH30) nell'header di OGNI run measure78; braccio base con header
  A-TH26 (rustc -V + sha di ENTRAMBI i lock).
- **Braccio base**: 7593d8e via `base-arm-build.sh` (A-AH31); **lock-cmp
  in forma PRUNE-ONLY** (emendamento DICHIARATO di A-AH32, al giudizio del
  Concilio WP-85): il byte-cmp letterale ha MORSO alla prima uscita —
  diff = UNA riga cancellata (edge dev-dep `"mimalloc"` assente dal
  manifest 7593d8e), set (nome, versione) IDENTICO, zero drift; il diff
  grezzo è archiviato in-header (`m83.base.header`).
- **Contabilità VOID (A-BG29, ricomputata dal manifest)**: 1 quarantena
  (`evidence/void/*-ac35d24-lockcmp-bite/`, manifest sha256): campagna-1
  morta fail-closed al build base (dente lock-cmp) — **21 slot-run VOID
  (18 fase C lever + 3 CR), 108 file**, mai `rm` (KS-AH-83-2).
- Strumenti ARMATI provati in ogni build citato (lezione S-82.0):
  fase R con `PHPR_MEM_CENSUS` + righe presenti; fase CR con
  `PHPR_REQ_NS=1` + righe `reqns:` drenate FUORI dalle finestre
  temporizzate (WP-64); `alloc_id=memcount-v2-s82` in-band su ogni riga
  census (A-AH33, pin census-twin).

## Verdetti (da verdict83.out)

- **VC slope CPU — PASS (KB-83-3 CHIUSA nella forma nuova)**: estimatore
  min-of-R, R=9 per cella, f=5% ex-ante; risoluzione 20,0 µs/req <
  banda/3 (123,7) ⇒ il claim è verdict-grade: **slope_lever = 110,0
  µs/req vs slope_base = 6920,0 µs/req** (bound 7291,0) — la leva taglia
  la pendenza compile-side di **~63×**. Scope NOMINATO: axum W=1 hello
  closed-sequential, binari union non strumentati, bracci interleaved
  stessa sera (A-TH26). Il programma "N-doppio" refutato dal Concilio non
  è mai stato rieseguito: ha chiuso il CAMBIO DI FORMA (A-BB31).
- **VCR per-request [derivata]** (A-BG32, braccio lever SOLO — il binario
  base pre-leva non ha il probe, DICHIARATO): regime su N=2000 ×3:
  median 330-360 µs/req, min 167-184, p90 420-503 — la distribuzione
  per-request esiste ora come attribuzione; include l'overhead
  dispatcher→worker (il min ~170µs concorda con lo smoke di sviluppo).
- **VR retained SCOMPONIBILE — PASS (algebra ESATTA)**: riga finale r1
  (4 fixture): `rw_bytes=14.250.027 − program_floor_share=1.009.360 +
  rw_main_net=7.407.810 = rw_budget=20.648.477` (**19,69 MB ×W**,
  arm=cli-server, main_evicted=0, uclog=0) — Σnet(entries)==rw_main_net e
  Σfloor_inc==program_floor_share ESATTE. Il 20,65MB di S-82.0 resta
  UPPER; il doppio conteggio scorporato vale ~0,96MB (KL-84-1 soddisfatta).
- **VR per-entry [derivata] — il "flat" di Leijen DISSOLTO (A-DL20)**:
  ord=1 net=7.349.977 B, median(altri)=8.208 B ⇒ **residuo one-time del
  primo lower ≈ 7,00 MB = 35,6% del budget** — massa di PROCESSO
  (prelude/interner al primo lower), non per-entry. Il "1,85MB/main
  flat" era l'artefatto dell'aggregato÷4.
- **VR A-DL22 [derivata]**: hello-only W=1: rw_budget=10,14 MB vs
  marginale VP 3,44 MB/worker ⇒ ratio 2,95; **frazione condivisibile
  (upper est) ≈ 69%** — input della delibera peak ×W (KL-84-2
  soddisfatta: la delibera ora HA la scomposizione).
- **VA register split — PASS**: steady a3==0 ESATTO sui tre bracci;
  [derivata] **register = +4 call/req**, **include-HIT ≤ +52 call/req**
  (upper bound vs opcache inheritance-cache, A-DS25) — il composito +56
  di S-82.0 è SPLITTATO (A-BB33/KB-84-3 soddisfatte).
- **VW path≥384B — NAMED-DEVIATION (KB-84-2: esito nominato, mai claim
  silenzioso)**: sul path canonico da 415 B, **a_bytes=1662 ≠ 2×len=830**
  e **a_calls=4,0** (non 2): il modello-floor "2 allocazioni da len" è
  **FALSIFICATO al confine**. Aritmetica dal raw: 1662 = 2×415 + 2×416 =
  2×len + 2×(len+1) — due copie EXTRA con terminatore sul path lungo
  (candidato: spill del buffer inline / CString per canonicalize).
  KB-83-1 CHIUSA come falsificazione onesta: il modello nuovo
  (2 vs 4 copie a soglia) va pinnato in WP-84.

## Aperture dichiarate (per NOME — mai chiusure in silenzio)

1. **Delibera peak ×W (p6)**: ora ha TUTTI gli input (scomposizione ESATTA,
   frazione condivisibile ~69%, F15b armato) — mitigazione deliberata =
   main ESENTE in partizione-per-TIPO (KS-MS-84-3); implementazione +
   F15b flip + rimisura peak = prossimo passo (KS-DS-84-3/4).
2. **VW modello nuovo**: soglia della quarta copia non localizzata nel
   codice (solo aritmetica dal raw) — sito da nominare prima di pinnare.
3. **A-PP18** (p7): resta APERTA, calendarizzata PRIMA di ogni
   riconciliazione Δglobal a W>1 (A-PP24).
4. Emendamento lock-cmp PRUNE-ONLY: al giudizio del Concilio WP-85.
