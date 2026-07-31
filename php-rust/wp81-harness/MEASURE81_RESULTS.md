# MEASURE81_RESULTS.md — A/B churn verdict of the A-BB6 lever (S-81.0 step 7)

**Ogni cifra qui viene dal ricomputo SCRIPTATO dei raw** (KG-82-1:
`wp81-harness/verdict81.sh` → `verdict81.out`, committati entrambi) — nessuna
trascrizione a mano. Cifre = churn LORDO, upper bound (gross=1 in-band).

## Identità (ENFORCED dal driver su OGNI run)

- git tree **3f32c16** == `git=` del feature-matrix.log; SESTETTO: union
  9f9f8d92ff969729 · census **12a8777c8c38fdc4** · census-axum-only 13c429be
  · axum-only e2183043 · default cee8c63e · **mem-census d5bba760069639e0**
  (KS-AH-82-4: riga matrix + lane CI same-commit).
- **driver_sha=436b453ef0980c03** (A-AH21: measure78.sh+gate-feature-matrix
  fingerprint) nell'header di OGNI run e in coda a ogni raw `.log`; porcelain
  esteso agli script harness — la campagna abortita a metà dal proprio
  tripwire (verdict81.sh scritto mid-campaign ⇒ 13 run rifiutate) è la PROVA
  che il check morde; i raw misti sono stati rimossi DICHIARANDOLO
  (commit 3f32c16) e l'intera campagna è stata rifatta a UN rev.
- Equivalenza sorgenti tra i commit harness-only della sessione: COMANDATA
  (`git diff --stat <rev1> <rev2> -- crates/` vuoto, KH82-2) — verificata
  empiricamente dagli hash sestetto IDENTICI su 552e6fb→2dc11eb→3f32c16.
- Baseline pre-leva: `wp80-harness/MEASURE80_RESULTS.md` (git 6910767,
  census 5c9c6eec). Il confronto è tra campagne, non build-adiacente
  stessa-sera per il churn: i contatori sono DETERMINISTICI su entrambe le
  campagne (spread 0,00% osservato IN-campagna qui, non ereditato —
  A-SK18/KS-SK-82-4/KG-82-2).

## Protocollo

measure81-campaign.sh = R=3 × (2 arm × 4 fixture) + idle60 × 2 arm, 26 run,
tutte ENFORCE (righe==110, depth≤1, inflight≤1, a3_trip==0, W=1, boot-probe
fuori canale, pin idle ==4 righe con self-test A-SK14). rc=0, zero FAIL.

## Verdetto churn (da verdict81.out — TUTTE le predizioni §10 PASS)

| arm | fixture | a_calls r1/r2/r3 | spread% | a1 | a3 | b_calls | resid_c | retain |
|---|---|---|---|---|---|---|---|---|
| census | hello | 2.0/2.0/2.0 | 0.00 | 0.0 | 0.0 | 731.0 | 44.1 | 1.0 |
| census | include_gate | 2.0/2.0/2.0 | 0.00 | 0.0 | 0.0 | 767.0 | 43.1 | 3.0 |
| census | include_heavy | 2.0/2.0/2.0 | 0.00 | 0.0 | 0.0 | 27982.0 | 44.1 | 6.0 |
| census | bare | 2.0/2.0/2.0 | 0.00 | 0.0 | 0.0 | 722.0 | 41.1 | 1.0 |
| censuscli | hello | 1140.0 ×3 | 0.00 | 0.0 | 0.0 | — | — | — |
| censuscli | include_gate | 1176.0 ×3 | 0.00 | 0.0 | 0.0 | — | — | — |
| censuscli | include_heavy | 28405.0 ×3 | 0.00 | 0.0 | 0.0 | — | — | — |
| censuscli | bare | 1132.0 ×3 | 0.00 | 0.0 | 0.0 | — | — | — |

- **hello a_calls su HIT: 2 call/req** (baseline 80.476 → **−99,997%**;
  KS-AH-80-4 v2 «≥90%» superata di 3 ordini; puntuale <4.000 ✓).
