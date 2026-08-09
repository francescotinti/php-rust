# Revisione S-122 — lente MISURA (revisore singolo)

VERBALE — revisione adversariale S-122, lente MISURA

**Reperto che invalida il claim 1 e trascina il 2.**

1. **BANDA_LAYOUT è confusa con la posizione nel round.** `s122-layout-band.sh` esegue l'ordine fisso P0→P1→P2→P3 in tutti e 5 i round, senza rotazione. P0 (il pin) risulta minimo o pari-minimo in 6/6 categorie [nota di ricevuta: 4/6 + calls pari; re va al contrario — il reperto resta non separabile]; arith/prop/str/arr crescono monotoni con l'indice, re decresce. Una dose-risposta alla taglia .text non produce questa posizione sistematica; un effetto d'ordine sì. La banda misura in prevalenza «primo del giro», non il layout — e i tre probe non sono tre estrazioni indipendenti.

2. **Lo stesso artefatto sta dentro L-ST1.** `s122-ab-st1.sh:50` misura sempre TA prima di TB: A=pin è «primo del giro» anche lì. Banda str (+5,00, P0 il più veloce) e D_med (−5,00, A il più veloce) hanno modulo e verso identici perché sono la stessa cosa misurata due volte. Il margine zero non è coincidenza: è mancata auto-cancellazione.

3. **Risoluzione.** `/usr/bin/time -p`, quanto 0,01 s; tutte le cifre sono multipli del quanto (str 2,50 · re 5,00 ns/iter). BANDA_LAYOUT_str = 2 quanti, _re = 1 quanto, _calls = 0. Sul RAW `st1-out/st1-full-runs.tsv`, un solo tick sulla coppia mediana (A 0,87→0,88) porta D_med a −2,50: REFUTAZIONE CONFERMATA → NON-DISTINGUIBILE. Un tick su dieci run.

4. **Asimmetria residua nel giudice.** `s122-st1-giudice.sh`: soglia_ref = max(BL; zavorra; 2×quanto) è l'unica soglia dell'apparato che omette 2×spread_A, presente sia in soglia_promo sia in tutte le guardie. Con spread_A str = 7,50 la refutazione sarebbe −15,00 e non reggerebbe. Il p.6 dichiara «due lati» ma li pesa 3:1.

5. **L-RE2**: mediana di 2 = media. −15,00 nasce da 4 e 2 tick; un tick (1,08→1,07) dà −10,00 = banda, niente early-stop. Fermata prudente, non un dato.

6. Minore: la famiglia 1,3×min non ha mai escluso nulla (fam=5/5 ovunque; arr P2 211,67 ammesso).

## Azioni S-123

1. Ri-misurare BANDA_LAYOUT con ordine dei binari PERMUTATO per round, includendo una seconda copia del pin come quarto binario: la dispersione pin-contro-pin è il vero pavimento.
2. Portare i run bersaglio a ≥5 s (N×5 su str/re/arr) o a un timer sotto il millisecondo: nessuna soglia sotto ~4 quanti è dicibile.
3. Alternare l'ordine dei due binari per coppia nell'A/B e ri-giudicare L-ST1 sul nuovo TSV prima di trattare la refutazione come acquisita.
4. Emendare soglia_ref con 2×spread_A, simmetrica a soglia_promo; se non regge, L-ST1 torna non-provata.
5. Ripetere lo smoke L-RE2 a R≥5 con ordine alternato prima di archiviare il candidato.
