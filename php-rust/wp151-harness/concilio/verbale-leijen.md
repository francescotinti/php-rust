# Verbale LEIJEN — S-151 (lente: allocatore / footprint fisico)

`VERDETTO: CONCORDO CON EMENDAMENTI` sull'impianto A1..A4. La SEQUENZA regge;
la FORMA di A3 «arena contigua + bump + reset a fine richiesta» come scritta
in Gemini §2.2 è REFUTATA nella metà allocatore; sopravvive solo la metà
refcount-esplicito/RefCell, che non è la mia lente. Fatti di codice verificati:
Zval=16B con niche `Option<Zval>`==16B (assert `array.rs:88-93`); oggetto oggi
= `Rc<RefCell<Object>>` + `Props{layout Rc, slots Vec<Option<Zval>>, dyn_entries}`
(`object.rs:18,713`); stampo `PropsTemplate` GIÀ collassa l'init (OL1-F1,
objalloc −20,4%); Vm FRESCA per richiesta, RetainSet per-richiesta (`worker_pool.rs:1-31`,
riuso Vm RESPINTO WP-77.2); mimalloc globale (`php-cli/main.rs:20-24`),
PURGE_DELAY=0; FFI `mi_heap_new`/`mi_heap_malloc_aligned` GIÀ in-tree
(`memcensus.rs:705-857`, letto defer del census).

## Refutazioni di lente (motivano gli emendamenti)
1. **Il bump-reset è una finzione sotto la semantica PHP.** Il vincolo del
   MANDATO (distruttori a timing osservabile ⇒ refcount resta) implica che gli
   oggetti muoiono A METÀ richiesta, a rc=0. Un bump puro non riusa la memoria
   dei morti ⇒ il picco dell'arena = TOTALE allocato per richiesta, non il
   live-set (su ORM: milioni di oggetti churnati, objmap 43,4 = round-trip GC).
   Alternativa (b): free-list/SlotMap dentro l'arena = re-implementare in casa
   i bin sharded di mimalloc, guadagnando ~zero (pop da free-list è già il
   costo mimalloc). Gemini fonde (a) e (b) in una frase sola: non decidibile
   così com'è.
2. **«Reset a fine richiesta» compra poco per architettura.** La Vm è fresca
   per richiesta e muore col suo heap logico; PURGE_DELAY=0 restituisce le
   pagine subito. Un'arena PRE-allocata persistente per-worker (per amortizzare
   la pre-allocazione) trattiene il picco del PEGGIOR request per N worker —
   contro la disciplina footprint vigente — oppure si rialloca a ogni richiesta
   (costo che nessuno ha prezzato). E il reset NON salta i distruttori: lo
   sweep ordinato a request_shutdown (ordine Stogov) va eseguito comunque.
3. **Doppio allocatore non necessario: mimalloc ha già i first-class heaps.**
   `mi_heap_*` è GIÀ linkato nel census (L-71.1). Un mi_heap per-richiesta (o
   per-canale-oggetti) dà località e distruzione O(1) SENZA un secondo
   allocatore, senza frammentazione incrociata, con le stesse size-class. Ogni
   confronto A/B di A3 deve avere questo come braccio di controllo, non solo
   «mimalloc default».
4. **Località promessa diluita:** i valori delle props (ZStr, Rc<PhpArray>,
   Ref-cell) restano sull'heap mimalloc comunque — l'arena compatta solo gli
   header ObjectData, che in mimalloc sono già size-class-uniformi (poca
   frammentazione esterna da attaccare).
5. **Props inline `[Option<Zval>; 8]` = tassa piatta 128 B/istanza** (16B×8,
   niche verificata). In un'arena a slot uniforme OGNI oggetto paga il max.
   Il risparmio è 1 alloc (il Vec slots), il cui costo-init lo stampo ha già
   compresso. Decidibile SOLO con la distribuzione pesata per istanziazioni (Q3).
6. **Il tetto CPU è già stretto:** TETTO movimenti 1,27 s ≈ 3,4% del gap ORM;
   CHURN 32% è clone/drop Zval (44% inline da run_loop), NON tempo allocatore.
   Le cifre Gemini (30–45%, −35% Doctrine) restano NON firmate: la metà
   «allocatore» di A3 attacca una fetta che i nostri strumenti non hanno mai
   mostrato dominante.

## Q1 (sequenza)
A1→A2→A3 GIUSTA dalla mia lente: i funnel del census vivono in php-types
(`Props::census_sync`, choke `next_id`), che A2 (split vm/mod.rs) non tocca ⇒
i CONTEGGI sopravvivono al refactor. I TEMPI no: FR1 ha mostrato prezzi
strutturali (+3180 B/+26 bl) da solo layout. Emenda R3.

