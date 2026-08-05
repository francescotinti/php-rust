# Verbale Sedia 2 — Niko Matsakis (ownership/aliasing/borrow) — Concilio WP-102

## VERDETTO

S-100 sui miei temi: **nessuna refutazione capitale**. La bozza §S-101 ha
**una refutazione capitale** (H-C1 così come nominata punta al canale
sbagliato sul micro che la motiva) più una clausola refutata al punto 3.

## Refutazioni

**R1 (CAPITALE) — H-C1 «clone del valore letto» è mal mirata sul suo stesso
micro.** `Zval` è `#[derive(Clone)]` con varianti heap `Rc<…>`
(`php-types/src/zval.rs:14`): il "clone" è un bump di refcount, mai una
copia profonda. Su prop.php le proprietà sono `Long` (Copy: clone
gratuito). Il churn del ciclo di vita Zval viene dal **RICEVITORE**: ogni
`PropGet` fa `obj.deref_clone()` (`run.rs:3394` — bump di `Rc<Object>`),
poi Pop/drop, e ogni drop di `Zval::Object` passa da `gc_note`
(`vm/mod.rs:3898`) che fa un `rc.borrow()` per nota. Il ~27% aggregato
(drop 12,6 + clone 7,6 + gc_note 5,3 + deref_clone 2,1) NON attribuisce
tra ricevitore e valore. Prior art in-tree: `ThisPropGet` (`run.rs:3418`)
già PRESTA il ricevitore sull'IC-hit senza clone. Prima riga di H-C1:
census che decompone i quattro simboli per SITO (ricevitore vs valore vs
Sweep/Pop di traffico), altrimenti si ottimizza il canale minore.

**R2 (clausola §S-101 punto 3) — «l'emissione non cambia ⇒ batteria+corpus
bastano» è FALSA per H-C1.** Un cambio di aliasing/lifetime a runtime è
PIÙ pericoloso di un cambio d'emissione: sposta il momento in cui
`strong_count` tocca 1, cioè l'ORDINE dei `__destruct` e la finestra del
cycle-collector («under-noting delays a destructor», vm/mod.rs:3892). Il
collaudo WP di parità è dovuto anche per H-C1.

**R3 — il profilo come tariffa.** ~27% è la quota dell'INTERO micro, non
il recuperabile: Sweep/Pop droppano anche temporanei che nessun prestito
elimina. L'atteso di H-C1 si calcola dal canale contato (n° bump+note
eliminati/iter × costo per evento misurato), mai da 27%×T.

**R4 — estensione BinaryAdd (reg_lower.rs:291-295): sound, con due bordi.**
La riscrittura è in-place 1:1 (nessun addr/exc_table da rimappare — bene),
`visit_addrs`/`bin_op_of` la classificano esaustivamente, il fallback del
handler rientra nel funnel generico. Ma: (a) il tripwire «flag-on zero
`Binary(Add)`» vive nei test del funnel — i corpi FUORI funnel
(prop_init, const-thunk) legittimamente lo conservano: il tripwire deve
dichiarare il suo perimetro per NOME o decade in silenzio; (b) la
riscrittura tocca anche regioni blocked/park: l'equivalenza lì è provata
solo dal differenziale, che va esteso se mai BinaryAdd divergesse dal
generico su un solo diagnostico.

## Emendamenti

- **A-MA-102-1**: prima di iscrivere H-C1, census per SITO dei quattro
  simboli Zval-lifecycle (ricevitore vs valore vs traffico); l'ipotesi si
  nomina sul canale che il census indica (probabile: prestito del
  RICEVITORE su PropGet, à la ThisPropGet).
- **A-MA-102-2**: lista scritta PRIMA degli hazard di aliasing con una
  fixture ciascuno: `__get`, hook get/set, `&$o->x` (Ref nel slot),
  riassegnazione intra-espressione (`$o->x + ($o->x=…)`), `unset` durante
  la lettura, lazy ghost/proxy, readonly/typed-uninit (`Undef` fatale),
  ordine `__destruct`, ciclo GC con proprietà nel ciclo, e gli op che
  scrivono lo stack IN PLACE (`BinaryAdd` fa `*last_mut()=v`: mai un
  alias di un slot proprietà sullo stack).
- **A-MA-102-3**: nessun borrow-guard `RefCell` può attraversare un
  confine di op o un rientro VM: il sigillo sia di TIPO (scope del borrow
  chiuso prima del push), non una review.
- **A-MA-102-4**: perimetro del tripwire zero-`Binary(Add)` dichiarato
  per NOME (funnel sì, prop_init/thunk no) e asserito via `all_funcs`.

## Kill-switch

- **KS-MA-102-1**: H-C1 non si scrive finché census A-MA-102-1 e fixture
  A-MA-102-2 non sono in-tree e verdi su ENTRAMBI i motori.
- **KS-MA-102-2**: atteso dal canale contato; se < pavimento sonda ⇒
  rinuncia pre-registrata.
- **KS-MA-102-3**: qualunque cambio d'ordine dei distruttori o del
  free-order (corpus per NOME, gate GC) ⇒ reject, senza appello.
- **KS-MA-102-4**: se H-C1 si scrive, coppia WP di parità OBBLIGATORIA
  anche senza cambi d'emissione (R2).
