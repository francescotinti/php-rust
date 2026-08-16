# Verbale HEJLSBERG — Concilio S-146, quesito B3/filone conteggi

Sedia: ingegneria dei compilatori (forma d'emissione, dataflow a compilazione,
costo per-sito). Bozza INDIPENDENTE; fascicolo letto per intero; nessun .rs
aperto (finestra di misura attiva).

## VERDETTO

**CONCORDO CON EMENDAMENTI** sul quesito; l'ordine d'istruttoria che propongo
INVERTE la priorità implicita («TakeSlot S-140 prima»): la sonda stessa dice
che il collo è il memcpy (69,5%), e il take NON elide il memcpy — lo elide
solo il borrow.

## Posizioni a–e

**a) CON EMENDAMENTI.** La forma-flag (bit `take` su LoadSlot, deciso a
compilazione) è ammissibile sotto il tetto WP-39..44: nessun corpo caldo
nuovo, branch per-sito dentro il braccio esistente (RC-1), BTB-predetto. Non è
gratis: si istruisce con (1) taglia `nm -S` di run_loop PREDETTA prima;
(2) disasm bl-count prima/dopo (lezione H-C2); (3) flag=0 ⇒ path byte-identico
al corrente (fallback per costruzione, precedente L-FR1); (4) banda-layout
pre-registrata. O1-outlining NON è prerequisito di QUESTA forma (era condizione
sui corpi NUOVI); ma taglia oltre predizione ⇒ stop.

**b) CONCORDO** con Stogov: per ogni take il perimetro fedele è il nucleo
senza identità (str, scalari). Rilievo di lente: la classe
borrow/through-borrow non ha bisogno di ALCUN perimetro liveness — non muove,
non anticipa morti, semanticamente invisibile (HC1 S-140 e L-FR1 S-145,
entrambe spedite).

**c) MI OPPONGO com'è posta.** Il primo censimento dovuto su ORM non è
F1-liveness ma il censimento dei DIGRAMMI (coppie `LoadSlot;CallArg` /
`;StoreSlot` / `;Dim` / `;Binary`, conteggi monobinari ×2, r1==r2): le fette
peephole si prezzano lì. F1-liveness su ORM SOLO se la forma-flag sopravvive
al gate prezzo (R2) — i tassi del media group WP (would_take_rc 47,1%) non si
trasferiscono a ORM per fede.

**d) CONCORDO con inversione d'ordine**: borrow/through-borrow PRIMA di ogni
take, perché elide il movimento intero, memcpy compreso — l'unica classe
coerente con KS-B4. **Arena-conteggi: ARCHIVIARE** — mai definita; ogni
lettura sensata ricade nei veti confermati 9/9 (alloc-removal senza costo
sostitutivo, contenitori sul call path).

**e) CON EMENDAMENTI.** B3 compra al massimo 1,52 s modellati (~3–4% del
tempo phpr; ~0,3× degli 8,6×) — nessun claim su parità né sui ~4,4 s di glue
fuori modello; coerente col mio «residui ≈6,5× a B completa». KS-B1 (−25%
churn+memops) è dimensionato su B1/B2: ereditato tal quale, B3 nasce
falsificata anche in pieno successo. La scommessa va ri-registrata alla scala
del perimetro modellato.

## Emendamenti

- **R1 (refutazione centrale)**: take converte clone→move ma il memcpy dei
  16 B e la store di Undef RESTANO; compra solo inc-dec+nota+glue di drop
  (≤~31% del prezzo pair, dalla partizione). La sonda deve prezzare
  «move+undef» vs «clone+drop» per classe PRIMA di aprire il flag.
- **R2**: gate prezzo pre-registrato per la forma-flag: risparmio/mov ×
  conteggio indirizzabile (dal censimento c) ≥ max(4 ns-equiv., rumore,
  banda) — sotto soglia, il flag non si scrive.
- **R3**: l'ordine per moltiplicatore PER-TIPO (scalar 91,1M · str 104,1M ·
  arr 60,9M · rc 111,5M) è inattuabile a compilazione: i tipi sono dinamici,
  l'emissione vede solo PATTERN. Ordine = contributo assoluto per pattern
  (conteggio×prezzo); il per-tipo resta tie-break osservativo
  (str ≈0,40 s ≈ rc 0,37 > scalar 0,26 ≈ arr 0,23).
- **R4**: ogni fetta peephole eredita il protocollo L-FR1 per nome: criterio
  ≤10 righe, R=5 ABAB, disasm agli atti, guardie con giudice DENTRO lo
  script (az.rev. S-145 #3).

## Kill-switch pre-registrabili

- **KS-H1**: bl-count run_loop post>pre, o taglia `nm -S` oltre predizione ⇒
  stop fetta.
- **KS-H2**: censimento digrammi ORM senza alcun pattern sopra il gate R2 ⇒
  B3 chiusa SENZA codice.
- **KS-H3**: fail NUOVO per NOME in weakrefs/destructor ⇒ STOP (invariato).
- **KS-H4**: scommessa B3 riscalata; micro churn 5/5 ma ORM fermo oltre
  l'orizzonte più severo (Klabnik/Gregg) ⇒ ridimensionare, non estendere.

## Mandato inverso

Oggi sappiamo che il prezzo per-movimento è PAVIMENTO (2,88 ns) e non
contatori; ieri non sapevamo che il 69,5% sta nel move stesso. Questo
retrocede il take (ieri candidato naturale) e promuove il borrow — che il
take non sostituisce.
