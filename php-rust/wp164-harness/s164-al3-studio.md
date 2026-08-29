# S-164 p.4 — studio L-AL3 «FrameExt riciclato via pool sul fast path closure» (PRIMA dell'edit; criterio A/B da finalizzare DOPO la coppia)

## Sito (census s163, ledger APPENDICE: AL2-fast = 1 alloc/chiamata residua)
- `push_closure_frame_one` (vm/mod.rs:11436): `frame.ext_mut().closure_id = Some(cl.id)`
  → `ext_mut` = `self.ext.get_or_insert_with(Box::default)` (vm/mod.rs:2754) ⇒ **1 Box<FrameExt>
  allocato a OGNI chiamata closure-fast** (il pool WP-30 ricicla SOLO slots+stack:
  `pooled_frame` vm/mod.rs:4632, `recycle_frame` vm/mod.rs:4651 fa `drop(frame)` ⇒ dealloc del box).

## Audit semantico (fatto, S-164)
- Lettori di `ext`: SOLO gli accessor (2744/2751/2758) + il walker census 2216 (content-based).
  NESSUN sito tratta `ext.is_some()` come flag: box con campi tutti default ≡ None.
- `ext` è l'ULTIMO campo di Frame (2604, «DECLARATION POSITION IS LOAD-BEARING»): in
  `recycle_frame` si può fare `let ext = frame.ext.take()` PRIMA di `drop(frame)` e resettare
  DOPO ⇒ i contenuti Rc-bearing dell'ext droppano ESATTAMENTE dove droppavano (dopo ogni
  altro campo) ⇒ sequenza di release Rc bit-identica (vincolo del doc 2594-2603 rispettato).

## Disegno (leva minima, ogni altra forma INVARIATA)
1. `FramePool`: freelist `exts: Vec<Box<FrameExt>>` cap piccolo (8) + `take_ext`/`put_ext`.
2. `recycle_frame`: `let ext = frame.ext.take(); drop(frame); if let Some(mut b) = ext {
   *b = FrameExt::default(); pool.put_ext(b); }` (oltre cap ⇒ drop = dealloc come oggi;
   il reset rilascia i contenuti nello STESSO punto dell'ordine di drop attuale).
3. `push_closure_frame_one`: `if frame.ext.is_none() { frame.ext = pool.take_ext(); }` prima
   del `closure_id = Some(..)` ⇒ zero alloc a regime sul fast path.
   (Il generico `push_closure_frame` NON si tocca: resta 2 alloc, dichiarato.)
4. Reset senza Zvals nel caso fast (solo closure_id: Option<u32>): il reset generico
   `*b = default()` copre anche box provenienti da frame ricchi (extra_args ecc).
5. Igiene RetainSet: dopo reset i box non tengono Zvals ⇒ nessun rischio pin per-richiesta;
   verificare comunque dove vive frame_pool nel ciclo richiesta (request_end).
6. NEL medesimo edit (az.rev. S-163 #4 / S-162 az.4): `unreachable!` sui DUE bracci morti
   gemelli `call_fn_one` + `call_method_one` — da dichiarare nel criterio dell'edit.

## Vincoli per il criterio A/B (da PRE-registrare prima dell'edit, dopo la coppia)
- Driver: m-missload (bersaglio nominato) + guardia m-arrload (non-bersaglio, solo-regressione).
- Census attesa NELL'UNITÀ DELLO STRUMENTO (hostcall_n = TUTTE le alloc): Δ = 1 alloc/chiamata
  ESATTA sul tag del sito, altri nomi zero; **esito census che FERMA pre-registrato ANCHE in ns**
  (az.rev. S-163 #3): banda per-sito NON fondata ⇒ si dichiara A UN LATO (soglia inferiore
  4 ns/iter); D sotto soglia col census esatto ⇒ leva NON paga ⇒ STOP dichiarato, niente promo.
- Disasm bl run_loop prima/dopo (protocollo); gemello A dal tree s163 nel target CANONICO;
  dente loc mod.rs da PRE-dichiarare; batteria per il branch `recycle_frame` (tocca TUTTI i frame).
