# WP_SESSION_167 — ⚖️ CONCILIO (GO-CONDIZIONATO 9/9 alla campagna R1) + FETTA 0 a verdetto: pila e mispredict REFUTATI, interno-handler dominante
**In una frase**: convocato dall'utente, il collegio dei nove ha deliberato
all'unanimità la campagna sul nucleo — ma «prima misura, poi codice» — e la
sonda eseguita in giornata ha già dato il suo frutto: il divario aritmetico
non sta né nella pila né nei salti mal predetti (phpr predice MEGLIO di PHP),
sta nel lavoro in più che ogni istruzione compie dentro il suo gestore.
**SCOREBOARD** (pin INVARIATO **s166 phpr 092dcff431bef876 + server
caa4e4b2638686a9**): arith 5,4 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,2 ·
re 2,5 (nessuna leva di codice: sessione di DELIBERA+SONDA, oggetto sanzionato
dal ⚖️) · WP 1,746-1,749 (rif.) · ORM [7,023;7,053] (rif.) · **leve spedite: 0
— sanzionato dal concilio (fetta 0 = misura)** · incidenti: **1** (ENOSPC da
copione census senza pulizia della copia-sorgente; curato, raw fuori repo).

## Esiti secchi
1·⚖️ CONCILIO 9 sedie + 3 team: GO-CONDIZIONATO alla campagna R1
  «interno-handler prima, dispatch poi» (reperto Hejlsberg: phpr dispatcha ~3
  op/iter vs ~6-7 Zend); NaN-boxing/fn-table/arena chiusi; sequenza
  F0→F1 predecode→F2 BinOp-3ind→F3 outline; kill pre-registrati.
  Vincolano COUNCIL_WP167_REVIEWS.md + verbali/ (9+3).
2·Az.rev. S-166: az.1 copia-gate-v2 COLLAUDATO (mutante rc=3, positivo rc=0;
  v1 di S-134 RIPRISTINATO dopo mia sovrascrittura cieca, dichiarata) ·
  az.4 regola RIF nel criterio + sanature orm2 (r.242 inclusa).
3·FETTA 0 (criterio pre-registrato, 5 bracci): (a) reg-lower paga GIÀ 80,6
  ns/iter · (b) stride +0,44 ⇒ PILA REFUTATA · (c) xctrace bilaterale
  collaudato dal mutante (c1 11,9×): **mispredict REFUTATO (phpr 0,005 vs
  oracle 0,031)**; quote ⇒ interno-handler ~23 ns (61%) + delivery-indiziato
  ~14,5 · (d) drop-census INDISPONIBILE (mutante: copertura parziale) ·
  (e) non scattato. **GO-leva F1/F2 SUB JUDICE del gate mock ≥90% (Gregg)**.
4·Rettifica a storia: un commit «rc=0» scritto PRIMA dell'esito, rettificato.

## ⭐ Lezioni (max 3)
- ⭐⭐ i MUTANTI degli strumenti pagano DOPPIO: uno ha smascherato prima il
  parser e poi la copertura parziale del census; l'altro ha fissato la
  semantica di colonna e refutato il canale branch — mai leggere uno
  strumento nuovo senza il suo mutante.
- ⭐⭐ un'attribuzione DA QUOTE è indizio bilaterale forte ma NON firma
  sottrattiva (§4): il 99% «indiziato» della fetta 0 alloca i mock, non li
  sostituisce.
- ⭐ un copione che copia un albero lo pulisce nell'epilogo (ENOSPC in
  finestra); raw fuori repo; mai commit concatenati a un collaudo.
