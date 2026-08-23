# Revisione S-155 (revisore singolo, lente MISURA)

## VERDETTO: REGGE CON RILIEVI

Verificato: criteri pair/ORM/sonda committati PRIMA dei run (3d33c29 12:08, ac4afda 12:14 vs run 14:50–15:09; gdc 400c944 15:02 vs 15:17); Δ ORM ricalcolato dai .time (34,38/34,43−0,08 vs 34,87/34,83−0,07, stesso metodo pavimento-parziale, stessa copia dichiarata s154→s155: Δ=[+0,41;+0,50] corretto); scope census a run.rs:3655 APRE prima di pop_keys:3658 (attribuzione args-Vec fondata al sorgente).

## Rilievi
1. **«Sotto 7×» oltre l'intervallo.** [6,972; 7,053] ATTRAVERSA 7: solo la gamba 1 è sotto; la gamba 2 (7,053, oracle2 peraltro ictx-SEGNALATA) è sopra. Per la disciplina a intervalli del progetto il titolo «prima volta sotto 7×» non è fondato; il Δ giù fuori rumore REGGE (lato phpr, non toccato dalle gambe oracle segnalate — che comunque non deflazionano oracle: 4,93<4,98).
2. **«Hit-path CE1 0-alloc CONFERMATO» misurato su UN ramo solo.** ce-count usa autoload=false → `class_index.get(LcKey)` diretto; il giudice m-classexists e il mix ORM dominante passano da `resolve_class_autoload` (autoload=true), 0-alloc solo per LETTURA del sorgente (LcKey SSO ≤64 B) — e un FQCN >64 B alloca. Perimetro da dichiarare, non «confermato» tout court.
3. **fe-count regge.** Lo scaling con l'arità (1 arg→16,0 B; 2 arg→32,0 B, interi esatti) discrimina l'args-Vec da una copia-nome (che resterebbe nel bucket 16 in entrambe le forme). Attribuzione solida.
4. **gdc: estrapolazione ×7 senza punto di controllo.** Il fit esatto a C∈{152,352} refuta da sé un termine di crescita raddoppiante (differirebbe di 1 tra i due C), ma 2393 resta fuori campo; inoltre 0,031 s è PURO-alloc — col fattore fuori-UB ~×3 del precedente BT2 si arriva a ≲0,1 s, comunque <0,293: «non pagante» regge con margine.
5. **dbal: righe summary phpr VUOTE nel verdetto** (anche in s154): l'estrazione è rotta; companion non arbitra, ma va sanata.
6. Anomalia «leve tentate 0» dichiarata con difesa plausibile (misure dovute; gdc uccisa a costo ~zero); incidenti 19 (=) onesto: l'attesa k=0 mancata è gestita dalla clausola pre-registrata, non è della classe incidente.

## Azioni
1. Riformulare il claim ORM: «gamba migliore 6,97; intervallo a cavallo di 7» — il sotto-7 si dichiara solo con intervallo intero <7 (o mediana N≥4).
2. Sonda ce-count in forma autoload=true 1-arg (driver = m-classexists ridotto) sul probe s155: chiude il perimetro del rilievo 2.
3. Terzo punto gdc a GDX≈2200 (C≈2352): un run, linearità alla scala ORM.
4. Fix estrazione summary dbal phpr nello script coppia (difetto ereditato da s154).
5. Census ORM al probe s155 (già in coda) per ripartire la magnitudine del funnel resolve_class_autoload.
