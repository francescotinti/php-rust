# Revisione S-163 — lente PROCESSO

## VERDETTO: REGGE CON RILIEVI
Il guadagno L-AU1 regge nella sostanza: il bundle «3 alloc/miss» sta nel criterio p.3 (commit 041aef16) PRIMA di smoke e census; il Δ misurato 600000 lo conferma ESATTO con repliche e bisezione; smoke/R5 riconciliati, 19 guardie pulite, catena di promozione a rc di script. Ma la dicitura «catena rc=0 senza rettifiche» è FALSA per un anello: il census ha emesso rc=5 e il suo `census.done` — canale che lo script stesso dichiara autoritativo — è stato sovrascritto A MANO. REGOLE §3 è esplicita: una guardia si emenda solo RIESEGUENDO il criterio emendato. La riesecuzione non c'è stata. Incidente di processo da contare; la promozione non cade perché l'attesa corretta era derivabile ex-ante e i gate di promo sono autonomi.

## Rilievi
1. **CAPITALE — rc=0 fabbricato a mano.** Lo script census (riga 12: «rc SOLO da census-out/census.done») ha emesso rc=5; la sessione ha scritto il .done con echo. L'APPENDICE è argomentata e verificabile, ma il canale autoritativo è stato violato per costruzione. La via pulita costava minuti: emendare l'attesa NELLO script, rieseguire, rc=0 emesso dall'arbitro. Precedente pericoloso: ogni futuro rc=5 è ora «appellabile per appendice».
2. **L'attesa del census non era pre-registrata.** Lo script con Δ=200000 nasce nello STESSO commit del verdetto smoke (27136dfc), dopo aver visto D=+45,5. Salva la correzione solo il fatto che 3 alloc/miss stava nel p.3 pre-smoke: correzione da sorgente legittima nel merito, ma su un arbitro nato post-trigger.
3. **La banda VINCOLANTE non morde mai SOPRA.** Secondo FUORI-UB SOPRA consecutivo (strmap s162, arrload s163) archiviato con la stessa frase qualitativa («plumbing, nessun coeff nuovo»). Il census refuta in spazio-alloc, non in spazio-ns: i ~15 ns oltre il tetto 30 restano non quantificati. Nessun esito del census in ns avrebbe FERMATO la leva.
4. **Harness chiuso mutato.** fx-sm/fx-sm-div estesi in-place in wp162-harness (221a8326): il replay forense dei verdetti s162 ora punta a fixture diverse. Mitigato dai gate BYTE-ID, ma l'estensione andava in wp163.
5. **Minore.** Predicato di quiete del riaggancio (6×30s <5%) scelto in corsa e irrobustito dopo i rifiuti; accettabile perché i gate del run restano arbitri e i tag bruciati sono a verbale.

## Azioni (S-164)
1. Rieseguire il census con attesa 600000 NEL sorgente; `census.done` dallo script; contare l'incidente «rc autoritativo scritto a mano».
2. Emendare il verbale s163: «senza rettifiche» → «con arbitrato census emendato a mano».
3. Pre-registrare per la prossima leva l'esito census che FERMA (condizione anche in ns); coeff per-sito da rifondare includendo il plumbing, o banda dichiarata a un lato.
4. Congelare i wp<N>-harness chiusi; estensioni solo nel harness corrente.
5. Predicato di quiete pre-registrato nel criterio quando il flare è già noto dalla p.1b.
