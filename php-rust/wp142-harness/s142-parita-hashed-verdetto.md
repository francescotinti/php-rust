# S-142 — verdetto parità Hashed/tombstoni/annidati (az.rev. S-141 #4)

Micro: `parita-hashed.php` (Hashed con chiavi str + tombstoni int/str, annidati
packed↔hashed, Rc condiviso, oggetti con `__destruct` = ordine di drop osservabile).
Binari: A = stash `phpr-s140` (f2708b75) · B = build fresca A′ HEAD (bba8a734,
gemello del candidato f751ef5b — 91 byte attribuiti: LC_UUID + `__DATE__/__TIME__`
dep C + 2 hash derivati; 0 byte di codice; S-141 build senza SOURCE_DATE_EPOCH,
dichiarato). Oracle 8.5.7 brew.

## Esiti
1. **A == B BYTE-IDENTICO nei 2 modi** (on/off): la domanda del revisore è chiusa —
   L-RD1 NON cambia la semantica di teardown di phpr su Hashed, tombstoni,
   annidati, Rc condiviso. Invarianza ora VERIFICATA su questi cammini (non più
   solo argomentata); resta il perimetro batteria/corpus/fixture/ORM della catena.
2. **phpr ↔ oracle: DIVERGENZA PRE-ESISTENTE** (identica in s140, NON della leva),
   NON a catalogo → catalogata oggi in PHPR_DIVERGENCES_FROM_PHP.md (§3.22):
   `unset($a[k])` su elemento d'array (packed E hashed) DIFFERISCE il `__destruct`
   del valore al drop dell'ARRAY (il tombstone tiene vivo il valore); Zend lo
   esegue all'`unset`. `unset($var)` è a parità. Sonda: `probe-unset.php`
   (in questa harness; 3 casi var/hashed/packed, oracle eager 3/3, phpr eager 1/3).
3. Fuori dal timing dei distruttori, output BYTE-IDENTICO vs oracle (count,
   array_keys post-tombstone, serialize annidati, Rc share).