- **Floor ex-post (A-BB23): a_calls(bare,HIT)=2 ≤ 200** — il floor ex-ante
  NON è bucato (KB-82-3); delta hello−bare su HIT = 0 ✓; nessuna
  decomposizione dovuta (A-BB24: 2 ≪ 5×floor). Il probe
  (canonicalize+stat+hash) è quasi alloc-invisibile — che NON dice nulla
  sulla CPU (A-BB25/KB-82-4: bound su ALLOCAZIONI; la CPU si giudica solo
  con la slope due-N, residuo dichiarato sotto).
- a1==0 e a3==0 steady (niente lower_prelude, include ancora HIT); a2
  assorbita nel salto (la unit MAIN contiene il prelude compilato).
- **b invariato**: hello 731 vs 730 (+0,14%), include_heavy 27.982 ==
  baseline ESATTO (la cifra corretta da A-BG22 — il run non è toccato).
- **resid invariato**: 44,1 call / 43.463 B == baseline ESATTO (media
  steady; nessun leak nascosto nel canale dichiarato, KG-81-3).
- retain_len = park-EVENTI: bare/hello 1 (main), include_gate 3 (2+main),
  include_heavy 6 (5+main) — pin A-DS8/F6 per NOME.
- cli arm (asimmetria superglobali DICHIARATA, A-DS10/KS-DS-81-3; confronti
  solo totale/a1/a3): hello 81.613 → **1.140** (−98,6%); include_heavy
  109.415 → 28.405 (resta la quota include-compile run-side b, come da
  contratto: la leva rimuove a1+a2 del MAIN).

## Idle window (churn-only)

Post-leva: **drift idle = 0 call / 0 B su entrambi gli arm a 60s**
(finestra == self-cost esatto: axum 38/41.524, cli 49/9.639). KL-82-1: la
frase precisa è «0 allocazioni Rust-allocator, tutti i thread» — canali
CIECHI per costruzione: (i) dealloc (il contatore non le incrementa),
(ii) malloc FFI (zlib/gd/tidy fuori dal GlobalAlloc), (iii) arene/metadata
mimalloc (mmap diretto; senza purge-thread: con PURGE_DELAY=0 e zero
traffico non scatta nulla), (iv) page-dirtying di memoria già allocata.
La residenza idle esige il twin union + vmmap + floor (KL-81-3).

## Fixture e gate (tree 3f32c16, battery COMPLETA PASS)

F1-F13 + F-probe + F-oneshot(3 denti) + F5/F8b/F8c(contatori) TUTTE VERDI
(gate-lever-fixtures{,2}.sh); corpus 1418 per NOME IDENTICO + refl 290
IDENTICO + workspace 0 fail; run-gate union + census-twin (full-body vs
oracolo sul binario census = battery CON main-HIT) + concurrent +
worker-panic + stdout-tandem + capture-order + doc-purge + DR-1 +
lever-pins. F4: deviazione DICHIARATA (nessun path main-impuro sul tree —
Concilio WP-83 giudica). Battery-su-HIT col pin ESPLICITO main_hit
per-richiesta (KS-SK-82-3): il claim formale resta «path misto» finché il
pin non è contato — residuo dichiarato.

## RESIDUI DICHIARATI (A/B NON completo su questi assi — mai chiusi in silenzio)

1. **Footprint twin** (V2 N vs 2N con floor vmmap 0,1MB + peak W=num_cpus):
   NON misurato in S-81.0 — il verdetto memoria RESIDENTE della leva è
   APERTO (il churn sopra non lo sostituisce, KH80-3/KL-80-1).
2. **CPU slope due-N** (100/200 req + costo scan supersede A-DS9, con
   risoluzione ex-ante KL-82-3): NON misurata — ogni claim CPU della leva
   resta ADVISORY d'ufficio.
3. **Retained ×W**: walker+controllo positivo ESISTONO (KL-82-2 onorata),
   ma la cifra retained su run reale mem-census + budget ×W (KS-MS-82-2)
   NON è stata prodotta — budget NULLO finché non misurata.
4. **Fixture autoload-run** (KB-81-3/KB-82-5): assente — «il HIT salta a3»
   resta ADVISORY.
5. Tolleranze: spread 0,00% = determinismo, NON base statistica (A-BG25);
   le bande usate (±15/±5KB/±5%) sono concessioni dichiarate della
   baseline WP-80.
