# S-114 istruttoria L-A «trigramma prop in prestito» (PRIMA del criterio; fonte: doc/DIFFERENZIALE_ZEND_PHPR.md §5+§9.1, letture run.rs/oop.rs al pin f71abd2a)

**Bigramma bersaglio** (adiacente nel corpo caldo di prop.php, dump S-112/113):
`PropGetSlotRecv {recv, slot, name, ic}` (run.rs:4168) + `BinaryTCPropSetPop {op, cidx, name, ic}` (run.rs:4320) = `$o->x = $o->y + C`.

**Costo storico per iterazione** (letture verificate):
1. `read_slot(slots[recv])` → clone Rc del ricevitore PUSH in pila (arrays.rs:828) che
   BinaryTCPropSetPop ri-POPpa (run.rs:4338) e muore a fine `prop_set_entry` — round-trip intero.
2. `reg_load_slot` → SECONDO clone Rc (get) che sull'IC-hit muore inutilizzato (falla H-P1, +3,3 misurati S-113, dentro banda 4,33).
3. `prop_set_entry` (run.rs:635): `lazy_prop_access` (borrow per il check lazy) + guardie IC (borrow, run.rs:696-708) + `write_property_at` (borrow su get_slot + borrow_mut su replace_slot, oop.rs:83-98) + `value.clone()` buttato (DISCARD=true) + un dispatch intero del secondo op.

**Forma della leva** — peephole RUNTIME nel braccio PropGetSlotRecv (emissione/pass NON toccati, dump identici per costruzione):
- `func.ops.get(ip+1)` = `BinaryTCPropSetPop` → probe bipartito SOLO-borrow; se TUTTO colpisce: yv sotto borrow (condizioni get VERBATIM da prop_get_entry run.rs:605-611: slot direttamente Object, ic.get(sk), class_id+1==cid1, lazy.is_none(), get_slot non-Undef, deref_clone) → `binary_fast(op, &yv, &const)` Some → guardie set VERBATIM da prop_set_entry run.rs:697-707 (set_ic.get(sk), class_id+1==cid2, lazy.is_none(), !is_enum_case, get_slot(sslot) presente, Ref ⇒ typed_refs vuoto) + `!INIT_PROPS` → commit: `write_property_at(&slots[recv], name, Some(sslot), value)` (STESSA funzione: Ref write-through identico) + `gc_note(old)`; `ip+2`.
- QUALSIASI condizione manca → sequenza storica VERBATIM da zero (probe solo-borrow: NESSUNO stato toccato prima del commit — warning Undef, __get/__set, lazy, enum, coercizioni restano al sentiero storico). Nessun helper sospendibile dentro la finestra fusa (veto rispettato): su double-hit non gira codice utente per costruzione (classi plain_set_props).
- Census: fusione compilata FUORI sotto `zval-census`/`op-census` (scn!/dcn!/census_record contano il sentiero storico; op-census non vedrebbe il secondo op).

**Risparmio di progetto su double-hit**: 2 cloni Rc (recv round-trip + get) + 1 dispatch intero + lazy_prop_access + value.clone + 2 push/2 pop di pila. Composizione sopra il singolo clone (~3,3): attesa QUALITATIVA sopra soglia 4,33 (nessuna cifra nel criterio, vincolo S-112).

**Non-bersagli**: arith/calls/str/arr/re non hanno PropGetSlotRecv nei corpi caldi (dump S-112, admission-out) → guardie solo-regressione con banda(cat) MISURATA S-114.
