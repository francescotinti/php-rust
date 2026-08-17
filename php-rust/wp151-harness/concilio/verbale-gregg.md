# VERBALE GREGG — S-151 (lente: metodologia di misura e attribuzione + mandato inverso)

## MANDATO INVERSO — cosa sappiamo di phpr OGGI che ieri non sapevamo
1. L'aggregato per-TAG nascondeva UN nome al 73,95% (debug_backtrace, s149):
   la granularità d'attribuzione batte l'aggiunta di canali; la leva è uscita
   dal NOME.
2. Il TETTO movimenti 1,27 s è FIRMATO ma oggi è STALE: fu misurato @ s145
   sotto un profilo in cui debug_backtrace pesava 81,9% di hostcall.n
   (n=275,0M). Post-BT1 quel profilo NON esiste più: né il tetto né la testa
   s147–s149 sono citabili senza rifondazione @ s150.
3. Il census è DETERMINISTICO a replica (0,000%, s147/s149): esiste quindi un
   gate nuovo — l'IDENTITÀ DI CONTEGGIO pre/post edit — più forte dei test e
   legale dove la byte-identità è vietata.
4. Il census è sensibile all'AMBIENTE (+3,2% hostcall.n da lunghezza path del
   workdir, candidato s150 p.2): i confronti cross-probe ereditano scarti di
   perimetro; solo l'identità intra-run è esatta.
5. Un'attesa con PAVIMENTO dichiarato rende l'oltre-attesa leggibile (S-150):
   ogni banda A3 dichiari i componenti NON prezzati.

## VERDETTO: CONCORDO CON EMENDAMENTI (sull'impianto A1..A4)

## Q1 — sequenza A1→A2→A3
Giusta, a DUE condizioni. (i) I siti censiti sono op del dispatch, non righe
di file: sopravvivono al refactor ⇒ census-prima è valido. Ma il probe è un
DIFF a sorgente: dopo A2 non ri-applica — la rifondazione probe va fatta @ s150
PRIMA di A2 (com'è nell'ordine) e MAI a metà refactor. (ii) I numeri A1
invecchiano 4–6 sessioni prima di A3: restano validi SOLO se ogni tranche A2
passa l'identità di conteggio (R4) — altrimenti A3 parte su numeri di un
motore che non esiste più. Interleaving ammissibile solo per memory/exec con
gli stessi gate; chirurgia-prima REFUTATA: senza census post-BT1 la testa
nuova è ignota (none.other 94,6M mai nominato).

## Q2 — gate delle tranche A2 (cosa sostituisce la byte-identità)
- Gate per tranche: batteria + corpus 1412×2 per NOME + fixture bilaterali +
  micro R=5 + disasm bl di run_loop (invariante atteso per tranche non-exec,
  delta DICHIARATO per tranche exec) + **IDENTITÀ DI CONTEGGIO census** (R4).
- Coppia WP: la regola utente ⚖️ 2026-08-12 dice «coppia a OGNI pin nuovo».
  Ogni tranche promossa = pin nuovo ⇒ coppia dovuta. Il piano 4–6 sessioni è
  in CONFLITTO col costo di 4–6 coppie: il conflitto si scioglie solo con
  deroga ESPLICITA dell'utente (es. coppia a tranche alterne + coppia finale),
  non in concilio. Va chiesta, non cablata.
- «Rischio zero» di Gemini (Fase 1) è NON firmato: spostare funzioni cambia
  partizione CGU e inlining ⇒ il perf-rischio esiste, coperto da micro+disasm+coppia.
- Partizione a rischio minimo: prima i satelliti foglia (dom, host/reflect,
  constants), run_loop/exec per ULTIMO e in tranche proprie.

## Q3 — A1 come partizione, A3 come decisione
- I 5 canali (clone-per-SITO/drop/gc_note/refcount/borrow) NON sono oggi una
  partizione: non esiste un totale-padre di cui siano fette. clone conta già
  le Rc::clone via Zval; un canale «refcount» separato (Rc::clone dirette,
  borrow di prop) DOPPIO-CONTEREBBE senza mappa di disgiunzione. Serve: (a)
  identità per-canale Σsiti==tot dallo stesso hook (come s147mv_tot==
  s145_clone_tot); (b) contatore di OVERLAP tra canali, atteso 0, stampato
  (come unnamed/overflow s149); (c) legge di CONSERVAZIONE cross-canale:
  nascite+cloni == drop+vivi_a_fine_request, per classe — è l'unica identità
  che lega i canali tra loro. Senza (a)+(b)+(c) i cinque numeri non sommano a
  niente di verificabile.
- Probe/inlining (veto in registro): il census è CONTEGGI mai tempo, quindi il
  veto morde altrove — ma lo stato di FUSIONE del probe cambia QUALI siti
  eseguono (s147 girò fused=false, «tetto su binario census»): lo stato va
  dichiarato nella ricetta e tenuto COSTANTE tra tranche-5 e ogni ri-census.
