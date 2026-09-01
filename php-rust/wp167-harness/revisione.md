# Revisione S-167 (revisore singolo, lente MISURA)

## VERDETTO: REGGE CON RILIEVI

Il nucleo operativo regge: refutazione mispredict solida (c1 phpr 0,005 vs oracle 0,031, mutante che morde 11,93×), direzione interno-handler coerente col reperto Hejlsberg, GO tenuto sub judice del gate mock ≥90%. Ma il «99% nominato» NON regge come cifra.

## Rilievi (ancorati ai numeri)

1. **Il «99%» è un'identità aritmetica, non un reperto.** Le quote sommano a 1 per costruzione (normalizzazione vals/Σ per campione): Σᵢ(qᵖᵢ×46,5 − qᵒᵢ×8,64) = 37,86 SEMPRE, qualunque semantica abbiano le colonne. Nominare c0+c2 (98,6% delle quote phpr) «chiude» il 99% per tautologia. Il contenuto vero è SOLO lo split 61/38 — e dipende dalla semantica di c0, mai fissata da un mutante suo. Il verdetto lo ammette («NON firme sottrattive») ma la sintesi lo rivende come chiusura.
2. **c0 «scende nei mutanti» è falso a metà**: p-mut c0=0,674 > p-dq 0,646 (sale su branchmut; scende solo su icachemut 0,611). E l'intestazione di s167-f0c.sh dichiara la legenda «Cycles/Delivery/Discarded/Useful»: se c0 fosse Cycles, quota×ns sarebbe double-counting. L'xml non porta nomi di colonna (solo uint64-array): la semantica di c0 resta aperta.
3. **Stride: refutazione a filo del rumore** — D=+0,44 con rumore 0,52 e soglia 0,5, e SENZA controllo positivo (unico braccio senza mutante che morde; nessuna verifica che i 14 dummy spostino davvero $s/$i di linea nel layout phpr). È un «non rilevato a risoluzione ~0,5», non una refutazione forte.
4. **Anomalia oracle non dichiarata**: su branchmut c1 oracle SCENDE (0,031→0,004) — il mutante morde solo su phpr; la bilateralità del collaudo è a metà, e le quote oracle vengono da UNA registrazione (599 campioni, N=1, nessuno spread).
5. **Skip di (e) meccanicamente lecito ma al limite**: firma front-end (delivery+discarded) = 1,938× < 2× per un soffio; con c2 phpr 2,3× oracle il tetto-fuso resta il braccio che prezzerebbe proprio il canale dominante. KS-BAK-167-1 non è evaso, solo rinviato: va detto esplicitamente.

## Azioni S-168 (concrete)

1. Eseguire i mock m1/m2/m3: sono l'UNICA chiusura non tautologica; il gate ≥90% resta l'arbitro del GO.
2. Fissare c0 con mutante proprio (lavoro utile puro aggiunto per iter, N esatto) e risolvere la legenda del template contro la documentazione dello strumento.
3. Ripetere stride con controllo positivo (mutante che DEVE produrre conflitto D-cache) o derubricare a «non rilevato ≤0,5».
4. Replicare xctrace R≥3 per lato (spread delle quote) e spiegare o-mut c1 0,031→0,004.
5. Emendare il verdetto: «99%»→identità dichiarata; correggere «c0 scende nei mutanti».