## Q2 (gate tranche A2)
Aggiungere ai gate un **gate FOOTPRINT**: peak vmmap Physical (1 gamba WP + 1
ORM) in banda vs s150 (riferimento t4: 1774–1847 MB, MISTO dichiarato — prima
serve una banda pulita, R4). Un modulo spostato può flippare inlining e muovere
il peak. Coppia WP piena solo a pin nuovo (regola vigente), non per tranche.
La motivazione Gemini «build dimezzata» NON è misurata: misurarla una volta,
non adottarla come attesa.

## Q3 (numeri che rendono A3 DECIDIBILE — richiesta formale al census tranche-5)
- **N1 live-set vs churn**: per workload (ORM, 1 richiesta WP, hk): oggetti
  TOTALI allocati vs PICCO simultanei vivi (max live), per classe. Decide
  bump-only vs slab e dimensiona l'arena: picco_live × sizeof(slot).
- **N2 distribuzione #props PESATA per istanziazioni**: p50/p90/p99, % istanze
  ≤4/≤8 slot, % con dyn_entries non vuoto. Decide la tassa inline-8:
  Δbytes = Σ istanze × (128 − 16×capacity_odierna).
- **N3 quota CPU allocatore sul CAMMINO oggetti**: % user CPU in malloc/free
  attribuibile a creazione/drop di RcBox<Object>+slots-Vec, BILATERALE, al
  netto dei pavimenti, per workload. È il TETTO del guadagno della metà
  allocatore di A3.
- **N4 bytes/oggetto oggi vs proposto**: dal canale obj del memcensus
  (`object.rs:338` già somma header+capacity) + size_of emesso dal binario,
  mai a mano.
- **N5 baseline footprint**: peak vmmap Physical di 1 gamba WP e 1 ORM @ s150
  (l'ordine S-151 è tutto CPU-framed: senza N5 il trade footprint↔CPU di A3
  non ha giudice).
- **N6 popolazione a fine richiesta**: oggetti vivi a request_shutdown —
  quanto copre davvero un «reset a fine richiesta».

## Q4 (dente righe)
Sede = BATTERIA (morde a ogni promozione; la CI ha backlog ~3 giorni, fatto
fresco del MANDATO ⇒ solo-CI non morde in tempo). Soglia 2.000 per file NUOVI;
per gli esistenti ratchet DECRESCENTE (mai sopra il conteggio corrente al pin).
Lezione bea7ea3 (auto-morso) recepita: pattern del dente su conteggio `wc -l`,
non su pattern testuali componibili.

## Q5 (cosa manca / mandato inverso)
(1) Manca QUALSIASI misura di footprint nell'ordine S-151 (→ N5): A3 baratta
footprint per CPU e non esiste la baseline. (2) Manca il braccio di controllo
mi_heap (R2): senza, un A/B «arena vs status quo» attribuirebbe all'arena ciò
che è della semplice affinità di heap. Cosa sappiamo oggi che ieri no: BT1 ha
mostrato che il gap ORM può crollare per UNA causa puntuale (8,4→7,1) — il
soffitto «strutturale» di Gemini è in parte fatto di cause nominabili; prima
di pagare un'architettura, il census deve dire quanta parte resta strutturale.

## Emendamenti
- **R1 (A3 riformulata)**: sostituire «arena contigua pre-allocata + bump +
  reset» con «store centralizzato a slot CON free-list e refcount esplicito»;
  il bump-only si ri-ammette solo se N1 mostra live≈totale (mai visto su ORM).
- **R2 (braccio di controllo)**: ogni A/B della metà allocatore di A3 include
  il braccio mi_heap-per-richiesta (FFI già in-tree) oltre a mimalloc default.
- **R3 (ri-ancoraggio)**: i TEMPI del census A1 decadono al primo pin
  post-refactor; prima di committare A3, replica CORTA dei canali su pin
  post-A2 (i conteggi valgono, i ns no).
- **R4 (gate footprint)**: banda peak vmmap Physical pre-registrata (WP+ORM,
  1 gamba) nei gate di ogni tranche A2 e di ogni atto A3.
- **R5 (inline-N dai numeri)**: la costante 8 di InlineProps non si fissa nel
  design: esce da N2 (p90 pesato), con tassa Δbytes dichiarata nel criterio.

## Kill-switch pre-registrabili
- **KS-LE-151-1**: N1: (totale allocato per richiesta ORM × sizeof slot) >
  1,5× peak footprint s150 ⇒ bump-only MORTO, solo slab/free-list.
- **KS-LE-151-2**: N3 sotto la banda del rumore del giudice ⇒ la metà
  allocatore di A3 si RITIRA; A3 procede solo come rimozione RefCell/refcount.
- **KS-LE-151-3**: N2: p90 pesato >8 oppure Δbytes inline >+10% del canale obj
  ⇒ inline-8 ridimensionato o refutato.
- **KS-LE-151-4**: tranche A2 con peak fuori banda R4 ⇒ tranche ferma,
  istruttoria prima della successiva.
