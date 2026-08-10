# Revisione S-126 — lente SEMANTICA

## Reperto principale
La nomina L-OL1 non discende dal criterio pre-registrato. P.6 nomina «la categoria con rapporto MASSIMO se >6 E visibile nel profilo»: il massimo è evalcls (316,9), NON visibile; il caso «massimo non visibile» non è coperto, e p.8 lo risolve DOPO aver visto i verdetti spostando la nomina su objchurn. Il «quindi» del claim è post-hoc, non lettura pre-registrata. Aggravante semantica sul «67%»: presuppone additività mai verificata — objalloc+objmap = 1393 ns vs churn 1820: 427 ns/iter (23%) NON attribuiti (insert `data['k']` + drop DIFFERITO via overwrite dello slot di `$map`, sentiero di drop DIVERSO dal drop immediato per riassegnazione di objalloc). Il 67% estrapola tra micro con semantiche di drop diverse.

## Reperti secondari
1. hf: «diff 17 nomi = famiglia php -S» è falso per ≥3 nomi (testIntlLocale, 2× trustedHosts: unit puri). Inoltre le righe summary hf nel verdetto sono VUOTE: il denominatore dello 0,92% non è citabile dal verdetto (viola «cifre solo dal verdetto»). La canonicità forse regge sul conteggio; la motivazione no.
2. dbal: il claim cita il raw 8,29-8,33 come «canonica» ma il verdetto porta anche net 8,57-8,60 (pavimenti 0,19 vs 0,06); il criterio non fissa quale sia la cifra canonica. E «mock-leggera» è asserito SENZA lo strumento di densità che la stessa leva dichiara mancante: lo scagionamento del compile regge solo sul profilo, non sulla mappa.
3. «Compile ≤1% leaf» è un lower bound sui frame nominati: memmove/memcmp (~7-8% dei campioni) sono leaf non attribuiti e possono nascondere compile (lessico = memcmp-intensivo).
4. Gate contesa ictx su conteggi assoluti tra gambe di durata 8× diversa: le due gambe phpr dbal sono SEGNALATE per costruzione. Serve ictx/secondo.

## Vagliate e respinte
- evalcls estrapolabile? Il claim lo usa solo come apertura/cliff e pretende lo strumento di densità prima di ogni leva: coerente (resta non provata la linearità per-classe a 20k classi).
- aboff «voce chiusa»: chiusura su non-risoluzione pre-registrata (p.5); la deriva monotona A1→B2 rende onesto il rifiuto di firmare il −1,36%. Ma il tetto va citato ≤3,8%, non «~2%».
- Interpolazione `"n$i"` nel ctor contamina objalloc, ma str è a parità (S-125): gonfia il pavimento comune e semmai DILUISCE il 9,9×; non ribalta la nomina.

## Azioni S-127
1. Ri-committare la regola di nomina (massimo-visibile vs massimo-assoluto) PRIMA dell'istruttoria d'ammissione L-OL1.
2. Sub-micro bilaterale per il residuo 427 ns (drop immediato vs differito; ctor senza interpolazione) per chiudere l'additività.
3. Rifare la cattura summary hf (denominatore NEL verdetto) e correggere a verbale i nomi non-php -S.
4. Fissare nel criterio mappa la cifra canonica (raw vs netto-pavimento) e riallineare PERF_MAP.
5. Gate contesa espresso in ictx/s.
