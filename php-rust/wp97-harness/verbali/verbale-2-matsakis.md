# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — WP-97

## VERDETTO: CON EMENDAMENTI

Il dataflow F1 è sano: direzione dell'errore scelta (scritture non modellate
sottocontano; letture non modellate = elenco F2), def per-arco su
IterNext/CatchMatch corretti, `StoreGlobal` come uso-mai-def evita il
sovraconteggio giusto. F2 copre i canali statici di condivisione. Ma il
numeratore `would_take_safe_rc` è INQUINATO e la mossa F3 sposta il punto di
morte dei valori fuori dai sentieri che il GC osserva. Non benedico la banda
1,91–2,75% come predizione: è un TETTO.

## Emendamenti

**A-MS-97-1 (numeratore Ref non misurato).** Uno slot che a runtime regge
`Zval::Ref` senza op marcante nel bytecode della funzione — lato interno di
`use(&$x)`, slot del main aliasato da `global` dentro una callee — passa i
predicati statici F2 ed entra in `would_take_safe_rc` (zvalcensus.rs:111
ricorre nel Ref e conta l'interno), ma per il guard di §WP-96 punto 1 NON è
takeable (si de-referenzia = clone). La frazione è ignota. Prima della
decisione di perimetro F3: aggiungere `WOULD_TAKE_SAFE_REF` (un `matches!`)
e un run di rimisura; la banda P3 va derivata su safe−ref, non su safe.

**A-MS-97-2 (guard a WHITELIST, non blacklist).** «Ref si de-referenzia»
è una blacklist: lascia takeable oggetti E array — e un array può contenere
oggetti, quindi l'anticipo di `__destruct` non è confinato al tipo Object.
Il guard runtime sia `matches!(cell, Zval::Str(_))` per la prima spedizione
(rischio distruttori zero per costruzione, banda onesta = `_str`
0,84–1,21%, che per §P1 impone il confronto col piano B, non l'abbandono).
Il perimetro F2 intero solo dopo, con corpus distruttori-ordine dedicato.

**A-MS-97-3 (il drop deve restare sui sentieri notati).** Oggi l'ultimo
handle in uno slot muore su `StoreSlot` (→ `gc_note(&old)`, run.rs:632) o
al teardown del frame. `mem::replace(_, Undef)` sposta quella morte dentro
il consumo inline (binary_value_ab, Pop già nota, gli altri?): se il
sentiero di consumo non registra la radice, i cicli il cui ultimo root
muore lì non vengono MAI raccolti. F3 deve provare per elenco che ogni
consumatore di un valore preso passa dalla stessa disciplina `gc_note`, o
instradarlo. Ownership trasferita ≠ lifecycle osservato.

**A-MS-97-4 (chiave cache = identità, non contenuto).** La chiave
(ptr Func, ptr ops, len) assume indipendenza dei due riusi d'indirizzo, ma
i due blocchi vengono liberati INSIEME e riallocati insieme: con mimalloc e
size-class uguali (eval ripetuto, per-request nel server) il riuso è
correlato, non «coincidenza doppia». Inoltre la chiave non vede mutazioni
in place di `ops` (quickening/IC futuri). Accettabile in sola misura;
VIETATO portare questo keying in F3 — l'emissione sia compile-time, e si
documenti l'invariante «Func mai mosso, ops mai riscritto».

**A-MS-97-5 (due buchi di elenco).** (a) `CallBuiltinRefCell` in
`effect()` cade nel ramo vuoto e in `renounce()` è solo name-check: se il
cell può sopravvivere alla call, lo slot è condiviso e non rinunciato —
provare che ogni sito è preceduto da PushRef/MakeRef marcante, o marcarlo.
(b) design95 elenca `debug_zval_refcount` fra gli osservatori;
`observes_scope` ha 7 nomi e non lo include — riconciliare.

## Kill-switch

**KS-MS-97-1**: divergenza per NOME sui test-trappola (`use(&$x)` dopo il
take, ordine distruttori, `compact`) o sul corpus → si ritira l'emissione,
non si difende.
**KS-MS-97-2**: in F4 `slot_reads_avoided` sotto la predizione F2 oltre la
frazione Ref misurata (A-MS-97-1) → meccanismo non agito, banda da
ri-derivare prima di rivendicare qualunque Δ.

## Refutazioni capitali

**Una**: `guadagno_cpu_atteso_safe_pct` letto come predizione P3 è
REFUTATO — il numeratore contiene esecuzioni Ref non-takeable in quantità
non misurata; è un maggiorante, e firmare P3 su un maggiorante viola la
regola «predizione derivata da una misura». Il programma F3 resta in piedi
sugli emendamenti sopra.
