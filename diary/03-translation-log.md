# Fase 3 — Translation log

> Generato con assistenza AI (Claude Fable 5). Una entry per step.

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
