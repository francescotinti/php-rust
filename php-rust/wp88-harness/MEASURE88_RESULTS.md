# MEASURE88_RESULTS.md — misure S-88.0 nelle FORME ordinate dal Concilio WP-89

Campagna measure88 (§Sintesi WP-89 p5): riconciliazione slope A-BB55≡A-DL42
(W∈{4, 8, 12, 16}, mode-census, mi_arena in-band, banda SOLO su b) + A-BB56
warm-both-then-pair/stagger con predizioni ex-ante + A-DS45 fase uc_log
armata. Verdetto macchina: `wp88-harness/verdict88.a1.g1.out` (VERDICT88
PASS, attempt=1 PULITO al primo colpo, fail-closed, per-attempt e
per-generazione). Cifre BYTES-FIRST con companion VERIFICATO; metrica
SEMPRE nominata (KB-88-2); mai max−min (KB-88-3).

## Identità

- git campagna: 202f8b1 · **attempt=1** (nessun VOID: i denti v5/v6 non
  hanno trovato nulla da mordere in-run; ledger campagna APPEND-only con
  esito del verdetto in-band, A-AH49/KG-89-1)
- battery-88pre: **PASS (16/16 CONTATO) a 0b83f2b** — 16° gate NUOVO
  `axum-tests` (A-PP41: suite axum-server ESEGUITA nel perimetro, a_pp38
  con osservabile publish A-PP45 pinnato per NOME) + F16b (A-DS42:
  a_ds38 ARMATO, ≥2 main_evicted su file); stamp LEDGERATO committed
  (202f8b1) con snapshot matrix per NOME+sha256 (A-AH48); campagna
  consumata via **battery-equivalence --same-rev v6** (A-SK50: PRIMA
  campagna con allowlist della finestra evidence-only + ledger-prefix +
  toolchain -Vv in-repo — KS-SK-89-1/KS-AH-89-1 armate e verdi)
