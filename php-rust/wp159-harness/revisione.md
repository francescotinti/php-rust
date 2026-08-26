# Revisione S-159 — REVISORE SINGOLO, lente SEMANTICA

## VERDETTO: REGGE CON RILIEVI
Claim 1 (L-AM1) e claim 2 (sonda) reggono. Claim 3 contiene una classificazione che contraddice il criterio pre-registrato.

## Rilievi
1. **ORM attesa-RF2: «COMPATIBILE» viola il criterio p.4.** Δ_norm=[−0,77;−0,16] è A CAVALLO di −0,293 (Δ_max=−0,16 non sotto −RES, Δ_min=−0,77 sì): la scaletta pre-registrata impone **NON RISOLTA**, sul lato peggiorativo. Lo script giudica dal solo estremo destro. Dichiararlo nel commit NON basta: il verdetto a registro è la parola sbagliata della scala che la sessione stessa ha pre-registrato. Serve emenda.
2. **Coeff 12,0 «ns/alloc-sito» è un nome improprio**: il census conta PASSAGGI-dispatcher (hostcall per-nome), non allocazioni; il coeff = D/2 impacchetta alloc+free+match+deviazione hostcall. Come UB falsificabile ha funzionato (D=+11,0 dentro 12,0±2,5, una conferma indipendente), ma a PERF_MAP va etichettato «per passaggio-dispatcher».
3. **fx-am dichiara «tutte le forme» ma il perimetro fast ha buchi nominabili**: nessuna closure-generatore (esclusa da `!is_generator` ma mai provata bilaterale), nessuna closure 1-param CON hint (`fn(int $x)` → simple_call=false, instradamento non testato), nessun func_get_args/num_args dentro la closure fast, nessuna static closure/bind. La forma by-ref è stata RIMOSSA dal gate (divergenza pre-esistente dichiarata): proprio il caso che proverebbe l'esclusione.
4. **phpr1 ictx segnalata per la 3ª finestra consecutiva**: la coppia ORM continua a produrre RIF contesi; rimisurare senza diagnosi non converge.

## Attacchi respinti
- Capture in slot 0: entrambi i cammini copiano captures POI i param-slot (mod.rs 11414-11421 vs 11452-11458) — ordine identico, esito identico.
- Hoist tra elementi: corpo closure immutabile per (module_id, fn_idx); push_closure_frame_one ri-risolve comunque per-elemento; ridefinizioni/autoload non toccano una closure già catturata.
- argc/func_get_args/extra_args: entrambi i cammini passano ESATTAMENTE 1 arg (braccio WP-37 di bind_params ≡ intake `argc=1; slots[0]=decay_arg`), extra_args vuoto identico.
- ArgPlace: i valori vengono da deref_clone di elementi array, mai ArgPlace.
- by-ref/variadic/hint/generator: esclusi dall'ammissione da simple_call (compile/func.rs:119,281).
- bound_this/scope/static_class/closure_id (static $x): righe speculari.
- Eccezione nel callback: drive_to_return identico; fixture caso 11 morde il fast path.

## Azioni
1. Emenda: riclassificare attesa-RF2 a NON RISOLTA (lato peggiorativo) in registro/roadmap; correggere la scaletta perché giudichi su ENTRAMBI gli estremi dell'intervallo.
2. Estendere fx-am: closure-generatore, closure 1-param con hint, func_get_args nella closure fast, static closure — bilaterale al prossimo gate.
3. In PERF_MAP etichettare il coeff 12,0 «per passaggio-dispatcher (alloc+free+match)»; prima di riusarlo come UB, esigere che le componenti non prezzate restino dichiarate per nome.
4. Prima della prossima coppia ORM: indagine dedicata sulla gamba phpr1 ictx (causa, non rimisura).
5. La conferma post-pin m-arrmap D=+8,0 (attesa +11,0, rumore 3,0) è al bordo: alla prossima finestra su pin s159 ri-leggere il D con drift-tree quantificato.
