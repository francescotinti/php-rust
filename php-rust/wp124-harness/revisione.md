# Revisione S-124 — lente PROCESSO

## Reperto principale
**L'arbitro dell'admission non era pre-registrato.** Il claim poggia su «criterio committato PRIMA» + «predizioni esatte 6/6», ma gli script che EMETTONO quel verdetto (`s124-classifica-report.sh` con la definizione di «in grado» = ±0,25/±0,30, più build/run/chain/promozione) sono entrati nel repo solo in **fb140d1**, DOPO entrambi i run di admission (run1 bocciata a 43d7610, run2 passata a 2d2c180) e dopo l'A/B run1. bcf1d61 committa criterio, giudice v3 e s124-ab.sh — l'arbitro del pilastro «6/6» no. Verificato con `git show --stat`. Per REGOLE §2 un verdetto emesso da uno script mai committato è un artefatto non collaudato-nell'atto: il claim di legittimità è INDEBOLITO nel suo pilastro pre-registrazione (le misure non sono contestate).

## Reperti secondari
1. **«6/6 esatte» sovradichiara**: re −3 non è predizione verificata ma esito costruito — la patch 2d2c180 (preg.rs/host.rs) sta FUORI dal perimetro dichiarato al criterio (zstr.rs + 7 siti + ~14 testuali) e il criterio non è mai stato ri-committato emendato. Il modello stesso era internamente incoerente (riga Vec-fed «2→2» vs predizione re −3): la ridiagnosi è dichiarata, la dicitura del claim la traveste da conferma del modello.
2. **SL delle guardie da regime estraneo**: le bande v2 (SL calls 0,73 che boccia run1 e assolve run2 a +0,81) vengono da binari layout-probe S-123 su pin s120; i candidati sono binari nuovi. Pre-dichiarate valide, mai rivalidate; «canale icache FIRMATO» è attribuzione senza A/B proprio (REGOLE §4: direzione+meccanismo, non firma).
3. **Registro fuori posto**: batteria, pin, corpus, fixture, micro e «PROMOZIONE COMPLETA» sono appesi a `s124-str1-verdetto.out` — il file del run BOCCIATO; `s124-str2-verdetto.out` non contiene i gate (REGOLE §8).

## Vagliate e respinte
- **Tolleranze tarate sui dati**: respinta — scarti massimi 0,04 vs bordo 0,25/0,30; mai outcome-determinative. Resta solo la non-pre-registrazione (reperto principale).
- **Retry B2 = insistere finché la guardia passa**: respinta — A/B rifatto integrale su candidato nuovo, soglie invariate, bersaglio rivince con margine; emendare il candidato non è emendare il giudice.

## Azioni S-125
1. Gli script d'arbitrio (admission incluse le tolleranze) si committano NEL commit del criterio, prima del primo run; verdetto da script non committato = incidente contato.
2. Patch estesa oltre il perimetro dichiarato ⇒ ri-commit del criterio emendato PRIMA di rieseguire l'arbitro.
3. Registro promozione in file proprio o nel verdetto del run promosso.
4. Rivalidare le bande layout sul regime post-patch (aggancio al ±zval-census stesso head già rinviato).
5. Nei report: «5 predette + 1 costruita dopo ridiagnosi», non «6/6 esatte».
