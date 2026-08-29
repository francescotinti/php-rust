# Revisione S-162 — lente SEMANTICA

## VERDETTO: REGGE CON RILIEVI
Lo specchio è fedele riga per riga: resolve_fn_one (calls.rs:684-702) replica l'ordine di invoke_named (strip di UN backslash, find_fn_ci, poi linked_functions via LcKey — calls.rs:1099-1137); push_fn_frame_one (mod.rs:11467-11481) equivale al braccio simple-call esatto di bind_params ad arità 1 (calls.rs:363-369: argc=len, slots[i]=decay_arg); enter_callee per simple_call si limita al push in entrambe le vie (calls.rs:1273-1283). Hoist idempotente CONFERMATO nel codice: DeclareFn rifiuta nomi già callable (run.rs:3350-3356), il link eval/include è fatale su redeclare (mod.rs:6467-6485), find_fn_ci salta i condizionali (bytecode.rs:2278-2296). «::» non risolve mai in resolve_fn_one (nessun nome funzione lo contiene) → resta al ramo statico di invoke_value (calls.rs:543). Nessuna forma catturata con esito diverso trovata. I rilievi sono buchi di PRESIDIO, non di semantica.

## Rilievi
1. **Ammissione più larga delle fixture**: simple_call ignora default e return-hint (compile/func.rs:119: esclude solo hint di PARAMETRO, by-ref, variadic, generator; bytecode.rs:1520-1522). Quindi `function h($x=5)` e `function r($x): int` via stringa ENTRANO nel fast path ma NESSUNA fixture le esercita: in fx-sm f2 ha 2 parametri, f3 ha hint di parametro. Equivalenza verificata a mano, non presidiata da gate.
2. **Forme non coperte da fixture**: funzione namespaced come stringa ('ns\f', '\ns\f') e generator user-fn via stringa (esclusa da simple_call, resta sul braccio no-frame di call_callable, calls.rs:651-659). fx-am le copre solo come closure.
3. **Divergenza pre-esistente non a catalogo**: `array_map('undef', [])` — entries vuote, il loop generico non chiama mai e restituisce [] SENZA errore (host.rs, loop 1-array); l'oracle valida il callback prima e lancia TypeError. §3.26/2 documenta solo il caso array non-vuoto. Non toccata dalla leva.
4. **Census: la spiegazione regge**: le 2 alloc/elemento del cammino generico ritrovate nel sorgente — vec![v] (host.rs, loop generico) + bytes.to_vec() del nome (calls.rs:550); find_fn_ci non alloca, cb.clone() è bump di Rc. Nessuna alloc orfana.
5. Il braccio no-frame di call_fn_one è morto per costruzione (enter_callee simple_call spinge SEMPRE): specchio dichiarato, ma un ramo irraggiungibile con pop su frames[baseline-1] merita un debug_assert o unreachable!.

## Azioni
1. Estendere fx-sm: `h($x=5)`, `r($x): int`, namespaced, generator-via-stringa (S-163, prima di nuove leve sul sito).
2. Aggiungere a fx-sm-div + §3.26 il caso callback-invalida con array VUOTO.
3. Sonda dovuta sul FUORI-UB SOPRA (D=+65 vs tetto 19): decomporre il bundle prima di iscrivere il quinto coefficiente a tabella.
4. Chiudere il braccio morto di call_fn_one con unreachable! (edit da criterio, non urgente).
