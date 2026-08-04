# Verbale sedia 1 — Hoare (WP-97, su S-95.0 A-ZV2 F1+F2 e programma §WP-96 F3)

Perimetro: design linguaggio/runtime Rust, safe-only. Mandato: refutare.

## VERDETTO: CON EMENDAMENTI

Le bande F1/F2 (47,11% rc, safe 90,21%, ALTA) reggono come misura: la
direzione d'errore dichiarata è rispettata quasi ovunque e il difetto che ho
trovato sposta i conteggi in modo trascurabile. NON regge la premessa del
§WP-96 «F3 emesso solo dove F2 lo consente»: `movable_safe` non è una base di
emissione sound (vedi Refutazione capitale). Il Rust è safe ovunque
(l'`unsafe` di `libc::atexit` è solo nella build di misura); bitset difensivi
(`get` fuori intervallo = falso) corretti.

## Emendamenti

**A-TH-97-1 — Archi eccezionali non devono subire il kill delle def.** In
`analyze` (liveness.rs 427-434) il contributo exc è fuso in `live_out[i]` e
poi `inb` sottrae i `defs`: su un arco eccezionale la def può non essere
avvenuta. Regola sound e prudente: se `exc_edges[i]` non è vuoto, le def di
`i` non uccidono (o si fonde `live_in[handler]` in `inb` DOPO il kill).
Ricontare F1/F2 col fix prima di scrivere l'opcode.

**A-TH-97-2 — Niente `_ => {}` in `effect()` e `renounce()`.** Il wildcard è
un buco di soundness a futura memoria: ogni variante `Op` nuova che legge uno
slot diventa silenziosamente «nessun effetto». Match esaustivo con elenco
esplicito dei no-effect; audit una-tantum delle varianti oggi nel wildcard
(es. `CallBuiltinRefCell`: è in `renounce` per `observes_scope` ma non ha
use/def in `effect()` — verificare che non porti uno slot).

**A-TH-97-3 — Il guard runtime su `Ref` è necessario ma non sufficiente, e la
banda MEDIA obbliga il confronto.** L'osservabilità della vita non è solo
`__destruct`: `WeakReference::get`, finalizzazione risorse (flush/close di
stream), timing GC. Se a inizio F3 si sceglie il nucleo stringhe (rischio
zero), quella è banda MEDIA: per la STESSA regola a tre bande di §P1 va
confrontata col piano B a parità di conto, non adottata in silenzio.

**A-TH-97-4 — Contabilità GC del valore preso.** Oggi la morte del valore
passa da `gc_note(&old)` in `StoreSlot`/`Pop`; un valore spostato che muore
nel consumatore (es. dentro `binary_value_ab`) deve comunque passare da
`gc_note`, o il cycle collector perde radici (lezione WP-72: gli Rc non
raccolgono i cicli).

**A-TH-97-5 — Emissione a compile-time e predizione ri-derivata.** F3 non
deve riusare la cache address-keyed di `zvalcensus` (collisioni da riuso
d'indirizzo accettabili solo in misura); dopo QUALUNQUE fix all'analisi la
predizione di `slot_reads_avoided` per il controllo positivo F4 va
ri-derivata dai contatori nuovi, o il controllo di Bak è vacuo.

## Kill-switch

**KS-TH-97-1**: esiste una variante `Op` che legge uno slot per indice, cade
nel `_ => {}` e non è coperta dalle rinunce F2 → conteggi F1/F2 e ogni F3 su
di essi invalidi.

**KS-TH-97-2**: `TakeSlot` emesso da `movable_safe` senza A-TH-97-1 → il
lavoro F3 è invalido (parità violabile per costruzione).

**KS-TH-97-3**: il fix A-TH-97-1 porta `safe_su_would_take_pct` sotto 60 o la
banda safe fuori dall'ALTA → P2/P3 da ri-derivare, non difendere.

## Refutazioni capitali

**Sì, una — prospettica su F3, non sulle bande.** Controesempio: `$m` vivo;
`echo $m` (fuori da ogni regione protetta); poi `try { builtin_out(..., $m) }
catch { echo $m; }` dove il builtin è abbassato a `CallHostBuiltinOut` con
`out_slot = $m` e lancia (TypeError/ValueError) PRIMA di scrivere l'out.
L'analisi: def di `$m` sottratta anche sul contributo dell'arco exc →
`live_out` dell'`echo` non contiene `$m` → `movable_safe` (load fuori
regione, slot non rinunciato). Con `TakeSlot` il catch vede `Undef` (warning
«Undefined variable»); Zend stampa il valore. Stessa classe: `StoreSlot` con
coercizione typed-ref, `CallHostBuiltinScanf`. La sessione S-95.0 resta
valida (sola misura); il programma §WP-96 è invalido finché A-TH-97-1 non è
applicato e ricontato. Aggiungere il controesempio ai test-trappola del
punto 3 del §WP-96.
