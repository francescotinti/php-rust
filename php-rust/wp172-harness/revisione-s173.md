# Revisione S-173 — lente MISURA (revisore adversariale, subagent in sola lettura; rotazione: S-171 semantica, S-172 processo)

**VERDETTO: REGGE CON RILIEVI**

Il claim (P3 +7,60, C +11,40, 2,95× su prop-dq) regge sui dati grezzi; i rilievi colpiscono la *precisione dichiarata*, non la direzione.

1. **«Rumore drop-1 0,07» = un tick esatto** (0,01 s/150M = 0,067; `s173-ab-leva.sh:76` prende i 4 valori più vicini alla mediana, spread 0,01 s in `f3-runs.tsv`). È il pavimento di quantizzazione, non una stima del rumore. Nessun outlier/deriva/effetto posizione (A: pos1 7,90/7,89, pos2 7,90, pos3 7,89/7,92 ≤1 tick): la rotazione compensa davvero.
2. **Riproducibilità tra run, stessa notte, stesso codice P1+P2**: 53,67 (abbc, braccio e396498b, 02:10) vs 52,53 (f3, pin 5f2dff7d, 02:23) vs 52,87 (S-172): banda **1,14** ns/iter; oracle 14,07/13,93/14,00 (1%). L'incertezza reale è ~±0,6, non 0,07 — coperta dalla soglia 4, ma il verdetto la tace.
3. **0,94 non ri-derivato**: costante hard-coded (`s173-ab-leva.sh:80`) importata da `wp171-harness/s171-m8-1g-verdetto.out:12` (`soglia_dec`, giudice arith). La banda osservata su prop-dq (1,14, punto 2) la supera.
4. **Mediana per colonna vs differenze appaiate**: f3 A−B 7,53 (vs 7,60), B−C 3,87 (vs 3,80; coppia5 = 4,00 esatto), A−C 11,40 — verdetti invariati, P4 a 2 tick dalla soglia. Su **abbc** invece: colonna 4,87, appaiata **5,60** (Δ 0,73 = 15% della cifra); riga 1 B outlier (59,73 = 8,98 s), C monotona 53,93→52,80 (deriva −1,1 su 5 coppie), rumore 0,67/0,93. La «CIFRA a P2 = +4,87» (`s173-abbc-verdetto.out:14`) dipende dallo stimatore: onesto sarebbe «+4,9÷5,6, nominata».
5. Pre-registrazione: criterio 02:03:15 (commit 8c0635bd 02:03:57), P3 commit 2006d11d **02:03:59** (2 s dopo: codice scritto in parallelo), P4 02:07:44. Igiene: abbc 02:06:58–02:10:09; build B 02:11:24–02:14:59, C →02:17:34; f3 02:19:16–02:23:44 (quiete FAIL tentativo 1, mediaanalysisd 13–17%). **Nessuna build durante le misure** — regge. Hash abbc 260fbc3b/e396498b = S-172 — regge.
6. **Cifra composta** (REGOLE §3): `gaps/REPORT_GAP_173.md:27` «residuo 27,2 = 7×1,75 + 5,8 + 7 + corpi ≈2» usa il dispatch 1,75 di S-169 (altro binario); `s173-verdetto.out:4` idem. Etichettata «direzione» ma scritta in cifre.
7. Claim «sotto 3×» detto esplicitamente sul giudice prop-dq (`s173-verdetto.out:2`, `WP_SESSION_173.md:4,7`), scoreboard micro/prop.php (30M) placeholder — regge.
8. (fuori lente, da segnalare) `s173-mutante3-verdetto-corsa1.out` rc=5 → corsa 2 rc=0 per **riclassificazione** di 5 blocchi «fuori dominio»→«a verdetto» (p3-numstr/null/bool-dst, p4-dst-double/str) e p4-self-rhs ROTTO→INTATTO con fixture 94→100 righe: la spiegazione «dst diventa Long dopo la 1ª iterazione» è plausibile ma non è provata da un blocco che resti INTATTO col reset.

**Azioni S-174**
- Riportare nei `.out` la banda tra run (stesso codice, binari diversi) accanto al drop-1; ri-derivare 0,94 su prop-dq.
- Stampare ANCHE la mediana delle differenze appaiate; cifra P2 come intervallo [4,87;5,60].
- Togliere le cifre dalla decomposizione del residuo o marcarla «ipotesi, non misura».
- Mutante P3/P4: aggiungere blocchi con reset dst nel loop che restino INTATTI (prova del dominio, non riclassificazione).

## Replica della sessione (S-173, dichiarata; le azioni restano az.rev. S-174)
- Rilievo 8: i blocchi col reset DENTRO il loop esistono già nella corsa 2 (`p3-null-dst-each`, `p3-bool-dst-each`, `p3-numstr-dst-each`, `p4-dst-double-each`, `p4-dst-str-each`, aggiunti in fx-sl3 94→100 righe, elencati in INT3/INT4 di `s173-mutante-sl3.sh`) e sono rimasti INTATTI (verdetto corsa 2): la prova del dominio richiesta c'è; la riclassificazione riguarda solo le forme originali senza reset. Il revisore ha letto i conteggi ma non le liste: resta vero che il .out non lo dice in chiaro — az.rev. S-174: il verdetto nomini i blocchi `-each` intatti.
- Rilievo 4: mediane appaiate calcolate dai tsv a fine sessione e riportate nei `.out` come nota post-revisione (f3: A−B 7,53 · A−C 11,40 · B−C 3,87; abbc: B−C 5,60): P2 = intervallo [4,87;5,60] nominato; P4 resta sotto soglia con entrambi gli stimatori.
- Rilievo 6: le decomposizioni del residuo in REPORT_GAP_173/NEXT/verdetto sono marcate «IPOTESI (cifre da altri binari), non misura».
