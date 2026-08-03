# Verbale sedia 5 — Lars Bak — Concilio WP-93 (revisione S-91.0)

Perimetro: statistica, stimatori. Metodo: ricomputo INDIPENDENTE (python, non l'awk del repair) dai 20 raw `m90.slope.*.memcensus` (tr -d '\0', campi per NOME: win=9 pboot/pwork, win=0 exit_collect_mi). Estrazione mia vs ladder del `.out`: **20/20 byte-identica** — nessun mismatch.

## VERDETTO: PASS CON EMENDAMENTI
Le cifre di testata riprodotte al byte; due difetti statistici reali: la banda pooled df=18 NON è citabile come CI onesto (indipendenza refutata dai raw) e il MAD flag inclusivo MASCHERA un terzo outlier (w16.r1).

## Q1 — b_peak(mediana) riprodotta; outlier dentro è giusto
Mie cifre: b=20.289.945,6 · se=1.084.655,4 — **al byte col .out**. Esclusione PRE-mediana:
- solo w4.r1: W4med(4)=153.944.064 → b=20.285.030 (Δ=−4.915, −0,02%).
- politica coerente (TUTTI i flaggati: w4.r1+w12.r3): W12med(4)=330.825.728 → b=20.357.120 (Δ=+67.174, +0,33%), se SALE a 1.272.980 e i marginali peggiorano (W8→12 25.100.288 OUT alto, W12→16 15.269.888 OUT basso).
Verdetto: **mediana-su-5 con l'outlier DENTRO è lo stimatore giusto** (breakdown 40%: tollera 2/5); l'esclusione compra ≤0,33% al prezzo di un grado di libertà del ricercatore e di se più larghi. Q1 sostiene il .out.

## Q2 — leave-one-out: SÌ, esiste un outlier mascherato
LOO (mediana+MAD delle 4 SORELLE, run giudicata fuori) col criterio A-BB67 «TUTTI e 3 i contatori»: flagga **w4.r1, w12.r3 E w16.r1**. w16.r1 (pboot 48.758.784 max, pwork 399.638.528 = +9,4 MB sulle sorelle, exit idem) è mascherata dal criterio inclusivo: sul pboot la sua stessa presenza porta dev==MAD==131.072, esattamente dentro 2·MAD; con LOO dev=229.376 > 2·98.304. Effetto sulle testate: **ZERO** (outlier alto: né min né mediana lo pescano; mediane invariate). Difetti collaterali: (a) MAD==0 (w8 pboot) rende il criterio DEGENERE e il codice lo tratta silenziosamente come non-outlier; (b) su singoli contatori LOO flagga 13 run — il congiuntivo ALL-3 è la sola cosa che tiene il flag utilizzabile.

## Q3 — pooled df=18: se riprodotto, banda NON citabile come 95%
Mie cifre: b=20.274.872,3 · se=427.628 · banda t18 [19.376.426, 21.173.319] — riprodotte. MA: con disegno bilanciato la pendenza pooled ≡ pendenza sulle 4 MEDIE per-W (verificato: identiche); l'informazione extra dei 20 punti sta solo nell'se, che presume 20 errori indipendenti. Dai raw: MS_between(df2)=2,86e14 vs MS_within(df16)=4,65e13, **F(2,16)=6,16 > F0,95≈3,63** ⇒ le run sorelle NON sono rumore scambiabile attorno alla retta (ICC≈0,51, design effect ≈3,0 ⇒ se corretto ~744.541, vicino all'se per-medie 846.130). CR0 con G=4 cluster (se=397.448) è downward-biased, inutilizzabile. **La banda [19.376.426, 21.173.319] NON è citabile come 95%**; citabile: la banda df=2 sulle medie [16.633.973, 23.915.772] (t=4,303) o il pooled etichettato «condizionato a indipendenza NON provata».

## Q4 — coerenza gradi/cifre di MEASURE90 emendato
Verificate al byte dai miei conti: b_boot 2.252.800/se 6.345,5 · b_work 17.276.928/se 501.347,7 · somma 19.529.728 · b_peak(med) 20.289.946/se 1.084.655/banda 2se · robustezza 0,972/0,999 · coverage marginale 0,778 · residui per-W · conversioni MiB (2,15/16,48/18,63/19,35) · delta 45.875. **COERENTI**. Due punti che il .out non sostiene: (1) la dicitura «pooled … CI onesto» (§b_peak): onesto nei df (A-BB68), NON nell'indipendenza (Q3); (2) «nessuna delle due può diventare il punto della sua W via min» — come ASSERZIONE è refutata dallo stesso doc (PEAK-min HA pescato w4.r1): va riformulata come norma. Inoltre «additività within-run 20/20» è un'identità aritmetica (il .out lo dichiara): mai contarla come verifica.

## Emendamenti
- **A-BB69**: MAD flag in forma LEAVE-ONE-OUT (sorelle = R−1, run giudicata esclusa); w16.r1 flaggata per NOME; MAD==0 ⇒ stato DEGENERE dichiarato in output, mai silenzio.
- **A-BB70**: politica outlier di testata: outlier DENTRO la mediana; esclusione pre-stimatore BANDITA salvo delibera, e allora TUTTI i flaggati + Δb dichiarato.
- **A-BB71**: pooled df=18 declassato a ADVISORY-conditional; F-test intra-W (ex-ante) obbligatorio prima di citare qualunque banda n>W-points come 95%; emendare la riga «CI onesto» in MEASURE90.
- **A-BB72**: riformulare la frase «can never become its W point via min» in forma NORMATIVA; l'additività within-run etichettata sempre «tautologia».

## Kill-switch
- **KS-BB-93-1**: banda pooled citata come 95% senza F-test di scambiabilità intra-W superato ⇒ FAIL del giudice.
- **KS-BB-93-2**: flag outlier calcolato con la run dentro le proprie sorelle ⇒ FAIL (solo LOO).

## Refutazioni capitali
- **SÌ (1)**: «pooled df=18 = CI onesto» — refutata dai raw (F=6,16): la banda [19.376.426, 21.173.319] non è un 95%.
- **NO** sulle cifre di testata: b_peak(mediana), se, b_boot, b_work, somma, robustezza, coverage marginale — tutte riprodotte al byte dai raw; l'outlier mascherato w16.r1 non sposta alcuna testata (effetto zero su min e mediane).
