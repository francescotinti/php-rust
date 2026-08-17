# Criterio S-152 p.1 — sonde-prezzo canali C1–C5 + mock store-indicizzato → GO/NO-GO A3c (pre-registrato PRIMA di build e run)

1. Oggetto: prezzo PROPRIO (ns/evento) dei canali del census tranche-5
   (conteggi VALIDI da `wp151-harness/s151-census-verdetto.out`, per replica)
   e prezzo del SOSTITUTIVO mock store-indicizzato (forma A3 ratificata:
   bucket+free-list, handle con incref, gen-check). Probe monobinario
   `--features sonda-price` dalla ricetta pin A′ (stessa del candidato s150
   cbbe71735effb165: SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0, lto=fat CGU=1),
   CARGO_TARGET_DIR dedicato `/private/tmp/phpr-sonda152-target` (la build
   canonica del pin NON viene toccata); hash probe REGISTRATO nel verdetto;
   probe conservato ×2 (wp152-harness/sonda-prep/ + stash) con path+hash.
2. Segmenti NUOVI `s152.price.*` in `vm/sondaprice.rs` (feature-gated, il
   binario di parità non li compila; s145/s149 RESTANO e si ri-emettono —
   mv_*/note_* sono i prezzi correnti C5/C4). N_MV=200M leggeri, N_PAIR=20M
   con alloc:
   - `c1_pair_ns`: coppia `Rc::clone`+drop dell'handle Object REALE (N_MV);
   - `c2_borrow_ns` / `c2_borrow_mut_ns`: borrow()/borrow_mut()+drop guard
     sull'handle reale (N_MV);
   - `c3_size_pair_ns`: coppia malloc+free alla taglia
     `size_of::<RefCell<Object>>()` via `Vec::<u8>::with_capacity` (N_PAIR;
     PROXY dichiarato del solo header+cella: l'init dei campi e le props sono
     INVARIANTI tra i mondi e non entrano nel netto); `obj_size` ad audit;
   - `mock_deref_ns`: store `Vec<MockSlot{gen,rc,payload}>`, accesso
     idx+gen-check su slot CALDO (N_MV) — LB OTTIMISTICO del sostitutivo
     (nessun modello cache/working-set: DICHIARATO, governa il §6);
   - `mock_dup_rel_ns`: coppia incref+decref `Cell<u32>` sullo slot (N_MV);
   - `mock_decq_ns`: push dell'id su coda decrementi `Vec<u32>` riusata,
     clear ammortizzato incluso (N_MV) — il drop sostitutivo (Matsakis);
   - `mock_alloc_ns`: pop+push free-list con gen-bump (N_PAIR) —
     l'alloc sostitutivo di C3;
   - braccio mi_heap (Leijen R2): `miheap_pair_ns` = mi_heap_malloc+free a
     taglia Object su heap dedicato (libmimalloc-sys/extended SOLO nella
     feature sonda-price). Se il binding non monta in finestra: slittamento
     DICHIARATO non-gate (C3=6,4M eventi non muove S1–S3, v. §6).
3. Giudice: chiavi dal file `PHPR_SONDA_OUT`; R=2 repliche + smoke; banda per
   chiave = [min,max] delle 2 repliche; replica >2% su una chiave usata in
   decisione ⇒ terza replica dovuta; prezzo citato SEMPRE come intervallo.
4. Collaudo-nell'atto: smoke a esito ESATTO (stdout `SONDA-OK` E chiavi s145
   11/11 E s149 6/6 E s152 tutte presenti, >0 dove prezzo); gli ATTESI dello
   smoke sono verificati da un SECONDO attore PRIMA del run di record
   (rev. az.4 S-151). Smoke fallito ⇒ STOP rc=8, nessun run.
5. Igiene: misura di TEMPO ⇒ lock `/private/tmp/phpr-measure.lock` di
   finestra + quiescenza in retry (mutex CI) + sentinelle busy stampate.
6. ARITMETICA GO/NO-GO (soglie GIÀ registrate s151-criterio-census.md §6,
   D_gap=[30,52;30,56] s; conteggi per replica dal verdetto census):
   - banda_netta = Σ canali NON-movimento (C1 ESCLUSO, concilio §2):
     · C2: 340,9M × (mix borrow/borrow_mut − mock_deref); quota borrow_mut
       ∈ [11,8M; 60,4M] dal top-10 (residuo 48,6M non attribuito: agli
       estremi della banda, DICHIARATO);
     · C4: 43,2M × (note_cont_repeat − mock_deref) [flag nello slot];
     · C5: SOLO specie obj cambia col sostitutivo (str/arr/scalar clone
       identici nei due mondi): quota obj ∈ [105,8M; 157,4M] dal top-10
       (residuo 51,6M agli estremi, DICHIARATO) × (mv_obj − mock_dup_rel);
     · C3: 6,4M × (c3_size_pair − mock_alloc).
     Prezzi min/max delle repliche → banda_netta = [lo, hi].
   - S3 «somma canali Object» = somma LORDA prezzo_corrente×conteggio su
     C1+C2+C3+C4+C5 (C1 al lordo con c1_pair; per C5 l'intero canale 191,2M
     al prezzo mv per specie, mix agli estremi come sopra) — lettura del §6
     DICHIARATA qui (il §6 non la specifica; C1 resta comunque fuori dalla
     banda_netta pro-A3).
   - GIUDIZIO pre-registrato: (a) se banda_netta_hi (estremo più favorevole
     ad A3, già costruito su mock LB-ottimistico) fallisce UNA QUALUNQUE tra
     S1≥0,50 s · S2≥1,53 s, oppure S3_hi<4,58 s ⇒ **NO-GO ROBUSTO a
     fortiori: A3c CHIUSA** (stile veto NaN-boxing; restano A3a/A3b
     micro-judged e le leve per NOME); (b) se banda_netta_lo E S3_lo superano
     TUTTE le soglie ⇒ GO (col caveat mock-ottimistico: il criterio A3c
     pieno dovrà prezzare cache/working-set, re-entrancy e weak-table —
     Bak R8 — PRIMA di ogni codice); (c) a cavallo ⇒ INDECIDIBILE alla
     risoluzione: terza replica + mock working-set census-like (~180k slot)
     DOVUTI prima del verdetto, nessun GO/NO-GO dichiarato prima.
7. Componenti NON prezzati, DICHIARATI: sovrapprezzo first-note gc (s145);
   costo cache/working-set del mock (HOT = LB); re-entrancy e weak-table
   (dovuti nel criterio A3c SE GO); il netto C2 presume il borrow sostituito
   da accesso via `&mut Vm` + gen-check (= mock_deref); pavimenti di specie
   C5 e quota borrow_mut agli estremi come da §6.
8. Esiti pre-registrati: NO-GO ⇒ A3c chiusa a verbale e il peso S-152 passa
   a pesca-outlier (atto 2) e touch-map A2 (atto 3); GO ⇒ criterio A3c
   pieno pre-registrato in sessione dedicata; INDECIDIBILE ⇒ atti del §6c
   poi rigiudizio, senza toccare le soglie.
