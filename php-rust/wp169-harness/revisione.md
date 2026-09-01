# Revisione S-169 (revisore singolo, lente PROCESSO)

## VERDETTO: REGGE CON RILIEVI

I numeri reggono (m5 D=−14,00 rumore 0,4; m4b/m3 riproducibili a R=9; rbm alza solo c3 su entrambi). Non regge il MODO: quattro difetti di una classe, una regola §3 saltata, lock vivo a sessione chiusa.

## Rilievi

1. **Pre-flight saltato (a).** Nessun `/apri-sessione`: Data (≥10G) e CI_FEED non letti — il feed alle 22:10–22:18 già diceva `disk-low(7G)`. L'ENOSPC era PREVEDIBILE: pre-flight omesso, non «ktrace imprevisto».
2. **Guardia emendata senza riesecuzione (REGOLE §3, rev. S-112).** e2 di m5/m7 vacua (m5-verdetto r.11 `A=0.04 B=0.04 oracle=0.00`, output «Could not open input file»). Il criterio p.6 la sostituisce con e2 di m4b9 «per costruzione»; §3 impone la riesecuzione (~1 min). Il «corpo banale 5,59» (verdetto r.6) usa un e2 mai misurato nel run di m5; B=m5 su arith-e2 e dump MAI eseguiti. (c): lecito come STIMA dichiarata, non come cifra di verdetto.
3. **Classe unica: sostituzione omessa nel copione derivato, cieca al gate (b).** Quattro casi: path E2 (nessun token di sessione: gate cieco per costruzione); `H=wp168-harness` (riga con `s169`: `grep -v SESSFAM`, copia-gate r.34/38, assolve la RIGA); header xctrace r.1 «repliche R=3 … memstall» descrive S-168; `grep "s169\|s-168"` (ab-mock r.23) accetta il lock precedente. Controllo mancante: gate per TOKEN, `[ -s "$E2" ]` sui path letti, parità A==B contro output ATTESO (`0`).
4. **«Esclusione esaustiva» poggia su c2 debole (d).** xctrace r.15: c1/c2/c3 «ognuna dal proprio mutante»; ma c2 viene dall'icachemut S-167 (+0,038, «semantica NON fissata», s167-f0-verdetto r.7): con c0 e c2 incerte l'esclusione non chiude. «Mispredict FIRMATO» a N=1: 0,007 vs 0,024 con spread oracle ±0,021 (S-168 r.14); in ns/iter 0,33 vs 0,21 ⇒ phpr scarta DI PIÙ. Conclusione giusta, forza e verso sbagliati.
5. **Portate estese (e).** e2O oscilla 3,16–3,60 ⇒ oracle 1,58–1,80/op, non «1,80». Otto Nop identici = dispatch a bersaglio costante senza operandi: 1,75 è un LIMITE INFERIORE, il «17,5 non nominato» ne eredita l'errore. Un driver, un handler: «corpo di OGNI handler» (session file r.33) è estrapolazione.
6. **Lock vivo, CI ferma (f).** ci-runner r.61 cede a `phpr-measure.lock`: creato 00:03, presente dopo la rotazione (00:52); coda 304 job (≈100 h). Push a ogni passo è §10; costo NON dichiarato: CI bloccata finché il lock vive.

## Azioni S-170
1. Rimuovere il lock a fine sessione (passo esplicito di chiusura); coda CI nello scoreboard.
2. Rieseguire m5 e m7 con e2 valido + dump di arith-e2 sotto m5.
3. copia-gate per TOKEN, `[ -s ]` sui path letti, parità contro output atteso; rigenerare i tre manifest.
4. Mutante positivo c0 (memset/str_repeat); c2 «indiziata»; c3 anche in ns/iter.
5. Pre-flight obbligatorio anche in «stessa conversazione»: Data, CI_FEED, modello.
