# TEAM-MISURA (relatore) — riconciliazione GREGG · BAK · LEIJEN — S-151 fase 2
Verdetti individuali: 3× CONCORDO CON EMENDAMENTI. I verbali restano la fonte vincolante.

## §Convergenze (con attribuzione)
1. **Sequenza A1→A2→A3 giusta** a condizioni: probe rifondato @ s150 PRIMA di A2 e mai a metà
   refactor (Gregg Q1); numeri decisionali per CANALE specie×operazione, che sopravvivono al
   refactor, inventario per-SITO deperibile (Bak Q1); funnel in php-types non toccati da A2
   ⇒ conteggi sopravvivono, TEMPI no (Leijen Q1). Chirurgia-prima refutata (Gregg Q1).
2. **Cifre Gemini mai in un criterio** (30–45% / −35% / 40%): Gregg R6, Bak R1, Leijen ref.6.
   Tensione tetto↔Gemini risolta CONTRO Gemini: col refcount esplicito (vincolo semantico) il
   guadagno da movimento resta cappato; il caso A3 vive sui canali non-movimento (Bak fisica;
   Gregg Q3: canali borrow/refcount/drop mai contati né prezzati da noi).
3. **«Rischio zero» Gemini Fase 1 respinto agli atti**: precedente FR1 +3,00 ns / +3180 B /
   +26 bl da solo layout (Bak Q2, Leijen Q1); CGU/inlining mutano (Gregg Q2).
4. **Nessun canale entra in una banda senza PREZZO** (aritmetica prezzo×conteggio): Gregg R2,
   Bak Q3 (lezione HC1), Leijen N3. Pavimenti dichiarati e bilateralità minima oracle:
   Gregg R3, Bak R7, Leijen N3.
5. **Soglia GO/NO-GO A3c pre-registrata PRIMA di leggere il census**: Gregg R3, Bak Q5-3/KS-2,
   Leijen KS-LE-151-2. Sotto soglia: A3c chiusa stile veto NaN-boxing, restano fette
   micro-judged / A3a+A3b (Gregg KS-G2 ≡ Bak KS-2).
6. **Caccia all'outlier per-NOME dentro A1, prima di aprire A2** (BT1 era outlier algoritmico,
   non memory-model): Bak R6/Q5-1, Gregg mandato-inverso 1, Leijen Q5. Teste: none.other
   94,6M · class_exists 9,7M · __reflect_* 12,4M.
7. **Dente A4**: sede BATTERIA (CI backlog ~3 gg = companion/advisory), nuovi ≤2.000, ratchet
   per-NOME sugli esistenti mai-crescere/al-ribasso, meccanica wc -l/fs (mai pattern testuali,
   bea7ea3), collaudo in NEGATIVO prima dell'ingresso (Gregg R8+Q4, Bak R5/Q4, Leijen Q4).
