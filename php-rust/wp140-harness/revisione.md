# Revisione S-140 — revisore singolo, lente MISURA

## Reperto principale (indebolisce la prima metà del claim)
Il reticolo del giudice è 3,33 ns/iter (tick 0,01 s di `/usr/bin/time` su N=3e6): D può valere solo {0; 3,3; 6,7; 10; …}. La promozione poggia su D=+6,7 = DUE tick, con rumore drop-1 = 1 tick, banda-layout = 1 tick nella STESSA direzione (il null-lever 275e5836 ha già reso +3,3 in banda6) e swing inter-run same-pair osservato di 2 tick (objdatains: +6,7 in t1 → +13,3 in t2, stessi binari). La soglia 4,0 (1,2 tick) sta sotto il primo valore misurabile non nullo e la formula max() non compone rumore e layout: l'evidenza netta sopra il controllo nullo è UN tick, cioè la risoluzione stessa. La spedizione si salva per i mitiganti — segno B<A 5/5 in t2 e 5/5 in conferma post-pin (D=+10,0), prezzi per-check 1,65/1,1/1,67 conciliabili entro il tick — ma «D=+6,7» va letto 6,7±3,3 e l'evidenza portante è la conferma, non il t2.

## Reperti secondari
1. **Check modello decorativo**: `s140-ab-hc1.sh` r.121 testa SOLO `delta > 80` (bound v1, non i 40 del v2) e stampa «dentro il modello (10-40)» perfino per il D=+3,3 del NULL-lever. Il t1 (3,3) aveva già refutato il range v1 senza dichiararlo; il v2 rifonda il range sul risultato di t1 (t2=6,7 al bordo basso del refit): la riga «vs modello» non conferma nulla.
2. **«Guardie 10/10» non riscontrato**: nel r5t2 le guardie sono 9 (t1 dichiarava «9/9»).
3. **Coppia**: conferma = non-falsificazione a bassa potenza (S-140 tutta interna al rif, margini 0,013/0,008; banda propria 0,012). Nel verdetto convivono DUE bande (r.46 «0,012 sostituisce 0,041», r.60 canonica 0,033): trappola per S-141.
4. **Deroga push «atti di pin»**: compare solo nel criterio pair committato 12:29, DOPO i push 12:09/12:12 (sanatoria ex post vs v1 p.7). Inoltre `quiet_wait` oltre 4 h marca skipped-busy SENZA requeue: i commit di pin possono restare senza CI in silenzio.

## Vagliate e respinte (con la prova)
- **Criterion-shopping v2**: respinto — v2 + m-hintcall6 committati in bc60c86 11:49:44, verdetti t2 in 7161a27 12:29:37; soglia invariata; la densità scala il segnale, non il criterio.
- **quiet_wait**: verificato in `ci/ci-runner.sh` r.95, dentro il loop, prima di OGNI job, MLOCK incluso: l'emenda regge tecnicamente.
- **Aritmetica unione banda**: corretta (min 1,752, max 1,785 ⇒ 0,033; S-140 da sola 0,012).
- **Sonda/census**: contatori conformi al criterio (avoided≈6e6; ORM 35,6M check ⇒ ~0,13% suite, quota dichiarata).

## Azioni S-141
1. Giudici ns-scale sub-tick: N≥1e7 o cronometro in-giudice (µs); regola: soglia ≥ banda-layout + 2 tick.
2. Harness: bound v2 nell'header/check, check modello bilaterale o lato basso dichiarato non-gate, nome giudice corretto.
3. Una sola banda canonica nel verdetto pair (0,033 cross-finestra); 0,012 etichettata intra-finestra, mai da sola.
4. Deroga pin-push scritta in REGOLE PRIMA degli atti; requeue (non skip) per skipped-busy.
5. Prossima leva churn: census preventivo della quota suite PRIMA della spedizione (HC1 vale ~0,13%).
