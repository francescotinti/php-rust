# Verbale S-146 — sedia KLABNIK (lente: chiarezza, spec dei gate, testabilità)

## VERDETTO: CON EMENDAMENTI (quasi-opposizione su TakeSlot come default)

Il fatto che decide tutto è NELLA sonda stessa: TakeSlot **non riduce i
movimenti** — sposta invece di copiare, ma il memcpy (69,5%, 1,06 s) resta
pagato. Ciò che TakeSlot compra è la coppia inc-dec: **0,21 s a frazione
100%**. La banda del giudice della scommessa (coppia ORM ±0,7%) vale ~0,3 s.
Il tetto della leva è SOTTO la risoluzione del suo giudice **per costruzione,
con i numeri già firmati**: non serve alcun censimento per saperlo. Anche
nella lettura larga (inc-dec+nota = 0,46 s, ammesso che il take eviti la
nota — non provato) il margine è 1,5× la banda, sotto la mia soglia di
distinguibilità 2×. Si applica la lettura severa (precedente S-144 memops).

## Posizioni a–e

**a) Forma d'emissione — CONCORDO con riserva.** Vero che il flag `take`
compilato non è un corpo caldo in più (design96 #1) e sta sotto il tetto
A-LB-97-1; ma la forma resta ACCADEMICA finché il tetto 0,21 s non è
sfondato da un censimento ORM firmato. Non si istruisce una forma per una
leva pre-uccisa. Per le leve borrow-first la forma è quella già spedita
(peephole in place, fallback per costruzione — L-FR1); disasm bl-count
obbligatorio su ogni tocco a run_loop (criterio-B p.4, resta).

**b) Perimetro semantico — CONCORDO.** Il vincolo Stogov (CV non consumati,
morte anticipata osservabile) morde solo le leve che muovono la fine-vita.
Le borrow-first sono semanticamente invisibili per costruzione (zero
liveness): il perimetro fedele è gratis. Fixture weakrefs/destructor a gate
restano.

**c) Censimento F1 su ORM — MI OPPONGO, ora.** Un censimento si deve a una
regola che ne ha bisogno; la regola qui decide già sui numeri S-145. Ciò che
serve invece è un **census SITI-CONSUMATORI su ORM** (monobinario, r1==r2,
criterio ≤10 righe firmato PRIMA): movimenti per sito × prezzo per classe
(2,88–3,85 ns) = guadagno modellato per fetta, ordinati.

**d) Alternative — CONCORDO: la leva che si istruisce è FR1-ext /
borrow-first** (chiave da SLOT, FieldRead/isset, siti consumatori): è
l'unica che elimina il MOVIMENTO, cioè il pavimento 69,5%. Due precedenti
spediti, zero liveness. **Arena-conteggi: si ARCHIVIA** — una leva senza
definizione non può avere criterio né giudice; riapribile solo per NOME con
design ≤1 pagina (perimetro semantico + giudice nominato).

**e) CON EMENDAMENTI.** Il perimetro modellato è 1,52 s su 37,6 s: B3 al
massimo teorico compra ~4% del gap — **non compra neanche la tappa ≤3× da
solo**; è un addendo. La glue ~4,4 s resta fuori modello: nessun claim.

## Emendamenti

**R1 (regola pre-registrata, PRIMA del census siti):** una fetta si apre se
guadagno modellato ≥ max(4 ns/iter sul SUO micro nominato, banda); una fetta
suite-judged pretende tetto modellato ≥ 0,6 s (2× banda coppia). Sotto:
micro-judged con suite a sola guardia (keep-partial-wins).
**R2:** il moltiplicatore 4,5–6,5% (SCREEN R=1) è MORTO per B3: non si
promuove, si SOSTITUISCE coi prezzi sonda-B. Ogni banda derivata da lì
(design95 §P1, righe guadagno_* del recount) è NON CITABILE nei criteri B3.
**R3 (KS riscritti):** KS-B1 («−25% churn+memops») era tarato sul programma
B1/B2: per B3 la predizione è la somma dei guadagni modellati spediti.
KS-B2 (4 sessioni, più severo con Gregg-5) resta, ma l'esito è **stop
famiglia + riconvoca**, non revert delle leve che tengono le proprie guardie.

## Kill-switch pre-registrabili

- **KS-B3-K1:** tetto modellato < soglia R1 o micro non nominato ⇒ fetta
  non si apre.
- **KS-B3-K2:** TakeSlot chiuso per tetto (0,21 s < 0,3 s a frazione 1);
  riapribile solo se un censimento ORM firmato mostra componente
  acquistabile ≥ 0,6 s.
- **KS-B3-K3:** guadagni modellati spediti ≥ 0,5 s con coppia ORM in banda
  ±0,7% ⇒ modello falsificato ⇒ stop famiglia, riconvoca.
- **KS-B3-K4:** 4 sessioni B3 con ORM fermo ⇒ stop famiglia + riconvoca.
- Gate semantici invariati; fail NUOVO per NOME in weakrefs/destructor ⇒
  STOP fetta.