- DECIDIBILITÀ A3: oggi NON è decidibile da A1 da solo, per due assenze.
  (1) Prezzi: la sonda s145 prezza solo la coppia clone; drop/gc_note/borrow/
  refcount non hanno prezzo ⇒ conteggi non convertibili in banda (R2).
  (2) Sostitutivo: il veto «alloc-removal senza costo SOSTITUTIVO» vale —
  vm.objects[id] paga bounds-check+indirezione+generazione. Disuguaglianza
  GO/NO-GO pre-registrabile (R3): banda_netta = Σ_canali eventi_eliminabili ×
  (prezzo_canale − prezzo_sostitutivo_mock) ≥ 2× soglia, con soglia RIDERIVATA
  su ORM s150 (0,7% × ~35,5 s ≈ 0,25 s) e PAVIMENTO dichiarato (località/
  branch/I-cache NON prezzabili a conteggi — è lì che vive l'eventuale
  oltre-attesa, non nella banda). Sotto 2×: solo fette micro-judged; la
  tensione Gemini↔tetto si scioglie così, per misura: il tetto 1,27 s copre
  SOLO il canale clone-movimenti — i canali borrow/refcount/drop che Gemini
  aggrega nel «30–45%» non sono mai stati né contati né prezzati da noi.
  Nessuna cifra Gemini è citabile in un criterio (R6).
- Semantica (altre sedie; il census la arma): fixture pre-registrate su
  __destruct order/weak/spl_object_id — spl_object_id conta 365k nell'ORM:
  il riuso di indici è OSSERVABILE ⇒ store generazionale obbligatorio.

## Q4 — dente A4
Sede: BATTERIA (morde a ogni promozione; CI con backlog ~3 giorni non morde
in tempo — CI solo companion). Forma: RATCHET per-file meccanico (wc -l):
legacy congelati al conteggio corrente per NOME (mai crescere), file nuovi
≤2.000; eccezioni nel test, emendabili solo DICHIARANDO. Anti-auto-morso
(lezione bea7ea3): collaudo in NEGATIVO (file oltre soglia ⇒ rosso) prima
dell'ingresso in batteria.

## Q5 — cosa manca dall'ordine (invalida il resto se trascurato)
1. La SONDA-PREZZI per canale (R2): senza, A1 produce conteggi non
   convertibili e A3 resta indecidibile — l'ordine la omette.
2. La soglia 0,7% va RIDERIVATA sulla coppia ORM s150 (denominatore 41,9→35,5 s
   ⇒ 0,293→~0,25 s): usare la vecchia = denominatore a memoria (veto).
3. La deroga-coppia per A2 va chiesta all'utente PRIMA della tranche 1 (Q2).
4. Il micro-mock del costo sostitutivo store-indicizzato (INDIZIO) va
   costruito in fase A1, non a chirurgia iniziata.

## EMENDAMENTI
- R1 (A1): identità per-canale + overlap atteso 0 + legge di conservazione
  nascite+cloni==drop+vivi, per classe; violazione ⇒ verdetto NULLO.
- R2 (A1): sonda-prezzo pair-style PER canale (drop/gc_note/borrow/refcount);
  senza prezzo, il canale non entra in nessuna banda.
- R3 (A3): GO/NO-GO = banda_netta (eventi × prezzo netto sostitutivo-mock)
  ≥ 2× soglia s150-riderivata, pavimento dei non-prezzati DICHIARATO.
- R4 (A2): gate per tranche = identità di conteggio census (0,000% replica ⇒
  == atteso) + disasm bl run_loop + micro R=5 + batteria/corpus/fixture.
- R5: nessuna cifra census s147–s149 (tetto 1,27 s incluso) citabile pro o
  contro A3 prima della rifondazione @ s150.
- R6: cifre Gemini (30–45%/−35%/40%) mai in criteri: ipotesi da firmare R1+R2.
- R7 (probe): ricetta ESATTA con env e LUNGHEZZA PATH del workdir controllata;
  stato fusione dichiarato e costante; smoke a esito esatto.
- R8 (A4): ratchet per-file in batteria, collaudo in negativo, CI companion.

## KILL-SWITCH PRE-REGISTRABILI
- KS-G1: overlap tra canali > 0 o Σsiti≠tot ⇒ census NULLO, riconvoca.
- KS-G2: banda_netta A3 < 2× soglia s150 ⇒ chirurgia declassata a fette
  micro-judged; nessun blocco multi-sessione dedicato.
- KS-G3: identità di conteggio violata su una tranche A2 ⇒ la tranche NON è
  refactor puro: revert, o riclassifica come leva con A/B proprio.
- KS-G4: micro fuori banda o disasm run_loop mutato su tranche non-exec ⇒
  stop, causa nominata prima di procedere.
