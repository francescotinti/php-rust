# Verbale sedia 2 — Niko Matsakis (ownership/aliasing/borrow) — Concilio WP-101

Oggetto: S-99.0 (pre-misura rollout, patch INT1, controfattuale registro) + bozza S-100 (promozione flag-on).

## VERDETTO

**La lettura ownership del controfattuale 4a REGGE; la fedeltà drop/clone di INT1 REGGE; ma il verbale porta TRE refutazioni capitali sul contorno** — l'attribuzione 57/43 poggia su una sottrazione CROSS-TREE, il flip come abbozzato SPEGNE H-B2 sui siti stack residui dell'emissione on, e A-MA-100-2 è evaporata dal registro.

Conferme dal codice (run.rs:1005-1066, HEAD): il hit-path di BinarySS/SSDst/SC è due prestiti condivisi di `frames[top].slots` + guardie `matches!` + `binary_fast` `#[inline(always)]` su prestiti + push/`reg_store_slot` del risultato. Zero clone, zero drop non banali, zero marshalling. Una BinaryAddSS rimuoverebbe SOLO il carico del payload `b` e il match inlineato: predizione D=0 coerente. Aliasing sano: `l==r` è legale sotto prestiti condivisi; `dst==l|r` in SSDst è sicuro per NLL (res computata a prestiti chiusi, semantica eval-then-assign di PHP); slots e stack sono campi disgiunti; su `?` di `reg_store_slot` res viene droppata senza stato parziale. INT1 (patch): sul hit entrambe le forme droppano due Long banali (C2: overwrite in-place + rhs a fine scope; INT1: lhs+rhs a fine scope) — parità drop/clone confermata, miss-path identico modulo ordine dei pop.

## Refutazioni capitali

**R1 — C0 è un ALBERO DIVERSO.** C0 = stash `phpr-s97-ha1`; INT1/C2 = HEAD±patch. Solo INT1−C2 (43%, traffico Vec) è same-tree-pulita; C0−INT1 (57%, "call/marshalling") sottrae attraverso il drift di S-98. In più INT1 cambia una cosa NON dichiarata: il tag-check di lhs passa da lettura through-Vec (`stack.last()` + strato Option) a locale posseduto — un termine di addressing finisce nella quota "call/marshalling". La direzione è robusta, le cifre 57/43 NO.

**R2 — Il flip ritira H-B2 dove non si fonde.** `reg_lower.rs:184-187`: «the production flag-on pipeline only ever emits `Binary(Add)`». Sotto emissione on i siti stack NON fusi perdono la specializzazione BinaryAdd (il −16,2% tenuto). E il punto 4 della bozza pre-registra il controfattuale Sub/cmp sul giudice flag-OFF — un modo in via di ritiro. Il giudice giusto post-flip sono i siti stack RESIDUI dell'emissione on.

**R3 — Ledger leak.** A-MA-100-2 (wildcard `_ => None` in `bin_op_of`, reg_lower.rs:188-194) non è né nei saldati né nel backlog della rotazione: caduta in silenzio. Resta viva: una futura variante specializzata (BinarySub…) de-fonde in silenzio — l'esatto anti-pattern che i match esaustivi S-96 hanno bandito.

## Emendamenti

- **A-MA-101-1**: prima di citare 57/43 come decomposizione ereditabile, costruire C0' SAME-TREE (HEAD con emissione BinaryAdd disattivata); fino ad allora pubblicare solo l'ordinamento («call+marshalling ≥ metà») come banda.
- **A-MA-101-2**: saldare o re-iscrivere A-MA-100-2 — `bin_op_of` a match esaustivo sugli Op Binary-like (variante nuova ⇒ NON COMPILA).
- **A-MA-101-3**: prima del flip, decidere CON MISURA la sorte di H-B2 sotto emissione on (estendere BinaryAdd ai siti stack residui della pipeline on, oppure pre-registrare la rinuncia); spostare il giudice del punto 4 sui residui post-flip.
- **A-MA-101-4**: «ramo costante per sito» conflaziona sito bytecode e sito macchina: il match di `binary_fast` inlineato nell'arm BinarySS è UN branch hardware condiviso da tutti i siti — la banda [0, 0,5] è provata sul micro a pochi siti, non su stream misto di BinOp. La sonda dell'eventuale riapertura (D≥0,7) va fatta su stream MISTO.

## Kill-switch

- **KS-MA-101-1**: il flip NON riusa `PHPR_REG_LOWER` presence-tested né con significato invertito — oggi `enabled()` è `is_some()`: `PHPR_REG_LOWER=0` ACCENDE il pass. L'opt-out post-flip è value-parsed; sigillo eager + dente anti-putenv ri-collaudati sulla semantica NUOVA, pena flip VOID.
- **KS-MA-101-2**: nessuna cifra 57/43 fuori-banda in criteri futuri finché C0' same-tree non esiste; ogni criterio derivato da essa è VOID.
