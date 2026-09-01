# Verbale PEDERSEN — S-167 (bozza indipendente, disciplina di confine per-richiesta)

## VERDETTO: GO-CONDIZIONATO

## ROTTA (una riga)
Campagna strutturale SÌ, ma dispatch-first (R1): è l'unica rotta boundary-inerte; R3-arena retrocessa (premessa già refutata da L-AL3, rischio classe-RetainSet), R2 solo in fette drop-neutrali.

## REFUTAZIONI DAL MIO ANGOLO
1. **R3 (arena per-richiesta) è un pericolo di confine SENZA payoff prezzato.** L-AL3 (S-164) ha stabilito che un'alloc mimalloc su fast path ≈0 ns: il mass-free d'arena compra ~0. In cambio: (a) qualunque Zval/stringa che migri nel RetainSet persistente diventa un puntatore in arena liberata — la stessa classe di bug pagata in S-8x (leak chiuso col pin PER-RICHIESTA); (b) i __destruct PHP sono OSSERVABILI (output, ordine gc_queue FIFO+gc_birth, teardown eager http-kernel, deadlock mysqli): un mass-free li salta o li riordina, violando parità; (c) l'ordine vincolante «output capture PRIMA di request_end()» impone che i buffer d'output NON stiano nell'arena — la superficie di eccezioni cresce, i sigilli si assottigliano.
2. **R2 (Zval) rischia sul confine solo se cambia la topologia drop/refcount**: handle-id (lezione next_id: pulire TUTTE le tabelle per-id), GC, distruttori. Taglia enum / small-string inline sono boundary-neutrali; NaN-boxing resta a veto; le fette Object già prezzate sono cadute (A3c NO-GO, L-TD1 borrow ≤1 ns).
3. **R1 (dispatch) è boundary-inerte**: nessuno stato nuovo sopravvive alla richiesta, nessun riordino di Drop, nessuna tabella per-id toccata. Il residuo «dispatch 36,3» è NOMINATO; arith è fermo da ~15 sessioni ed è puro loop+op. Minima superficie, massimo bersaglio.

## EMENDAMENTI
1. R3 ammessa SOLO per dati senza Drop osservabile (frame/operandi, dove ARCO REGISTRI già opera); MAI Zval/stringhe candidabili al RetainSet; ogni fetta R3 esibisce sigillo di TIPO (stile VmGate ZST) che nulla d'arena sopravvive a request_end().
2. Ogni fetta strutturale aggiunge al gate la parità two-request byte-identica sullo stesso processo (capture prima del reset), oltre a corpus per NOME ×2.
3. Fette R2 accettate solo se drop/refcount-invarianti, con gate handle-id + ordine distruttori.
4. Ogni leva R1 porta disasm bl-count di run_loop prima/dopo (lezioni H-C2, MC1-inline).

## KILL-SWITCH
Se una fetta rompe parità two-request o corpus per NOME → revert al byte, incidente contato. Se R1 non muove arith oltre soglia in 2 sessioni consecutive → la campagna passa al vaglio R4 (ridefinizione onesta).

## PRIMA FETTA
Predecode/fusione sul cammino arith (cmp+jmp, load+op+store) con PHPR_REG_LOWER attivo. Giudice: run-micro arith R=5, soglia pre-registrata arith ≤42,1 ns/iter (−10% da 46,8, fuori quantizzazione 0,5) + prop/mc entro banda-layout; gate: corpus 1412×2 verde, batteria 1748, coppia WP al pin nuovo.
