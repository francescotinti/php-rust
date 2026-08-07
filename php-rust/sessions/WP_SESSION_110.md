# WP_SESSION_110 — S-110: coppia WP in banda (debito saldato) + TESI FRONTEND FIRMATA (xctrace)

**In una frase**: la suite WordPress conferma che il terzo lotto non costa
nulla sul carico reale (1,867×, in banda), e la prima misura coi contatori
hardware su ENTRAMBI i motori ha firmato perché aritmetica e proprietà restano
lente: phpr butta un terzo dei cicli ad aspettare le istruzioni (frontend),
l'oracle il 3% — la prossima leva ha un bersaglio nominato (threaded-dispatch).

**SCOREBOARD** (micro NON rimisurati: pin 92909544 invariato, valori S-109):
**arith 9,3 = · prop 7,9 = · calls 5,1 = · str 5,3 = · arr 3,9 = · re 3,5 =**
WordPress: **full ON 1,867× DENTRO banda [1,81;1,88]** (rif resta 1,842) /
OFF 1,869× (↓ da 1,911, sotto la famiglia storica) · media 2,673/2,612 (↓↓,
off sotto il rif 2,64; quinta lettura, voce aperta) · peak full ON 1843,53 MiB.
**Leve perf di codice spedite: 0 (dichiarato)** — finestra leva = leva
MISURATIVA (d) dall'ordine S-110, eseguita e FIRMATA. **Data**: 2026-08-07
(21:2x–23:5x) · **Modello**: Fable 5 · **Commit**: ded4a74→(chiusura) pushati.

## Esiti secchi
1·coppia bimodale VERDE ×4 gambe (criterio PRE ded4a74; parità per NOME, sola
eccezione wp_is_stream ×2; debito lotto-3 ASSOLTO) → 2·leva (d): T1-stock FAIL
(niente eventi grezzi), forgia .tracetemplate IGNORATA IN SILENZIO (smascherata
a esito-esatto), kpc senza sudo ⇒ **criterio v2 EMENDATO E COMMITTATO PRIMA
della misura** (9ff53cf: quote top-down nominate MetricTable) → 18 registrazioni
bilaterali R=3 → **TESI FIRMATA: delivery arith 9,75× · prop 5,96× · controllo
arr 1,04×** (firma di FAMIGLIA, causa IC/ITLB/redirect non ripartita) →
3·gh-status-sync PUBBLICATO (corpus fresco 2652/1415/1238; blockquote perf
riscritto) → azioni revisore S-109 saldate (commit-per-passo ✓ 14 commit,
admission bipartita ✓, diff-prelude N/A senza leva d'emissione). 🔵 Incidenti
d'apparato NOMINATI: figlio tracciato bloccato in open() su volume esterno (TCC
headless) ⇒ giudici da copie interne byte-identiche; scratch xctrace ha RIEMPITO
il disco interno (15G ktrace, rc=134) ⇒ 4 guardie nello script.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il profilo bilaterale a contatori paga al primo colpo**: «icache-bound»
  vietata come premessa è diventata FIRMA con criterio pre-registrato — ed è il
  controllo di specificità (arr 1,04×) a renderla firma e non suggestione.
- ⭐⭐ **Strumenti GUI-first mentono due volte in headless**: figlio bloccato in
  open() (TCC) e forgia del template fallita IN SILENZIO con rc=0 — la smaschera
  solo il braccio a esito-esatto (0 occorrenze dell'evento atteso).
- ⭐ **Lo scratch di uno strumento è parte del suo perimetro di disco**: la
  guardia va sul volume dove lo strumento SCRIVE, non dove salva l'output.
