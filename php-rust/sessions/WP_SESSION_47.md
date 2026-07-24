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

## Gate (in corso / da completare)

- cargo test --release: **1639/0** su `fa100ad` (2 sentinelle GC incluse).
- corpus per nome vs baseline 1421 (`gate-out-wp46-archived/corpus.fails`): (TBD)
- A/B 6 round old=`e6af390`: (TBD)
- gate22 completo col conteggio: (TBD)
- full-suite vs run33 (88 nomi): (TBD)

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

## Prossimo (WP-48)

(TBD a fine gate: residui gc_047/gc_030/gc_022, shrink unit 0,3G Fase 1.1,
interning const-array, str residual 3% = metadati moduli, Laravel a valle.)