8. **Gate tranche A2 (unione, tutti d'accordo sul nucleo)**: batteria + corpus 1412×2 per NOME
   + fixture bilaterali + micro R=5 banda-layout + disasm bl run_loop prima/dopo come
   sostituto della byte-identità (Gregg R4, Bak R4) + identità di conteggio census (Gregg R4)
   + gate footprint peak vmmap (Leijen R4). Ordine tranche: freddo/foglia prima, raggio
   d'inlining di run_loop per ultimo o mai (Gregg Q2, Bak Q2).
9. **Staleness**: nessuna cifra s147–s149 rifondabile senza replica @ s150; tempi/prezzi
   decadono al primo pin post-refactor ⇒ replica CORTA pre-A3 (Gregg R5, Leijen R3).
10. **Scarto +3,2%**: sensibilità ad ambiente/lunghezza path (Gregg m.i.4, R7); se non
    istruito entro la sessione census ⇒ numeri A1 declassati a INDICATIVI (Bak KS-3).

## §Conflitti-e-dissensi
- **C1 — citabilità del TETTO 1,27 s oggi**: Gregg R5 lo dichiara STALE (misurato @ s145 sotto
  profilo debug_backtrace 81,9%, non citabile pro/contro A3 prima della rifondazione); Bak R1
  e Leijen ref.6 lo usano come cap vigente. Sintesi di team: il tetto resta IPOTESI direzionale
  (Bak Q3-1 chiede lui stesso «conferma di tranche-5») ma NON entra in criteri finché
  tranche-5 @ s150 non lo rifonda. Dissenso registrato, governa Gregg R5 sui criteri.
- **C2 — soglia A3c, due formule**: Gregg R3 = banda_netta (eventi × prezzo netto
  sostitutivo-mock) ≥ 2× soglia riderivata s150 (0,7%×~35,5 s ≈ 0,25 s ⇒ ≥ ~0,5 s);
  Bak Q3 = UB(canali 2+4) ≥ 5% del gap ORM su binario census (~2× miglior leva storica).
  NON equivalenti. Proposta del relatore: pre-registrare UNA soglia = la PIÙ severa delle due
  (GO solo se entrambe soddisfatte); la scelta finale spetta a main/utente PRIMA del run.
- **C3 — coppia WP per tranche A2**: Bak «a ogni pin nuovo, non negoziabile»; Gregg concorda
  ma nomina il conflitto col costo di 4–6 coppie ⇒ deroga da CHIEDERE all'utente prima della
  tranche 1, mai cablata (Gregg Q2/Q5-3); Leijen legge «solo a pin nuovo, non per tranche».
  Sintesi: regola vigente intatta; la deroga è quesito all'utente (posizione Gregg).
- **C4 — perimetro A2**: Bak R3 = perimetro MINIMO (solo moduli che A3 toccherà + dente),
  riduce le sessioni-senza-leva; il MANDATO dice 4–6 sessioni accettate. Emendamento all'ordine
  da ratificare dall'utente. Gregg/Leijen non si oppongono.
- **Dissensi di lente (non conflitti)**: Leijen R1 refuta bump-reset (metà allocatore Gemini)
  e impone free-list/slab salvo N1 live≈totale; Leijen R2 braccio mi_heap obbligatorio negli
  A/B allocatore; Bak R8 re-entrancy e weak-table generazionale prezzate nel criterio A3c;
  Bak R2 scomposizione A3a/b/c promo-gated. Nessuna sedia contesta.

## §Spec-census-esatta (ciò che il team ESIGE dal tranche-5 @ s150)
- **Canali** (partizione con DISGIUNZIONE dimostrata, Gregg Q3): C1 clone/drop handle Object
  per sito (rifonda il tetto) · C2 borrow/borrow_mut Object per sito · C3 malloc/oggetto a
  costruzione (Rc header+Vec props+dyn) · C4 gc_note con arg Object per sito · C5 clone/drop
  VALORI Zval a PropGet/PropSet (Bak Q3 1–5). Più: testa hostcall per-NOME nuova + scan
  outlier none.other 94,6M (Bak R6).
- **Identità obbligatorie** (violazione ⇒ census NULLO, KS-G1): Σsiti==tot per canale dallo
  stesso hook; contatore OVERLAP tra canali atteso 0 e STAMPATO; legge di conservazione
  nascite+cloni==drop+vivi_a_fine_request, per classe (Gregg R1).
- **Numeri Leijen**: N1 totale vs picco-live per classe/workload · N2 distribuzione #props
  PESATA (p50/p90/p99, %≤4/≤8, dyn_entries; fissa inline-N, Leijen R5) · N3 quota CPU
  allocatore bilaterale netta-pavimenti · N4 bytes/oggetto dal binario · N5 baseline peak
  vmmap WP+ORM @ s150 · N6 vivi a request_shutdown.
- **Prezzi**: sonda pair-style per canale drop/gc_note/borrow/refcount (Gregg R2, senza prezzo
  il canale non entra in banda); micro-mock costo sostitutivo store-indicizzato costruito IN
  A1 (Gregg Q5-4); prezzo re-entrancy (Bak R8); braccio mi_heap-per-richiesta (Leijen R2).
- **Ricetta probe** (Gregg R7): env esplicito, lunghezza path workdir controllata, stato di
  FUSIONE dichiarato e costante tra tranche-5 e ogni ri-census, smoke a esito esatto.
- **Gate A2 derivato**: identità di conteggio sui TOTALI per canale pre/post tranche
  (0,000% replica ⇒ ==); per-sito libero di rimescolare (sintesi Gregg R4 + Bak Q1).

## §Priorità-per-l'ordine-S-151+
1. PRIMA del run census: pre-registrare soglia A3c (sciogliere C2, main/utente) + criteri
   GO/NO-GO + pavimenti non-prezzati dichiarati.
2. Rifondazione probe @ s150 con ricetta R7; dentro A1: sonde-prezzo, mock sostitutivo,
   scan outlier per-NOME, istruzione scarto +3,2%, baseline footprint N5.
3. Quesiti all'utente prima della tranche-1 A2: deroga-coppia (C3) e perimetro minimo (C4).
4. Dente A4 in batteria subito (ratchet + collaudo in negativo), CI companion.
5. Solo DOPO A1 rifondato: ordinare A3a/b/c coi numeri (Bak R2); A3c resta sotto kill-switch
   KS-G2/KS-2; tempi da replicare corti sul pin pre-A3 (Leijen R3).
