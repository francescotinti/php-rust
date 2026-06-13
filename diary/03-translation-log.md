# Fase 3 — Translation log

> Generato con assistenza AI (Claude Fable 5). Una entry per step.

## Step 2 — Operatori e conversioni + oracle + differential

- **Riferimento C:** Zend/zend_operators.c (full port semantico: is_numeric_string :3620,
  compare :2306, compare_long_to_string :2260, smart_strcmp :3421, smart_streq :3373,
  increment_string :2613, pow_function_base + safe_pow, zendi_try_get_long :378),
  zend_operators.h (dval_to_lval/safe/cap, THREEWAY :516), zend_strtod.c (zend_gcvt),
  zend_smart_str.c:116
- **Target:** php-types: numstr.rs, dtoa.rs, convert.rs, ops.rs, diag.rs
  + tests/differential.rs
- **Oracle:** php 8.5.7 compilato dal sorgente locale (`/tmp/php-src`, build
  `--disable-all --enable-cli`, copia in /tmp per evitare lo spazio nel path che
  rompe autoconf)
- **Differential: 37.835 casi (47 valori × 47 × 17 binop + 6 unari), 0 mismatch**
  byte-per-byte, diagnostica inclusa. Iterazioni: 2.711 → 8 → 0 mismatch.
- **Errori dei report di seconda mano corretti leggendo il C / sondando l'oracle:**
  - [spec] trailing whitespace È ammesso nelle stringhe numeriche PHP 8 (l'agente diceva il contrario)
  - [spec] int vs stringa non-numerica in `<` → confronto come stringhe (non `l!=0`)
  - [spec] NAN→bool è truthy CON warning 8.5 "unexpected NAN value was coerced to bool"
- **Scoperte non documentate trovate dal differential (sarebbero state bug):**
  - stringa numerica con overflow intero → int **satura** a LONG_MAX/MIN (emula strtol),
    silenziosamente se `zend_is_long_compatible` (es. "9223372036854775808"|0 silente,
    "1e100"|0 deprecato)
  - double non rappresentabile in contesto int → Warning "not representable as int";
    NAN|0 emette **due** diagnostici (Warning + Deprecated, per FITS_LONG(NAN)=true)
  - NAN→string: warning solo nel cast esplicito, NON in concat
  - `pow` int overflow: il loop square-multiply **continua in double dal punto di
    overflow** (5**100 e MIN**MAX divergono da `pow(base,exp)` ricalcolato)
  - `~true` → "Cannot perform bitwise not on true" (value name, non type name)
  - conversione operandi sequenziale: op1 fallisce → niente warning da op2
- **Test:** 24 unit/integration + 37.835 differential
- **Tempo:** ~2.5h (inclusa build oracle in parallelo)

## Step 1 — php-types: PhpStr, Zval, PhpArray

- **Riferimento C:** Zend/zend_types.h:335-432, Zend/zend_string.h:114-133,
  Zend/zend_hash.c:257,1099,1182-1183,3300, Zend/zend_long.h:112
- **Target:** crates/php-types (zstr.rs, zval.rs, array.rs)
- **Decisioni applicate:** D-G1, D-G2, D-G3, D-G4
- **Round di iterazione AI:** 1 (più una correzione pre-compilazione)
- **Test pass al primo tentativo:** sì (12/12)
- **Errori incontrati / scoperte:**
  - [semantica] Il modello iniziale di `nNextFreeElement` (flag overflow) era
    impreciso: il C inizializza a `ZEND_LONG_MIN` (zend_hash.c:257), tratta MIN
    come "append parte da 0" (zend_hash.c:1099) e **satura** a `LONG_MAX`
    (zend_hash.c:1183); l'errore "next element is already occupied" deriva dal
    fatto che lo slot saturo è occupato, quindi dopo `unset($a[PHP_INT_MAX])`
    l'append a MAX **riesce di nuovo**. Verificato sul C prima del commit,
    test dedicato aggiunto. Conseguenza osservabile della RFC 8.3
    "negative array index": `$a[-5]=1; $a[]=2;` → chiave -4 (test coperto).
- **Test scritti:** 12 (3 zstr, 2 zval, 7 array: canonicalizzazione chiavi,
  collisione "8"/"08", ordine post-unset/update, next_free, append-at-MAX,
  compattazione)
- **Tempo:** ~25 minuti
