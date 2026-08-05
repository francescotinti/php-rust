# Verbale Sedia 8 — Dmitry Stogov (Zend engine, semantica PHP) — Concilio WP-102

**VERDETTO: S-100 regge sul flip; REFUTAZIONI PUNTUALI su §3.12 (catalogata male
in tre punti) e sulla FORMA di H-C1 (il "prestito" non è la semantica Zend).
Nessuna refutazione capitale.**

## Refutazioni (con verifica su fonte C 8.5.7 e oracle, stasera)

1. **§3.12 REFUTATA COME SCRITTA — tre difetti.** (a) È **mode-dependent**:
   in weak il ref va a 0; in **strict** lo stato RESTA 1 ma il TypeError
   catturato CAMBIA in `Cannot assign null to reference held by property
   T::$i of type int` (verificato sull'oracle brew). La voce cataloga solo il
   weak e dichiara "messaggio a parità" — in strict è falso. (b) Il commento
   della fixture (b) dice "regime strict del FILE ASSEGNANTE" ma il file NON
   ha `declare(strict_types=1)`: collauda il weak raccontandosi strict.
   (c) **Sotto-scopata**: anche il typed-**prop** diretto (`$t2->i += "abc"`
   → 0, verificato; `zend_binary_assign_op_typed_prop` ha la stessa forma).
   **Meccanismo VERIFICATO** (non più "plausibilmente"): il path d'errore di
   `add` fa `ZVAL_UNDEF(result)` quando `result != op1`
   (`convert_op1_op2_long`, zend_operators.c); `zend_binary_assign_op_typed_ref`
   opera su `z_copy`; in weak `zend_verify_ref_assignable_zval` →
   `i_zend_verify_type_assignable_zval` = −1 → `zend_parse_arg_long_weak` su
   UNDEF (tipo 0 < IS_TRUE, ≠ IS_NULL) → **0 senza warning**. "Azzera" è un
   artefatto di un UNDEF che attraversa la coercizione debole; phpr
   "conserva" coincide con lo STATO Zend strict e col principio "op fallito ⇒
   nessuna scrittura", non col messaggio strict.
2. **§3.11**: il warning oracle nasce dal fetch CV **RW** (`zval_undefined_cv`),
   non dall'AssignOp: la cura appartiene alla famiglia fetch-undef (stessa di
   §3(c)), non a un cerotto sul compound; solo così l'ordine
   rhs→warning→errore-op diventa collaudabile una volta per tutte.
3. **H-C1 refutata NELLA FORMA "prestito/refcount al posto del clone"**: Zend
   `FETCH_OBJ_R` non presta — fa `ZVAL_COPY_DEREF`: copia di valore 16-byte +
   ADDREF **solo se refcounted**. Sul giudice prop.php i valori sono INT:
   l'oracle non fa NESSUN refcount, eppure phpr spende ~27% in
   clone/drop/gc_note — il ciclo di vita Zval costa anche sui **scalari**
   (gc_note su Int?). La COPIA vera in Zend non avviene mai in lettura: gli
   array sono CoW, la separazione scatta in scrittura.
4. **Il borrow nudo è indifendibile** in almeno tre casi PHP: mutazione
   interlacciata nello statement (`$o->a + $o->mut()`: il valore osservabile
   è PRE-mutazione — l'addref lo garantisce, un prestito dello slot no);
   `__get`/hook/lazy (il valore è un temporaneo, non c'è slot); slot
   contenente ref (serve deref).
5. **Census**: verificato che il corpo del loop è IDENTICO pre/post optimizer
   (9 op a 0x10000 e 0x20000) ⇒ la decomposizione 2,0×6,2 non soffre del
   disallineamento census/cronometro su QUESTO giudice; il criterio però va
   scritto prima di riusare la tavola su calls.php, dove l'optimizer fonde.
6. **Tetto pre-registrato**: azzerare TUTTO il ciclo di vita Zval porta prop
   da 12,4× a ~9×: H-C1 **non** chiude H-C (il 6,2×/op resta dispatch+handler
   generici vs SPEC/TAILCALL). Vietato presentarla come la cura.

## Emendamenti

- **A-ST-102-1**: riscrivere §3.12 (meccanismo verificato; scopo esteso al
  typed-prop; tabella weak/strict con messaggio sostituito-e-chained in
  strict); fixture (b): twin strict + twin typed-prop; correggere il commento.
- **A-ST-102-2**: fix §3.11 nominato nella famiglia fetch-undef (CV e prop),
  giudice = trap (e) per l'ordine osservabile.
- **A-ST-102-3**: controfattuale H-C1 in due stadi: (i) censimento delle
  SPECIE di Zval che attraversano gc_note/drop nel giudice; (ii) bypass
  bookkeeping per non-refcounted (semantica-neutra); solo dopo, addref per i
  refcounted — mai borrow nudo.
- **A-ST-102-4**: fixture H-C1 obbligatorie PRIMA del codice: mutazione
  interlacciata, `__get`/hook/lazy, ref-in-slot, warning undef-prop,
  destruct-timing (riusare la trap f).
- **A-ST-102-5**: tavola census×costo su calls.php: census nei DUE regimi
  optimizer prima di fissare il denominatore.

## Kill-switch

- **KS-ST-102-1**: nessuna riga H-C1 senza le fixture A-ST-102-4 verdi
  sull'oracle, attese scritte PRIMA.
- **KS-ST-102-2**: se il censimento mostra gc_note su non-refcounted, il
  bypass scalari si misura DA SOLO prima di ogni schema refcount.
- **KS-ST-102-3**: la ri-baseline sei categorie (p.1 bozza) precede ogni
  cifra H-C1: rapporti citati senza quella misura non fanno fede.
