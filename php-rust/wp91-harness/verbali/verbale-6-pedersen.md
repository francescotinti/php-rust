# VERBALE — Pedersen, sedia 6, Concilio WP-91

VERDETTO: CONCORDO CON EMENDAMENTI.

## Q1 — A-PP46 failpath (measure89-campaign.sh)
I due handler sono reali: `subshell_failpath` (r.164-173) e `teardown_fail`
(r.177-188) uccidono l'albero, verificano gone best-effort 5s e ledgerano
`failpath=… server_gone=yes|no` con `attempt=$ATT phase=$label` — leggibile
per attempt. MA restano rami residui:
1. **wait_up fallito** (r.243, 280, 312): `kill_server_tree; fail` — TERM
   solo, NESSUNA verifica gone, NESSUNA riga `server_gone=`. È l'unico ramo
   di abbandono cieco rimasto: esattamente la classe che A-PP46(2) dichiara
   di chiudere.
2. **Nessuna escalation KILL**: se TERM è ignorato, tutti e tre gli handler
   ledgerano `server_gone=no`/`still alive` e ESCONO lasciando il superstite
   vivo (onesto ma incompleto; il pre-flight successivo lo paga con un VOID).
3. **Terminalità VOID invertita nel ramo subshell**: il fail() dentro la
   command-substitution scrive `verdict=VOID` PRIMA che il parent appenda la
   riga `failpath=… server_gone=`; KG-89-1 vuole la VOID terminale.
4. Censimento del report: i siti chiamanti sono 8 (196, 245, 256, 282, 287,
   314, 333, 334), non "7" come in WP_SESSION_89.md r.30. Off-by-one, non
   capitale. Doppio-fail: escluso (il fail del subshell scrive UNA sola VOID;
   il parent esce senza secondo fail).

## Q2 — A-BG51 pid-echo (main.rs r.358-380)
Il layer `map_response` avvolge il Router con `.fallback(php_handler)`:
copre TUTTE le risposte del router principale, incluso `__reqns` (r.211, è
dentro php_handler) e gli early-return d'errore. **Tier0 NO** (r.315-338,
Router separato senza layer) e l'esclusione NON è dichiarata da nessuna
parte: il commento dice "every response", la riga di sessione "ogni risposta
axum". La campagna non esercita tier0, quindi nessun assert falso in-band —
ma la deviation va nominata. Inoltre "asserts it on the FIRST request of
every phase" è letteralmente falso: i probe di `wait_up` (fino a 100 curl su
`/__reqns`, r.139-146) PRECEDONO `assert_http_pid`. Materialmente sano
(l'assert avviene prima di ogni richiesta misurata, contro il LISTEN-owner
già verificato), ma la dicitura va corretta.

## Q3 — A-PP47 due-lane (worker_pool.rs r.1443-1514)
Riqualificazione ONESTA: refutata a macchina (log armato: probe_fail×2, zero
put — F8c by design), non a parole; la lane fatale ora è PINNATA al contatore
(put==0 AND probe_fail==2), la publish osservabile vive sulla lane pulita.
Pool W=1 (r.1397) ⇒ "same worker" garantito. Lane pulita: file pid-suffisso
in temp_dir, creato SOLO se l'env è armato; nel gate il run armato è
`--test-threads=1` filtrato su a_pp38 ⇒ niente flakiness parallela. Residui:
(a) panic prima di `remove_file` lascia il fixture (benigno); (b) full-suite
parallela con PHPR_UNIT_CACHE_LOG nell'ambiente inquina il log condiviso ⇒
FAIL spurio (fail-closed, da dichiarare). Il gate arma e conta giusto: UCL
non-vuoto (dente F16b), put==1/hit==1 esterni — che mordono anche su
pubblicazione della fatale (put diventerebbe ≥2). Manca il contatore esterno
`probe_fail==2`.

## Q4 — A-PP48
NPASS==13: morde in ENTRAMBI i versi (prune⇒12, aggiunta⇒14, ignore⇒12),
ancorato alla suite unittests src/main.rs (r.32), non al MAX. Estrazione
prima del run armato che appende allo stesso LOG: ordine corretto.
I conteggi A-PP45==5/A-PP47==13 (gate-lever-pins.sh r.1217-1219, `grep -c`
su TUTTO il file): dente contro delete/aggiunta non dichiarata, **incenso
contro il gutting sostitutivo** — neutralizzare l'assert conservando la
stringa/commento con la sigla mantiene il conteggio. Mitigato dai contatori
esterni del gate-axum (put/hit/vacuità); l'unico pin SOLO in-test è
npfail==2: quello è il buco reale.

## Emendamenti
- **A-PP50**: wait_up fail-path ledgerato — nei 3 siti sostituire con
  handler che kill-tree + verifica gone + riga `failpath=wait_up
  server_gone=`; nessun ramo di abbandono senza riga server_gone.
- **A-PP51**: escalation KILL — dopo il loop 5s con gone=no, `kill -KILL`
  sull'albero + ri-verifica; ledger `server_gone=killed9|no`.
- **A-PP52**: contatore esterno fatal-lane — gate-axum-tests conta
  `main_probe_fail`==2 sull'UCL armato (chiude il gutting comment-preserving
  che A-PP48(b) non morde).
- **A-PP53**: dichiarare per NOME l'esclusione tier0 dal pid-echo (o
  aggiungere il layer) + correggere "FIRST request of every phase" in
  "prima request ASSERITA, post-wait_up, pre-workload"; dichiarare il
  vincolo "run armato = solo targeted" della lane pulita.

## Kill-switch
- **KS-PP-91-1**: nessuna campagna m90+ consumabile finché esiste un ramo di
  abbandono server senza riga `failpath=/server_gone=` (A-PP50 = gate di
  merge).
- **KS-PP-91-2**: ogni claim "teeth intact" fondato sul solo censimento
  sigle (A-PP48(b)) è ADVISORY finché il contatore esterno npfail (A-PP52)
  non è armato.

Firmato: Pedersen, sedia 6, Concilio WP-91.
