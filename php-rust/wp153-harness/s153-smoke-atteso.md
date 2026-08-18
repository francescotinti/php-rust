# s153-smoke-atteso — attesi BLIND dello smoke L-TD1 (dichiarati PRIMA del run)

Smoke = `s153-ab.sh <B> <hash8> smoke-td1 2` (R=2, early-stop manuale a segno
opposto prima del run di record R=5).

## Attesi ESATTI (verifica del secondo attore: esito ESATTO, mai «diverso da»)

1. **Parità output** su TUTTE le categorie: `diff` A vs B VUOTO. In
   particolare l'output di objdropdef/objchurn/objdatains è ESATTAMENTE
   `1500000` + newline (Σ di `id & 1` per id=0..2.999.999 = 1.500.000,
   derivato dal sorgente del giudice, non dal binario).
2. **Guardie d'ingresso**: A hash8 = `cbbe7173`; B hash8 = quello passato,
   != cbbe7173; quiescenza rc=0; lock presente e mio (`s153`).
3. **Giudice objdropdef, R=2**: D = A−B con segno **POSITIVO** su entrambe le
   coppie (segni 2/2). Un segno opposto in una coppia = early-stop, niente
   R=5, istruttoria. Magnitudine attesa: D ∈ [4; 26] ns/iter (centro modello
   17,6 = UB; R=2 non fa banda — la cifra di record è SOLO quella dell'R=5).
4. **Companion** objchurn/objdatains: segno + atteso (non gate allo smoke).
5. **File attesi a fine run**: `ab-out/smoke-td1.rc` (autoritativo; 0 o 4 —
   con R=2 il trange non è drop-1 e il python può dire SOTTO SOGLIA senza
   che lo smoke sia fallito: allo smoke arbitra il SEGNO 2/2, non la soglia),
   `s153-smoke-td1-verdetto.out`, `ab-out/smoke-td1-runs.tsv` con 2 righe
   per categoria.

## Cosa il secondo attore DEVE contestare

- attesi non esatti (es. «output uguali» senza il valore 1500000 derivato);
- lock/quiescenza non verificati dallo script (leggere s153-ab.sh righe 40-49);
- l'uso della cifra smoke come cifra (vietato: solo segno);
- A o B non identificati per hash PRIMA del run.
