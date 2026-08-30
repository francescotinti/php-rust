# S-165 criterio L-MC1 «MethodCall.borrow k≤2» — PRE-registrato PRIMA dell'edit

1. **LEVA**: in `Op::MethodCall` con argc≤2, IC-hit col ricevitore letto IN PLACE
   (pattern ThisMethodCall) + bind diretto pila→slot (H-D forma 2, S-105):
   ammissione = recv `Zval::Object` DIRETTO in pila (non-Ref), `ic.get(cid)` hit,
   callee `simple_call && n_params==argc`. Un `ArgPlace` nel tail NON respinge:
   si materializza NEL fast path (pop grezzo→`arg_place_read` in ordine sorgente
   →`flush_diags(cur_line)`→`decay_arg`), specchio ESATTO del funnel (callee
   simple_call ⇒ maschera by-ref tutta falsa). Ogni altra forma INVARIATA
   (funnel `method_call`). Soundness IC=ThisMethodCall (WP-36): Fiber mai in
   cache (funnel devia PRIMA del fill; il fill di `dispatch_instance_call` è
   l'unico scrittore), Generator/Closure non sono `Zval::Object`.
   Nello STESSO edit: `unreachable!`×2 su `call_fn_one`/`call_method_one`
   (az.rev. S-163 #4, caduti col revert AL3).
2. **GIUDICE**: `m-mc2.php` NUOVO (gemello metodico di calls.php: `$o->f($s,1)`,
   k=2, arg1=ArgPlace su `$s` + arg2 letterale — esercita la materializzazione);
   N=20.000.000 dal SORGENTE; ns/iter=(mediana raw−floor)/N·1e9; floor=empty.php
   PER-binario (med3).
3. **SEGNO**: +B (B=edit più veloce). **SOGLIA**: max(4 ns/iter, rumore drop-1).
   Obbligo az.rev. S-164 #5: verdetto «non pagante» SOLO con rumore≤4 — se a
   R=3 il rumore drop-1 supera 4, si estende (R=5) PRIMA di ogni verdetto.
4. **SMOKE**: R=3 coppie interleaved (ordini AB/BA/AB), early-stop a segno
   opposto oltre soglia. **BANDA VINCOLANTE [4;30] ns/iter** — UB dal bundle
   CONTATO nel sorgente per l'hit ammesso: 1 alloc args-Vec (`split_off`)
   ≈0 ns (lezione AL3 s164-al3-STOP.md) + salto INTERO di
   `method_call`+`dispatch_instance_call` (scan ArgPlace su Vec, 3 shunt-match
   Generator/Fiber/Closure, borrow class_id ripetuto, chiamata `bind_params`
   generica, 2 chiamate nidificate) + `pop_keys`. Fuori banda ⇒ ARBITRATO
   census PRIMA del R=5: attesa `s144_vecargs_note` Δ = N ESATTO su m-mc2
   (unità dal sorgente census: evento = Vec con capacity>0 al bind; il path
   diretto NON passa da bind_params), `arity_note` INVARIANTE (aggiunta nel
   fast path), altri nomi Δ=0.
5. **GUARDIE SOLO-REGRESSIONE** (comparatore al3sm riusato, stesse N e soglie
   fondate): arith/prop/calls/str/arr/re + obj* + arrload/strmap/arrfilter/
   arrmap/refl/hostargs/backtrace/missload. Guardia che morde ⇒ arbitrato
   DICHIARATO prima del R=5 (emenda rev. S-161 #4), mai assoluzione ex post.
6. **GATE EDIT**: disasm bl-count `run_loop` A/B (otool, pattern TAB — fix
   S-164) con Δ DICHIARATO a verbale; dente loc: run.rs 6815→cap NUOVO
   PRE-dichiarato nello stesso commit (salita = solo il fast path), calls.rs
   resta sotto 2000 (unreachable! RIDUCE); parità braccio B = smoke 2 modi di
   `pin-phpr.sh --braccio` + fixture bilaterali del comparatore.
7. **VERDETTO**: D≥soglia in banda ⇒ R=5 (criterio R=5 da pre-registrare con
   soglia fondata sul rumore smoke); D<soglia con rumore≤4 ⇒ NON PAGANTE,
   STOP e revert al byte (p.3b AL3). Il rapporto A/B è DIREZIONE+cifra sul
   giudice proprio; nessuna ripartizione su categorie non misurate (§4).
