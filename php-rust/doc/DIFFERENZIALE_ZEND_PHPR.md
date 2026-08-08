# Differenziale Zend ↔ phpr — «Zend fa X, phpr fa Y» (istruttoria permanente, v1 S-113)

Fonte: letture VERIFICATE su php-8.5.7 (file:riga citati) e su crates/ al pin
s112 f71abd2a. Scopo: dare alle leve S-114+ un'attesa da MECCANISMO. Non è un
piano: ogni riga diventa leva solo passando da istruttoria→criterio PRE→A/B.

## 1. Dispatch (famiglia più grande: pesa su TUTTI i giudici)
- **Zend**: VM ibrida a computed-goto — ogni handler termina con
  `goto *(void**)(OPLINE->handler)` (zend_vm_execute.h:371-374): NESSUN loop
  centrale, il salto indiretto è replicato PER-handler e il branch predictor
  impara le transizioni op→op tipiche. In più ogni opcode è SPECIALIZZATO a
  build-time per tipo di operando (`_SPEC_CV_CONST` ecc.): l'handler non
  decodifica nulla a runtime.
- **phpr**: UN `run_loop` con `match` centrale (291 KB, 5864 bl): ogni op
  ritorna al top del loop e passa dallo STESSO salto indiretto → un solo sito
  da predire per tutte le transizioni. La specializzazione esiste ma è per
  FORMA di operando (BinarySS/SC/TC…), decisa dal pass, non per tipo runtime.
- **Cifra**: arith 5,5× CON tutte le corsie attive ⇒ ~9-10 ns/op di tetto
  contro ~2-3 di Zend; S-110 (xctrace): 32,5% dei cicli phpr in fame frontend
  su arith vs 3,3% oracle. S-111 ha refutato UNA forma (pre-filtro
  hot-cluster), non la famiglia. Rust stabile non ha computed-goto; le forme
  candidate (tail-call per-handler, loop replicati per cluster SENZA
  pre-filtro sui freddi) vogliono istruttoria dedicata + banda layout misurata.

## 2. Zval e refcount
- **Zend**: zval = 16 B (zend_types.h:355); gli SCALARI non hanno refcount —
  `ZVAL_COPY_DEREF` (zend_types.h:1533) fa addref SOLO se
  `Z_OPT_REFCOUNTED`. Soprattutto: il VM lavora su **puntatori** a zval che
  vivono nei CV slot del frame — leggere `$o` NON copia e NON tocca refcount.
- **phpr**: Zval enum (php-types/src/zval.rs:15) — Long/Bool/Double copy puri
  (pari a Zend), ma Str/Array/Ref/Closure/Object sono `Rc` e il VM passa Zval
  **posseduti per valore** tra helper: ogni transito di un Object = clone
  Rc++ e drop successivo. prop paga ~3 cloni del ricevitore/iter (H-P1 ne
  toglieva 2: **+3,3 ns/iter misurati S-113** — il quanto del clone è ~1,6).
- **Extra phpr**: `Rc<RefCell<Object>>` aggiunge il borrow-flag a OGNI
  accesso (check+set+clear); Zend non ha l'equivalente (mutazione libera).

## 3. Loop bigram (CmpJmpSC / IncDecSlotJmp)
- **Zend**: IS_SMALLER Long-Long inline + SMART_BRANCH fonde compare+salto
  (zend_vm_def.h:677-695); PRE_INC fa `fast_long_increment` IN PLACE sul CV
  (zend_vm_def.h:1529-1545).
- **phpr**: CmpJmpSC = stessa fusione (parità di forma ✓); IncDecSlotJmp
  idem. Residuo: solo il costo dispatch (§1). Qui phpr NON ha corsie mancanti.

## 4. Aritmetica
- **Zend**: ADD (zend_vm_def.h:47-83) = 2 check type_info + `fast_long_add`
  con overflow in asm (zend_operators.h:704: su aarch64 sono 5 istruzioni con
  `bvs` al ramo freddo).
- **phpr**: `binary_fast` copre le stesse coppie (post H-A2 anche Shl/Shr);
  overflow via checked-op. Corsie PARI; il 5,5 residuo è §1+§2, non corsia.

## 5. Proprietà (prop 7,6 — divario peggiore, meccanismo COMPLETO)
`$o->x = $o->y + 1; $s += $o->x;` — confronto per passo:
- **Zend get** (FETCH_OBJ_R, zend_vm_def.h:2069): `container=EX_VAR(...)` =
  puntatore al CV (ZERO refcount, riga 2076); cache-hit = 2 confronti
  (`zobj->ce == CACHED_PTR`, riga 2105) + **offset FISSO**
  `OBJ_PROP(zobj, prop_offset)` (riga 2110) + `ZVAL_COPY_DEREF` (riga 2116:
  per un Int è una copy da 16 B senza addref). Niente borrow, niente bounds
  check, l'hit non guarda nemmeno le eccezioni.
