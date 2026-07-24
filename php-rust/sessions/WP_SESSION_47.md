# WP_SESSION_47 — Ri-attribuzione owner-level: il "3G di cicli" era la reflect-cache

> ⚡ **WP-47 (2026-07-24, `583dd30`→…)** — **Attribuzione di seconda
> generazione RIUSCITA al primo colpo: il root-walk esteso a TUTTI i campi
> Zval-bearing del Vm riconcilia arr al 100,0% (3.114.573/3.114.573) e str
> al 96,7% (23,03M/23,82M) — e l'owner del gap è UNO: `reflect_method_info_cache`,
> 455.978 entry / 2,48G = 93% del grafo attribuito.** La diagnosi WP-45
> "3,08G cicli irraggiungibili" è definitivamente falsificata: dati VIVI,
> tenuti da una memo-cache VM senza eviction.

## Il metodo (mandato: zero-fix finché non riconcilia)

1. **Tabella decisionale PRE-REGISTRATA** (`sessions/WP47_DECISION_TABLE.md`,
   commit `583dd30`) PRIMA di leggere i numeri: holder→leva per 9 scenari.
2. **Root-walk esteso** (`06e13e1`): `census_walk_frame` (slots+stack+
   **dyn_vars**+this+ret_cell+**iters**+ext — il walk vecchio leggeva solo
   slots/stack), fibers, frames vivi, autoloaders, error/exception/shutdown/
   signal handlers, ob_stack, enum_cache, lazy/reflect tables, filtered
   streams; **contatori reached-vs-live per canale DENTRO `deep_size`**
   (`tag=walk_recon`): la riconciliazione è esatta alla singola allocazione,
   non stimata. Const-pool ZStr crediti al canale reached (`e0e…`).
3. **Census sul gruppo media** (binario separato `phpr-mem-target/…/phpr-memgc47`,
   mai il binario di parità): riconciliazione immediata, poi split della
   categoria in 4 costituenti con conteggio entry → colpevole inchiodato.

## Il colpevole e la fisica

- `reflect_method_info_cache: HashMap<(ClassId, nome), Zval>` — memo dei
  descrittori ReflectionMethod (WP-era precedente), **mai evitto**. Ogni
  **mock PHPUnit è una classe fresca con ClassId nuovo**; la generazione del
  mock riflette i metodi → una entry (~5,4KB media: descrittore con
  docComment/params/attributes) per metodo di classe MORTA. 456k entry sul
  gruppo media.
- Effetto collaterale CPU (spiega il verdetto WP-46): i descrittori sono
  array Rc condivisi → il drop del clone del caller li NOTA come possible
  root container (`strong>1`) → erano gran parte dei **726k root "vivi"**
  che il collector camminava a ogni collect senza liberare nulla.

## Le leve

1. **`fa100ad` — epoch eviction, cap 8192** sul sito di insert (miss-only,
   zero costo sugli hit): al cap la cache si svuota e il working-set si
   ri-memoizza. PHP-invisibile (i descrittori sono value COW; l'identità Rc
   non è osservabile). **Mechanism-check census: peak 5,12G→1,81G (−65%),
   arr vive 3,11M→728k, str vive 23,8M→855k, roots_total 2,67G→169M,
   root per collect 726k→276k; riconciliazione arr ancora 100% (728.468/728.468).**
2. **`d684cd7` — recupero CPU collector** (Obiettivo 2): `gc_classify` a
   2 passate — via la children-map dell'intero grafo e i cloni Zval di TUTTI
   i figli (scalari inclusi): un solo Handle per nodo scoperto, live-BFS
   ri-legge gli archi by-borrow, child-list ricostruite SOLO sul sottografo
   white; + **isteresi della soglia**: lo step-down (mai sotto la base 50k,
   intoccata) solo se ≥1% dei root processati è morto — il collect che
   libera 10² su 10⁵ root vivi non riabbassa più il trigger (la causa del
   +80% full-suite di WP-46).
3. NON eseguito (verbalizzato): mark intrusivi nei container (PhpArray/Closure
   non hanno campo mutabile in safe Rust — solo gli Object hanno GcMark);
   collect al confine test (nessun segnale di boundary visibile alla VM).

## Gate

- cargo test --release: **1639/0** sia su `fa100ad` che su `d684cd7`
  (sentinelle `gc_object_cycle_collect_sentinel` +
  `gc_container_cycle_collect_acceptance` anche rilanciate singolarmente:
  verdi).
- Corpus per nome vs baseline 1421: **IDENTICO due volte** — su `fa100ad`
  (solo eviction) e su `d684cd7` (classify 2-passate + isteresi).
