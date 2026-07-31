# MEASURE78_RESULTS.md — WP-78 fase misura (2026-07-31, S-78.1)

**Protocollo**: design78.md (emendato S-78.1.7). Ogni cifra cita run tracciato
(raw in `measure-out/`). Gate sulla build misurata: feature-matrix PASS ·
G-APERTURA-2 PASS · KH78-1 PASS (tutti git 275c518, bin 5fdc971680d5e6a2).
**Binari**: union (gemello NON strumentato) **5fdc971680d5e6a2** — tutte le
cifre footprint/peak; census build **8f4146db** — SOLO contatori (KB-78-5).
Workload: `hello.php` POST/GET sequenziale chiuso (A-BG9), warm-up 10 escluso
solo dalle metriche per-richiesta, picco = intero processo (KG-79.C).
MIMALLOC_PURGE_DELAY=0; vmmap V1 (post-warm-up) / V2 (post-batch); exit
verdict-grade solo con riga join (KL-78-4).

## 1. Tier-0 (pavimento Axum: `--tier0`, no pool, no Vm) — R3

| run | peak (time -l) | vmmap V1 | vmmap V2 |
|---|---|---|---|
| r1 | 10.665.984 | 4288K | 4336K |
| r2 | 10.731.520 | 4384K | 4432K |
| r3 | 10.633.216 | 4240K | 4288K |

Peak medio **10,68MB**, spread 0,92% ≤2% ✓. (Riga join N/A: nessun pool per
costruzione — exit pulita da graceful shutdown.)

## 2. A/B stesso binario, stessa fixture, N=100 (A-BG4/A-BG10) — R3

| braccio | peak | V1 | V2 | exit stats |
|---|---|---|---|---|
| `--axum --workers 1` r1/r2/r3 | 59,82 / 59,69 / 59,70 MB | 17,6/17,5/17,5M | 18,1/18,0/18,0M | VERDICT-GRADE (join) |
| `--cli-server` r1/r2/r3 | 53,99 / 53,99 / 53,99 MB | 13,2/13,3/13,3M | 13,2/13,3/13,3M | UNAVAILABLE-BY-DESIGN (KG-79.B) |

- Spread peak axum 0,22% ✓; cliserver deterministico al byte.
- **Delta A/B: peak +5,7MB (+10,6%), residente steady +4,8M (18,0 vs 13,2)** =
  SOMMA dichiarata di {runtime tokio + canale mpsc + pool/worker +
  registry-once-per-worker + retention} (A-BG10) — NON decomposto oltre in
  questo run.
- **A-BB1 (≤+2% alloc)**: NON GIUDICABILE come confronto A/B — il braccio
  cli-server non è strumentato a contatori (e non ha exit-stats pulite,
  KG-79.B). L'attribuzione per-fase del braccio axum è in §4; il verdetto
  alloc-A/B è deferito a quando il path cli avrà contatori o drain pulito.

## 3. Probe amplificazione RSS(N) vs RSS(2N) (A-BG5, prova regina) — R3+R3

| N | V2 (R3) |
|---|---|
| 100 | 18,1 / 18,0 / 18,0 M |
| 200 | 18,0 / 18,0 / 18,0 M |

**Delta per-richiesta ≈ 0 intero-esatto**: il +0,5M V2−V1 visibile a N=100 è
assestamento post-warm-up che PLATEAUA (identico a N=200), non crescita
lineare. **Nessun leak per-richiesta** — coerente con la chiusura KS-DS-78-4
di S-78.1.5 (RetainSet per-richiesta) e con used_n=0 (§4).

## 4. Census per-fase (build strumentata 8f4146db, `--workers 1`) — R3 + include

Steady-state (righe req≥11, INTERO-ESATTE tra richieste E tra run):

| workload | a: lower+compile | b: vm+run+render | c: shutdown/end | retain_len | live_objs | depth_max |
|---|---|---|---|---|---|---|
| hello.php | 80.476 call / 13.015.960 B | 730 call / 95.627 B | 0 / 0 | 0 | 0 | 1 |
| include_gate.php | 80.513 call / 13.037.032 B | 767 call / 98.333 B | 0 / 0 | 2 | 0 | 1 |

- **Attribuzione: fase a = 99,1% delle call e 99,3% dei byte per richiesta**
  (~13MB di churn alloc per RICOMPILARE prelude+script identici a ogni
  richiesta). Report per-fase ⇒ KB-78-1 soddisfatto; canale isolato ⇒
  KS-AH-78-3 soddisfatto. **Questo è il dato frequenza×taglia (WP-57) che
  sblocca il design della cache Module A-BB6** (post-censimento, come da
  ordine): la leva vale ~80k call/13MB a richiesta sul canale compile.
- Controlli positivi (KG-79.A): contatori visti muoversi ✓; retain_len
  DISCRIMINA (0 hello / 2 include) ✓; depth_max=1 in TUTTI i campioni
  (meccanismo chiuso-sequenziale verificato, KH78-2) ✓; c=0 è un dato reale
  (il teardown libera senza allocare), non un contatore cieco — il canale
  c è coperto dal controllo positivo dei contatori globali.
- used_n (live_objs) = 0 post-request_end su ogni richiesta (KS-DS-78-2 ✓).

## 5. Linearità in W (A-DL2/KL-78-2) — ≥100 req/worker, R3

| W | N req | V2 (R3) | peak (R3) |
|---|---|---|---|
| 1 | 100 | 18,1 / 18,0 / 18,0 M | 59,8 / 59,7 / 59,7 MB |
| 4 | 400 | 55,9 / 55,1 / 55,0 M | 114,0 / 117,4 / 121,0 MB |
| 10 | 1000 | 134,1 / 134,1 / 133,7 M | 186,3 / 212,0 / 198,8 MB |

- Fit base + W·k: k = 12,43 (W1→4) / 13,12 (W4→10) / 12,89 (W1→10) M/worker;
  W=4 misurato 55,3M vs 56,7M predetto dal fit estremi (−2,5%).
  **Lineare in prima approssimazione — KL-78-2 NON scatta.**
- k ≈ **12,4–13,1M per worker**: dominante atteso = copia Registry per worker
  (Leijen c.4) + stack + stato worker. Leva futura (non commissionata):
  registry condivisa read-only tra worker.
- ⚠️ Spread del PICCO a W=10: 186–212MB (12%) — oltre il 2%: NON mediato
  (KB-78-4); attribuito ai transitori concorrenti di primo-compile dei 10
  worker; il residente V2 resta a spread 0,3%. Da indagare solo se il picco
  multi-worker diventa metrica di verdetto.

## 6. R-G4 (CPU attribution)

NON commissionato — non eseguito (design78 §Ordine punto 6).

## Nota di identità (incidente metodologico registrato)

Il primo tentativo Tier-0 è girato su un binario DEFAULT (cli-only,
c21c29591a898621) lasciato sul path dal build workspace: l'hash stampato dal
driver ha smascherato il run (server mai partito). Rimedio: rebuild union +
verifica hash PRIMA del batch — conferma che l'identità-del-binario nel
driver non è decorativa (KS-AH-78-1).
