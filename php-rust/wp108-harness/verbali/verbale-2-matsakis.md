# Verbale sedia 2 — MATSAKIS (ownership/aliasing/borrow) — Concilio WP-108 su S-106

## VERDETTO

**H-A1 (BinarySTDst) NON REFUTATA sul mio perimetro** — l'equivalenza col tris è provabile per NOME, non solo dichiarata. **UNA refutazione di portata sull'igiene**: il backstop ArgPlace D-12 è «rumoroso» solo in build che i gate non eseguono mai. D-9: declassamento conforme ma stato di ATTESA, non terminale.

## R-MA-108-n (rilievi/refutazioni)

- **R-MA-108-1 — ordine di osservazione INVARIATO, per NOME.** Nel tris (`run.rs` finestra contigua, stessa riga, `blocked[]` esclude target di salto ed exc-boundary dagli interni) il LoadSlot leggeva `slots[l]` DOPO la valutazione completa del rhs (già in pila); il fuso legge dopo il pop del rhs. Pop e Swap sono senza effetti ⇒ lo stato osservato è identico. `read_slot` (arrays.rs:829) ritorna un clone OWNED (Ref → clone del referent, borrow statement-scoped): nessun borrow vivo attraversa `binary_value_ab`, quindi una rientranza (es. `__toString`) vede in entrambe le forme uno snapshot già preso e può mutare `slots[l]`/`slots[dst]` con lo stesso esito: il tris aveva la stessa finestra. Nessuna finestra NUOVA.
- **R-MA-108-2 — l==dst sano; percorso d'errore identico.** Snapshot-poi-store come nel tris; `reg_store_slot` (run.rs:298) replica `StoreSlot` INTERO (guardia typed-ref, write-through `store_slot`, `gc_note` sul vecchio). Profondità di pila al punto del `?`: tris = push 1, pop 2; fuso = pop 1 — net −1 entrambi ⇒ l'handler exc vede lo stesso stato.
- **R-MA-108-3 — conteggi: parità deliberata.** dcn! su lhs/rhs/dst-old nei due bracci; scn Pop=1 onesto; il duplicato transiente esiste ancora (lhs owned) e muore nello stesso punto, dentro `binary_value_ab`. NIENTE conta diverso per la leva. NOTA a catalogo (NON delta H-A1): la coda `Dup;StoreSlot;Pop` assorbita da `bin_dst` elide il `gc_note` del Pop sulla copia del risultato — pre-esistente a TUTTA la famiglia *Dst, da censire una volta, non da imputare alla leva.
- **R-MA-108-4 — REFUTAZIONE (portata: igiene, non leva).** Il backstop D-12 (`calls.rs:307-312`): `debug_assert!` è compilato via in release — e batteria/corpus/micro girano `--release` per regola; il contatore esiste solo in build `mem-census`. Sul percorso di collaudo REALE un funnel mancato degrada ancora a Null in SILENZIO: esattamente il difetto che D-12 doveva curare. «Backstop rumoroso: SALDATO» è vero a lettera di codice e falso a lettera di gate.

## A-MA-108-n (emendamenti)

- **A-MA-108-1**: il dente D-10 (direct-bind negatives) DEVE avere una gamba con `debug_assertions` attive O una gamba census con assert contatore==0 a fine run; finché manca, «backstop rumoroso» non si dichiara saldato nei report.
- **A-MA-108-2**: valutare (admission propria, churn dichiarato) un contatore incondizionato nell'arm ArgPlace in release: l'arm esiste già nel match, costo zero sul percorso non-ArgPlace. Il degrade a Null resta il comportamento release GIUSTO (mai il descriptor a user code, mai abort in produzione).
- **A-MA-108-3**: D-9 — «il commento declassato resta» vale come ATTESA: il dente drop-order diventa PREREQUISITO di admission della leva D-20 (metà-Zend); leva sul decay/call-path senza dente = VOID (estende KS-MA-107-1).

## KS-MA-108-n

- **KS-MA-108-1**: una fusione che sposta il PUNTO di clone si giudica sull'ordine di osservazione tra op SENZA effetti; la prossima finestra che assorbe un'op CON effetti (LoadVar-warning, FetchDim) NON eredita questo verdetto.
- **KS-MA-108-2**: un'asserzione che vive solo in build mai eseguite dai gate è documentazione, non un dente.

## Giudizio ordine S-107

Sequenza SANA. Emendo: punto 1 — il dente D-10 espliciti la gamba debug_assertions (A-MA-108-1); punto 2 — la cura §3.15 tocca il binder dinamico = perimetro ArgPlace: la verifica D-12 va PRIMA o INSIEME, non dopo; D-9 agganciato a D-20 (A-MA-108-3). Nessuna obiezione alla leva punto 3.
