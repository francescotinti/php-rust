# Verbale Sedia 4 — Hejlsberg (compilatori incrementali, interning, pipeline)

## VERDETTO: CON EMENDAMENTI

S-97.1 è metodologicamente pulita (criterio scritto prima, onorato; parità per
NOME; controllo positivo del flag). Le refutazioni colpiscono la BATTERIA di
parità, il POSTO del residuo AssignOp e il testo del programma H-B*.

## Refutazioni capitali

**RC-1 — La batteria v3 gira a un punto di pipeline DIVERSO dalla produzione,
e per `{main}` è plausibilmente VACUA.** In produzione `lower_func` gira dentro
`compile_body` (func.rs:163-165), PRIMA della cessione WP-65; poi
`compile_program_impl` fa `main.seed_slots = len; main.slot_names =
Box::default()` (mod.rs:326-327). Il harness di test `lowered()`
(reg_lower.rs:360-382) applica il pass all'output di `compile_program` — DOPO
la cessione — dove `{main}.slot_names` è VUOTO: `fold_slot` (riga 85, egualità
col nome) non può matchare alcun `LoadVar` di `{main}`. I 13 snippet della
batteria sono TUTTI script top-level: per i loro corpi `{main}` il pass testato
è con ogni probabilità l'identità. La sessione ha appena imparato «pretendere
la prova positiva che il flag ha morso» (tail/binario stantio) e la sua stessa
batteria non la pretende: `stage2v3_rewrites_hot_windows` asserisce forme fuse
solo in `fn f`, mai in `{main}`. Finché manca il controllo positivo, le gambe
`{main}` della batteria non sono evidenza.

**RC-2 — «Il conteggio è quasi chiuso (11 vs 7)» confonde lo strumento col
motore.** 11 è flag-ON, dormiente; la strada di parità resta a 19 (fattore
2,7×). La scala H-B2/H-C/H-D si attiva «dopo H-A1 e H-B1», ma H-A1 è caduta e
il suo effetto non raggiungerà MAI il binario spedito senza una decisione di
promozione che il programma non nomina. Il ladder condiziona su uno stato
irraggiungibile per costruzione.

**RC-3 — Il residuo AssignOp nel pass è il posto SBAGLIATO; e il quesito
sull'ordine è mal posto.** (a) Ordine: `thread_jumps` è strettamente in-place
(nessun op inserito/rimosso), quindi l'insieme delle finestre è essenzialmente
invariante all'ordine — spostare il pass prima del threading non allunga
nulla. Il limitatore vero è la guardia UNA-RIGA (`f.lines[j] == line`,
reg_lower.rs:189): i micro sono loop a riga singola, il codice WordPress è
formattato multi-riga — la resa del pass su codice reale è NON misurata e
plausibilmente molto sotto il 42% del micro. (b) Il fold di `LoadSlot` non è
bloccato da una necessità ma da un'EREDITÀ: la regola «mai foldato» esiste per
la parità del warning di `LoadVar`, e `LoadSlot` è silente — non c'è warning
da risintetizzare. Le guardie NECESSARIE sono altre: semantica INTEGRALE di
`StoreSlot` (write-through dei ref, gc_note) e NIENTE store se `Binary` lancia
(ordine osservabile via TypeError/distruttori). (c) Il precedente è già in
albero: `ConcatAssignSlot` (expr.rs:99, WP-55) è ESATTAMENTE la forma fusa
slot-diretta di AssignOp, emessa flag-off nel percorso di parità. Il residuo
va lì — emissione, non `fuse_window`: beneficia la parità (19→~15), è immune
alla guardia di riga, e paga il collaudo WordPress per regola n.2.

**RC-4 — Costo di compilazione del pass: NON misurato.** O(n) ma ~4 traversate
più rebuild completo con `clone()` di ogni op anche a zero finestre; ammortato
dall'unit-cache (`reg_mode` in chiave), accettabile SOLO finché flag-on resta
strumento di misura. L'oracle compile-side esiste (`--list-tests`, WP-64) e
non è stato usato.

## Emendamenti

- **A-HE-99-1**: controllo positivo nella batteria: asserire ≥1 forma fusa in
  `{main}` con il pass invocato al punto di pipeline di PRODUZIONE (env-flag
  in-process o funnel reale), non su modulo post-cessione.
- **A-HE-99-2**: il residuo AssignOp si progetta come fratello di
  `ConcatAssignSlot` all'emissione flag-off (guardie: StoreSlot integrale,
  no-store-on-throw), con collaudo WordPress; NON come quarto fold del pass.
- **A-HE-99-3**: sequenziare rispetto a H-B1: se l'emissione fusa entra, la
  baseline 8,24 ns flag-off si muove — pinnare prima/dopo esplicitamente; e
  riscrivere le condizioni di attivazione H-B2/H-C/H-D senza il riferimento a
  H-A1 dormiente (nominare la decisione promozione/scarto).

## Kill-switch

- **KS-HE-99-1**: nessuna forma registro va flag-off senza coppia compile-side
  `--list-tests` E collaudo di parità WordPress; violazione → reject.
- **KS-HE-99-2**: ogni fusione AssignOp all'emissione porta PRIMA del merge un
  test del percorso di lancio (TypeError dal Binary ⇒ slot NON scritto,
  osservabile via distruttore).
- **KS-HE-99-3**: finché A-HE-99-1 non è verde, le gambe `{main}` della
  batteria v3 non contano come evidenza di parità del pass.

*Refutazioni capitali: sì (RC-1..RC-4). Indipendenza: nessun verbale altrui letto.*
