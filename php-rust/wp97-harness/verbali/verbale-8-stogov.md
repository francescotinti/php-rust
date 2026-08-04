# Verbale sedia 8 — Stogov (engine Zend / fedeltà oracle) — WP-97

## VERDETTO

**F1+F2 in sola misura: metodologia corretta** (liveness prima del dispatch,
come da mia consulenza §2; direzione d'errore dichiarata; determinismo al
contatore). **MA il §WP-96 pone F3 come scelta libera fra due opzioni pari, e
non lo è: la banda ALTA (take su tutto il perimetro F2, oggetti/array inclusi)
è REFUTATA come infedele all'oracle.** Zend non consuma MAI un CV — solo
TMP/VAR — esattamente perché svuotare lo slot anticipa la morte del valore.
L'anticipo è osservabile anche SENZA `__destruct` dichiarato: riuso di
`spl_object_id`/`spl_object_hash`, scadenza anticipata di `WeakReference`/
`WeakMap`, side-effect di chiusura risorse (flock, tmpfile), `__destruct`
ereditati. Quindi anche la "via intermedia" per assenza di dtor dichiarato è
pre-refutata. **La scelta fedele: emissione su `movable_safe` (perimetro F2
intero) + guard di TIPO a runtime nel handler — si sposta SOLO `Str`
(Long/Null/Bool/Double sono già copy-free); Ref si de-referenzia; oggetti/
array/risorse restano sul percorso clone byte-identico.** Guadagno atteso =
banda MEDIA (0,84–1,21%), non ALTA: P3 va RI-derivata, non difesa. Il guard
runtime è comunque obbligatorio per il Ref (punto 1 del §WP-96): estenderlo al
tipo costa un confronto, non un branch nuovo nel percorso comune.

**Warning "Undefined variable"**: sì, preservabile, ma per contratto:
`TakeSlot` deve replicare ESATTAMENTE il ramo Undef di `LoadVar` — warning
PRIMA di produrre il valore, stesso testo, stessi modi di soppressione
(isset/coalesce), slot lasciato Undef, Null prodotto. Il warning rientra nella
VM via `set_error_handler` (mia consulenza §5): serve un phpt apposito.

## Emendamenti

- **A-DS-97-1**: F3 = TakeSlot emesso su `movable_safe`, handler con gate di
  tipo: move solo `Str`; Ref→deref+clone; ogni altro tipo→clone identico a
  oggi. Nessuna eccezione "oggetto senza dtor".
- **A-DS-97-2**: P3 per F4 ri-derivata ex-ante alla banda str
  (`guadagno_cpu_atteso_str_pct_min/_max`); un risultato in banda ALTA è
  "effetto non capito" (design95 §P3) e va nominato, non rivendicato.
- **A-DS-97-3**: contratto Undef di TakeSlot (sopra) + phpt: variabile
  indefinita in sito movibile con `set_error_handler` che rientra.
- **A-DS-97-4**: `renounce()` copre solo gli osservatori chiamati DENTRO la
  funzione; il canale re-entrante (warning→error handler→`debug_backtrace`
  con args, `getTrace()`) può osservare i PARAMETRI. O si escludono i param
  slot dal take, o si prova che phpr materializza gli args del backtrace
  indipendentemente dagli slot.
- **A-DS-97-5**: guardia di esaustività su `effect()`: il `_ => {}` tratta
  come fall-through ogni op non modellato — innocuo in misura,
  correttezza-critico in F3 se un op porta un indirizzo di salto non
  matchato. Test che enumera ogni variante di `Op` con target. Audit di
  `CallBuiltinRefCell` (in `renounce` solo per nome, mai marcato per slot).
- **A-DS-97-6** (backlog per NOME, NON F3): estensione array-di-scalari solo
  con bit "no-destructible" mantenuto su `PhpArray`, misura e gate propri.

## Kill-switch

- **KS-DS-97-1**: qualunque divergenza dtor/weakref/object-id nei gate →
  la classe di siti torna al clone; mai "fix forward" che allarga.
- **KS-DS-97-2**: guadagno F4 oltre il doppio della banda str → stop e
  nominare l'effetto prima di rivendicare.
- **KS-DS-97-3**: guardia A-DS-97-5 trova un op non matchato → si ricontano
  F1/F2 PRIMA di emettere.

## Refutazioni capitali

1. **"Perimetro F2 intero = banda ALTA" come opzione ammissibile per F3**:
   infedele (CV mai consumati in Zend; morte anticipata osservabile).
2. **"Oggetti senza `__destruct` sono sicuri"**: falso — spl_object_id,
   WeakReference, risorse.

Consulenza S-95.0: usata correttamente (§2 ordine, §3/§4 a backlog); il §5
(re-entrancy dei diagnostici) NON è ancora entrato nei predicati → A-DS-97-4.
