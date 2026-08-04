# Verbale Sedia 2 — Matsakis (ownership/aliasing/borrow) — Concilio WP-99

**Oggetto**: report S-97.1 (H-A1 caduta, codice dormiente) + programma H-B1.

## VERDETTO: CON EMENDAMENTI

S-97.1: nulla da refutare sul piano borrow del codice spedito. H-B1 **come
formulata è irrefutabilmente NON scrivibile in Rust safe**: va vincolata (A-MA-99-1)
e sottoposta a una sonda preventiva (A-MA-99-2), altrimenti MI OPPONGO.

## Analisi del codice (run.rs)

**Fast-path a borrow (BinarySS/SSDst/CmpJmpSS ~979-1077).** Il pattern è sano:
`fr = &self.frames[top]`, due borrow condivisi sugli slot, `binary_fast` ritorna
`Option<Zval>` **posseduto** → per NLL il borrow muore al `break 'r v` e il push
successivo su `self.frames[top].stack` è libero. Non è fragile per caso: è il
COMPILATORE a garantirlo — se un refactoring facesse ritornare a `binary_fast`
un prestito (`&Zval`, `Cow`) o le passasse `&mut self`, NON compila. Il
canale che invece il borrow-checker **permette in silenzio**: passare a
`binary_fast` un `&mut self.diags` per split di campo (frames shared + diags
mut su campi distinti COMPILA) e farle emettere warning — divergendo dal
funnel del miss-path (doppio diag / ordine). Da sbarrare per contratto:
KS-MA-99-1. Alias `l == r` ($x+$x): due borrow condivisi, legale e corretto.
Nota minore: BinarySC/SCDst/CmpJmpSC materializzano `cv` PRIMA del fast-path
anche sul hit — un bump Rc per dispatch sui const ZStr (commentato deliberato;
su `arith` i const sono Long, costo nullo).

**reg_store_slot (244-257) vs StoreSlot (655-672).** Stesso ordine essenziale:
entrambi clonano l'Rc della cella (per staccare il prestito da `self.frames`
prima di `typed_ref_assign(&mut self)`) → coercizione → `store_slot` →
`gc_note(old)`. La differenza è il **cammino d'errore**: StoreSlot fa
pop→coerce(`?`)→push→pop, quindi su `Err` il valore è già stato POPPATO e
consumato; in reg_store_slot il valore non è mai stato sulla pila. Profondità
di pila diverse all'unwind ⇒ se il catch tronca la pila, può cambiare
l'ORDINE dei drop dei temporanei abbandonati (dtor osservabili in PHP). Oggi
flag-only, zero rischio parità; se mai promosso: fixture nominata in
A-MA-99-3.

## H-B1 — refutazione della forma, proposta della forma vincolata

1. **La formula letterale è impossibile in safe**: un `&mut Frame` (prestito
   di `self.frames`) tenuto attraverso il match mentre gli handler chiamano
   metodi `&mut self` è E0499 da manuale. `mem::take` del frame: VIETATO —
   gli osservatori ambientali cross-frame (backtrace, `current_frame_args`,
   `var_dyn_read`) vedrebbero un frame fantoccio (unsoundness semantica, non
   di memoria). Puntatori raw: fuori policy. `split_last_mut` per-opcode:
   è l'indicizzazione attuale sotto altro nome.
2. **I confini dichiarati (call/ret/throw) sono SBAGLIATI**: `cur_line(top)`
   legge `frames[top].ip` per la riga dei diag, e i warning nascono DENTRO
   gli handler (binary_value_ab). Un `ip` locale ricaricato "solo ai confini"
   sposta le righe dei messaggi in silenzio. KS-MA-99-2.
3. **Forma SAFE realistica (A-MA-99-1)**: loop interno su split di campo —
   `let fr = self.frames.last_mut()` + campi read-only, vivo ATTRAVERSO le
   iterazioni; `fr.ip` resta il campo vero (niente staleness); qualunque
   opcode che richieda un metodo `&mut self` (funnel, gc_note, chiamate) =
   confine: break al loop esterno. Costo onesto: il set caldo va reso
   method-free (gc_note/typed_refs oggi sono metodi ⇒ o si esce, o si
   rifattorizza a funzioni libere su campi splittati). Guardia profondità
   dove `frames` cresce: corretta (i frame crescono solo ai call).
4. **Refuto il conto dei guadagni**: 4 bounds check preveduti + 2 `len()`
   sono rami quasi-gratis su un colosso da ~9,9 ns/opcode; e il beneficio
   copre SOLO il sottoinsieme fast-path — proprio S-97.1 mostra che il costo
   sta negli opcode che entrano nel funnel `&mut self`. Prima di scrivere:
   **sonda per ADDIZIONE** (A-MA-99-2), due indicizzazioni ridondanti in più
   per opcode dietro flag; se `arith` non sale oltre il rumore R=3, togliere
   le quattro non può dare un calo netto ⇒ H-B1 cade senza essere scritta.

## Emendamenti

- **A-MA-99-1**: H-B1 riformulata come "loop interno su split-borrow di campo,
  confine = ogni opcode che richiede un metodo `&mut self`", non "&mut Frame
  attraverso il match".
- **A-MA-99-2**: sonda per addizione (+2 indicizzazioni/opcode dietro flag)
  PRIMA di implementare; criterio numerico scritto in apertura.
- **A-MA-99-3**: fixture nominata (non urgente finché flag-only): TypeError da
  typed-ref dentro un catch con temporaneo dotato di distruttore sulla pila —
  ordine dei drop StoreSlot vs reg_store_slot.

## Kill-switch

- **KS-MA-99-1**: `binary_fast` resta `fn(BinOp, &Zval, &Zval) -> Option<Zval>`
  — pura, senza canale diags né accesso a self. Ogni allargamento di firma =
  gate rosso (divergenza fast-path/funnel che il borrow-checker NON vede).
- **KS-MA-99-2**: vietato cachare `ip` in un locale staccato dal frame con
  write-back "ai confini": o `ip` resta il campo del frame mutato attraverso
  il prestito vivo, o flush prima di OGNI sito che può emettere diag/errore.
- **KS-MA-99-3**: vietati `mem::take` del frame e puntatori raw nel run_loop
  (safe-only); qualunque forma che renda il top-frame invisibile agli
  osservatori cross-frame è respinta a priori.

## Refutazioni capitali

**Sì, due**: (1) la formulazione letterale di H-B1 non è scrivibile in Rust
safe e il suo insieme di confini è sbagliato (ogni sito di diag è un confine
per l'osservabilità di `ip`); (2) il conto dei guadagni attribuisce al
preambolo un costo mai misurato — la sonda per addizione deve precedere il
codice.