- **A/B 6 round old=`e6af390`** (oracle 21,12/21,30 stabilissimo):
  - **peak footprint fisico: old 4,877-4,924G → new 1,743-1,792G = −63,8%**;
    vs oracle ~0,41G il gap passa **da ~11,9× a ~4,3×** — il più grande
    salto footprint della storia del progetto.
  - user CPU: old 62,77s medio → new 63,60s = **+1,3%** (6/6 round
    new>old, delta 0,2-2,0s) — la ricostruzione dei descrittori evitti
    costa più di quanto il classify snellito recuperi sul media group.
    **TENUTO per direttiva no-revert**; il bersaglio vero dell'isteresi è
    la full-suite (3,71× WP-46).
- **Gate22 COMPLETO VERDE** (binario `d684cd7`, 10:08→10:31, archivio
  `gate-out-wp47-archived/`): corpus 1421 (0 nuovi-fail) · sess 28 /
  date 351 / refl 290 IDENTICI · ORM 3484 3E/13F per nome · hk 1665 0E/0F ·
  cargo 1639/0 · gd/mysqli/media BYTE-ID · http DIFF-set identico (16 item
  WP-14) · **option 413 IDENTICO col conteggio (413/1061)** · **restapi
  3508 IDENTICO col conteggio (1E/1F = oracle)**.
- **Full-suite (run34): fail-set BYTE-IDENTICO a run33** (88 nomi; 30.472
  test, 0E/2F/86W/73S). Master-CPU **≈19:20 = 3,42×** (da ~21:00 = 3,71×
  WP-46: l'isteresi + classify 2-passate recuperano ~8%; riferimento
  storico WP-40 resta 11:39 = 2,06×). Wall ~23 min = 2,0× (da 2,3×).
  RSS telemetry max 5,4G mid-run (transiente; a fine run 1,35G), con
  purge=0. Baseline per nome resta `run33-fails.txt` (=`run34-fails.txt`).

## ⭐⭐ Lezioni

- ⭐⭐ **"Irraggiungibile dal walk" è una proprietà del WALK, non del dato**:
  due sessioni (WP-45/46) hanno inseguito cicli morti perché il root-walk
  copriva 11 categorie su ~25. La riconciliazione reached-vs-live PER
  ALLOCAZIONE (contatori dentro deep_size) trasforma l'attribuzione da
  stima a bilancio esatto: quando arr ha riconciliato al 100,0% il
  residuo era zero e l'owner era per forza nella lista camminata.
- ⭐⭐ **Ogni memo-cache keyed su un id APERTO (ClassId che cresce coi mock)
  è un leak sotto PHPUnit**: bounded per programmi normali, monotona sotto
  generazione di classi. Il censimento con `n=` nelle categorie l'ha
  inchiodata in un solo re-run.
- ⭐ Il death-avg estimator del canale arr sovrastimava (1,9G stimato vs
  1,37G esatti dal walk): con reached_b il bilancio non dipende più
  dall'estimator.
- ⭐ La leva footprint era anche la leva CPU: i root che il collector
  camminava inutilmente erano i descrittori della cache.

## Dove vive il gap residuo (dal census post-fix, per WP-48)

Peak media 1,77G vs oracle 0,41G = 4,3×. A fine run (census): unit 0,30G
(2046 moduli leakati — **Fase 1.1 shrink**, ora il canale singolo più
grosso) · arr esatti 79M · str 55M · obj 45M · created 115M ·
non-censito ~1G al picco (transienti mid-run, rounding size-class, preg
engines, dom arenas, frammentazione mimalloc — il delta 1,77G-peak vs
~0,6G live-end). Str residuo walk 3% (≈39MB, metadati moduli fuori
const-pool). Full-suite CPU ancora 3,42× vs 2,06× WP-40: il collect
sul grafo vivo della full resta il sospetto (prossime leve: collect al
confine test se si trova un segnale, rooting più selettivo).

## Prossimo (WP-48)

1. Fase 1.1: shrink unit (~0,3G, `shrink_to_fit`/`Box<[T]>` su Func/Module
   + drop seed HIR post-link) — ora il canale dominante del residuo.
2. Recupero CPU full-suite: attribuzione del 3,42× (op/gc census sulla
   full o su un gruppo grande) prima di altre leve.
3. Coda: cold-box Object, Fase 2 CPU (RET_DEREF+ret_shape, Sweep elision),
   interning const-array, residui gc (gc_047 iteratore al break, gc_030
   trace shape, gc_022 temp mid-statement), bug isset __get annidato.
4. Laravel a valle (rotta WP-first).
