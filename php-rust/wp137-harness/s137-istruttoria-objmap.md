# s137-istruttoria-objmap — canale «valore-oggetto 43,4» ATTRIBUITO (meccanismo), leva RIFIUTATA

1. **Conteggi** (build emendata census `--features gc-census` sul worktree s136 +
   patch s137tp INERTE, binario 14,4M, MAI pinnabile; run diretti, sola aritmetica
   di contatori deterministici):
   - m0_obj2048 (`$map[$i&2047]=$e`, $e oggetto): notes 8.997.966,
     **inserted 3.000.001 (1/iter)** · **sweeps main 3.000.001 (1/statement)** ·
     **demoted 3.000.002** · freed 0 · collects 0.
   - m1_int2048 (`$map[k]=int`): notes 8.997.971, inserted 2, sweeps main 2,
     demoted 2 — GC a riposo.
2. **Meccanismo per NOME** (round-trip per statement): il displaced Zval::Object fa
   `gc_note` → `gc_note_slow` (borrow + set_pos + `Rc::clone` + `gc_buf.push`);
   lo Sweep di fine statement poppa il candidato, `strong_count>2` ⇒ **demozione
   con `gc.clear()` che CANCELLA `buffered`** (vm/mod.rs, sweep ~4161-4190) ⇒ la
   nota successiva RI-BUFFRA. Il dedup dei mirror-flag copre solo l'hash insert
   (cycle_root), non il round-trip.
3. **Leva piccola RIFIUTATA con precedente**: il dedup note-time (`count > 2` ⇒
   salta il buffer) è **REFUTATO a storico** — commento in `gc_note_slow`
   (vm/mod.rs ~3944-3952): perde la finestra dei drop non-agganciati nello stesso
   statement, distruttore ritardato = flake WP-21 (REST sideload unique-filename).
   Un dedup across-statement sicuro richiede la collectability al DROP-SITE
   (modello Zend: distruzione refcount-driven, buffer solo per i cicli) ⇒ è il
   piano [[php-rust-gc-cycle-collector-plan]] (via il strong-ref di `created`),
   NON una leva locale.
4. **Perimetro residuo di objmap** post-attribuzione: chiave 10,0 (presize vs
   grow) · quota Rc/borrow del round-trip comprimibile SOLO col piano GC.
   L'apertura «objmap residuo 116,7» si RIQUALIFICA: valore-oggetto 43,4 =
   GC-design-bound (cura nominata), non più candidato a leva micro.
5. Esito S-137: **leva NON tentata (A/B non eseguito) = anomalia DICHIARATA**;
   ragioni per NOME: (a) blocco leve dim-write pre-registrato (s137-criterio-sonda
   p.5, sonda NON CHIUSA); (b) objmap valore-oggetto design-bound (p.3 qui);
   (c) nessun altro candidato con prezzi misurati nella finestra (dispatch 36,3
   senza meccanismo nominato; chiave 10,0 sotto soglia).