- **phpr get** (PropGetSlot/Recv, run.rs:4162/4169): clone Rc del ricevitore
  (`reg_load_slot`) che sull'hit muore inutilizzato + `scope_key` + `ic.get`
  + borrow RefCell + check class_id/lazy + `get_slot(u32)` (indicizzazione
  bounds-checked) + `deref_clone` + push su Vec. **4 costi che Zend non ha:
  clone del ricevitore, borrow-flag, bounds check, transito owned.**
- **Zend set-RMW** (`$o->x += …`: ASSIGN_OBJ_OP, zend_vm_def.h:1008):
  `get_property_ptr_ptr` UNA volta col cache_slot (riga 1054) poi
  `zend_binary_op(zptr, zptr, value)` **IN PLACE sul puntatore** (riga 1077):
  zero pop/push del valore, zero secondo lookup, zero copia.
- **phpr set** (BinaryTCPropSetPop→prop_set_entry): borrow per l'hit-check,
  poi `write_property_at` = ALTRO borrow (check Ref, oop.rs:87) + borrow_mut
  + `replace_slot` + `gc_note(old)` + `value.clone()`. Tre borrow e una
  copia dove Zend fa un binop su un puntatore.
- **Nota IC**: l'IC phpr riempie solo da classi `plain_set_props` (pubbliche,
  simmetriche, non tipizzate, senza hook) ⇒ sull'hit niente coercizione:
  le forme in-place sono LEGALI per costruzione sull'hit.

## 6. Array (arr 4,2)
- **Zend**: FETCH_DIM_R (zend_vm_def.h:1933) → `fetch_dimension_address_inner`
  (zend_execute.c:2802): packed = indice diretto; chiave stringa = lookup
  hash con stringhe interned (hash pre-calcolato nella zend_string).
- **phpr**: dual-repr PhpArray (pari sul packed); MA arr.php usa chiavi
  "k$i" COSTRUITE a ogni iterazione: entrambi i motori ripagano la costruzione,
  phpr paga in più l'hash non-interned della chiave temporanea. Divario già
  il più basso dei giudici non-regex: rendimento marginale basso.

## 7. Chiamate (calls 5,2)
- **Zend**: INIT_FCALL (zend_vm_def.h:4057) + DO_UCALL (4169): frame su
  zend_vm_stack = **bump allocator contiguo**, i CV del callee vivono nel
  frame stesso; `execute_data = call; LOAD_OPLINE; ZEND_VM_ENTER` — il
  "ritorno" è un cambio di due puntatori.
- **phpr**: Op::Call (run.rs:3204) con simple_call S-105: args pila→slot
  senza contenitori, frame POOLED (riuso). Residuo: frame Rust 176 B +
  Vec slots non contigui allo stack chiamante + Rc del ret_cell. Già molto
  limato; il resto è §1.

## 8. Dove phpr è GIÀ pari o avanti
Smart-branch ✓ (CmpJmp*) · corsie Long complete ✓ (post H-A2) · IC proprietà
✓ (PropIc ≈ CACHE_SLOT, ma con costi extra §5) · superistruzioni che Zend NON
ha (BinarySCSCDst fonde 4 op; LoadVarPushConst; IncDecSlotJmp) · mimalloc ·
frame pool. La fedeltà byte resta il vincolo che Zend non deve pagare.

## 9. Leve derivate, in ordine di attesa (per S-114+; ognuna passa dal rito)
1. **L-A «trigramma prop in prestito»** (giudice prop): probe BIPARTITA
   get-IC E set-IC del trigramma `$o->x = $o->y + C` (PropGetSlotRecv +
   BinaryTCPropSetPop): se ENTRAMBI gli IC colpiscono → read y (borrow) +
   binop fast + store x (borrow_mut, in place, niente write_property_at né
   clone del value né recv sulla pila); se UNO manca → sequenza storica
   VERBATIM da zero (nessuno stato toccato prima del commit: legale col veto
   finestre, la sospensione sta tutta nel sentiero miss — prior art W9a).
   Attesa: somma dei costi §5 = i +3,3 di H-P1 PIÙ set-side e round-trip
   di pila ⇒ progettata per stare sopra 4+banda. Composizione, non clone
   singolo (lezione S-113).
2. **L-B famiglia dispatch** (tutti i giudici): la più grande e la più
   rischiosa; UNA forma già refutata (S-111). Prossima istruttoria: contare
   il costo del ritorno-al-top vs salto diretto (disasm + contatori) PRIMA
   di progettare qualunque forma. Prerequisito assoluto: banda layout
   misurata (S-114 punto 1).
3. **L-C op-count Sweep**: Zend non ha un'op Sweep; phpr ne spende 2-3/iter
   (dispatch puro, il corpo è già fast-path vuoto). Toglierle dove il pass
   PROVA che non servono è un cambio di emissione (admission NON invariata):
   solo con criterio dedicato e corpus pieno.
4. **L-D micro**: get_slot a offset fisso (u32 non bounds-checked via
   accessor dedicato), fold di scope_key. Quanti da ~1-2 ns: SOLO dentro L-A,
   mai da soli (lezione S-113).

*Regola d'uso: questo file si AGGIORNA quando una lettura lo smentisce; le
righe citate sono state verificate a mano su php-8.5.7 in data 2026-08-08.*
