# Verbale sedia 9 — Brendan Gregg — Concilio WP-93 (revisione S-91.0)

Perimetro: metodologia di misura, doc==macchina per NOME, leggibilità ledger. Mandato: REFUTARE.

## VERDETTO: PASS con emendamenti minori. Refutazioni capitali: NO.

## Q1 — Censimento MEASURE90 emendato ↔ repair90-estimators.out (righe .out citate)

Ogni cifra del perimetro ha la sua riga; censimento per NOME:
- b_peak(mediana) 20.289.946 → r87 `REPAIRED b_peak ... 20289946` AL BYTE; se 1.084.655 → r87; banda [18.120.635, 22.459.256] → r87. Min-based 19.723.059 → r67+r82 (tether g2 PASS).
- b_boot 2.252.800 → r37; b_work 17.276.928 → r52; somma 19.529.728 → r88; continuità 19.575.603/45.875 → r89; residuo 193.331 → r82 (193331,2).
- Robustezze 0,972 / 0,999 → r86 (tautology=no).
- VCOV per-W: tutti e 12 i ratio → r96-r99, AL BYTE. Pooled 0,576 «in nessun raw» → r100.
- Marginale 15.777.004 = 0,778 → r101; invisibile 4.480.174 + intercetta 71.934.811 → r102.
- Residui per-W mode_min (−2.097.152 / 0 / 262.144 / 393.216) → r91; dpost nonzero (w4.r5 +5.046.272, w16.r2 +1.048.576, w16.r3 +5.177.344; 17/20 zero) → r93; outlier w4.r1/w12.r3 → r94-r95; ADVISORY=7 per NOME → r36; marginali OUT a mediana (22.183.936, 13.271.040) → r59-r61; clamp⇔overlap 10/10 → r113; 3/4 pd1000 dt∈{1,5} ricostruibile da r109-r112; t 4,303/2,101 → r2/r83-r85.
- cal 7.801.102: NON nel .out — fonte dichiarata verdict90.a1.g2.out r46, verificata. Legittimo (non è cifra riparata).

DUE nit di trascrizione, non refutazioni: (1) se b_boot doc «6.345» vs macchina 6345,5 — TRONCAMENTO, mentre ovunque il doc arrotonda half-up (501347,7→501.348; 1084655,4→1.084.655; 20289945,6→20.289.946): regola incoerente su UNA cifra, propagata a NEXT_SESSION:40. (2) delta continuità senza segno (macchina −45875).

## Q2 — Gradi in GAP_TREND (r101) e NEXT_SESSION

GAP_TREND: mediana «RIPUBBLICATA», min-statistic «DECLASSATA», somma «(ADVISORY)» — gradi giusti. NEXT_SESSION §Misure: b_boot «VERDICT-GRADE come MAGNITUDINE» (consentito), b_work e somma «ADVISORY», continuità «suggestiva» — conformi. NESSUNA cifra citata sopra il grado deliberato. Un buco: b_peak(mediana) in NEXT_SESSION sta sotto l'intestazione «gradi» SENZA token di grado, unica riga muta in mezzo a righe etichettate — leggibile per omissione come verdict-grade, mentre i 4 punti PEAK sono ADVISORY per NOME e la testata è «DA RIFARE» (KS-BB-92-1).

## Q3 — bite-test-historic.log

AUTOSUFFICIENTE: header con mandato (A-SK-68/WP-92 P2), head=c71bf9e, budget from HEAD per doc; ogni FAIL con file, NUMERO di riga, token e ragione (M84: 2; M85: 4 — coerenti con WP_SESSION_91 p2); i PASS M86/87/88 col criterio stampato e la label «NEVER verdict-grade». Collocazione: citato nel manifest a wp81-harness/gate-cifre-manifest.tsv:15; M84/85 ancorati verdict= (r28-r29). Nit: «committato in questo stesso commit» — il log atterra in ce1ee27 (FIGLIO di c71bf9e, parent verificato); il log non può nominare il proprio sha, la frase dica «commit figlio di head».

## Q4 — Lezioni ⭐⭐ WP_SESSION_91

Tutte e tre con evento-prova nominato in-sessione: (1) numericità ← Scoperta 4 (awk macOS bracket-class, `[ "" -ne 8 ]` vacuo); (2) bite-test decide ← Scoperta 2 + log (attesa sbagliata 3/5); (3) autorità da HEAD ← forge working-tree, «porta di TRE forge su quattro», denti T13-T16. Nessuna da riformulare.

## Emendamenti

- **A-BG62**: regola di arrotondamento DICHIARATA (half-up) per ogni trascrizione decimale→intero nei doc; correggere se b_boot 6.345→6.346 (o citare 6.345,5) in MEASURE90:27 e NEXT_SESSION:40.
- **A-BG63**: delta di continuità col SEGNO (−45.875 B) in MEASURE90 e GAP_TREND ove citato.
- **A-BG64**: token di grado ESPLICITO su b_peak(mediana) ovunque citata (RIPUBBLICATA, testata DA RIFARE — mai muta sotto intestazione «gradi»).
- **A-BG65**: i log di bite-test dichiarino «commit figlio di head=<sha>»; riga di ledger che lega log→sha di atterraggio.

## Kill-switch

- **KS-BG-93-1**: trascrizione doc di una cifra macchina con arrotondamento non conforme alla regola A-BG62 = FAIL del gate cifre (dente sulla coppia companion decimale→intero).
- **KS-BG-93-2**: riga di cifra sotto intestazione di gradi SENZA token di grado = FAIL (il grado si legge sulla riga, non si eredita dal silenzio).

## Refutazioni capitali: NO — doc==macchina regge sul perimetro; i quattro rilievi sono di trascrizione/etichetta, non di sostanza.
