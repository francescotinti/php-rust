# GREGG — sedia metodologia di misura/attribuzione (bozza indipendente, S-167)

## §COSA-SAPPIAMO-OGGI (mandato inverso)
1. **Borrow in-contesto ≤~1 ns** (L-TD1, S-153): ogni rotta «togli i borrow» è già esclusa.
2. **Alloc mimalloc su fast path ≈0 ns** (L-AL3, S-164, census esatto): pooling/arena come classe di leva PURA è esclusa; R3-arena non può pagare per il canale alloc.
3. **Banda-layout fondata** (S-165: missload 8,0 · arrfilter 6,0, bracci nulli): sotto ~5-8 ns/iter un D su giudice host-call non è un segnale senza braccio nullo — il metodo ORA sa distinguere leva da layout.
4. **Quantizzazione del giudice** (S-164): arith è FERMO davvero (5,42-5,45), il «creep» era tick. Il fermo di arith è un fatto misurato, non un'impressione.
5. **Il pattern che paga**: 100% delle promozioni ha semplificato il dispatch/cammino (5-40 ns/sito); zero promozioni da spostamento di memoria. Nel modello prop il residuo NOMINATO è dispatch 36,3.
6. **A3c NO-GO prezzato** (S-152/153): fette Zval sui canali Object sotto soglia.
Conclusione inversa: il sapere accumulato **esclude già** R3 (canale alloc), le fette borrow/pooling di R2, e ogni fetta con D atteso < banda-layout senza braccio nullo. L'unica rotta coerente con TUTTE le evidenze è **R1 dispatch** — ma su arith è un'analogia, non una cifra.

## VERDETTO: **GO-CONDIZIONATO**
Rotta (una riga): campagna SOLO R1-dispatch, subordinata alla firma del modello-del-tempo arith bilaterale; R2/R3 restano chiuse salvo cifra nuova dal modello.

Il dossier ammette: arith non ha decomposizione. Deliberare GO strutturale senza sonda = allocare sessioni su un'attribuzione presa in prestito da prop. La PRIMA FETTA è obbligata ed è la sonda stessa.

**Forma firmabile del modello arith** (come S-129/131):
- **chiusura ≥90%**: somma componenti nominate entro il 10% dei 46,8;
- **per-passo**: bracci SOTTRATTIVI pre-registrati (loop nudo, fetch operandi, op, write-back Zval, dispatch), mai profilo aggregato;
- **bilaterale**: stessa scomposizione sull'oracle (8,6) ⇒ gap PER-COMPONENTE (feedback-one-sided);
- risoluzione dichiarata (tick 0,5 ⇒ R alto/valori veri); **PHPR_REG_LOWER on E off** (quanto del dispatch sopravvive al reg-lowering è quesito aperto del dossier).

## EMENDAMENTI
1. Nessun codice R1 prima della firma del modello arith.
2. Ogni fetta futura con |D| atteso <8 su giudici host-call pretende braccio nullo; fondare banda-layout anche per categorie arith/prop.
3. Il «dispatch 36,3» di prop si rimisura con reg-lowering on/off prima di dimensionare la campagna.
4. Verdetti mai su tick: sempre valori veri a risoluzione dichiarata.

## KILL-SWITCH
Se il modello attribuisce a dispatch+decode <40% del gap arith (38,2 ns/iter), o chiusura <90% dopo 2 sessioni ⇒ campagna R1 SOSPESA, concilio su R4 (ridefinizione onesta).

## PRIMA FETTA
Giudice: m-arith (46,8 vs 8,6) + bracci sottrattivi pre-registrati su ENTRAMBI i motori. Soglia di firma: chiusura ≥90% bilaterale, componenti a errore ≤ banda propria del braccio, doppia corsa reg-lowering on/off.
