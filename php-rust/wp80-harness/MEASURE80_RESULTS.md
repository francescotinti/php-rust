# MEASURE80_RESULTS.md — misura di riferimento R≥3 (S-80.0.6, Council WP-81 p6)

**Sostituisce design79 §1** (ADVISORY per A-BG17/KG-81-1: cifra senza raw nel
repo). Ogni cifra qui sotto cita il raw COMMITTATO e l'identità ENFORCED.

## Identità (KS-AH-80-1 + KG-81-2, per OGNI run)

- git tree **6910767** == `git=` del feature-matrix.log (driver ENFORCE, match
  verificato run-per-run; archivio matrix: `wp78-harness/matrix-archive/
  feature-matrix.6910767.*.log` + copia per-run `<run>.matrix`).
- binario census **5c9c6eec481d5133** == `bin[census]` (entrambi gli arm).
- Battery integrale PASS sul tree 1305327 (stesse sorgenti crates di 6910767 —
  i commit successivi toccano solo harness/evidence): matrix quintetto ·
  run-gate union 5260f50b · census-twin · concurrent overlap≥2 · worker-panic
  3 fasi+near-miss · stdout-tandem 6/6 · capture-order · doc-purge · DR-1.
- Corpus 1418 per NOME IDENTICO + refl 290 IDENTICO + workspace 1652/0 a
  phpr ef90cb19 (`wp80-harness/evidence/`).

## Protocollo

measure78.sh (ENFORCE: righe==110, depth_max≤1, **inflight_max≤1** — il nuovo
osservabile closed-sequential A-TH9 —, **a3_trip==0**, W=1, boot-probe fuori
canale). R=3 per (arm, fixture); WARMUP=10, MEASURED=100; steady = righe
11-110. **Spread R=3 = 0,0%** su ogni canale (contatori deterministici:
run byte-identiche). Cifre = **churn LORDO, upper bound** (gross=1 in-band,
A-DL10). Raw: `wp78-harness/measure-out/census.80.<fx>.r{1,2,3}.*` e
`censuscli.80.<fx>.r{1,2,3}.*`.

## Arm axum (`census:`) — medie steady per richiesta

| fixture | a_calls | a_bytes | a1_calls | a1_bytes | a2_calls | a2_bytes | a3 | b_calls | b_bytes | c_calls | c_bytes | resid_c | resid_B | retain |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| bare | 80.450 | 12.998.624 | 74.288 | 10.825.612 | 6.162 | 2.173.012 | 0 | 721 | 95.358 | 0 | 0 | 41,1 | 43.368 | 0 |
| hello | 80.476 | 13.015.960 | 74.288 | 10.825.612 | 6.188 | 2.190.348 | 0 | 730 | 95.627 | 0 | 0 | 44,1 | 43.463 | 0 |
| include_gate | 80.513 | 13.037.032 | 74.288 | 10.825.612 | 6.225 | 2.211.420 | 0 | 767 | 98.333 | 0 | 0 | 43,1 | 43.896 | 2 |
| include_heavy | 81.012 | 13.391.378 | 74.288 | 10.825.612 | 6.724 | 2.565.766 | 0 | 27.982 | 1.867.510 | 14 | 1.568 | 44,1 | 45.156 | 5 |

- **a1 (prelude) = 74.288 call / 10.825.612 B, IDENTICO su ogni fixture e ogni
  run** — la costante che la unit cache TL estesa al MAIN salta su HIT.
- a3 = 0 steady su tutte (unit-cache HIT sugli include ✓, discriminatore);
  a3>0 visto sul COLD delle prime righe (controllo positivo in-cargo).
- c si muove in vivo su include_heavy (14/1.568) — canale non cieco (KB-80-5).
- depth_max=1 E inflight_max=1 su TUTTE le run: il claim closed-sequential è
  ora VERDICT-GRADE (KH81-1 soddisfatto — dec-after-send).
- live_objs=0 su ogni riga (KS-DS-78-2 ✓). retain_len atteso: 0/0/2/5 ✓.

## Arm cli (`census-cli:`) — finestra engine totale, medie steady

