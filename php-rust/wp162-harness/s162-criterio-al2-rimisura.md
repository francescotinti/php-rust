# s162-criterio-al2-rimisura.md — rimisura AL2 su stash FERMI (rev. S-161 #1; PRE-REGISTRATO prima del run)
1. Oggetto: cifra PROPRIA del coeff sito-autoload L-AL2 (oggi a registro «direzione firmata, magnitudine NON tarata»).
2. Bracci FERMI: A=phpr-s161-gemelloA (==pin s160 ceeb6e76) · B=phpr-s161 (==pin s161 ec0a636a); hash verificati PRIMA del run, mismatch ⇒ rc=9 STOP.
3. Giudice: m-missload.php N=10.000.000 (N dal sorgente; tick 1,0 ns = soglia/4, §3 az.rev. S-154), marcatore `ML-OK 10000000` bilaterale + diff output A/B; floor3 per-binario su empty.php.
4. Misura: R=5 coppie ABAB alternate, mediane, rumore drop-1 (matematica s158 INVARIATA); igiene di RECORD: lock s-162 + quiescenza rc=0 + sentinelle language-server inizio/fine nel `.out` (incidente 15).
5. Segno atteso: + (A senza leva più lento). D ≤ max(rumore, 0) ⇒ NESSUNA cifra: il registro resta «solo segno», si dichiara.
6. Registro (D > rumore, segno +): coeff sito-autoload = D ± rumore(drop-1) nella TABELLA PER-SITO (rev. #2); confronto DICHIARATO con R=5 (+5,0) e conferma post-pin (+3,0) = drift a verbale, non gate.
7. Intorno pre-registrato dall'unica cifra esistente (R=5 s161): |D−5,0| > 2,0+rumore ⇒ reperto FUORI-INTORNO a verbale (si dichiara, NESSUNA taratura post-hoc); il modello per-sito NON pre-dice questo sito da altri siti (veto S-161).
8. Esiti a FILE: wp162-harness/s162-al2-rimisura-verdetto.out; rc AUTORITATIVO SOLO da al2rim-out/rim.done.
