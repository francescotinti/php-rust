# Team engine (Hoare · Matsakis · Stogov) — nota di riconciliazione WP-97

Verdetti: 3× CON EMENDAMENTI. F1+F2 in sola misura reggono per tutte le sedie; nessuna benedice il §WP-96 così com'è.

## Convergenze

1. **Arco eccezionale non sound**: la def sottratta anche sul contributo exc rende `movable_safe` una base di emissione invalida — A-TH-97-1 (con controesempio capitale e KS-TH-97-2/3); Stogov KS-DS-97-3 impone il riconteggio F1/F2 prima di emettere.
2. **Esaustività di `effect()`/`renounce()`**: il `_ => {}` è un buco a futura memoria; audit di `CallBuiltinRefCell` chiesto da tutte e tre — A-TH-97-2, A-DS-97-5, A-MS-97-5(a); più il gap `debug_zval_refcount` (A-MS-97-5b) e il canale re-entrante backtrace/args (A-DS-97-4).
3. **Guard runtime a WHITELIST, prima spedizione solo `Str`**: A-MS-97-2 e A-DS-97-1 coincidono (move solo Str; Ref→deref+clone; resto→clone identico). Refutazioni convergenti: banda ALTA infedele (Stogov cap. 1-2: CV mai consumati in Zend; "oggetto senza dtor" non è sicuro — spl_object_id, WeakReference, risorse; idem Hoare A-TH-97-3).
4. **P3 va RI-derivata, non difesa**: il numeratore safe è un maggiorante (refutazione capitale Matsakis) — A-MS-97-1 (`WOULD_TAKE_SAFE_REF`, banda su safe−ref), A-DS-97-2 (banda str), KS-TH-97-3.
5. **Destino del valore preso**: ogni consumatore di un valore mosso deve passare da `gc_note` o i cicli perdono radici — A-TH-97-4 ≡ A-MS-97-3.
6. **Emissione compile-time, mai cache address-keyed**: A-TH-97-5 ≡ A-MS-97-4 (riuso correlato con mimalloc; invariante «Func mai mosso, ops mai riscritto»).
7. **Test-trappola per NOME**: controesempio Hoare (try/catch su builtin-out), KS-MS-97-1 (`use(&$x)`, ordine dtor, `compact`), A-DS-97-3 (phpt Undef con `set_error_handler` rientrante).

## Conflitti

- **Destino del perimetro F2 intero**: Matsakis (A-MS-97-2) lo ammette DOPO, con corpus distruttori-ordine dedicato; Stogov lo pre-refuta per sempre come F3 (array-di-scalari = leva separata per NOME, A-DS-97-6); Hoare non decide ma esige il confronto banda MEDIA vs piano B per §P1 (A-TH-97-3), non l'adozione silenziosa.
- **Sufficienza del guard**: per Stogov il gate di tipo nel handler È la scelta fedele; per Hoare il guard è «necessario ma non sufficiente» (WeakReference, flush risorse, timing GC restano da coprire lato analisi).

## Priorità proposte per WP-96

1. Fix A-TH-97-1 + riconteggio F1/F2 (verifica KS-TH-97-3).
2. Match esaustivi + audit `CallBuiltinRefCell` + guardia op-target (A-TH-97-2/A-DS-97-5/A-MS-97-5).
3. Rimisura `WOULD_TAKE_SAFE_REF` e ri-derivazione P3 su banda str/safe−ref (A-MS-97-1/A-DS-97-2).
4. TakeSlot compile-time su movable_safe con whitelist Str + contratto Undef + disciplina gc_note (A-DS-97-1/A-MS-97-2/A-DS-97-3/A-TH-97-4/A-MS-97-3/A-TH-97-5).
5. Test-trappola per NOME (punto 7 sopra) + chiusura gap renounce (A-DS-97-4/A-MS-97-5b).
6. Confronto banda MEDIA vs piano B per §P1 prima di rivendicare (A-TH-97-3).
