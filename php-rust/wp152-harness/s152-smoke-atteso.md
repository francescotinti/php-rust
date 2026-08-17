# S-152 — attesi di smoke della sonda canali (dichiarati BLIND, PRIMA di ogni esecuzione del probe)

Verifica dovuta da un SECONDO attore (rev. az.4 S-151) PRIMA del run di
record: coerenza attesi ↔ criterio (`s152-criterio-sonde.md`) ↔ codice
(`crates/php-runtime/src/vm/sondaprice.rs`) ↔ check dello script
(`s152-sonda-canali.sh:check_keys`).

## Guardie (tutte a esito ESATTO)
1. stdout del driver = esattamente `SONDA-OK` (nessun altro output).
2. File `PHPR_SONDA_OUT` presente con TUTTE le righe `chiave=valore`, e SOLO
   nel formato `s{145,149,152}.price.<k>_ns=<float>` o `=<int>` per i meta.
3. Chiavi s145: 9 prezzi (`cal mv_scalar mv_str mv_arr mv_obj note_scalar
   note_cont_repeat pair_zcell pair_arr0`) tutti > 0, più `n_mv=200000000` e
   `n_pair=20000000` → 11 righe.
4. Chiavi s149: 4 prezzi (`pair16 pair32 pair48 splitoff3`) > 0, più
   `zval_size=16` (audit taglia: diversa ⇒ shape da RIDICHIARARE, non gate)
   e `n_pair=20000000` → 6 righe.
5. Chiavi s152: 9 prezzi (`c1_pair c2_borrow c2_borrow_mut c3_size_pair
   mock_deref mock_dup_rel mock_decq mock_alloc miheap_pair`) tutti > 0, più
   `obj_size=<int >0>` (informativo, taglia `RefCell<Object>`),
   `n_mv=200000000`, `n_pair=20000000` → 12 righe.
6. Ordinamenti di plausibilità NON-gate (si dichiarano se violati, non
   fermano): `mock_deref ≤ c2_borrow` atteso (il sostitutivo hot fa meno
   lavoro del RefCell) · `mock_alloc ≤ c3_size_pair` atteso (riuso slot vs
   malloc) · `c1_pair` nell'ordine di `mv_obj` (stessa coppia refcount,
   senza il wrapper Zval).
7. rc del checker = 0; smoke fallito ⇒ STOP rc=8, NESSUN run di record.
