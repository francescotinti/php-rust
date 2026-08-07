# WP_SESSION_110 — S-110: coppia WP in banda (debito saldato) + TESI FRONTEND FIRMATA (xctrace)

**In una frase**: la suite WordPress completa conferma che il terzo lotto di
ottimizzazioni non costa nulla sul carico reale (1,867×, dentro la banda
pre-registrata), e la prima misura coi contatori hardware su ENTRAMBI i motori
ha firmato il perché aritmetica e proprietà restano lente: phpr butta un terzo
dei cicli ad aspettare le istruzioni (frontend), l'oracle il 3% — la prossima
leva ha ora un bersaglio nominato (threaded-dispatch).

**SCOREBOARD** (micro NON rimisurati: pin 92909544 invariato, valori S-109):
**arith 9,3 = · prop 7,9 = · calls 5,1 = · str 5,3 = · arr 3,9 = · re 3,5 =**
WordPress: **full ON 1,867× DENTRO banda [1,81;1,88]** (rif resta 1,842) /
OFF 1,869× (↓ da 1,911, sotto la famiglia storica) · media 2,673/2,612 (↓↓,
off sotto il rif 2,64; quinta lettura, voce aperta) · peak full ON 1843,53 MiB.
**Leve perf di codice spedite: 0 (dichiarato)** — la finestra leva è andata
alla leva MISURATIVA (d) per NOME dall'ordine S-110, eseguita e FIRMATA.

**Data**: 2026-08-07 (21:2x–23:5x). **Modello**: Fable 5. **Commit**: ded4a74→(chiusura) pushati.

## Esiti secchi
1·coppia bimodale VERDE ×4 gambe (criterio PRE ded4a74; full ON 1,867 in banda,
parità per NOME, sola eccezione wp_is_stream ×2; debito lotto-3 ASSOLTO) →
2·leva (d): T1-stock FAIL (niente eventi grezzi), T1-custom forgia .tracetemplate
IGNORATA IN SILENZIO (smascherata a esito-esatto), kpc senza sudo ⇒ **criterio
v2 EMENDATO E COMMITTATO PRIMA della misura** (9ff53cf: quote top-down nominate
MetricTable) → raccolta bilaterale 18 registrazioni R=3 → **TESI FIRMATA:
delivery arith 9,75× · prop 5,96× · controllo arr 1,04×** (specificità ✓;
firma di FAMIGLIA, causa IC/ITLB/redirect non ripartita) → 3·gh-status-sync
PUBBLICATO (corpus fresco 2652/1415/1238; README blockquote perf riscritto) →
azioni revisore S-109: commit-per-passo ✓ (14 commit), admission bipartita ✓,
diff-prelude N/A (nessuna leva d'emissione). 🔵 Incidenti d'apparato NOMINATI:
figlio tracciato BLOCCATO in open() su volume esterno (TCC headless) ⇒ giudici
da copie interne byte-identiche; scratch xctrace ha RIEMPITO il disco interno
(15G ktrace, rc=134, harness in ENOSPC) ⇒ sbloccato via overwrite mirato, 4
guardie nello script (TMPDIR esterno, cap 60s, fail-closed <5G, pulizia).

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il profilo bilaterale a contatori paga al primo colpo**: la tesi
  «icache-bound» vietata come premessa è diventata FIRMA con un criterio
  pre-registrato e 18 registrazioni — e il controllo di specificità (arr 1,04×)
  è ciò che la rende una firma e non una suggestione.
- ⭐⭐ **Strumenti GUI-first mentono due volte in headless**: il figlio tracciato
  si blocca in open() (TCC) e la forgia del template fallisce IN SILENZIO con
  rc=0 — solo il braccio a esito-esatto (0 occorrenze dell'evento) la smaschera.
- ⭐ **Lo scratch di uno strumento è parte del suo perimetro di disco**: 15G di
  ktrace in $TMPDIR interno con trace-output sull'esterno; la guardia va sul
  volume dove lo strumento SCRIVE davvero, non dove salva l'output.
