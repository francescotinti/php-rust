# Revisione S-131 — lente MISURA (revisore singolo)

## Reperto principale
**Lo stimatore del rumore (range drop-1 intra-batch) è refutato dai dati della sessione stessa.** Il binario 0fdf1c49 come giudice objdatains in 4 batch dà 1250,0 · 1253,3 · 1268,3 · 1253,3: spread TRA batch 18,3 ns > soglia 13,3. Le due stime appaiate di D divergono di 21,7 ns (smoke +45,0 vs R=5 +23,3), oltre entrambe le bande dichiarate (6,7 e 13,3): l'offset di batch non si cancella nell'appaiamento (nello smoke A alto E B basso). Inoltre smoke D=45,0 SUPERA l'UB pre-registrato ~30 ns e il verdetto tace. «D=+23,3» porta incertezza reale ~±10–20 ns; la promozione sopravvive per direzione 7/7 + submicro −33,3, NON per il quadro soglia/rumore. Aggravante: a R=2 il range quantizzato (tick 3,33 ns) può leggere 0,0 (S-130 smoke) e la soglia crolla al pavimento 4,0 ≈ 1 tick.

## Reperti secondari
1. **Verdetto smoke contaminato**: riga 1 «quiescenza FALLITA (rc=1) — STOP» del primo tentativo (append `>>`), poi header del retry che dichiara rc=0 dal file .rc SOVRASCRITTO; invocazione arbitro cambiata post pre-registrazione («fix path neutro», dichiarato).
2. **Pair, bordo alto 1,797**: leg1-off ha la firma delle gambe ESCLUSE in S-129 (ictx oracle +17% vs mediana, oracle CPU la più veloce) sotto il gate 1,5×; senza di essa il riferimento è 1,757–1,785. «Warm-up efficace» non dimostrato: warm-up media-only, e la full di leg1 è comunque il massimo.
3. **Riferimento full = miscela di configurazioni**: on 1,757–1,767 e off 1,785–1,797 sono popolazioni disgiunte; la larghezza dell'intervallo è gap di flag, non rumore (canone ereditato da S-129).
4. `trange`: tie-break dipendente dall'ordine di arrivo (sort stabile); B' poteva essere 16,7 invece di 13,3 (non flippa).

## Vagliate e respinte
- **Bande fondate**: ricalcolate dai raw S-130 (7 gambe B, drop-1): objchurn 20,0 · objalloc 6,7 · objmap 13,3 — ESATTE.
- **Quantizzazione 0,01 s**: a R=5 soglia min 4,0 > 1 tick, D=7 tick — non flippa.
- **Direzione on<off**: già in S-129 (1,765/1,767 vs 1,805) — attesa.
- **Ordine commit**: criterio 76f9251 (21:58) < codice b87d7fa (22:01); verdetti prima delle letture. Aritmetica del giudice riverificata.

## Azioni S-132
1. Soglia giudice = max(drop-1, spread storico tra mediane di batch sul pin corrente), registrato a verbale.
2. Riconciliazione obbligatoria nel verdetto R=5: |D_smoke−D_R5| e D vs UB del modello, dichiarati se fuori banda.
3. Giudice pair: riferimento anche per configurazione (on-only = default) + firma per gamba (ictx%, rank CPU oracle) accanto al gate 1,5×.
4. Tie-break deterministico in `trange` (funzione del multiset).
5. Verdetto = file nuovo per tentativo; mai append su tentativo fallito.
