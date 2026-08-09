# Revisione S-121 — lente PROCESSO (revisore singolo)

## Verdetti

**Claim principale (L-ST1 «REFUTATA in modo PULITO»)** — **RIDIMENSIONATO**. Il *processo* regge: ordine commit verificato (criterio ae92801 → patch/census 1fb4aae → giudice 298ff80 → verdetto bc10f9c), patch mai nei sorgenti, ripristino al pin AL BYTE, regola smoke implementata coincidente col p.7 (concorde ≤−1,00 su tutte le coppie E |D_med|>banda; soglie 2,50/3,33), nessuna deroga. Ma «refutata» sovra-afferma: la banda-zavorra 2,50 è esattamente **UN quanto del timer** (0,01 s / 4 M = 2,5 ns/iter; tutte le misure zavorra sono multipli di 2,5) — è satura alla risoluzione dello strumento e misura solo il run-to-run su UN binario, non il layout tra binari (alternativa che il verbale stesso NOMINA). Asimmetria: la promozione esigeva ≥5,00 (2×quanto), la refutazione s'è accontentata di >2,50; con banda-v2 7,50 il full avrebbe giudicato (−6,25 dentro). Smoke R=2 senza filtro-famiglia: la coppia −7,50 pesa metà del verdetto. Verdetto onesto: «fermata dalla regola pre-registrata, refutazione PROVVISORIA».

**Grado server** — **REGGE**: .done rc=0 voids=0 nei due modi, tre pin FAIL-CLOSED da file, env -i, watchdog, conteggi pinnati 413/3508.

**ABAB** — **REGGE con riserve**: parità failnames 4/4 verificata; «non ripartibile» corretto e il .out sotto-afferma bene (N=2 = direzione). Ma il segno concorde è NEGATIVO 2/2 (s120 più lento su WP) e lo spread-arbitro 18,73 s è dominato dall'unica gamba B2.

**Fixture preg + §3.18** — **REGGE con riserva**: golden bilaterali fail-closed, catalogo aggiornato; però il gate NON è cablato in `scripts/pin-phpr.sh`: morde solo se invocato a mano.

**Colonna arr** — **REGGE** (D1/D2/D3 dichiarati, 245,01×100k ≡ 4,02×6,1M).

## Ciò che il claim tace

- Criterio p.2 prevedeva «arr atteso in CALO»: **non realizzato** (245,01→245,01) e il census-verdetto controlla solo gli AUMENTI — predizione mancata assente da ogni verbale; indebolisce il modello «builtin by-value anche lì».
- La lezione ⭐⭐ «bookkeeping > malloc» è affermata come fatto ma è UNA delle due ipotesi (l'altra: layout).
- Il candidato 2e1eda8d resta stashato: un retry senza banda-layout riuserebbe lo stesso arbitro viziato.

## Azioni S-122

1. Misurare la banda-LAYOUT str (≥3 build perturbate dello stesso sorgente-pin) PRIMA di ogni retry; nel frattempo early-stop str usa max(zavorra; 2×quanto=5,00).
2. Con lo stash già pronto, eseguire il full A/B L-ST1 (costo basso) e chiudere refutazione vs layout.
3. Emendare il census-verdetto: le predizioni secondarie del criterio (arr in calo) si verificano e l'esito va a verbale anche se mancato.
4. Cablare s121-fx-preg-gate.sh nella catena pin/corpus-gate.
5. Nei report ABAB dichiarare il segno («2/2 s120 più lento, dentro rumore»), non solo «non ripartibile».
