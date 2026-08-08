# Revisione S-116 (revisore singolo, lente: MISURA)

## Verifiche fatte
Ricalcolate dal TSV (`ab-out/la-full-runs.tsv`) tutte e 6 le categorie con (t−floor)/N×1e9: prop D per coppia +29,33/+28,67/+27,00/+32,00/+29,33 → mediano +29,33, 5/5, spread_A 4,00 — coincide. calls: −6,00/−6,00/−6,50/−6,50/−6,50 → mediano −6,50, spread_A 0,50, soglia −max(1,00; 5,50)=−5,50 — coincide, sfondamento STRETTO corretto. arith/str/arr/re ricalcolate: tutte coincidono col verbale; regola famiglie 1,3×min verificata sui raw (zero esclusioni dovute, tutte le A entro 1,3×min); tie a 2 decimali implementati nello script (round(…,2)); rc letti dai file (full-rc=1, admission-out/rc=0, batteria rc 0/0/0).

## Il punto che ridimensiona
I «3 campioni L-A» su calls sono TRE MISURE DELLO STESSO PAIO DI BINARI (zero rebuild da S-114): a livello di layout il campione L-A è N=1, contro N=2 layout nulli. La ripetibilità di un binario non è sistematicità della leva: un artefatto di layout specifico di QUESTA build riprodurrebbe identicamente −6,5/−7,0/−6,5. Anche trattando i 5 campioni come scambiabili, la probabilità che i 3 peggiori siano i 3 L-A è 1/10 (p=0,10); a livello layout, 1/3. In più: a calls la grana del cronometro è 0,5 ns/iter (0,01 s / 2e7) e il floor è UN campione per categoria che trasla tutte le D — il margine sopra la guardia (1,0 ns) vale 2 quanti. «FATTO» è troppo; «questa build di L-A paga 6,5-7,0 su calls, oltre banda nulla N=2» è ciò che i dati reggono.

## Verdetti
1. **REGGE** — numeri, famiglie, tie e rc verificati dai raw; guardia sfondata per regola pre-registrata.
2. **RIDIMENSIONATO** — repliche di un solo layout ≠ campioni indipendenti; p≈0,10 (1/3 a livello layout); banda nulla N=2 con due valori identici sottostima il max nullo.
3. **RIDIMENSIONATO** — 3×1742 verificato (rc e nomi), ma run2-3 riusano la build di run1: chiusa la flakiness dei TEST su UNA build, non della BUILD; se il 1741 di S-113 era build/ambiente, N effettivo ≈ 2 build.
4. **REGGE** — p.7 richiede TUTTI i gate; verdetto acquisito al micro. Costo dichiarato: soglia held-out emendata mai esercitata, spread_pin resta non pubblicato.

## Azioni
1. Rebuild di 2c18b2e con layout perturbato (link fresco o zavorra diversa) e rimisura calls: separa tassa-di-codice da tassa-di-questa-build.
2. Portare la banda nulla calls a N≥4 layout prima di dichiarare tasse.
3. Alzare N di calls (o floor mediano su R campioni per coppia) finché un quanto ≪ margine di guardia.
4. Batteria: una run aggiuntiva con build FRESCA (target pulito) per coprire la build; riformulare la lettera come «flakiness test chiusa».
5. Eseguire comunque l'held-out emendato sul paio conservato: pubblica spread_pin e testa la tassa calls sui giudici reali.