| fixture | total_calls | total_bytes | a1_calls | a1_bytes | a3 |
|---|---|---|---|---|---|
| bare | 81.579 | 13.124.591 | 74.288 | 10.825.612 | 0 |
| hello | 81.613 | 13.142.102 | 74.288 | 10.825.612 | 0 |
| include_gate | 81.687 | 13.165.901 | 74.288 | 10.825.612 | 0 |
| include_heavy | 109.415 | 15.290.995 | 74.288 | 10.825.612 | 0 |

⚠️ **Asimmetria superglobali dichiarata (A-DS10/KS-DS-81-3)**: il braccio cli
semina `set_web_request` REALE, l'axum no (A-BB4 deferito) — confronti
censuscli↔axum solo su totale/a1/a3 e SOLO con questa dichiarazione. Il
delta hello cli−axum (81.613 vs ~81.250 = a+b+c+resid) ≈ +0,4% è coerente
col seeding web; non è un verdetto A-BB1 (che esige la decomposizione b+c).

## Idle window (A-AH13/A-DL5/A-BG20/KL-81-3 — churn-only)

Probe self-cost: axum 38 call/41.524 B; cli 49 call/9.639 B. **Drift idle
VERO = 0 call / 0 B su entrambi gli arm, sia a 10s sia a 60s**
(`*.80.idle60.*`): la finestra tra i probe è identica al solo self-cost.
Nessun rumore cross-thread da assorbire nelle diff di fase. 🔵 Nota di
metodo: il driver S-80.0.2 aveva reintrodotto un off-by-one nel SOMMARIO
idle (il boot-probe è census-global → 4 righe probe; l'awk head-anchored
leggeva boot→p1 = warm-up come "self-cost"): fixato con tail-3 nello stesso
commit di questi risultati; i RAW erano corretti, solo il sommario derivato
era errato — ricomputato dai medesimi raw.

## Floor non-compile ex-ante (A-BB16/KB-81-2 — dichiarato PRIMA della leva)

Su HIT la fase a residua = probe main (canonicalize+stat+fp del source) +
lookup + bookkeeping. Componenti misurabili OGGI: probe HIT delle unit
include = b(include_gate)−b(hello) = **37 call/2 unit ≈ 18,5 call/unit**;
bookkeeping per-richiesta ≈ resid 41-44 call. **Floor dichiarato: ≤ 200
call/req** (stima ~50-100; bound con margine ×2). La soglia §10
(a_calls HIT < 4.000) sta 20× sopra il floor ⇒ predizione falsificabile,
non-vacua (KB-81-2 soddisfatto). La quota che la leva DEVE rimuovere:
a1+a2 = 80.476 call / 13,0MB su hello (l'82%+17% = 100% della fase a).

## KS-AH-80-4 ridefinita (KS-SK-81-4 — UNA quantità)

**v2**: `a_calls` steady su HIT deve calare ≥90% vs baseline build-adiacente
(hello: da 80.476 a <8.048) — a1⊆a mai più sommati. Predizione puntuale
§10 invariata: <4.000. Vincolo separato: `a1_calls`==0 a caldo.

## Predizioni a caldo aggiunte (KG-81-3/A-BG21)

- **resid**: post-leva ≈ invariato (44±15 call / 43,5±5KB su hello — il
  fs::read del main resta nel resid). Un leak da leva si nasconderebbe QUI:
  canale dichiarato con predizione, verdetto A-BB6 la cita (KG-81-3).
  Precisazione A-BG23 (Concilio WP-82): la predizione è sulla MEDIA steady;
  la riga req=11 è un residuo di transiente (resid=157 su hello, regime da
  req=12) — dichiarato nella baseline, il picco per-riga non è nel bound.
  Le tolleranze ±15/±5KB/±5% sono CONCESSIONI dichiarate, non statistica
  (spread 0,0% = determinismo, zero informazione di varianza — A-BG25): la
  prima campagna post-leva ri-deriva lo spread osservato (KG-82-2).
- **b**: invariato ±5% (hello 730/95.627; include_heavy 27.982/1.867.510 —
  cifra CORRETTA in chiusura S-80.0: la prima trascrizione diceva 2.798, un
  ×10 da output troncato, trovato dal ricomputo indipendente del Concilio
  WP-82/A-BG22 sui raw) — la leva non tocca il run.
- **a2**: →0 su HIT (assorbita nel salto a1+a2).
