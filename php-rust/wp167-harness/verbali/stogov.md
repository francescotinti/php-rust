# Sedia STOGOV — bozza indipendente (S-167)

## VERDETTO: GO-CONDIZIONATO
**Rotta:** campagna R1 in forma Zend — predecode con operandi pre-risolti a offset di slot + handler specializzati per tipo con slow-path outlined + fusione cmp+jmp; R2 ridotta a «Zval copy-cheap», veto NaN-boxing CONFERMATO; R3 rinviata.

## Gli 8,6 ns di Zend (dal sorgente, zend_vm_def.h)
1. **Handler specializzati** per (opcode × tipi operandi): `ZEND_ADD SPEC CV_CONST` fa UNA cmp su `Z_TYPE_INFO == IS_LONG`, `fast_long_add_function`, `opline++`. Caso generale in HELPER fuori linea, `SAVE_OPLINE` solo lì.
2. **Operandi CV = offset nel frame**: `EX_VAR(opline->op1.var)` è una `lea`. Zero lookup, borrow, match.
3. **Dispatch HYBRID**: `goto *(void**)(OPLINE->handler)`, opline/execute_data in registri callee-saved. Un salto indiretto predetto per-sito.
4. **Zval 16 byte**, scalari senza refcount: copia = un move, niente drop-glue.
5. **Niente pila operandi**: risultati dritti in slot frame (TMP = slot). SMART_BRANCH fonde cmp+jmp senza zval booleano.
6. **SAVE_OPLINE differito**: il fast path non tocca stato VM in memoria.

## Manca a phpr, in ordine di prezzo
(a) operandi pre-risolti a offset con fast-path monomorfo per tipo — QUI vive la torta «dispatch 36,3»: il costo non è il salto, è ciò che l'handler fa prima di toccare i dati (fetch, match sull'enum, guardie); (b) slow-path outlined (lezione H-C2/MC1: icache); (c) fusione cmp+jmp; (d) dispatch threaded — **(d) da solo è la trappola della dipendenza**: saltare più in fretta dentro un handler grasso rende ~2-3 ns/op, annegati. Le cadute L-TD1 (borrow ≤1 ns) e L-AL3 (alloc ≈0) lo confermano: il prezzo è lavoro per-op, non memoria.

## Insegnamento chiave
Zend fa 8,6 ns con union taggata a 16 byte **senza NaN-boxing**: R2 radicale non è prerequisito della parità.

## Emendamenti
1. PRIMA della fetta: decomposizione arith dedicata (chiusura ≥85%) e misura di quanto del «dispatch 36,3» sopravvive con PHPR_REG_LOWER on.
2. Ogni handler specializzato: fallback `#[inline(never)]` + bl-count disasm prima/dopo (protocollo H-C2).
3. Verificare zero drop-glue sulla copia Zval scalare calda.
4. Sequenza obbligata (a)→(b)→(c); (d) solo dopo, a decomposizione aggiornata.

## Kill-switch
Corpus 1412×2 rosso per NOME, batteria non-1748 o coppia WP fuori banda ⇒ revert al byte; D micro sotto soglia ⇒ fetta caduta, la prossima solo a decomposizione rifirmata.

## PRIMA FETTA
Predecode + handler specializzati per le op del driver arith (add/mul/shr LONG, cmp+jmp fuso, store a slot). **Giudice**: micro arith R=5 sul pin, criterio pre-registrato. **Soglia GO**: 46,8 → ≤35 ns/iter (−25%), banda-layout con gemello.
