# Revisione S-132 — lente SEMANTICA (revisore singolo)

## Reperto principale
Il claim sopravvive sui cammini raggiungibili, ma «equivalente per costruzione» ha UN buco reale: l'istanza la cui props-table non usa il layout di classe. Esiste: la destroy-phase del GC svuota le props con `Props::new()` a layout VUOTO (vm/mod.rs:5355), class_id intatto. Se un `__destruct` ripopola una prop dichiarata (leaf `replace` by-name → finisce in `dyn_entries`, object.rs:920-936) e poi la usa non-leaf: il VECCHIO `get(name)` la trovava nei dyn_entries e descendeva raw; il NUOVO `get_slot_mut(si)` (object.rs:844) risponde None → MagicDescend (arrays.rs:550), e il drain senza `__get` con next step Prop erra incondizionatamente «Attempt to modify property on null» (arrays.rs:1634-1640) dove prima la scrittura atterrava. Divergenza vera, confinata alla finestra di teardown, coperta da NESSUN gate. Non invalida la promozione (irraggiungibile da batteria/corpus/ORM/hk), ma il «by construction» va corretto in «by construction fuori dalla finestra teardown».

## Reperti secondari
1. `slot0` poggia su tre punti distanti mai asseriti insieme: stampatura per-classe (compile/class.rs:404), guardia `obj_class == s` scope-private (oop.rs:439-445), costruzione `with_layout`. `get_slot_mut` non ha il fallback di `replace_slot` (che ritorna `Err` su indice estraneo): un futuro costruttore di Props disallineato rompe in SILENZIO solo il ramo lettura.
2. Hooked BACKED non-leaf: descend raw senza invocare hook — pre-esistente e invariato dalla leva; il commento nuovo non lo delimita.
3. Claim secondario: l'on-only CANONICO è ben definito (coppie proprie per gamba, pre-registrato in s132-criterio-pair.md p.4-bis; il misto è etichettato companion), ma 1,752–1,768 è N=2 coppie e l'off-only residuo N=1 dopo l'esclusione leg1-off: intervallo di due punti, non banda.

## Vagliate e respinte
- (b) omonima dinamica di una dichiarata: impossibile — get/get_mut/contains/set sono tutte layout-first e mai cadono ai dyn_entries per nomi nel layout (object.rs:869-916).
- (c) Denied±presente: `prop_indirect_guard` esce subito per non-Slot (oop.rs:766); key0 identica (storage_key); ordine guardia→defer preservato.
- (d) enum/lazy: condizioni identiche; il blocco finale contains/set/get_mut resta raggiungibile per lazy non-leaf (arrays.rs:607-613).
- (e) get vs get_mut: nessun effetto collaterale; stesso state via `as_deref()`.
- (f) rw: propagato invariato; `Some(Undef)` conta «presente» in entrambe le vie.

## Azioni S-133
1. Fixture destructor-window (oggetto in ciclo GC, `__destruct` riscrive prop dichiarata leaf poi non-leaf), diff vecchio/nuovo.
2. `debug_assert!` in field_write_prop_step: `slot0.is_some() ⇒ presenza slot == contains(key0)`.
3. Dare a `get_slot_mut` la disciplina di `replace_slot` o documentare l'invariante sul tipo Props.
4. Annotare nel PERF_MAP che il riferimento on-only è N=2 coppie.
5. Delimitare nel commento L-LO1 il caso hooked-backed non-leaf come pre-esistente.
