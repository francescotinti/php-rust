# Revisione S-153 — lente SEMANTICA — VERDETTO: **REGGE**

## Reperti per punto
a) **Chiavi/ZStr condivise: immutabili per costruzione.** L'unico mutatore in-place è `ZStr::try_append` (zstr.rs:125), gated `rc==1` con fallback COW `concat2`; i blocchi del pool hanno rc≥2 per sempre (il pool tiene un ref) ⇒ mai mutati. `string_offset_write` (vm/arrays.rs:1195-1206) ricostruisce SEMPRE la stringa (`to_vec` + `PhpStr::new`), mai scrittura nel blocco: `$t[0]['type'][0]='X'` non può corrompere `ty_arrow`. Le chiavi non sono mai mutate: `ksort` riordina coppie, `array_change_key_case` costruisce array nuovo; nessun `&mut Key` nel workspace. La `hash: Cell<u64>` condivisa è cache idempotente, single-thread.
b) **thread_local sano.** php-server usa `std::thread::spawn` (worker_pool.rs:388) ⇒ un pool per worker; `ZStr` è `!Send/!Sync` (NonNull, zstr.rs:62-68) e NON esiste `unsafe impl Send` per ZStr/PhpStr: il compilatore vieta l'attraversamento. Reperto collaterale pre-esistente (fuori claim): `unsafe impl Send for UdfCallable(Zval)` in vm/pdo.rs:45-46 è l'unica scappatoia che trasporta Rc — motivata (stesso thread), da sorvegliare.
c) **Ordine/condizioni = oracle 8.5.7** (verificato live): file,line,function,class,object,type,args; `type` solo con class; `object` solo con PROVIDE_OBJECT; `args` omessa su eval/IGNORE_ARGS. Identico al pre-leva (fixture backtrace 10/10 in promo).
d) **getTrace/getTraceAsString = cammino SEPARATO** (mod.rs:13419, chiavi fresche per frame) non toccato; i soli consumatori di BtFrame sono ho_debug_backtrace e ho_debug_print_backtrace (`as_bytes`, resa identica); tipi enforce dal compilatore.
e) **TD1 revert pulito**: diff c8adb54..HEAD su mod.rs = SOLO hunk BT2 (Vec<u8>→ZStr + BtFrame); gc_note_frame/gc_sweep/gc_release_cascade intatte; zero occorrenze "L-TD1" nei crates.
f) **Numeri riconciliati**: D=+266,7 (733,3→466,7), soglia 66,7, riconciliazione |0,0|, guardie 12/12, segni 7/7 = 2 smoke + 5 R5 (tutti B<A); TD1 −3,3 vs 4 (gemello smoke +6,7 anch'esso sotto soglia 10); promo: batteria 1748/0/2, corpus 1412 zero flip, ORM 16 nomi, hk 0E/0F, pin ×2 — tutto coerente col session file. **Rilievo (non invalida)**: il giudice backtrace ha tick=66,7 ns (N=150000, timer 10 ms): la conferma post-pin +333,3 dista esattamente 1 tick dall'attesa; il "−36%" puntuale porta ±1 tick (≈±9 pp) di quantizzazione.

## Azioni
1. S-154: eseguire la sonda k post-leva già DOVUTA (D=+266,7 > UB 160: meccanismo non ancora nominato per intero).
2. Citare il guadagno come banda: −36%±9 pp finché il giudice resta a N=150000.
3. Alzare N del giudice backtrace (≥×4) per portare il tick sotto la soglia di 4 ns/iter.
4. Dente nuovo: dopo due backtrace, scrivere `$t[0]['type'][0]='X'` e asserire che il terzo resta `->` (fissa la proprietà COW su cui poggia la leva).
5. Annotare UdfCallable (pdo.rs) come unico unsafe-Send portatore di Zval, guardia se mai le UDF usciranno dal thread della Connection.
