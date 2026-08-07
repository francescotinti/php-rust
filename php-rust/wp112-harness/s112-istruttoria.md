# S-112 istruttoria (PRIMA del criterio, vincolo NEXT_SESSION §S-112)

**Candidata (b) arr RMW-su-dim — DECLASSATA per attesa sotto-pavimento.**
Dal dump sul pin 92909544 il corpo interno di arr è 8 op/iter:
`CmpJmpSC · LoadVarPushConst · StringifySlot · ConcatN(2) · FetchDim ·
BinarySTDst · Sweep · IncDecSlotJmp`. Il bigramma nominato dal census S-108
(`FetchDim→BinarySTDst`) NON è una finestra legale: FetchDim sospende in coda
(`enter_object_method` per ArrayAccess::offsetGet, run.rs) e il vincolo S-108
(«la finestra termina al primo helper sospendibile») più il veto in vigore
(«finestre fuse oltre un helper sospendibile») lasciano come forma massima
`[ConcatN, FetchDim]` (ConcatN è pura; StringifySlot sospende via __toString).
Attesa: −1 dispatch, −2 transiti ≈ +3,5 ns/iter (calibrazione S-106: −2
dispatch −4 transiti = +7,0) < pavimento 4 ns/iter ⇒ verdetto prevedibile
«sotto soglia»: non si spedisce una leva progettata per fallire il suo criterio.

**Candidata (c) Sweep/Zval — ISTRUITA, forma concreta trovata nei dump.**
Sweep di per sé ha già il fast-path a buffer vuoto (WP-39/50) e la fusione è
vetata (veto GC). Ma il corpo arith sul pin è 4 op/iter (CmpJmpSC ·
BinarySCSCDst · Sweep · IncDecSlotJmp) a ~80 ns/iter contro ~8,6 dell'oracle:
il collo non è il numero di dispatch (famiglia frontend refutata S-111), è il
costo per-op del ciclo di vita Zval. Due falle nominate leggendo run.rs:
1. **`binary_fast` NON copre Shl/Shr** (commento: «Shl/Shr → generic»). La
   guardia-b inline di `BinarySCSCDst` (opb=Shr in arith: `$i>>2`) fallisce
   QUINDI OGNI iterazione e paga `reg_load_slot` + `binary_value_ab` outlined
   + `apply_binop_ovl` outlined, 50M volte per run del giudice peggiore (9,3).
   La semantica (Long,Long) di ops::shl/shr è pura salvo y<0 (ArithmeticError):
   fast-pathabile VERBATIM con `None` sul ramo d'errore.
2. **Le code RMW non hanno la guardia inline**: `BinarySCSCDst` (coda opd,
   run.rs:1813-1814) e `BinarySTDst` (run.rs:1711) chiamano
   `self.binary_value_ab` outlined anche quando la sua PRIMA riga
   (`binary_fast`) risolverebbe: hoisting puro della guardia già precedentata
   nello STESSO braccio (combine, run.rs:1809). Zero semantica nuova per
   costruzione.
Beneficiari dai dump: arith (siti 1+2), prop (BinarySTDst a 0017 del suo
corpo), arr (BinarySTDst a 0028). calls/str/re non usano questi sentieri nel
corpo caldo ⇒ guardie non-bersaglio.

**Scelta: leva H-A2 «fast-path Long del sentiero RMW»** (famiglia (c),
tre siti per NOME: A2a shift, A2b coda BinarySCSCDst, A2c coda BinarySTDst).
Nessuna modifica al pass/emissione: admission a emissione INVARIATA (più
forte del solito). Nessuna stima esterna nel criterio (vincolo (e)).
