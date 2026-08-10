# s127-admission-lettura.md — L-OL1: ammissione ESEGUITA, forma nominata (cifre dai verdetti citati)

Ordine rispettato: regola di nomina (c71620b) → admission census (`s127-admission-verdetto.out`)
+ disasm + profilo (`s127-admission-profile.out`) → forma. Regola di nomina s127 soddisfatta:
objchurn eleggibile (>6 E multi-% nel profilo del run reale S-126).

## Census (ACQUISITA, R=2 identici)
`new En`+ctor+drop = **9,00 alloc + 9,00 free/oggetto** (oracle Zend: ~1-2). Decomposizione
coerente coi sub-micro: interpolazione `"n$i"` = 2 (9→7 in objallocni) · data-insert = +4 ·
drop differito = +1 · overwrite mappa = 1. 9 coppie mi_malloc/free ≈ 150-180 ns: NON spiegano
da sole il delta 1093 ns ⇒ il resto è macchineria.

## Disasm pin (protocollo S-104, riferimento "prima")
`alloc_object` = **383 istruzioni, 36 bl** (caldo: 2× `Props::set` per NOME, `BTreeMap::insert`
su `created`, `next_id`, `gc_buf_push` [5 bl], 2× `mi_malloc_aligned`). Morte via sweep:
selezione buffer → `BTreeMap::remove` → `resolve_method_runtime(__destruct)` per morte →
cascata release (con 2 remove SipHash su std-HashMap per-id SEMPRE VUOTE: `var_dump_debug`,
`stringify_args`).

## Profilo micro (indizio unilaterale)
Nessun dominatore singolo; gruppo max = lookup per NOME (HashMap::get 1004 + SipHash 433 +
slot_of 889 + quota memcmp 1396) ≫ GC/morte (sweep 731 + cascata + BTree) ≫ alloc (~800) ≫
coercizioni ctor (474). Il costo è spalmato sull'intera filiera nascita/ctor/morte.

## FORMA NOMINATA: L-OL1-F1 «stampo» (template Props per classe)
`OnceCell<Props>` su `CompiledClass`, costruito UNA volta (stessa logica attuale), poi
`alloc_object` fa `template.clone()`: slots per INDICE (niente slot_of/`Props::set` per default),
default `[]`/costanti condivisi COW (`Zval::Array(Rc<PhpArray>)` senza RefCell ⇒ ogni scrittura
passa da `make_mut`; enum-case default = singleton, condividerli è semantica PHP corretta).
**Predizioni ammissione della forma** (arbitro census, da verificare PRIMA dell'A/B):
alloc/free/iter **−1,00 ESATTO** su objalloc/objallocni/objdatains/objdropdef/objchurn
(sparisce l'alloc del default `[]` per-oggetto), objmap **0,00**; bl-count `alloc_object` CALA.

**Forma B scartata con meccanismo** (morte-immediata al sito di nota): contraddice la decisione
di progetto documentata in `gc_note_slow` («Always buffer, no note-time classification», rete
di sicurezza per drop non agganciati nello stesso statement, flake WP-21) — è una leva
multi-sessione con arsenale fixture proprio, non UNA forma. Resta apertura per NOME.
Micro-trim registrati per NOME (non questa forma): guardia is_empty sulle 2 remove SipHash
per-morte · flag has_destruct precompilato · hasher FxHash sulle 2 std-HashMap per-id.
