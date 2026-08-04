# Verbale sedia 5 — Bak (VM: alloc-rate, code-cache, percorsi caldi, dispatch) — WP-98

## VERDETTO

**Il passo 2 ha ABUSATO del mio tetto. Verdetto di chiusura NON verdict-grade:
va declassato a SOSPENSIONE, non archiviazione.** Il §WP-97 è nell'ordine
giusto per due terzi, ma la sua prima voce è sbagliata e la sua motivazione al
punto 1 è tecnicamente falsa.

**L'abuso.** A-LB-97-1 dice «Δ netto bracci caldi ≤ 0», cioè impone un
CONTROLLO. Non ha mai detto «un corpo caldo costa ~1%». Il passo 2 ha letto il
tetto come una TARIFFA e l'ha sottratta a una banda. I nostri stessi numeri
falsificano la tariffa: WP-44 v1 (2→4 corpi) = **+1,17%**, v3 (2→9 corpi) =
**+1,01%**. Nove corpi costano MENO di quattro. Se ci fosse un prezzo per
corpo, quell'ordine sarebbe impossibile: la varianza di layout domina il
termine per-corpo. Accanto: WP-41 = +0,62% per ~60 siti inline, WP-33 = +2,9%
per UN branch mai preso. Il costo va da 0,6% a 2,9% e non è monotòno nel
numero di corpi. Un intervallo che copre 5× non sottrae niente a una banda di
0,84–1,21%.

**L'abuso di grado, che è il più grave.** design96 §4 dichiara con cura che il
guadagno è SCREEN — e poi gli sottrae un costo MISURATO (A/B interleaved,
oracle di giornata) preso da una leva di FORMA DIVERSA. Il documento applica la
disciplina del grado al lato che vuole salvare e non al lato che uccide. Un
costo misurato su un'altra leva non è una misura di questa.

**Il cerchio.** Il costo vero era predicibile a costo quasi nullo: `nm -S`
sulla taglia predetta. Non è stato calcolato «perché non si predice la taglia
di un braccio che non si scriverà» — ma la taglia era l'INPUT della decisione
di scriverlo. La decisione ha consumato il proprio output.

## Emendamenti (A-LB-98-n)

- **A-LB-98-1 (la tariffa è vietata)**: nessun conto netto può sottrarre un
  costo di corpo caldo da una banda finché quel costo non è misurato SU QUESTA
  FORMA. Il tetto A-LB-97-1 è un controllo a posteriori, non un addendo a
  priori. Il verdetto del passo 2 va riscritto come «non decidibile senza
  misura», che è un'altra cosa da «netto zero».
- **A-LB-98-2 (prima voce: il denominatore, non O1)**: il moltiplicatore
  4,5–6,5% è R=1 senza spread e ha diviso o moltiplicato ogni decisione di due
  sessioni. Ri-profilo **R≥3, stesso workload, ZERO cambi di codice**, più i
  contatori discriminanti della mia consulenza §1 (mispredict indiretti/op,
  L1I-miss/op, normalizzati su `op-census`). È misura sull'OGGETTO, non
  apparato; rimette in moto il cronometro in mezza giornata; e dice se O1 ha un
  canale PRIMA di scriverla. Regola scritta prima, come in consulenza §1.
- **A-LB-98-3 (controllo positivo di O1, che è DOPPIO)**: (a) meccanismo —
  `nm -S run_loop` con taglia predetta prima, PIÙ l'elenco dei simboli
  outlineati intersecato con `op-census`: zero dei top-40 outlineati, ogni
  outlineato sotto soglia di frequenza. La taglia da sola prova che LLVM ha
  outlineato, non che il working set caldo sia calato. (b) canale — L1I-miss/op
  DEVE calare; `op-census` totale INVARIANTE. Solo dopo, l'orologio.
- **A-LB-98-4 (il mio bite test era mal posto — lo correggo)**: «outlinea
  bracci mai eseguiti, predici CPU invariata» è incoerente, perché outlinare
  codice freddo interlacciato È il meccanismo i-cache. Il null control corretto
  è outlinare un Δ-taglia equivalente in una funzione FUORI da `run_loop`.
- **A-LB-98-5 (punto 1 del §WP-97, la frase falsa)**: «branch per-sito, quindi
  ben predetto dal BTB» è un errore di livello. Il flag `take` è statico per
  SITO DI BYTECODE, ma l'istruzione di branch sta a UN PC dentro `LoadSlot` ed
  è eseguita da tutti i siti: la predizione dipende dalla sequenza dinamica del
  bit, non dalla sua staticità. Col nostro raw quel bit è preso il 42,33% delle
  volte (18,65% sul solo nucleo stringhe): entropia quasi massima, il caso
  peggiore per un condizionale a due vie sul percorso di lettura più caldo che
  abbiamo. Vero che non aggiunge un braccio; il prezzo è che sposta la
  mispredizione da un indiretto (dove il predittore ha storia) a un diretto mal
  bilanciato. Le due forme vanno MISURATE, non argomentate.

## Kill-switch (KS-LB-98-n)

- **KS-LB-98-1**: se A-LB-98-2 dà mispredict/op < 0,10 **e** L1I-miss/op < 0,02,
  O1 non ha canale: non si scrive, si passa a O2 (dieta della testa).
- **KS-LB-98-2**: O1 con taglia calata ma L1I-miss/op invariato → la leva non è
  provata; nessun Δ tempo rivendicabile, anche se favorevole.
- **KS-LB-98-3**: `op-census` totale non invariante su O1 → è cambiato il
  lavoro, non il layout: revert, la coppia non è valida.
- **KS-LB-98-4**: qualunque documento futuro che sottragga una cifra di costo
  per-corpo-caldo senza misurarla su quella forma → respinto in radice
  (A-LB-98-1).
- **KS-LB-98-5**: terza sessione consecutiva senza cronometro sulla rotta
  CPU-VM → la rotta si dichiara SOSPESA per nome nell'handoff.

## Refutazioni capitali

**Tre.**

1. **La tariffa non esiste**: 2→9 corpi è costato meno di 2→4 (WP-44), quindi
   il costo di «un corpo caldo in più» non è una costante e non può essere
   sottratto da una banda. Il verdetto «netto non distinguibile da zero» del
   passo 2 è **refutato nella sua derivazione** — la conclusione può restare
   vera, ma non è provata da quel conto.
2. **O1 non è la prima voce giusta**: la mia stessa consulenza scommetteva sul
   PROLOGO (O2), non sull'i-cache. Mettere O1 per prima significa scrivere la
   leva del ramo che non ho mai dato per favorito, senza il contatore che
   discrimina i tre rami. Prima il discriminatore (A-LB-98-2), che è anche il
   modo più economico di far ripartire il cronometro.
3. **«Per-sito, quindi ben predetto» è falso** (A-LB-98-5): confonde la
   staticità del flag con la biettività del branch. Il bit è quasi 50/50.

**Sul cronometro fermo**: sì, è un problema, e non per igiene. Il pin phpr è
invariato, quindi il profilo di WP-95 descrive ancora questo binario — ma
descrive R=1. Due sessioni hanno prodotto decisioni derivate da un numero senza
spread. Non serve una leva per rimediare: serve ripetere una misura che già
sappiamo fare.