- binario mem-census b2fe7a43e62166da ENFORCED contro matrix + noprobe;
  identity contract v2 GIUDICATO (A-BG47/KG-89-2: server_exit==0,
  srv_pid riconciliato con OGNI riga pid=, count identity==1); thr-set
  == {0..W-1} ESATTO su ogni raw (A-PP42/KS-PP-89-2); port-owner assert
  su ogni run (A-PP43/KS-PP-89-4); MIMALLOC_PURGE_DELAY=0 con READ-BACK
  in-band (A-DL41: `tag=mi_option purge_delay val=0` su ogni raw — il
  controllo positivo dell'ordinale morde da solo);
  PHPR_MI_COLLECT_EXIT=1. DECLARED DEVIATION invariata: il collect gira
  sull'heap CONDIVISO v3 all'atexit (A-DL39 = design). Canale
  $PHPR_MI_STATS NON armato e NON-corpus (A-DL40/KL-89-4).

## Verdetti (da verdict88.a1.g1.out)

- **VSLOPE-HI — A-BB55≡A-DL42, la riconciliazione È ARRIVATA e la banda
  NON torna**: mode-census per W (KB-89-2, min-of-R declassato ADVISORY
  ovunque: ≥2 modi byte-distinti a ogni W). Modi DOMINANTI su
  metric=committed_postcollect_win0:
  W=4 148.307.968 B = 141,44 MiB [derivata: companion /1048576] (×2 su R=5)
  W=8 230.948.864 B = 220,25 MiB [derivata: companion /1048576] (×2)
  W=12 324.403.200 B = 309,38 MiB [derivata: companion /1048576] (min dei 5 modi distinti)
  W=16 399.769.600 B = 381,25 MiB [derivata: companion /1048576] (×2).
  **residue_dominant = 0 B a OGNI W**: ogni modo dominante è multiplo
  ESATTO di 65.536 B (named-constant: granulo d'arena) — la
  quantizzazione predetta da Bak/Leijen è confermata al byte.
  **Cross-check A-BB55 riuscito**: `tag=mi_arena key=committed current`
  == `tag=mi_proc commit` AL BYTE su ogni run (il committed di processo
  è INTERAMENTE arena mimalloc; arena_count=1). Delta dominanti:
  W4→8 82.640.896 B = 78,81 MiB, per worker 20.660.224 B = 19,70 MiB [derivata: Δ/4]
  W8→12 93.454.336 B = 89,12 MiB, per worker 23.363.584 B = 22,28 MiB [derivata: Δ/4]
  W12→16 75.366.400 B = 71,88 MiB, per worker 18.841.600 B = 17,97 MiB [derivata: Δ/4]
  — MONOTONI (KB-89-1 soddisfatta). **b (LSQ sui modi dominanti, W∈{4, 8, 12, 16}) =
  21.195.981 B = 20,21 MiB per worker — NAMED-DEVIATION**: fuori dalla
  banda KL-85-2 3.605.572 B ±5% ANCHE nel segmento alto e ANCHE
  sull'estimatore ordinato dal Concilio (banda confrontata SOLO con b,
  KL-89-2); b_min (min-of-R, ADVISORY) = 20.399.718 B = 19,45 MiB.
  Slack piatto (76.576–502.368 B): la ritenzione nei bin resta esclusa.
  **Delibera di lettura**: la granularità d'arena NON spiega la
  deviazione (residuo 0, b stabile — 21.195.981 B = 20,21 MiB — dal basso all'alto regime) —
  il costo marginale per-worker di QUESTO protocollo (100·W hello,
  worker std::thread) è 21.195.981 B = 20,21 MiB per worker di committed; la banda KL-85-2
  (derivata a W=10 steady su UN fixture, altra metrica di forma) NON è
  una banda di questo protocollo. Attribuzione del b = apertura WP-90.
- **VWARM — A-BB56, le predizioni hanno discriminato ma NON come
  atteso**: calibrazioni byte-riprodotte per la TERZA campagna
  consecutiva: pad87a net = pad87b net = **7.801.102 B** (r1==r2 al
  byte), floor_inc = **1.161.206 B** entrambe [metric=net-at-lower,
  net_window=process-counters, W=1 sequenziale].
  (a) **concbase — ⚠️ INVERSIONE vs measure87**: padA net = 15.602.518 B
  = calA+calB+314 B (2/2 ESATTO — la firma-inghiottimento PULITA che in
  m87 apparteneva a padB); il surplus sta ora su padB:
  r1 +1.560.464 B = 1,49 MiB [derivata: dB verdetto, companion /1048576]
  r2 +3.176.029 B = 3,03 MiB [derivata: dB verdetto, companion /1048576]
  — INSTABILE fra i run. Il surplus m87-padA
  (3.146.416 B = 3,00 MiB [derivata: media m87]) NON si è
  riprodotto: non è una proprietà
  del LATO ma della finestra aperta al momento del rumore/first-touch —
  attaccamento di TIMING. Si scioglie solo con A-BB50 (net per-thread).
  (b) **concwarm — il floor crolla come da struttura**: floor_inc
  1.161.206 B → **164.368 B** (2/2, ENTRAMBI i lati): il floor
  per-thread condiviso è stato pagato dal warm hello (ord=1) — conferma
  indipendente della natura strutturale per-thread del floor; net padA
  4.092.062 B (2/2 al byte), padB 819.340 / 893.036 B. Il label
  meccanico del discriminatore («PER-REQUEST») è **VOID-di-significato**:
  la formula era tarata sulla firma m87 (surplus su A in base) che non
  si è riprodotta; in regime di floor-collapse dA = net−(calA+calB) non
  misura più il surplus. Ri-giudizio al Concilio WP-90.
  (c) **concstag (20 ms) — CONTROLLO POSITIVO PERFETTO**:
  spans=NO-OVERLAP e net == cal **AL BYTE** su entrambi i lati
  (7.801.102 B, 2/2; floor_inc 1.161.206 B idem): con finestre
  disgiunte il process-counter riproduce la calibrazione ESATTA —
  **la firma-inghiottimento è PURAMENTE artefatto di overlap**, come
  predetto da P-STAG in forma ancora più forte (zero swallow, non solo
  «minore»). TUTTI i net concorrenti restano VOID come cifre per-thread
  (KB-88-1) fino ad A-BB50 attuato.
- **VUCLOG — A-DS45 nella forma RIQUALIFICATA**: log di PRODUZIONE W=1
  con supersede-with-putord = 2, **main_evicted = 0** (tripwire
  KS-DS-84-4 rispettato e giudicato come dente esplicito), pair-guard
  consumato su log NON-selftest. La lettera «≥1 coppia main_evicted in
  produzione» è REFUTATA-A-CODICE (dopo la partizione A-MS24 è
  strutturalmente impossibile su binario sano — il suo scatto VOIDA); il
  positivo ≥1-coppia vive in F16b (battery, ARMATO, ≥2 main_evicted su
  file). Ri-giudizio al Concilio WP-90.

## Aperture dichiarate (per NOME)

1. **Attribuzione di b**: ~21.195.981 B = 20,21 MiB per worker di
   committed marginale su questo protocollo — coerente in basso
   (m87 W1..4: 25.880.166 B = 24,68 MiB) e alto regime, granularità
   esclusa (residuo 0): il termine è REALE e non attribuito (candidati:
   theap/tcache per-thread, pagine arena per-worker mai restituite a
   purge_delay=0). Banda KL-85-2 da ri-derivare o ritirare (WP-90).
2. **Surplus timing-attached**: il surplus concorrente salta di lato fra
   campagne (m87: padA +3.146.416 B = 3,00 MiB [derivata: media m87];
   m88 padB r1 +1.560.464 B = 1,49 MiB [derivata: companion /1048576];
   m88 padB r2 +3.176.029 B = 3,03 MiB [derivata: companion /1048576])
   ed è instabile fra run — attribuzione impossibile con finestra di
   processo; si scioglie con A-BB50 (design87).
3. **Discriminatore VWARM**: formula da ri-tarare (il regime
   floor-collapse invalida dA come misura del surplus) — Concilio WP-90.
4. **A-DS45 lettera vs codice**: riqualifica supersede-lane da
   ratificare al Concilio WP-90 (refutazione della refutazione, seconda
   della sessione dopo new-pre-decl p1).
5. **A-BB50/A-DL39**: design (design87.md, integrato A-DL43); ogni
   canary concorrente resta vietato come cifra per-thread.
6. **A-MS27 / A-PP18 / A-PP27 / A-AH38**: invariati (backlog).
