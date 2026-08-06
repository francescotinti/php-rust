# Verbale sedia 1 — Hoare (Concilio WP-107, su S-105)

Perimetro: design linguaggio/runtime, safe-only, correttezza semantica delle forme.

## VERDETTO: CON EMENDAMENTI

La forma 2 (direct-bind, run.rs:2593) è semanticamente sana: il predicato è
identico al braccio fast di bind_params (calls.rs:324), `simple_call`
esclude hint/by-ref/variadic/generator (compile/func.rs:119), quindi ad
arità esatta nessuno slot resta Undef, argc è posto, nessuna coercizione è
saltata. Sul punto (b) CONCEDO: `decay_arg` (calls.rs:298) non muta mai —
`Ref → borrow().clone()` è un Rc-bump che PRECEDE il drop del wrapper,
nessun distruttore può scattare a metà loop e nessun codice utente corre;
l'ordine inverso è inosservabile anche per `f($a,$a)` con $a ref. La
concessione è CONDIZIONATA: regge finché D-R11 regge e decay_arg resta
libero da codice utente.

## Refutazioni

- **R-HO-107-1 (CAPITALE)**: il sigillo Copy (array.rs:99-107) è VACUO
  rispetto al proprio scopo dichiarato. `_seal::<bool>(); _seal::<i64>();
  _seal::<f64>()` verifica che tre PRIMITIVE siano Copy — tautologia
  eterna — senza legarsi ai payload delle VARIANTI. L'esempio nominato nel
  suo stesso commento («Bool → boxed») passa: `Box<_>` è 8 B ⇒ size 16 ✓,
  align 8 ✓, sigillo Copy ✓. Tre sigilli, zero morsi sul caso di progetto.
- **R-HO-107-2**: la doc A-HO-106-2 (zval.rs:258-264) scrive nel codice
  permanente «run_loop is icache-bound» come FATTO — la tesi che WP-106 ha
  declassato a ipotesi N=1 non citabile (KS-BA-106-1, NON-riproporre). Un
  doc-sigillo che cita una premessa vietata la ri-firma a ogni lettura.
- **R-HO-107-3**: Scoperta #4 del report eccede la misura: G2 conta
  l'ARITÀ al choke-point bind_params, non l'ELEGGIBILITÀ della forma 2
  (che esige simple_call — niente hint/by-ref/variadic/generator — E arità
  esatta E sito Op::Call; CallValue/CallNsFallback/metodi tengono il Vec).
  «Copre la maggioranza del carico reale» non è stabilito.
- **R-HO-107-4**: il predicato fast vive ora in TRE siti (run.rs:2593,
  calls.rs:324, calls.rs:1135) e la definizione di `simple_call` è
  duplicata testualmente (func.rs:119 e :281). Il dente esistente
  `simple_call_fast_bind_arities` (mod.rs:22459) è pre-leva e morde il
  BINDER, non il braccio inline di run.rs: un drift su una sola copia è
  un fork semantico silenzioso (es. coercizione hint saltata).

## Emendamenti

- **A-HO-107-1**: ri-sigillare via costruttori di variante:
  `const fn s<T: Copy>(_: fn(T) -> Zval) {}` poi `s(Zval::Bool);
  s(Zval::Long); s(Zval::Double);` — il tipo payload è pinnato ALLA
  variante; `Bool → boxed` ora NON compila.
- **A-HO-107-2**: riscrivere la doc zval.rs: «canale refutato PER MISURA
  (Δ negativo 5/5); icache-bound = ipotesi N=1 NON firmata, KS-BA-106-1».
- **A-HO-107-3**: dente VM-level sul braccio direct-bind: chiamate ad
  arità esatta verso funzioni hinted / by-ref / variadic / generator
  DEVONO prendere il sentiero generico (osservabili: TypeError di
  coercizione, aliasing, pack); più fixture `f($a,$a)` con ref sul fast.
- **A-HO-107-4**: unificare il predicato in un helper
  (`Func::takes_fast_bind(n)`) usato da entrambi i siti; `simple_call`
  calcolato in UN punto solo.
- **A-HO-107-5**: nominare il perimetro di «census 0,0000»: solo Op::Call;
  misurare la quota di bind ELEGGIBILI prima di estendere la leva agli
  altri siti.

## Kill-switch

- **KS-HO-107-1**: ogni allargamento di `simple_call` (o nuovo
  consumatore) senza rieseguire dente differenziale + fx21 = leva VOID.
- **KS-HO-107-2**: ogni modifica a `decay_arg` che introduca effetti
  osservabili (codice utente, mutazione) invalida la concessione (b): il
  commento d'ordine in run.rs va ri-provato o il fast path revertito.

Refutazioni capitali: SÌ (R-HO-107-1).
