# Team «hc-canale» — Concilio WP-102, fase 2

**Sedie**: 5 Bak · 2 Matsakis · 8 Stogov — Relatore: hc-canale
**Fonti vincolanti**: verbale-5-bak.md · verbale-2-matsakis.md · verbale-8-stogov.md
**Tema**: candidata H-C1 («prestito al posto del clone su PropGet»)

---

## 1. Verdetto di team

**H-C1 nella forma «prestito/refcount al posto del clone del valore letto» è
REFUTATA da tutte e tre le sedie e NON si iscrive all'ordine S-101 così
nominata.** Le tre refutazioni convergono sul fatto centrale: sul giudice
prop.php i valori sono `Long` (Copy), il clone del valore non alloca e non
tocca refcount — eppure phpr spende ~27% nel ciclo di vita Zval. Quindi il
meccanismo che costa NON è quello che H-C1 nomina, e la cura proposta
(prestito) potrebbe AGGIUNGERE traffico dove oggi non c'è (Bak R1), mira al
canale minore ignorando il RICEVITORE (Matsakis R1), e non corrisponde alla
semantica Zend, che non presta mai: `ZVAL_COPY_DEREF` = copia 16-byte +
ADDREF solo se refcounted (Stogov p.3).

Ulteriore vincolo condiviso (Bak R5 ≡ Stogov p.6): **anche a successo pieno
(tutto il 27% azzerato) prop resta ~9×** — H-C1 non chiude H-C e il tetto va
scritto PRIMA nell'ordine, mai negoziato dopo.

## 2. Riformulazione condivisa — H-C1 è SOSTITUITA da una forma a due stadi

Adottiamo la forma a due stadi di Stogov (A-ST-102-3), che sussume la
ri-nominazione per census di Bak (A-BA-102-1) e l'attribuzione per sito di
Matsakis (A-MA-102-1). Le nuove ipotesi, tutte SUBORDINATE al census di §3:

- **H-C1a «bypass scalari»** — se il census mostra gc_note/drop-bookkeeping
  su Zval NON-refcounted (Int ecc.), bypass del bookkeeping per i
  non-refcounted. Semantica-neutra per costruzione; si misura **DA SOLA**
  prima di ogni schema refcount (KS-ST-102-2).
- **H-C1b «canale ricevitore»** — se il census attribuisce il churn al
  ricevitore (`obj.deref_clone()` = bump `Rc<Object>` + drop→gc_note per
  ogni PropGet), estensione del prior art in-tree `ThisPropGet`
  (run.rs:3418) che sull'IC-hit già evita il clone del ricevitore. Sigillo
  di TIPO per ogni prestito: nessun borrow-guard attraversa un confine di
  op o un rientro VM (A-MA-102-3).
- **H-C1c «copy+addref condizionale»** — solo per valori refcounted e solo
  dopo (i) e (ii): la forma Zend, mai borrow nudo dello slot valore
  (indifendibile su mutazione interlacciata, `__get`/hook/lazy,
  ref-in-slot — Stogov p.4).

L'atteso di ciascuna riga si calcola **dal canale contato** (n° eventi
eliminati/iter × costo per evento misurato), MAI da quota%×T (Matsakis R3):
il 27% è la quota dell'intero micro, non il recuperabile — Sweep/Pop
droppano anche temporanei che nessun prestito elimina.

## 3. Ordine di misura — PRIMA di ogni riga di codice

1. **Ri-baseline sei categorie** (KS-ST-102-3): nessuna cifra H-C1 fa fede
   senza; pubblicata con ns/op per specie — traffico vs lavoro — mai solo
   il prodotto conteggio×costo (KS-BA-102-2).
2. **Census dinamico su prop.php, DUE motori**, decomposto su tre assi:
   - **specie** di Zval (Int / string condivisa / array / object) che
     attraversa clone/drop/gc_note (Bak A-BA-102-1: alloc/iter e
     refcount-ops/iter; se alloc/iter≈0 il meccanismo si ri-nomina);
   - **sito** (ricevitore vs valore vs traffico Sweep/Pop) per i quattro
     simboli drop 12,6 + clone 7,6 + gc_note 5,3 + deref_clone 2,1
     (Matsakis A-MA-102-1);
   - **canale** (alloc, refcount-bump, gc_note-call).
   Le tre predizioni di sedia si PRE-REGISTRANO come attese rivali:
   Bak = drop-glue/discriminante incondizionato su scalari; Matsakis =
   ricevitore Rc + gc_note; Stogov = gc_note su non-refcounted. Il census
   arbitra per nome.
3. **Profilo inline-aware** (A-BA-102-2) per aprire il 50% di run_loop:
   la post-misura di qualunque H-C1x giudicata contro la baseline sfocata
   attuale è un verdetto sfocato (Bak R3).
