# predictions67 — pre-registro K-66.1/G-67.5 (committate PRIMA dei letti)

Regole: bande bilaterali SOLO dove c'è meccanismo noto (M-66.5/G-66.3);
istruttorie senza kill-switch dove non c'è storia. Ogni cifra di memoria
dichiara l'unità (L-67.2). Binari: parità = build tree WP-67 (post debiti
1330fd9); census = phpr-memgc66 (build stesso tree, feature mem-census).

## P67-Q1 (B-67.1 census server, PRIMA di disegnare la publish impure)

Probe: memgc66 `phpr -S` su wpdev, N=10 richieste `/`, dump census
per-richiesta (run_module_with_hir dumpa a ogni run top-level — delta
dump_i − dump_{i−1} = costo della richiesta i).

- **Q1-a (meccanismo, bilaterale)**: a steady-state (delta R9→R10) i
  compile per-richiesta sono ESATTAMENTE 15 (la lista P66-R4) —
  Δcompiles ∈ [15, 15] con lista path stabile; ogni altro path a
  Δcompiles=0. KS: Δcompiles≠15 a R≥4 ⇒ la tassonomia P66-R4 era
  incompleta, STOP quota.
- **Q1-b (istruttoria, nessun KS)**: costo CPU dei 15 =
  Δ(lower_ns+compile_ns) per richiesta atteso nell'ordine di
  10-100 ms/richiesta (grandi seed elisi 191..435; nessuna misura
  storica per-richiesta); script-loader.php e ProviderRegistry attesi
  dominanti per-file (elide 242/435).
- **Q1-c (meccanismo, bilaterale)**: Δleaked_modules per richiesta
  steady-state = 15 + n_eval (n_eval ≥ 0, atteso 0 o piccolo costante);
  Δleaked_bytes > 0 e STABILE tra R9→R10 e R8→R9 (±10%). KS:
  Δleaked_modules < 15 ⇒ il contatore B-67.2 non vede i siti, fix
  prima di ogni quota.
- **Q1-d (istruttoria)**: Δleaked_bytes per richiesta atteso
  nell'ordine di 1-20 MiB (15 Module con seed grandi; unità MiB).

## P67-E1 (E-67.1 depurazione — check di coerenza sullo stesso letto)

- **E1-a (meccanismo)**: con la finestra depurata, per ogni path
  lcpath: lower_ns non contiene più il nested autoload ⇒ Σ lcsum
  additiva; su un run `--list-tests` (stesso workload di WP-63/65)
  dup_fp_ns + dup_cold_ns = dup_lc_ns ESATTO (identità per
  costruzione, verificabile a macchina).

## P67-P2 (P-2 de-leak — metro L-67.4/K-67.4, PRIMA del codice P-2)

- **P2-a (meccanismo, bilaterale)**: post P-2, probe N={1,100,1000}
  richieste (fixture template piccola, cache warm): pendenza
  Σcommitted mi_bin per-richiesta tra N=100 e N=1000 ≤ **50 KiB/req**
  (soglia pre-registrata; pre-P-2 la pendenza è ≈ Δleaked_bytes/req
  misurata in Q1, attesa ≫ soglia). Phys accanto, MAI da solo.
- **P2-b (meccanismo)**: post P-2, Δleaked_modules (contatore diretto,
  rinominato parked se applicabile) a fine richiesta con RetainSet
  droppato: i Module non in cache MUOIONO — bytes ritenuti a
  steady-state = cache bounded + richiesta corrente. KK67-2: pendenza
  sopra soglia o miss_cold in crescita ⇒ P-2 NON chiusa.
- **P2-c (sentinelle, [0,0])**: probe67-droporder byte-id pre/post
  refactor; gate67 matrice piena; seed_prefix_short=0; qualunque
  divergenza drop-order = STOP (K-M67.4).
- **P2-d (coppia full propria, KG67-1/KH67-3)**: CPU della coppia
  stessa-sera dentro lo spread build-adiacente storico (±1,5-2%);
  fail-set 88 BYTE-ID = run33. Cifre <1% SOLO con self-pair A-A′.
- **P2-e (KL67-1, terza run del metro)**: Σcommitted mi_bin dump-1 del
  master full census entro **±1%** di 1.686,1/1.684,8 MiB (65b/l661) a
  parità di leve censite; se diverge >1% il metro decade a
  counted-only. (La baseline P-2 sposta il counted: il confronto vale
  sul ramo PRE-P-2 se la run è pre-leva, altrimenti si dichiara
  l'atteso post-leva dal Δcounted.)

## P67-I1 (cacheabilità unit impure — DOPO P-2, forma vincolata §4)

- **I1-a (B-67.3, meccanismo, bilaterale)**: post fix, replay wpdev
  R≥3: {hit_cross=512, miss_cold=0, miss_dc=0, fp-seq invariata vs
  pre-fix}. KB67-2: le 15 diventano miss_dc ⇒ leva NON consegnata.
- **I1-b (fixture S-67.2, meccanismo)**: ordering-echo 3 lati: oracle
  Zend stampa U-top·[autoload]·U-after in R1 e RI-stampa [autoload] in
  R2; phpr cold = stesso output dell'oracle; phpr hit = IDENTICO
  all'oracle R2 (il replay ri-esegue gli echo dell'autoload). KS-S67.1:
  hit senza [autoload] ⇒ STOP publish.
- **I1-c (fixture P-67.4, meccanismo)**: figlio editato tra R1 e R2 ⇒
  MISS (mai hit con dep-list stantia).
- **I1-d (KS67-1)**: catena impura (dep del replay non hit pura) ⇒
  never-published — fixture con dep a sua volta autoload-in-lowering.

## P67-G1 (self-pair A-A′, B-66.1 — stessa sera della coppia P-2)

- **G1-a (istruttoria)**: A-A′ (stesso binario ×2) delta CPU atteso
  dentro ±1,5-2% (spread di classe run51/52); il valore misurato
  DIVENTA lo spread citabile della serata — nessun KS (prima
  esecuzione del protocollo).
