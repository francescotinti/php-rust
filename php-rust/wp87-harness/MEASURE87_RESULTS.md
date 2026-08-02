# MEASURE87_RESULTS.md — misure S-87.0 nelle FORME ordinate dal Concilio WP-88

Campagna measure87 (§Sintesi WP-88 p5): metà fisica A-DL38≡A-BB52 come
slope di committed steady-state + bracci candidati per NOME + coppia
disgiunta A-BB53. Verdetto macchina: `wp87-harness/verdict87.out`
(VERDICT87 PASS, fail-closed, blocchi collect-then-emit con FLAG di
pulizia a monte A-SK47/KS-SK-88-2). Cifre BYTES-FIRST con companion
VERIFICATO; metrica SEMPRE nominata (KB-88-2); mai max−min (KB-88-3).

## Identità

- git campagna: 38961f0 · **attempt=3** (a1 VOID: server ORFANO — il
  wrapper /usr/bin/time era il $!, il TERM uccideva il wrapper e il primo
  server restava sulla porta; VERDICT87 FAIL(67) agli atti; a2 VOID: il
  dente anti-orphan nuovo ha morso sul PROPRIO wrapper via pgrep -f;
  entrambe le generazioni committate sotto i LORO nomi, KG-88-1; ledger
  campagna APPEND-only con la storia completa, A-BG43)
- battery-87pre: **PASS (15/15 CONTATO) a 6d9a80f**, stamp LEDGERATO
  committed (a9e18fc), matrix COMMITTED, doctest VmGate CONTATI ==2
  (A-MS37); campagna consumata via **battery-equivalence --same-rev**
  (A-SK46: prima campagna della serie con la precondizione verificata da
  una MACCHINA — anchored PASS, sha256(OUT), stamp 4 campi, matrix
  committed, toolchain A-AH44; KS-SK-88-1 chiusa)
- binario mem-census 2a15f2085d4dcdf8 ENFORCED contro matrix + noprobe
  (KH86-1/A-TH37); mappa dispatch per-thread ESATTA su ogni raw (A-PP37);
  seq=/epoch= in-band su ogni run (A-BG44); MIMALLOC_PURGE_DELAY=0;
  PHPR_MI_COLLECT_EXIT=1 (snapshot win=0 POST mi_collect(true)).
  DECLARED DEVIATION: il collect gira sull'heap CONDIVISO v3 all'atexit,
  non per-theap (export per-theap = design A-DL39).

## Verdetti (da verdict87.out)

- **VSLOPE — A-DL38≡A-BB52 NAMED-DEVIATION**: min-of-R su
  metric=committed_postcollect_win0: W=1 68.681.728 B = 65,50 MiB · W=2
  103.153.664 B = 98,38 MiB · W=3 116.785.152 B = 111,38 MiB · W=4
  150.405.120 B = 143,44 MiB. Delta consecutivi: 34.471.936 B = 32,88 MiB
  · 13.631.488 B = 13,00 MiB · 33.619.968 B = 32,06 MiB. **Slope LSQ per worker =
  25.880.166 B = 24,68 MiB** — FUORI dalla banda KL-85-2
  3.605.572 B ±5% (protocollo PULITO: la fisica è la risposta). La
  derivazione KL-85-2 veniva da steady W=10 N=100 sequenziale su UN
  fixture; qui W∈{1..4} con 100 req/worker: il committed di processo a
  questi W è dominato da termini NON per-worker-costanti (segmenti arena
  interi, granularità di commit) — il Δ non è lineare (13,00 vs 32,88
  MiB). Slack (committed−used) min-of-R: 76.576 B (W=1) · 68.416 B (W=2)
  · 76.448 B (W=3) · 76.576 B (W=4) [fonte: verdict87.out] — lo slack
  post-collect è PIATTO: la chiusura contata-vs-fisica NON si gioca sullo
  slack ma sulla granularità di commit dell'arena. Companion
  metric=peak_memory_footprint (min-of-R): 38.535.672 · 54.936.104 ·
  78.463.528 · 91.980.328 B — **DECLARED-DEVIATION (A-SK52-forma,
  S-88.0)**: queste righe (e le omologhe `peak_memory_footprint_bytes`
  in verdict87.out) provengono dai .log di `/usr/bin/time -l` parsati
  con `tr -d '\0'` (verdict87.sh:116) — canale NUL-strippato,
  KS-SK-88-3 violata in lettera (Klabnik WP-89 Q2); VOID-a-rischio
  finché il canale .log non entra in nul_free con REFUSE o conteggio
  NUL in-band (KS-SK-89-2). Le cifre committed_postcollect_win0 NON
  sono toccate (canale .memcensus, nul_free verificato).
