# Verbale sedia 6 — Pedersen (confine per-richiesta, lifecycle harness, failpath)

## VERDETTO: CON EMENDAMENTI

## Q1 — Rami di abbandono residui senza server_gone=
Due rami sfuggono alla disciplina A-PP50/51:
1. **`head_unmoved` (measure90-campaign.sh:82-85) esce con `exit 1` SENZA riga di ledger** — né abort, né VOID, né server_gone. Ledger-silenzio puro: la campagna "svanisce". Non orfanogeno (chiamato solo a server spento), ma refuta la lettera "failpath ledgerati su OGNI ramo".
2. **`assert_server_gone` (r.238-245): sul sopravvissuto a TERM chiama `fail` senza escalation KILL e senza server_gone=** — l'unico ramo che lascia un php-server VIVO oltre la fine campagna. Il preflight successivo lo intercetta, ma la riga VOID non testimonia lo stato del processo.
Minore: sul cammino `subshell_failpath` l'ordine righe è VOID→failpath (inverso di `teardown_fail`); e i failpath non fanno `wait "$DPID"` → il .log di `/usr/bin/time` può essere incompleto alla riga di ledger.

## Q2 — Quiescenza al momento di /__census_pwork
**La dichiarazione KS-MS-91-2 come scritta è FALSA nella lettera.** Il curl sequenziale garantisce zero risposte outstanding, non zero *request_end* outstanding: per la Binding Rule (output capture BEFORE request_end) il worker spedisce il body e POI esegue il teardown — curl ritorna mentre il worker può essere ancora in request_end. Keep-alive non c'entra (curl = processo nuovo per richiesta); il rischio è la coda di teardown. Con commit MONOTONO il picco è salvo; la **Δ pboot→pwork può assorbire teardown della richiesta n-esima** → attribuzione b_work formalmente non provata. Il contatore `census::OUTSTANDING` esiste già (worker_pool.rs:630): il testimone costa una riga.

## Q3 — Dove si rompe nreq/W+1 round-robin
Verificato: dispatch = `fetch_add % senders.len()` (worker_pool.rs:610-614), strict RR; `/__reqns` (wait_up + pid-echo) e `/__census_p*` sono ROUTER-side (main.rs:211,245) — non consumano slot; solo hello (100·W, divisibile per W) e le W self. La distinzione delle W self regge a QUALSIASI fase del contatore sotto RR stretto. Si rompe con: (a) **traffico estraneo su 127.0.0.1:8297** — nessuna esclusività client asserita, un client locale qualsiasi ruba uno slot in mezzo al blocco self, non testimoniato; (b) curl che fallisce il connect (slot non consumato) — coperto da ok==nreq solo per gli hello, non per interposizioni. Testimone economico: pin del contatore REQUESTS == nreq prima del blocco self.

## Q4 — Copertura pid-echo + scoping bash
`assert_http_pid` copre UN /__reqns per fase; **nessuna request census è pid-asserita** e i curl census sono senza `-f`: un HTTP 4xx/5xx passa con curl exit 0 — successo vacuo a livello harness, righe mancanti scoperte solo dal giudice. Scoping: `local SRV BOOT_EPOCH` + `identity_row` è CORRETTO (dynamic scoping, chiamata dentro run_*; `set -u` protegge usi esterni). Fragilità residua: `ps -o lstart` / `date -j -f "%a %b"` senza `LC_ALL=C` pinnato. **Ledger**: phase=start pinna judge_sha=1b1e9e96; g1 FAIL con quel giudice; g2 PASS con judge_sha=97a8eff0 e `supersede_of=g1` — **nessuna riga di ledger autorizza la sostituzione del giudice** (catena gG fuori banda).

## Emendamenti
- **A-PP-60** "assert_server_gone con escalation": passa per ensure_gone e ledgera server_gone= prima di fail.
- **A-PP-61** "head_unmoved ledgerato": instradare su fail; vietato l'exit muto.
- **A-PP-62** "census curl -f + pid-echo": `-f` su tutti i census; header x-phpr-pid asserito almeno su pwork e self.
- **A-PP-63** "testimone di quiescenza": la riga pwork porta outstanding=0 da census::OUTSTANDING; ≠0 ⇒ Δ fase ADVISORY.
- **A-PP-64** "LC_ALL=C" nel prologo campagna.
- **A-PP-65** "supersessione giudice in ledger": riga consume/authorize con delibera gG per ogni cambio judge_sha post-start.

## Kill-switch
- **KS-PP-92-1**: Δ di fase (b_boot/b_work) senza testimone outstanding==0 ⇒ mai verdict-grade.
- **KS-PP-92-2**: verdict row con judge_sha ≠ pinned a phase=start senza riga di autorizzazione ⇒ generazione VOID.

## Refutazioni capitali
Una: la garanzia di quiescenza KS-MS-91-2 come dichiarata (riga 291-293 del harness) non è vera al livello allocatore — HTTP-complete ≠ request_end-complete; l'attribuzione b_work resta ADVISORY finché manca il testimone OUTSTANDING==0.
