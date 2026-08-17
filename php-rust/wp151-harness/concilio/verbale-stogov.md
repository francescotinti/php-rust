# Verbale STOGOV — S-151 (lente: semantica Zend Engine) — Fase 1, indipendente

**VERDETTO: CONCORDO CON EMENDAMENTI** su A1..A4. La sequenza regge, ma A3 è
AMBIGUA sul punto capitale della mia lente — il timing dei distruttori — e la
forma «arena contigua» di Gemini è REFUTATA dal C di Zend stesso.

## Fatti verificati nei sorgenti
- phpr: `Zval::Object(Rc<RefCell<Object>>)` (php-types/src/zval.rs:40),
  `WeakHandle(Weak<…>)` (:61), `Resource(Rc<RefCell<…>>)` (:54); distruttori
  **sweep-driven**, timing **DELIBERATAMENTE ≠ Zend** («intentionally differs
  from Zend's immediate refcount destruction (priced into the per-name
  baselines)» — vm/mod.rs:22331; sentinelle WP-39 `gc_drop_order_sentinel_*`
  pinnano l'ordine PROPRIO di phpr).
- Zend 8.5.7: `zend_objects_store` = **array di PUNTATORI** (`zend_object
  **object_buckets` + free_list, Zend/zend_objects_API.h), oggetti allocati
  singolarmente; dtor IMMEDIATO a rc=0 (`zend_object_release`). NON è arena
  contigua: la stabilità dei `zend_object*` attraverso dtor re-entranti dipende da questo.

## Q1 — A1→A2→A3: SÌ con clausola canale-stabile
Il census per CANALE sopravvive al refactor (il canale è semantico); la mappa
per-SITO no ⇒ probe rigenerabile post-refactor. Chirurgia-prima/interleaving:
NO — A3 tocca mod.rs+run.rs+host.rs+php-types, non esiste sottoinsieme
piccolo. MA prima del census va decisa **A3.0** (R1).