- **VARMS — candidati sottratti per NOME (ADVISORY R=3)**: braccio
  MIMALLOC_ARENA_EAGER_COMMIT=1 delta **0 B** e braccio
  RUST_MIN_STACK=1048576 delta **0 B** sul committed min-of-R W=2 —
  nessuno dei due candidati muove il committed post-collect a W=2.
- **VDISJ — A-BB53 predizioni ex-ante CONFERMATE**: calibrazioni
  byte-riprodotte (r1==r2): pad87a net = pad87b net = **7.801.102 B**
  (gemelle strutturali), floor_inc = **1.161.206 B** entrambe
  [metric=net-at-lower, net_window=process-counters, W=1 sequenziale —
  legale per KL-87-2]. Concorrenza W=2, overlap qualificato dal
  qualificatore NUMERICO (A-BB49) in **2/2**: (a) **firma-inghiottimento
  su coppia NEST-FREE**: padB net = 15.602.514 B = calA+calB+310 B
  [derivata: 7.801.102+7.801.102+310] in ENTRAMBI i tentativi; padA net =
  18.748.620 / 18.748.924 B = calA+calB + ~3.146.416 B [derivata:
  net−calA−calB] (surplus consistente fra i tentativi: la finestra del
  primo ord assorbe ANCHE il first-touch runtime del secondo worker); (b)
  **floor_inc IDENTICO AL BYTE fra cal e conc (delta=0, 2/2 per lato)** —
  il floor strutturale è per-thread e nest-free ⇒ nessuno sconto da nodi
  condivisi, come predetto. TUTTI i net concorrenti restano **VOID come
  cifre per-thread** (KB-88-1): il run esiste per SCOPARE la promozione
  (v. §Delibere), non per attribuire.

## Aperture dichiarate (per NOME)

1. **Metà fisica**: la forma slope-committed W∈{1..4} NON riproduce la
   banda KL-85-2 (25.880.166 B/worker vs 3.605.572 B) — le due forme
   misurano regimi DIVERSI (granularità di commit d'arena vs steady
   marginale W=10). La riconciliazione (slope a W alti, o
   commit-granularity in-band) è materia del Concilio WP-89.
2. **padA surplus ~3.146.416 B = 3,00 MiB [derivata: media dei due
   tentativi]**: consistente (Δ fra tentativi 304 B), non attribuito —
   candidato: first-touch del secondo worker dentro la finestra
   process-counters del primo. Si scioglie solo con A-BB50 (net window
   per-thread).
3. **A-BB50/A-DL39**: solo DESIGN in questa sessione
   (`wp87-harness/design87.md`); ogni canary concorrente resta vietato
   come cifra per-thread fino ad attuazione A-BB50.
4. **A-AH38 + dry-run KS-AH-86-1**: nessuna fase slope/base-arm
   equivalence-based in questa campagna (same-code consumption usata);
   l'apertura resta nominata.
5. **A-MS27 / A-PP18 / A-PP27**: invariati (backlog).

## ⚖️ Delibere eseguibili QUI (ordine WP-88 p6)

1. **Promozione della scomposizione additiva — SCOPED alle coppie
   ANNIDATE**: net(ord2) = net_own − shared è promossa da MODEL-GRADE a
   **VERDICT-GRADE nel solo scope delle coppie annidate** (simboli del
   lato piccolo ⊆ del grande, classe hello ⊆ pad85): lì il confound dei
   nodi condivisi non morde per costruzione (Δfloor_inc = 996.838 B
   ESATTO in entrambi gli ordini, VINV al byte). KB-87-1 è SOLLEVATA in
   questo scope e SOLO in questo. Per coppie disgiunte/parziali resta
   MODEL-GRADE: l'evidenza A-BB53 di QUESTA campagna mostra che sul
   nest-free la finestra di processo inghiotte (padB=calA+calB+310 B) —
   la generalizzazione esige A-BB50 attuato + delibera di Concilio.
2. **A-DL39 split heap per-worker**: design approvato come DOCUMENTO
   (design87.md §A-DL39); merge = delibera Concilio WP-89 con bump
   ALLOC_ID + ricalibrazione come precondizioni nominate.
3. **A-BB50 net window per-thread**: design approvato come DOCUMENTO
   (design87.md §A-BB50); precondizione di ogni canary concorrente; la
   coppia pad87a/b è il collaudo positivo/negativo previsto.
