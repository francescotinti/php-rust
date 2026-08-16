# Revisione S-148 — lente PROCESSO (revisore singolo, REGOLE §7)

## (a) Attacco principale
**I due «kill per conteggi» sono pronunciati a un perimetro più STRETTO del bersaglio nominato dal concilio.** Il tag `frame` copre solo `Frame::with_buffers` (vm/mod.rs:2717); FrameExt, dyn_vars, iters, ret_cell cadono in `none`. `arrgrow` copre 4 metodi di array.rs; i grow da reserve interni, clone, unset cadono in `none` (94,6M). Quindi 0,12×/0,23× sono LIMITI INFERIORI della causa «pool-Frame»/«growth-hashbrown»; il verdetto (unica fonte citabile, p.3) pronuncia «MORTI, zero codice» senza qualificatore, e NEXT_SESSION lo scolpisce come veto assoluto. Nel merito il kill probabilmente regge (per superare 24,8M la quota nascosta dovrebbe essere ~7× frame.n=2,99M), ma quel tetto NON è derivato in nessun documento: manca il ponte perimetro-tag→bersaglio-causa.

Secondo attacco: **drift di etichetta su `none`**. Il criterio pre-registrato (p.1) dichiara che `none` contiene ANCHE nativi host.rs/host_reflect, compile/eval, iteratori, Stringify, props-map — e l'anatomia vi colloca gli args-Vec — ma verdetto (p.2/p.4), session file e NEXT_SESSION lo etichettano «none/VM-inline». L'etichetta contraddice la definizione pre-registrata e può orientare male la sonda S-149; «69,56% di TUTTE le alloc nei builtin» non porta il caveat nativi-esclusi (direzione favorevole alla testa, ma il confine va detto).

A favore: pre-registrazione VERIFICATA su git (criteri 5f37a8f 19:02 → parser+golden d59b2b5 19:08 → raw mtime 19:16–19:18 → verdetto 19:19); identità Σtag==galloc esatta; copy-gate con manifesti; cap rispettati (NEXT 80/80, session 40/40); caveat hist dichiarato. t2: esclusione leg1 (202%) e sotto-campionamento deriva (N=5<6) applicano regole pre-registrate — claim secondario REGGE. Nota: leve spedite 0 per la SECONDA sessione consecutiva, dichiarato ma il ritmo-regola si consuma in censimenti.

## (b) Azioni per S-149
1. Rettificare in rotazione: i kill valgono «al perimetro dei tag s148 (with_buffers; 4 metodi array)»; derivare il tetto della quota residua in `none` prima di usare il veto contro leve frame-lifecycle o grow-altrove.
2. Rinominare `none` = «residuo (VM-inline + nativi host.rs + compile/eval + iter)» in NEXT/PERF_MAP; la tranche-4 per-NOME includa i cammini host.rs/host_reflect o ridichiari l'esclusione.
3. t3 puntando a 6/6 pulite: indagare la causa ictx di leg1 (anomala due volte) PRIMA del lancio, altrimenti il test deriva resta strutturalmente sotto-campionato.
4. S-149 contenga una leva A/B vera (regola ritmo): la sonda-prezzo non è leva; se slitta, dichiararlo.

## (c) Verdetto
**REGGE CON RETTIFICA** — ranking e testa solidi (pre-registrazione e identità verificate), ma i kill sono pronunciati oltre il loro perimetro misurato e l'etichetta «VM-inline» contraddice la definizione pre-registrata.
