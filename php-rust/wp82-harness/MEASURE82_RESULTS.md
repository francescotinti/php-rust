# MEASURE82_RESULTS.md — misure RESIDUE dell'A/B leva A-BB6 (S-82.0 p7)

**Cifre di misura** = verdetto SCRIPTATO dei raw (`wp82-harness/verdict82.sh`
→ `verdict82.out`, entrambi committati; fail-closed A-SK19). **Cifre
derivate** = righe `[derivata]` di verdict82.out (A-BG26). Documento
vincolato al corpus committato da `gate-measure-cifre.sh` (KG-83-3).

## Identità

- Campagna-4 a git **e2990b3**, matrix stesso-chain senza commit intermedi;
  binari nel matrix log per riga (union/census/mem-census); driver_sha con
  campaign script incluso (A-AH30) nell'header di OGNI run measure78.
- **Contabilità VOID (A-BG29, ricomputata dai manifest — run e FILE
  separati)**: 3 quarantene (mai `rm` — KS-AH-83-2), per-manifest:
  campaign-1 (worktree-subdir): 15 run measure78 / 81 file; campaign-2
  (`--locked` vs dev-dep p5): 15 run / 81 file; campaign-3 (chain-HEAD):
  15 refusal-summary da 287 B ciascuno (tripwire KG-81-2, zero dati) / 23
  file. Totale: **45 slot-run VOID, 185 file** in
  `wp82-harness/evidence/void/*/MANIFEST.txt`. ⚠️ L'header v1 di questo
  documento diceva «void_runs=162+23» e «46 REFUSAL» — cifre MAI
  ricomputate, BOCCIATE dal Concilio WP-84 (Gregg): questa è la forma
  riconciliata.
- Fase R (retained): supplemento `phaseR-supplement.sh` (campagna-4 aveva lo
  strumento NON ARMATO — `PHPR_MEM_CENSUS` è l'interruttore, la feature è
  solo il compilato; e il dump vive nel teardown di `run_module_with_hir` ⇒
  arm CLI-SERVER, il gemello A/B onesto): stesso `mem_hash` di campagna-4
  ENFORCE (b620d64c89abb584), full-body vs oracolo PASS PRIMA della cifra
  (KS-AH-83-1).

## Verdetti (da verdict82.out)

- **VF footprint twin — PASS**: coppia 100/200 delta −0,133 MB (NEGATIVO:
  shrink/purge-noise alla risoluzione vmmap, non crescita) ⇒ escalation
  A-DL19a alla coppia SCALATA dai raw fase C: **|V2(2000)−V2(1000)| =
  0,000 MB ≤ 0,1 MB floor** ⇒ residenza per-richiesta ≤ 0,1KB/req —
  **NESSUN leak residente sul path HIT** (V2 ≈ 20,2/20,1 MB stabili).
- **VP peak W=10 — DECLARED-WORSE (voce APERTA Concilio WP-84, KL-80-2 mai
  in silenzio)**: lever 232,0 MB (232/232/232, spread 0%) vs base pre-leva
  197,6 MB (190/202/201) = **+34,4 MB (+17,4% > banda 6,5%)** — coerente
  col costo retained ×W del main cached: [derivata] 3,44 MB/worker × 10.
  L'accettazione (costo ×W vs 13 MB/req di churn risparmiato) è decisione
  di Concilio, non di script.
- **VC CPU slope — NULL (KB-83-3, mai ADVISORY)**: spread delle slope
  970 µs/req ≥ banda/3 (125,7) ⇒ nessun claim CPU; N raddoppia
  (2000/4000) in WP-83. Medie GREZZE a verbale: lever 150,0 µs/req
  (120/170/160), base 7043,3 µs/req (6520/7490/7120) — il segnale è
  enorme ma il claim aspetta la risoluzione ex-ante.
- **VH battery-su-HIT TWIN-PAIR — PASS (KS-SK-83-3/KS-PP-83-2 CHIUSE)**:
  run B (union+uclog) segmento battery per OGNI path: probe==20, hit==20,
  **put==0** (path puro HIT, mai misto); run A (census witness): steady
  a_calls==2,0 ≤ 4 su tutte le 4 fixture, steady_n=30, binario nominato
  nei raw. Il residuo «path misto» di WP-81 è CHIUSO.
- **VR retained — PASS (KL-83-1 soddisfabile)**: riga unitcache mem-census
  con full-body PASS previo: **rw_bytes=14250027 [FLOOR cache-as-owner] +
  rw_main_net=7407810 [net-at-lower misurato]**, uclog==0 (KS-PP-83-1),
  main_evicted==0 DICHIARATO (KS-DS-83-1), entries=11,
  probe/hit/put=33/29/4 (4 main put una volta, 29 HIT). **Budget ×W
  [derivata]: (rw_bytes+rw_main_net) = 20,65 MB × W** (W=1 misurato;
  thread-local ⇒ ×W per costruzione) — coerente con il VP: +3,44 MB/worker
  osservati a W=10 sul solo working-set hello (1 main vs 4 qui).
- **VA autoload-run — PASS (KB-82-5 chiusa in forma PIÙ forte)**: riga 1
  (MISS) a3=1112 call — l'autoload SPARA e compila via a3 (controllo
  positivo); a regime **a3==0 ESATTO**: l'include dell'autoload runtime è
  esso stesso un cache-HIT — «il HIT salta a3» vale anche per l'autoload
  runtime; la quota RUN vive in b: [derivata] 787−731 = **+56 call/req**
  vs hello (register+closure+link dell'include-HIT).
- **Scan supersede (Bak, in-cargo `a_ds9_...bound`, campaign-only)**: dal
  raw COMMITTATO `measure-out/m82.scanbound.raw` (A-BG30 — la run v1 era
  citata senza raw, bocciata): put(K=8)=308 ns, put(K=64)=503 ns ⇒
  **coefficiente 3 ns/key** — bound 1 µs/key con margine ~300×
  (bound-check, mai win-claim: WP-38).

## Aperture dichiarate (per NOME — mai chiusure in silenzio)

1. **VP peak ×W**: voce Concilio WP-84 (accettazione del costo retained ×W
   o mitigazione — registry condivisa read-only è già nel backlog WP-83+).
2. **VC slope**: N=2000/4000 in WP-83 (KB-83-3).
3. ORM/hk perf build-adiacente: NON misurati (nessun claim fatto —
   KS-AH-83-4 non innescata).
