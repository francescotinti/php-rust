# WP_SESSION_52 — in-node marks (classify −42,7%) + Fase 1.3 cold-box Object: full 2,14×, media 2,83×

> ⚡ **WP-52 (2026-07-25, `1a52712`+`ebd44f3`)** — **Ob.1 in-node marks
> (epoch+slot `WalkMark` 8B su Object/PhpArray/Closure, stato del walk in
> tabella contigua che È la worklist FIFO, Ref su mini-mappa): classify_ms
> census 109.383→62.678 = −46,7s (−42,7%), OLTRE la quota (−25..−34s).
> Ob.2 Fase 1.3 cold-box (`rare: Option<Box<ObjRare>>`, −64B/istanza;
> dyn_entries ESCLUSO: è lo storage di stdClass = fast-path): −56,0B/oggetto
> alla cifra (87,5% del predetto, obj.live_n conservato 525.258=525.258),
> churn obj −2,02GB. Giudici: full stesso-giorno run40 725,1s vs old
> phpr-wp51b 787,4s = −7,9% raw (~−6,7% corretto) → 2,14×; media ab52
> old 59,175 vs new 58,838 = −0,57% new 6/6 → 2,83× (minimo di sempre);
> peak fisico +0,61% (+10,4MB, guardia ≤+2% ok) → 4,36×. Parità: corpus
> 1421 IDENTICO ×2, cargo 1639/0 ×2, fail-set BYTE-ID a run33 su census52,
> run40 E run40-old (88 nomi).**

## Ob.1 — QUOTA (registrata PRIMA di scrivere codice) e verdetto

- Calibrazione dall'A/B WP-51: fusione (3→2 hash/arco + rehash) = −33s con
  rehash ≈ metà ⇒ 1 hash-op/arco ≈ 16-17s di full ⇒ predetto −25..−34s.
  Sanity primi-principi: ~400-500M archi/run ×2 passate ≈ 0,9-1G hash-op.
- Byte: +8B/nodo su arr peak 769.897 + obj (GcMark 8→16B) ≈ +7,3MB.
- **Verdetto: −46,7s misurati nel contatore bersaglio** — il surplus oltre
  la quota viene dal `get` per-pop di pass-2, dall'iterazione contigua
  (Vec vs mappa) di external-check/whites e dallo zero costo-mappa.
- Conservazione ESATTA (stessi ordini di scoperta): collects 303=303,
  calls 152=152, freed 14.235.463 vs .479 (Δ−16, 10⁻⁶), notes Δ−2,5k su
  4,109G — stessa banda di micro-rumore del check WP-51.

## Ob.1 — design (per riproporre il pattern)

- `WalkMark { epoch: Cell<u32>, slot: Cell<u32> }` NEL nodo (Object dentro
  `GcMark`, campo nuovo su PhpArray e Closure): epoch-guarded ⇒ nessun
  reset tra walk (stale = epoch≠corrente; epoch Vm-level, mai 0);
  `Clone`/COW dà mark fresco (una copia è un nodo NUOVO).
- Stato mutabile del walk in `Vec<NodeRec{node,handle,in_edges,live}>`
  indicizzata dallo slot; la Vec È la worklist pass-1 (push order =
  discovery order = identico alla VecDeque di prima ⇒ stessi conteggi).
- `Ref` = ibrido su mini-mappa: `Zval::Ref(Rc<RefCell<Zval>>)` non ha una
  struct dove mettere il mark; cambiare la variante = superficie enorme.
- External-check ora itera in ordine di scoperta (deterministico) invece
  che in ordine-mappa: live/white SET identici, ordine whites arr/clo
  diverso (obj restano sortati) — parità verificata dai gate per nome.

## Ob.2 — Fase 1.3 cold-box Object

- 3 Vec rari (72B) → `rare: Option<Box<ObjRare>>` (8B) = −64B/istanza;
  accessor predicano senza allocare, `rare_mut()` alloca al primo uso.
- **Deviazione dichiarata**: `dyn_entries` NON boxato — è LO storage delle
  proprietà di stdClass/dynamic (WP ne abusa): +1 deref su un fast-path =
  rischio classe WP-44 per soli −16B. Presi 64 di 80B a rischio zero.
- Alla cifra: obj.live exit −29,41MB su 525.258 oggetti = −56,0B/oggetto
  (predetto −64B = 87,5% ≥ 70% ✓; il residuo = i Box ObjRare di chi usa
  le feature). obj.cum −2,02GB (36,13M alloc). `ARR_OVERHEAD` census 40→48.

