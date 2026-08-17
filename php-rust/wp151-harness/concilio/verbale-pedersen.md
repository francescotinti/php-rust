# VERBALE — sedia PEDERSEN (S-151, Fase 1 indipendente)
Lente: confine per-richiesta/per-test e lifecycle (RetainSet, request_end,
sweep distruttori, server persistente).
**VERDETTO: CONCORDO CON EMENDAMENTI** (A1..A4 ratificabile SOLO coi
vincoli R1–R8; senza R2/R3 sarei OPPOSTO ad A3).

## Fatti verificati a codice (non a memoria)
- Server: per richiesta = RetainSet FRESCO + Vm FRESCA; isolamento = MORTE
  della Vm (A-MS6/A-DS2, `worker_pool.rs:329-331`); il RetainSet pinna SOLO
  bytecode, MAI lo store oggetti (KS-DS-78-4).
- Ordine osservabile (`vm/mod.rs:3746-3781`): shutdown_fns → flush OB →
  dtor-walk → session flush → filtered streams → break_request_cycles.
  Capture output a `worker_pool.rs:996` (A-PP1) PRIMA di `request_end()`
  (:1000) che azzera stdout/rendered — il binding vive lì.
- Dtor-walk (`oop.rs:1082-1210`): Fase A globali reverse con osservatore
  `strong_count == 2 + extra` (slot + registro `created` + clone gc_buf);
  Fase B walk in ordine di creazione, round su snapshot di Rc FORTI (un
  dtor può ancora raggiungere un peer), dtor-spawned ripresi (d2/d4).
- `next_id` (`vm/mod.rs:3660-3682`): riusa gli id liberati, SCRUBBA 7
  tabelle per-id; `next_object_id=1` a ogni request_end; gate used_n=0 su
  `live_objects()`=`created.len()` (KS-DS-78-2).
- **VERIFICATO ASSENTE**: nel crate php-server NESSUN test `__destruct` né
  `spl_object_id` — output dei dtor e riuso id SENZA gate server-mode.

## Q1 (sequenza)
A1→A2→A3 regge; argomento in PIÙ per census-first: i conteggi per-canale
sono l'impronta comportamentale che arma il gate delle tranche A2 (R5).
Vincolo: le tranche che toccano `request_shutdown`/`oop.rs` preservano
l'ORDINE del teardown (semantica oracle-verified, non struttura).

## Q2 (gate delle tranche A2)
Batteria (include i test worker_pool) + corpus 1412×2 per NOME + fixture
bilaterali + micro R=5. Sostituto della byte-identità (vietata): **identità
ESATTA dei conteggi census per canale su workload fisso** (una mossa pura
non cambia un conteggio; precedente repliche 0,000%) + disasm bl-count dove
la tranche tocca run_loop. Coppia WP «a OGNI pin nuovo» = legge UTENTE
(2026-08-12): il concilio NON può diradarla — o ogni tranche la paga, o si
chiede all'utente deroga esplicita (proposta: tranche accorpate, 1 pin/
sessione).

## Q3 (A3: punti di rottura del ciclo di vita + copertura)
1. **Soglie osservatore Fase A**: `2+extra` codifica la topologia di
   possesso ATTUALE (slot+registro+gc_buf); lo store la cambia ⇒ OGNI
   soglia RI-DERIVATA, non tradotta. Coperto: d2, droporder, corpus.
2. **Fase B con `&mut Vm`**: oggi lo snapshot Rc tiene vivi i peer del
   round; con refcount esplicito lo snapshot deve PINNARE (refcount++) e lo
   store CONGELARE il riuso slot durante il walk. Coperto in parte (d4);
   **MANCA**: fixture «dtor libera un peer non ancora walkato».
3. **Confine capture**: l'output dei dtor fluisce in `rendered`, catturato
   PRIMA di request_end. Se A3 sposta il teardown dello store in `Vm::drop`
   o in `request_end` ⇒ output dtor PERSO in silenzio = violazione del
   binding. **MANCA il gate**: server due-richieste con `__destruct` che
   stampa, parità al byte (R2).
4. **used_n=0**: A3 sostituisce `created` ⇒ KS-DS-78-2 RI-PUNTATO al
   live-count dello store, firma mass-teardown riprovata.
5. **spl_object_id/riuso id**: arena generazionale (necessaria contro ABA
   degli ObjectId stantii) NON deve far trapelare la generazione nell'id
   PHP-visibile; gli scrub per-id di `next_id` vanno rimappati. **MANCA**:
   fixture server «id ripartono da 1 alla seconda richiesta».
