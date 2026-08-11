# s131-propstep-lettura.md — modello prop_step ACQUISITO: la torta di E−E2 ha nomi

**Esito formale**: SONDA-PROPSTEP ACQUISITA (rc=0, quiescenza rc=0 in header, conteggi
deterministici, parità stdout, tree ripristinato). Quote = MODELLO (build emendata, pin s130).

## Reperti (objdatains, per statement; p2 conferma ±1 ns; p5/p6 = ×2 coerente)
1. **E−E2 = 166,9** (S-130: 165,1 — trasferisce): **prop_step interno 130,7 (78%)** +
   dispatch fuori prop_step 36,3. Chiusura blocchi 93–94% su tutte le categorie.
2. **Partizione prop_step**: guardie (container-guard: prop_key+props.get+prop_slot_state+
   prop_indirect_guard) **49,4** · defer_check (key_read+prop_key+contains) **37,0** ·
   key+container_op (prop_key+enum-check+contains/get_mut) **34,3** · **borrow 1,5**
   (il try_borrow_mut NON è il costo!) · altro 8,5.
3. **Quota resolve per call-site (az.rev. S-130 #4 CHIUSA con rerun)**: statement = 4 siti
   probati (c9=3 prop_key ≈10,4 ns/chiamata = 31,2 · c10=1 prop_key_read 9,1) = **40,3 ns**
   + 1 sito ENUMERATO per NOME: **prop_indirect_guard** (oop.rs, resolve interna, costo
   netto ≈0 — cache-calda, dentro «guardie») · **ctor = 4 resolve = 70,8 ns** (17,7/chiamata,
   più care delle statement: cammino Denied/Dynamic del ctor) — conferma s130-e1a-lettura.
4. **Non-resolve nei blocchi = 81,9 ns/statement**: ≈3 lookup sulla props-map della STESSA
   chiave (props.get in guardie + contains in defer + contains/get_mut in container_op)
   + prop_slot_state + prop_indirect_guard(prop_info) + is_none/branching.
5. **Forma E1 NOMINATA dal modello (per S-131/S-132)**: **E1-KO «resolve-once»** — in
   `field_write_prop_step` UNA `resolve_prop_access` dopo l'header, riusata dai 4 siti
   helper (key0/declared0/denied0 ≡ per costruzione a prop_key/prop_is_declared_slot/
   prop_key_read); UB = 40,3 − 10,4 ≈ **30 ns/statement** (converge con UB E1a 31–35).
   Il passo successivo (S-132) è «lookup-once» sulla props-map (~3 lookup → 1, dentro
   gli 81,9 non-resolve) e la forma ctor (70,8 ns, tocca New/PropSet — forma separata).