4. **Tetto pre-registrato nell'ordine**: successo pieno ⇒ prop ~9,1×,
   ancora >3× sopra X≤3 (KS-BA-102-1); vietato presentare H-C1x come "la
   cura" di H-C (Stogov p.6).

## 4. Fixture semantiche OBBLIGATORIE (in-tree e verdi su ENTRAMBI i motori, attese scritte PRIMA — KS-MA-102-1, KS-ST-102-1)

Unione delle liste A-MA-102-2, A-ST-102-4, A-BA-102-3:

- **Per specie di valore** (Bak): int, string condivisa, array — criterio
  di successo PER SPECIE (la cura giusta per string può essere sbagliata
  per int).
- **Aliasing/osservabilità** (Matsakis ∪ Stogov): mutazione interlacciata
  nello statement (`$o->a + $o->mut()` — valore osservabile PRE-mutazione,
  l'addref lo garantisce, un prestito no); `__get` e hook get/set (valore =
  temporaneo, non c'è slot); lazy ghost/proxy; `&$o->x` (Ref nello slot,
  serve deref); riassegnazione intra-espressione (`$o->x + ($o->x=…)`);
  `unset` durante la lettura; readonly/typed-uninit (`Undef` fatale);
  warning undef-prop; op che scrivono lo stack IN PLACE (`BinaryAdd` fa
  `*last_mut()=v`: mai alias di uno slot proprietà sullo stack).
- **Ciclo di vita** (Matsakis R2 + Stogov): ordine `__destruct`
  (destruct-timing, riusare la trap f); ciclo GC con proprietà nel ciclo;
  finestra del cycle-collector («under-noting delays a destructor»).

**Collaudo di parità WP obbligatorio anche senza cambi d'emissione**
(KS-MA-102-4): un cambio di aliasing a runtime sposta il momento in cui
`strong_count` tocca 1, cioè l'ordine dei distruttori — «l'emissione non
cambia ⇒ batteria+corpus bastano» è FALSA per questa famiglia (Matsakis R2).
Qualunque cambio d'ordine distruttori/free-order ⇒ reject senza appello
(KS-MA-102-3).

## 5. Conflitti residui (posizione per sedia)

1. **Prestito del ricevitore: ammesso o no?** — Matsakis: sì, à la
   `ThisPropGet`, purché sigillato dal TIPO (scope del borrow chiuso prima
   del push, mai attraverso confine di op). Stogov: «il borrow nudo è
   indifendibile» e la forma Zend è addref, mai prestito — ma i suoi tre
   controesempi (p.4) colpiscono il borrow dello SLOT VALORE, non il
   ricevitore. Bak: neutrale sul canale, esige che il census lo nomini
   prima. **Composizione di team**: H-C1b resta iscrivibile come prestito
   del ricevitore SOLO con sigillo di tipo A-MA-102-3 E fixture
   destruct-timing/interlacciata verdi; se una fixture mostra osservabilità
   dell'ordine dei drop del ricevitore, si ripiega su addref (forma
   Stogov). Il borrow dello slot valore resta VIETATO per unanimità.
2. **Nome del meccanismo dominante** — tre predizioni rivali (§3.2): non è
   un conflitto da negoziare ma da arbitrare col census; le predizioni
   pre-registrate impediscono la ri-narrazione post-hoc.
3. **Grado di refutazione** — Bak la classifica capitale (strumenti ciechi
   + nemico forse inesistente), Matsakis capitale sulla mira, Stogov
   puntuale sulla forma. Irrilevante per l'ordine: l'esito operativo
   (nessuna riga senza census+fixture) è identico e unanime.

## 6. Priorità per l'ordine S-101

1. **P1** — Ri-baseline sei categorie (prerequisito di ogni cifra).
2. **P2** — Census dinamico specie×sito×canale su prop.php, due motori,
   con le tre predizioni pre-registrate; in coda il profilo inline-aware.
3. **P3** — Fixture semantiche §4 in-tree, verdi su entrambi i motori.
4. **P4** — H-C1a (bypass scalari) SE il census la nomina: misurata da
   sola, semantica-neutra, primo codice candidato.
5. **P5** — H-C1b (ricevitore) SE il census la nomina: sigillo di tipo +
   coppia WP di parità obbligatoria; ripiego addref se le fixture mordono.
6. **P6** — H-C1c solo dopo P4/P5, mai borrow nudo dello slot.
7. **Fuori tema ma vincolante per l'ordine** (dai verbali): riscrittura
   §3.12 e fix §3.11 in famiglia fetch-undef (A-ST-102-1/2); perimetro del
   tripwire zero-`Binary(Add)` per NOME (A-MA-102-4); L=12,9 non è
   coefficiente riusabile (A-BA-102-4).

**Kill-switch di team** (unione, tutti attivi): KS-BA-102-1/2,
KS-MA-102-1/2/3/4, KS-ST-102-1/2/3.