6. **Weak**: `teardown_weaks`=Rc::downgrade; il break dipende da
   upgrade-dopo-drop-dello-store-forte, concetto che nello store sparisce ⇒
   ri-esprimere come liveness (refcount>0); serve weak-count esplicito
   (WeakReference/WeakMap coperti dal corpus; teardown_note ≠ gate).
7. **Busy-cell K-M72.2**: senza RefCell `try_borrow_mut` Err non esiste ⇒
   ogni cella prendibile: delta osservabile possibile (più props prese, più
   drop di risorse). DICHIARARE + fixture risorsa-in-prop-ciclica.
8. **Residenza dello store**: il «reset del puntatore dell'arena» (Gemini)
   come RECLAMO di fine richiesta è INAMMISSIBILE come sostituto del
   teardown osservabile; store per-WORKER = KS-DS-78-4 aggravato + primo
   passo verso il riuso Vm. VINCOLO: **store = campo della Vm**.
- **Numeri per la DECIDIBILITÀ di A3** (e pronuncia sulla tensione TETTO):
  il tetto 1,27 s ≈ 3,4% cappa il SOLO canale movimenti ⇒ «azzerare il
  movimento ripaga» (Gemini) è già refutata su quel canale; A3 è decidibile
  SOLO se il census prezza SEPARATAMENTE i canali non-movimento (clone per
  sito, drop, borrow-check, refcount-op) e la scommessa si fonda su QUELLI.
  In più: census ANCHE su workload server multi-richiesta col teardown per
  richiesta (live a shutdown, round dtor, teardown_note reg/broken/busy/
  alive_after — già emessi): A3 ridisegna il teardown, il prezzo si misura
  PRIMA.

## Q4 (dente A4)
Sede: BATTERIA (morde a ogni promozione; la CI ha backlog ~3 giorni),
pre-commit al più advisory. Cap DURO ~2.000 sui file NUOVI; sui monoliti
ratchet DECRESCENTE agganciato alle tranche A2 (ogni tranche abbassa il
tetto del file che spacca). Conteggio meccanico (righe), non pattern.

## Q5 (cosa manca dall'ordine)
(a) I gate di confine server per NOME (R2, R3) non esistono: A3 riscrive
quella macchineria alla cieca. (b) Residenza dello store non pre-registrata
(R1). (c) Il census S-151 com'è ordinato è CLI-only: i costi di confine
restano non prezzati (R4). Fatto nuovo (Gregg): l'isolamento per-richiesta
è comprato con la MORTE della Vm, non con un reset — A3 non lo baratti.

## Emendamenti
- **R1**: pre-registrare: ObjectsStore = campo della Vm, muore con la Vm;
  nessun riuso cross-richiesta senza concilio (assert !persistente).
- **R2**: NUOVO gate server due-richieste con `__destruct` che emette
  output: parità al byte, capture prima di request_end. PRIMA di A3.
- **R3**: NUOVE fixture server riuso-id (id ripartono, per-id scrubbate) +
  «dtor libera peer non walkato». PRIMA di A3.
- **R4**: A1 include una gamba server multi-richiesta col teardown
  per-richiesta prezzato; canali movimento/non-movimento separati.
- **R5**: gate tranche A2 = identità esatta conteggi census per canale su
  workload fisso (sostituto della byte-identità vietata).
- **R6**: used_n=0 ri-puntato al live-count dello store in ogni build A3.
- **R7**: deroga coppia-WP per la finestra A2 chiesta all'UTENTE, mai
  decisa dal concilio.
- **R8**: ordine teardown (fns→OB→dtor→session→streams→break) dichiarato
  INVARIANTE di semantica per la durata di A2+A3.

## Kill-switch pre-registrati
- KS-P1: store dichiarato fuori dalla Vm (worker/static/thread_local) →
  tranche respinta, concilio obbligatorio.
- KS-P2: live-count store ≠ 0 post-request_end su gate server → no stash.
- KS-P3: parità due-richieste (incluso output dtor) diverge → promozione
  respinta.
- KS-P4: tranche A2 con conteggi census non identici → «mossa pura»
  falsificata → revert della tranche.
- KS-P5: id PHP-visibile diverge dall'oracle sulla fixture riuso-id →
  schema id di A3 si riprogetta prima di ogni altra riga.
