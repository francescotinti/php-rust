# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

## Perimetro 1 — disciplina di borrow del handler `Op::BinaryAdd` (run.rs 944-975): REGGE

Il handler è NLL-pulito. `pop()` produce un `rhs` POSSEDUTO; la guardia
`match (self.frames[top].stack.last(), &rhs)` copia `(*l, *r)` in `fast:
Option<(i64,i64)>`, quindi il borrow condiviso della pila TERMINA prima di
`last_mut()` — nessuna finestra in cui un ri-borrow invalidi. Il fallback
ri-entra in `binary_value_ab(&mut self, …)` con `lhs`/`rhs` GIÀ mossi fuori
dalla pila: zero borrow pendenti su `self` al confine di ri-entrata. Ordine
dei pop (rhs poi lhs) e panics (`expect`) IDENTICI a `binary_value`
(run.rs 213-216): equivalenza del funnel verificata sul corpo, non sulla
dichiarazione. Il hit-path è verbatim `binary_fast` (Long,Long) Add,
overflow sugli operandi ORIGINALI compreso.

**Riserva strutturale (non capitale)**: tra `pop(rhs)` e la scrittura
in-place la pila è in stato intermedio, non osservabile SOLO perché non c'è
alcun `?`/diag/drop non banale nel mezzo (Long ha drop triviale). È un
invariante di forma, non di tipo: una futura diag inserita lì lo rompe in
silenzio → A-MA-100-3.

## Perimetro 2 — forma B della sonda: il verdetto negativo TRASFERISCE, a fortiori

La sonda dà a B il suo caso MIGLIORE: nel programma noop nessun op caldo
attraversa il confine, e i 16 bracci freddi fanno `continue 'outer` senza
nemmeno ri-prendere `frames` (il macro ignora l'ident). La forma vincolata
dal Concilio WP-99 ha confine a OGNI opcode con metodi `&mut self` — nel
run_loop vero quasi ogni handler caldo (Binary→binary_value, CmpJmp,
IncDecSlot generico) — quindi la B reale è ≤ della B della sonda, che perde
già 0,53 ns/op sotto pressione. Un verdetto negativo su un bracket
FAVOREVOLE trasferisce nella direzione refutante per costruzione. Nessuna
refutazione possibile: la caduta a tavolino di H-B1 sta.

## Perimetro 3 — `bin_op_of` come sinonimo: DUE REFUTAZIONI CAPITALI

**R-1 (capitale, sul programma S-99 punto 3)**: la frase «BinarySS/SC/Dst
hanno lo stesso plumbing generico da togliere» (hb2-addspec.out, ripresa in
NEXT_SESSION) è FALSA sul codice. Il fast-path di `BinarySS` (run.rs
1011-1019) prende ENTRAMBI gli slot per PRESTITO e chiama `binary_fast`,
che è `#[inline(always)]` col braccio (Long,Long) Add già dentro: niente
call, niente marshalling di Zval per valore, niente pop/push. I 6,07
ns/occorrenza di D misurano il plumbing del percorso STACK; nel percorso
registro il residuo rimovibile è il solo carico+match del payload `BinOp`.
Un criterio del rollout «derivato da D=6,07» è quindi miscalibrato per
costruzione: sovrastima il tetto di un fattore ignoto e farà «cadere» o
«passare» la leva sul numero sbagliato.

**R-2 (capitale, latente)**: l'equivalenza BinaryAdd ≡ Binary(Add) vive in
UN punto (`bin_op_of`, reg_lower.rs 189-195) che ha il braccio `_ => None`.
Il dividendo dei match esaustivi conquistato in WP-96 («una variante nuova
di Op non compila») è VACUO proprio qui: BinarySub/BinaryMul del rollout
S-99 compileranno senza toccare `bin_op_of`, e la pipeline mista della
batteria perderà le fusioni in silenzio (divergenza perf-only che nessun
corpus di parità può mordere). La cura enumerabile manca dove serve.

**Nota (non capitale)**: «guardia CONTATA» è stato reinterpretato come
«contata nel tempo totale» su micro sempre-hit; il census conta le
OCCORRENZE di BinaryAdd, non il hit/miss. Il rollout eredita D misurato a
hit-rate 100%.

## Emendamenti

- **A-MA-100-1**: il criterio del punto 3 S-99 NON si eredita da D=6,07:
  si pre-registra un tetto NUOVO misurato sul giudice flag-on (forma
  registro), col controfattuale statico rifatto sul residuo vero (match
  del payload, non call+marshalling).
- **A-MA-100-2**: `bin_op_of` perde il wildcard o guadagna un test pinnato
  che enumera le grafie specializzate: un nuovo opcode aritmetico
  specializzato deve NON COMPILARE (o far fallire la batteria) senza una
  decisione esplicita lì.
- **A-MA-100-3**: commento-invariante + debug_assert nel handler BinaryAdd
  («nessuna operazione fallibile tra pop(rhs) e la scrittura in-place»);
  census con split hit/miss prima di derivare criteri di famiglia.

## Kill-switch

- **KS-MA-100-1**: VOID ogni spedizione di Add-nei-registri il cui
  criterio sia derivato da D=6,07 invece che misurato flag-on.
- **KS-MA-100-2**: VOID la garanzia di pipeline mista della batteria se un
  opcode specializzato nuovo compila senza toccare `bin_op_of` o il suo
  test di enumerazione.
