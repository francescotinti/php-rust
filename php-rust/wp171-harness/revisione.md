# Revisione S-171 — lente SEMANTICA (leva L-SL1, commit 67d3b941)

**VERDETTO: REGGE CON RILIEVI** — `long_arith_i64`/`long_cmp_i64` = sottoinsieme Long→Long/bool dell'arm `(Long,Long)` di `binary_fast` (run.rs 148-263 vs 270-325, 0-based), stesso ordine lhs/rhs; ogni `None` ricade sul corpo originale che ricomputa da zero, fast path senza effetti collaterali ⇒ overflow/Div/Mod/Pow/Concat/Spaceship/Ref/Undef/Double/stringa identici per costruzione; store in place su `dst` Long ≡ `store_slot`+`gc_note` (`is_gc_container(Long)=false`, zval.rs 296). Nessun caso che INVALIDI il claim; i rilievi sono di copertura e dichiarazione.

## Rilievi
1. **La fixture non prova che il fast path sia PRESO** (residuo maggiore, non invalida): fx-sl1 passa identica anche se ogni forma cadesse al lento. Manca il mutante abortivo di S-170 (m8a/m13a): `Some(r)→Some(r+1)` in run.rs 2165 e `long_cmp_i64` negata in 2232 devono rompere righe NOMINATE (dq100, bitops, lt-loop, dec-loop). La sola prova indiretta è il D timing.
2. **«typed-ref» è nominale**: `function typed(int &$x…)` (fx-sl1.php:40-42) NON crea un typed reference in PHP (solo le proprietà tipizzate); il ramo `reg_store_slot` con `typed_refs` non vuoto e `dst` `Ref` (run.rs 387-392) resta scoperto. Corretto per costruzione (codice originale), copertura dichiarata al p.5 del criterio ma non reale.
3. **Nessun dump delle forme non-driver**: che `bitops` (fx-sl1.php:29) e `assign-form` (:39) abbassino a `BinarySCSCDst` (reg_lower.rs 486/495) non è verificato — `bin_op_of` (349) accetta QUALSIASI `BinOp` in opa/opb/op/opd, quindi BitAnd/Or/Xor e Shl con r∈[1,63) su l negativo sono coperti solo per asserzione. Residuo minore (verbatim).
4. **Census cambia significato**: `dcn!` (2225 vs slow 480/491), `note_slot_read` (read_slot, arrays.rs 894) e `gc_note` non scattano nel fast path; i conteggi zval-census di CmpJmpSC/BinarySCSCDst tra s166 e s171 calano per NON-materializzazione, non per meno lavoro semantico. Coerente con «dcn = drop di Zval»; da dichiarare nel prossimo census.
5. **Dente loc +174** (loc_dente.rs 74-78): dichiarato, ma ~45 righe sono commenti e 14 la firma a 12 parametri di `binary_scsc_dst_slow` (run.rs 407-420): una struct/`&Op` l'avrebbe contenuta. Minore.
6. **fx-sl1-div congela §3.11/§3.13**: pin==stash s166 è giusto per la leva, ma resta un contratto «invariante», mai un atteso.

## Azioni S-172
1. Mutante abortivo su B (fast path BinarySCSCDst e CmpJmpSC) contro fx-sl1: righe rotte NOMINATE, poi revert al byte.
2. Aggiungere a fx-sl1 un typed reference vero (`class C{public int $p;} $r=&$o->p; $r += $i*3-($i>>2)` + overflow → TypeError/float) e Shl negativo in range.
3. Dump `--dump-ops` di fx-sl1 con conteggio `BinarySCSCDst`/`CmpJmpSC`/`IncDecSlotJmp` per riga, archiviato in wp171-harness.
4. Nota nel census S-172: delta `dcn!`/slot_read attribuito a L-SL1.
