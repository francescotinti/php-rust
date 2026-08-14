# Revisione S-136 — lente PROCESSO (revisore singolo)

## Reperto principale
Lo smoke FD1 ha emesso **rc=5 «ESITO: GUARDIA MORDE»** (re: D=−5,4 vs soglia −4,5, oltre il rumore drop-1 calcolato B'=2,5) e la sessione è passata all'R=5 **senza emendare il criterio**. Il p.7 del criterio dice, senza eccezioni di fase: «una guardia che morde OLTRE il proprio rumore fa cadere la leva»; il p.4 assegna allo smoke solo l'early-stop a segno opposto del giudice, ma è lo stesso giudice meccanico (s136-ab.sh) a scrivere l'esito rc=5, autoritativo per REGOLE §5. La regola risolutiva («le guardie si adjudicano solo a R=5, col drop-1 vero») compare solo ex-post nel session file, scelta a numeri visti e in direzione favorevole alla leva. Serviva l'emenda dichiarata tra smoke e R=5 (REGOLE §3/§5). Indebolisce il claim 2 senza invalidarlo: l'R=5 era l'arbitro pre-registrato e la guardia re a R=5 rientra (−5,8 vs −15,8).

## Reperti secondari (numerati)
1. **Firma leg3-on ictx 133% non pesata**: gamba contata «pulita» ed entrata nel canone on-only (1,779) senza una riga di verbale; il criterio pair prescrive «firma per gamba» ma non ne pre-registra la lettura — stesso difetto strutturale del reperto principale: strumenti senza conseguenza pre-registrata, adjudicati ad hoc.
2. **3 probe dump in finestra**: violano il vincolo auto-imposto «NESSUN build/misura fino al done»; dichiarate ma NON contate (scoreboard: «incidenti: 1»). Pesatura incoerente col LSP, che è contato.
3. **«Criterio PRIMA del codice» poggia sul solo ordine dei commit** (7b7455c 09:46 → 5b9ab80 09:51): 5 minuti tra criterio e leva+giudice A/B implicano codice in bozza prima della registrazione. Le soglie però non ne dipendono (derivate dal modello-tempo).

## Vagliate e respinte
Emende pair: conformi — commit dedicati, COPIA-GATE rifatto entrambe le volte (copia.diff aggiornato), gate INVARIATO; confrontabilità col rif 1,769 preservata (condizione di accettazione rc=0 identica a S-134; assestamento/retry spostano solo il *quando*). Gambe t1/t2/t3 (LSP inclusa): rc=8, nessuna nel canone. FUORI-UB +0,4: conforme p.5 (dichiarato nel verdetto R=5, sonda schedulata NEXT p.3). Soglie non adattate post-smoke: 69,6/13,3/82,9 già nell'header dello smoke.

## Azioni S-137 (3-5, concrete)
1. Nel prossimo criterio leva: pre-registrare il ruolo dello smoke sulle guardie (chi adjudica, a quale R); il caso re resta a verbale come precedente.
2. Pre-registrare la lettura della firma per gamba (soglia di «gamba sporca»); annotare leg3-on 133% nel verbale coppia.
3. Correggere lo scoreboard: incidenti 2 (LSP + probe in finestra), o togliere il vincolo assoluto dalle istruttorie. **[APPLICATA in S-136: conteggio corretto a 2]**
4. Eseguire la sonda di ripartizione +13,7 PRIMA di nuove leve sul cammino dim-write (blocco REGOLE §4).
5. Committare il criterio leva con working tree dichiarato pulito dal codice della leva (o bozza dichiarata).