## Giudici (tutti stesso-giorno, catena sequenziale orchestrate52.sh)

- **Census mechanism-check** (phpr-memgc52, probe OFF): sopra. Fail-set
  census BYTE-ID a run33.
- **Full**: run40 (new) ultimo sample **725,1s = 12:05** (morte +19s,
  ~743s corretti) vs run40-old (phpr-wp51b) **787,4s = 13:07** (morte
  +10s) = **−7,9% raw / ~−6,7% corretto**; 725,1/339 = **2,14×**
  (convenzione last-sample come run39 782,7). Old diurno 787 vs old
  notturno 782,7 = ambiente ~+1%: la coppia stesso-giorno è il giudice.
- **Media (ab52, 6 round interleaved + 2 oracle)**: CPU old 59,175 vs new
  58,838 = **−0,57%, new 6/6** → 58,84/20,82 = **2,83×** (minimo di
  sempre). Peak fisico old 1,7098G vs new 1,7202G = **+0,61% (+10,4MB)**
  → 1,7202/0,3947 = **4,36×** — guardia ≤+2% rispettata, MA il netto
  +10,4MB manca la predizione (−1..−2MB): il +8B/arr al picco costa più
  della stima count-based — probabile size-class mimalloc (80→96B = +16B
  reali/array × ~770k ≈ +12MB) — REGRESSIONE PICCOLA TENUTA e verbalizzata
  (direttiva no-revert; il full guadagna −6,7% CPU).

## Parità e gate (classe GC, protocollo WP-50)

- corpus **1421 IDENTICO per nome col conteggio ×2** (albero Ob.1
  `1a52712`, albero Ob.1+Ob.2 `ebd44f3`)
- cargo **1639/0 ×2**; smoke readonly/clone/typed-unset BYTE-ID con oracle
- 3 full stesso-giorno (census52, run40, run40-old): fail-set **BYTE-ID a
  run33** (88 nomi) su tutte; 30.472 test 0E/2F/86W/73S
- Binario stashed sul path canonico esterno: `phpr-wp52` (sha256
  `57607da3…`); old dell'A/B era `phpr-wp51b` (sha1 `4f79c9d0…`,
  verificato = release pre-modifiche al pre-flight)

## ⭐ Lezioni

- ⭐⭐ **I mark di un walk grande vanno NEI nodi (epoch+slot), lo stato in
  una tabella contigua**: −42,7% del classify senza toccare l'algoritmo
  (conservazione alla cifra). La quota calibrata dall'A/B precedente
  (−25..−34s) era il FLOOR: mappa-iterazione e get-per-pop valevano
  un altro ~30%.
- ⭐⭐ **RSS di ps sui full ≠ footprint**: new 4024 vs old 2897 stesso-giorno
  (+39%!) ma il peak FISICO media è +0,61% — i mark in-node scrivono in
  ogni nodo vivo a ogni walk e tengono residenti pagine idle/MADV_FREE.
  Mai giudicare footprint dall'RSS telemetrico (già WP-39, forma nuova).
- ⭐ **Il +8B/nodo va quotato in size-class, non in byte**: al picco il
  costo reale può raddoppiare (80→96B = +16B/array) — la predizione
  count-based ha mancato il segno del netto (−2MB predetti, +10,4MB reali).
- ⭐ Epoch-guarded mark: `Clone` deve dare mark FRESCO (una copia è un
  nodo nuovo) — con COW/`Rc::make_mut` un derive copierebbe slot stale
  (innocuo con epoch, sbagliato senza).
- ⭐ Cold-box: escludere dal box ciò che è storage di un TIPO comune
  (dyn_entries = stdClass), anche se la roadmap lo elencava — la lettera
  della fase cede al profilo del workload; deviazione verbalizzata.

## Prossimo (WP-53)

1. Residuo full 2,14× vs 2,06× WP-40 ≈ ~30s (classify census residuo
   62,7s: ora dominano borrow/iter/strong_count, non l'hashing — il
   ripiego "profilo del walk" di WP-52 è la lente giusta se si insiste).
2. reflect-cache: owner della cardinalità (memo keyed su ClassId dei mock,
   WP-47); ritorno cap 16384→8192 = decisione utente (~42MB standing).
3. Fase 2 CPU roadmap (RET_DEREF+ret_shape, Sweep elision) — mai aperta.
4. Laravel resta posticipata a valle della roadmap.