## Q2 — gate tranche A2
- Ogni tranche promossa = pin nuovo ⇒ **coppia WP a OGNI tranche** (regola utente 2026-08-12); diradarla si CHIEDE all'utente, non si delibera qui.
- Sostituto della byte-identità: batteria + corpus×2 (il gate giudica già
  NOMI E CONTENUTO-digest: pinna anche l'output dei 1412 fail) + fixture +
  micro R=5 in banda + **disasm bl-count di run_loop** come tripwire (un
  move-only può flippare l'inliner cross-module: la banda giudica, il disasm
  spiega). Partizione: freddi PRIMA (dom, reflect-hostcall, census),
  run_loop ULTIMO e **NON spezzato**: lo spacchettamento `exec/ops_*.rs` di
  Gemini §4.3 rompe l'inlining del dispatch (veto «probe che rompe
  l'inlining»; lezione WP-104).

## Q3 — A3: semantiche a rischio PER NOME + copertura corpus (1412 fail congelati / 5305 phpt Zend/tests)
1. **Timing/ordine distruttori** — «refcount esplicito» ha DUE letture:
   (a) rc che REPLICA il timing sweep attuale (zero flip, golden stabile);
   (b) distruzione immediata a rc=0 alla Zend (FEDELTÀ, ma cambia l'ordine
   osservabile ⇒ invalida per costruzione le sentinelle WP-39, muove digest e
   fail-set). DUE progetti con gate diversi (R1). Copertura: top-level 19/25
   phpt `__destruct` passano (guardia), 6 nel fail-set; gc/ 62/76 passano.
2. **__destruct re-entrante + risurrezione + once-only** — oggi `destructed`
   id-set autoritativo + GC_DESTRUCTED (mod.rs:4377,4402,5301). Senza RefCell
   il dtor esegue PHP che ALLOCA nel store mentre la cascata itera: con
   ObjectData inline nell'arena un push che rialloca invalida ogni `&mut`
   tenuto attraverso la chiamata — in safe Rust non compila o forza re-index a
   ogni tocco (il bounds-check che si voleva uccidere). **REFUTATA la forma
   Gemini §2.2-2 «arena contigua»**: la forma Zend-vera è store di puntatori
   stabili + free-list, id = indice bucket. Copertura: gc/bug70805{,_1,_2}
   (risurrezione in GC) NEL FAIL-SET ⇒ nessuna guardia.
3. **Weak/WeakMap** — con id riusati (pool `free_object_id` già in object.rs)
   un `WeakHandle(id)` nudo risorge sul riuso = ABA ⇒ weak = (id, generation);
   id nudo per spl_object_id (che DEVE riusare alla Zend). Copertura: weakrefs/
   **18/35 NEL FAIL-SET** (weakmap_basic_map_behavior, weakmap_weakness,
   weakmap_iteration, gh17442_1/2, gh20073…) ⇒ WeakMap quasi NON guardato.
4. **spl_object_id/hash (stabilità+riuso)** — test dedicati in ext/spl e
   ext/standard, **FUORI dal corpus**; copertura solo indiretta via `#N` nei
   digest ⇒ fixture (R5).
5. **Cicli/collect_cycles** (mod.rs:5146, WalkMark epoch) — Bacon-Rajan esige
   refcount ESATTI: oggi `Rc` li garantisce per teorema del type system; con
   rc manuale un clone/drop mancato corrompe il collector LONTANO dal sito.
   Copertura: gc/ 62 pass; gc_022/023/030/043/045..049, bug64960/65372/67314 fail.
6. **CoW array-con-oggetti + layering** — `Drop for PhpArray` (leva L-RD1)
   dovrebbe DECREMENTARE gli oggetti contenuti, ma php-types NON vede lo store
   del VM: o store alla-EG() (thread-local) visibile anche a php-types, o si
   vieta il drop naturale dei container (perimetro enorme, L-RD1 da rifare).
7. **&$obj** — `Ref(Rc<RefCell<Zval>>)`: ogni cella Ref è un sito rc in più.
   **Resource** resta Rc (fuori perimetro A3: dichiararlo). Serialize
   oggetti: test in ext/*, fuori corpus (R5).
8. **Prezzo vero: perdita del Drop-glue** — census S-140: 44% dei clone INLINE
   da run_loop ⇒ migliaia di siti dove il dec-ref diventa disciplina MANUALE
   (Zend la regge in C con 25 anni + debug-build con assert); il divieto del
   Copy-senza-refcount è metà del vincolo: serve CHI decrementa e COSA lo forza (R3).

**Numeri che il census DEVE produrre (decidibilità)**: (i) churn clone/drop
SPLITTATO per tipo di Zval — il CHURN 32% di S-140 è DILUITO, A3 azzera solo
la quota-Object; (ii) borrow/borrow_mut su RefCell&lt;Object&gt; per canale +
prezzo ns/op da A/B proprio; (iii) drop di container-con-oggetti; (iv) **quota
dei borrow tenuti ATTRAVERSO chiamate re-entranti** (metodo/hook/dtor):
restano re-index anche nello store — se dominano, il guadagno crolla.
Tensione tetto-movimenti 1,27 s vs Gemini: NON è contraddizione — il tetto
riguardava i MOVIMENTI di Zval, non il churn rc+borrow degli oggetti; ma le
cifre Gemini (30–45%, −35% Doctrine) restano NON firmate: fede a (i)–(iv).

## Q4 — dente
Sede = BATTERIA (morde a ogni promozione; CI con backlog ~3 giorni no;
pre-commit secondo dente facoltativo). Soglia ~2.000 SOLO sui file NUOVI;
sugli esistenti **ratchet di non-crescita per file** (tetto assoluto
inapplicabile con mod.rs 25,7k). Conteggio RIGHE, non pattern (bea7ea3).

## Q5 — cosa manca (mandato inverso)
BT1: UNA leva su UN builtin ha mosso ORM di −1,3× (8,4→7,1). Il census
per-nome (none.other 94,6M, class_exists 9,7M, __reflect_* 12,4M) può
contenere altre BT1: **A1 abbia mandato esplicito di pescarle PRIMA di
congelare 4–6 sessioni senza leve** — ogni BT1 ricalibra il gap residuo di A3.

## Emendamenti
- **R1 (A3.0, pregiudiziale)**: decidere PRIMA del census tra sweep-preserving
  e Zend-immediate; se immediate, pre-registrare «rifondazione golden + flip
  per NOME + riscrittura sentinelle WP-39» come atto di fedeltà, non regressione.
- **R2 (forma store)**: bucket di Box/puntatori stabili + free-list alla
  `zend_objects_store`; VIETATA l'arena contigua inline; weak=(id,generation).
- **R3 (collaudabilità rc)**: handle NON-Copy con clone/drop obbligati via
  store + shadow-mode di batteria (doppio conteggio Rc↔rc su run corpus) +
  audit fine-richiesta live-count==0; senza i tre, A3 non si spedisce.
- **R4 (census)**: aggiungere i numeri (i)–(iv) al piano tranche-5.
- **R5 (fixture pre-chirurgia)**: fx-objstore bilaterale: spl_object_id-riuso,
  serialize oggetti, WeakMap-weakness, risurrezione in __destruct.
- **R6 (A2)**: run_loop non si spezza; coppia WP a ogni tranche-pin salvo
  deroga esplicita utente; disasm agli atti per tranche.

## Kill-switch pre-registrabili
- **KS-ST-1**: census: quota-Object (churn+borrow) sotto soglia pre-registrata
  (proposta <15% del gap ORM netto, su binario census) ⇒ A3 FERMA al tetto.
- **KS-ST-2**: tranche A2 con micro fuori banda R=5 o digest mutato non
  dichiarato ⇒ revert tranche al byte.
- **KS-ST-3**: UNA divergenza Rc↔rc nello shadow-mode R3 ⇒ stop chirurgia,
  incidente contato. **KS-ST-4**: fixture R5 rosse sul pin ⇒ la chirurgia non
  parte senza baseline bilaterale agli atti.
