# TEAM-GOVERNO (Klabnik · Stogov) — riconciliazione S-189 fase 2
Generato con assistenza AI (Claude Fable 5). Fonti vincolanti: verbali/{klabnik,stogov}.md.
Relatore: letti SOLO dossier, i due verbali, REGOLE.md, NEXT 1-3; `next-budget.sh` read-only: 29/80 righe · 7.853/8.000 car. · riga 1 = 600/600 (PIENA).

## CONVERGENZE (2/2)
1. **Q1 GO su (A)** coda condivisa FIFO; (B) refutata (flag «libero» fuori lock, cieca sotto saturazione); (C) refutata (difetto di produzione, batteria 61 non lo copre).
2. **Knob `rr`: NO** (due vie al cap / due SAPI nello stesso binario). 3. **Invariante (Q2 i)**: Σcount == nreq, thr-set ⊆ {0..W-1}; W=1 unico modo di census.
4. **Timeout nel `rx.await`: NO** (invenzione, Stogov R9; infedeltà, Klabnik Q3); §3.38 è la sola cura fedele allo stallo. 5. **Nessun 503/504 mai**; WP-77.5 SUPERATA per nome.
6. **W=1 stalla per fedeltà**, provato in ambo i bracci. 7. **Restart del server a OGNI ripetizione** (Klabnik R1a `next_worker` da 0; Stogov R2b fantasma). 8. Criterio PRIMA del codice (REGOLE §3), gate per commit, lettere emendate per nome, §3.39 a catalogo NON bloccante.

## Q1 / Q2 / Q3 — verdetto del team
- **Q1: GO su (A)** con gli emendamenti di forma di questo verbale. Nessun dissenso.
- **Q2: (i) con (iii) per `__census_self`.** A W>1 la mappa è **NON VERDICT-GRADE per nome, fail-closed** (formula Klabnik; «advisory» di Stogov assorbita:
  un run a w≠1 emette `FAIL VDISP` e non entra in nessuna cifra). `__census_self` a W>1 risponde in-band `census-self VOID w=<W>`; **la barriera NON si
  costruisce in S-189** (Klabnik R9: apparato senza committente, timebox §1) ed entra in «Aperture per NOME» con le condizioni di Stogov R7 già scolpite:
  solo sotto `#[cfg(feature = "mem-census")]`, attesa a scadenza o verifica `OUTSTANDING == W` prima del wait, violazione ⇒ abort «instrument, not worker».
- **Q3: il dente basta come ACCETTAZIONE semantica SOLO con R1-R4 Klabnik + R1/R6 Stogov; NON basta come guardia nel tempo** ⇒ dente unitario in CI.
  Throughput: guardia SOLO-REGRESSIONE sul solo braccio AX, 0,97×, spread ≤5 %, R=3, C∈{4,8} (Klabnik R7; Stogov non si oppone); «A/P = 1,00 per
  costruzione» refutato; CPU/req non si rimisura.

## FEDELTÀ (ruling)
- **FIFO condivisa = semantica dell'oracle** (backlog del kernel + `accept` da ogni processo libero), non ottimizzazione: un processo che esegue non accetta.
- **Il master di `php -S` accetta anch'esso** (Stogov R1) ⇒ `PHP_CLI_SERVER_WORKERS=N` = **N+1 acceptor**. Braccio oracle «3 acceptor» del dente:
  **`PHP_CLI_SERVER_WORKERS=2`** (eguaglia il W=3 di phpr, niente asterischi); braccio «1 processo» = senza variabile. Dossier §3 e §3.9 (h): «N figli + master».
- **`Connection: close` su ogni risposta cli-server** ⇒ coda di connessioni ≡ coda di richieste; WordPress apre un handle curl per loopback: equivalente,
  residuo §3.9 (b) già a catalogo. **Esecuzione fantasma** (client interno scaduto, richiesta eseguita lo stesso): identica oracle/phpr, POST inclusi.
- **§3.38 ↔ §3.9 (h)**: sotto `php -S` vige `max_execution_time=30` e su macOS arm64 Zend arma `ITIMER_REAL` (wall): l'oracle tronca la richiesta esterna
  dell'editor temi (loopback 100 s) a 30 s di parete con banner 200; phpr aspetta 100 s e reverte. Le due voci si LEGANO per nome; nota a §3.38
  «macOS arm64: wall; Linux: CPU — busy33 neutra, un loopback NO». Il dente (3 s + 0,6 s) resta < 30 s.
- **§3.39 (nuova, 🟡, NON blocca la promozione)**: «wrapper `http://`: nessun timeout; PHP onora `timeout` del contesto o `default_socket_timeout`=60 s su
  connect+read; `stream_context_create(['http'=>['timeout'=>n]])` ignorato in silenzio (almeno `TODO(port)`). Effetto: un loopback via stream stalla senza
  fine a W=1; in WordPress i loopback passano da `Requests`→curl con `CURLOPT_TIMEOUT` ⇒ **curl NON coinvolto**». Il gate usa curl ⇒ fiducia nel gate
  indipendente da §3.39. Cura fuori perimetro S-189 (leggere `timeout`, in difetto l'ini → `timeout_global`; `ini_get` deve riportare 60).
- **Handler timeout = invenzione; 503 di coda piena = invenzione.** Testo §3.9 (l) CORRETTO (la motivazione «backlog unbounded» del dossier e di Klabnik R8 è
  falsa, Stogov R8): «coda interna unbounded + backlog di hyper dove il kernel di `php -S` limita a somaxconn (macOS 128) e scarta il SYN: divergenza di
  SOVRACCARICO, non di semantica; dichiarata, non curata; mai 503».

## LA FORMA
**Sequenza commit (Klabnik R11, emendata Stogov), ogni commit col suo gate, build mai red:**
- **C0 docs** (`.rs` zero): `wp189-harness/s189-criterio-loopback.md` ≤10 righe (segno, soglie, R=3, giudice = lo script, contratto `ms=N` = sola curl,
  esclusa la `usleep`), criterio guardia AX, §3.9 (h) riscritta + (l) nuova + §3.39 nuova + nota §3.38, «N figli + master». Gate: `next-budget.sh`.
- **C1 fixture + `gate-loopback.sh`**: `loop.php` (`d` in ms moltiplicato, porta in query string, curl `CURLOPT_TIMEOUT 3`, stampa `OK ms=`/`ERR errno=28 ms=`)
  + `pong.php`. Gate: bracci old (pin `97b58fd225ca7289` per hash, ROSSO GIUSTO) · oracle `WORKERS=2` (OK) · oracle 1 processo (ERR28 E risposta esterna 200)
  · new = tree PRIMA del cablaggio (ROSSO: il dente è rosso prima). `.out` con hash e git rev fuori repo; script e fixture in git.
- **C2 `dispatch.rs`** + test della coda (N prod/M cons ogni id una volta; drena-poi-esce; push dopo close restituisce il task; close sveglia il consumer).
  Gate: `cargo test -p php-server` ×3 feature, `loc_dente` (dispatch.rs sotto `CAP_NUOVI`), corsie `-D warnings`.
- **C3 cablaggio** `worker_pool.rs` (righe 543-648 INTATTE; commento ghost A-PP37 CANCELLATO, non aggiornato) + `__census_self` fail-closed + dente unitario
  (prova del rosso sul tree C2 nel `.out`). Gate: C2 + `gate-lever-pins.sh` (A-PP23 + riga di ASSENZA `next_worker`) + `gate-capture-order.sh` (dispatch.rs
  scansionato, atteso 0) + `gate-doc-purge.sh` + `gate-loopback.sh` VERDE 3/3 + `run-gate.sh` G-APERTURA-2 (static-counter stampa 1) + `battery61-axum` 6/6.
- **C4 lettere/parser**: riga `[ "$W" = 1 ] || FAIL VDISP non verdict-grade (S-189)` in `verdict86/87/88/89/90.sh` e `va-census-template.sh` (copia dichiarata);
  testi R12 Klabnik per A-PP36, KS-PP-87-3, A-PP37, A-PP42/KS-PP-89-2; nota a piè WP-77.5 col testo (l) corretto; `GIUDIZIO_C_AXUM` emendato (coda
  condivisa, W=1 stalla, nessun timeout); `COUNCIL_WP189_REVIEWS.md`. Riga 1 NEXT SOLO con firma.
- **C5 `scripts/pin-server.sh` → s189** (registro `PIN_REGISTRY` s189 + harness), guardia AX ⇒ KS-SK-189-6; `PIN_ATTESO` dei harness S-188 per nome.
  **PERF_MAP NON si tocca**: la guardia AX è solo-regressione, non record; un record vuole criterio proprio e `.out` d'igiene (REGOLE §3).
- **Rollback**: `git revert` di C3 E C2 insieme (C2 solo = `dispatch.rs` morto ⇒ `-D warnings` rosso); prova = `gate-loopback.sh` di nuovo rosso sul tree.
**Gate `gate-loopback.sh` — condizioni di accettazione (tutte, fail-closed):** restart del server per ripetizione · UN probe di prontezza su path statico (non
passa dal pool) · porta ≠ 8080 scelta dal kernel, passata in query string · docroot = sola cartella delle due fixture · richieste nel `server.log` == 4
(2 esterne + 2 interne), altrimenti boot VOID e ripetuto (max 3) · divario di lancio A→B dichiarato (100 ms) e provato dai timestamp del log · 3/3 identiche,
qualsiasi difforme = FAIL (mai 2 su 3) · controllo positivo «rosso per la ragione giusta» = A `ERR errno=28` con ms ∈ [2500;4000] E B `OK` < 1000 sul pin
con hash == `PIN_ATTESO` («entrambe ERR» = server fermo o W=1, NON convalida) · riga PASS `old=<hash> new=<hash> git=<rev>`.
**Dente unitario `no_idle_worker_while_queue_nonempty`** (test in `worker_pool.rs`, pool fresco W=2, costruito come i test census 1801/1841): si spediscono in
ordine, senza attendere, tre task — A = fixture PHP `usleep(1500000)`, B = fixture breve, C = fixture breve — raccogliendo i tre oneshot; asserzioni:
(1) le risposte di B e C arrivano entrambe PRIMA della risposta di A; (2) C arriva entro 1.000 ms dal dispatch; (3) A arriva entro 5.000 ms (nessun hang).
Round-robin da 0: C→worker 0 dietro A ⇒ (1) e (2) falliscono = ROSSO sul tree C2; coda condivisa: worker 1 prende C appena finita B ⇒ VERDE. Gira in CI
(`cargo test -p php-server`, `ci.yml` 98/105): un ritorno al round-robin fa rosso in CI, non in un verbale.
**Aggiramento e guardia meccanica**: (a) riga 1 riscritta «per far posto» senza scarico per nome ⇒ KS-SK-189-5; (b) round-robin ripristinato «per la
misura» ⇒ dente in CI + riga di assenza `next_worker` in `gate-lever-pins.sh`; (c) timeout/503 nel handler ⇒ KS-DS-189-3/8 + riga di assenza (letterali
`503`/`timeout(` nel handler di `main.rs`, se in C0 risultano assenti oggi); (d) barriera o knob costruiti in corsa ⇒ KS-SK-189-7.

## RIGA 1 DEL NEXT (testo proposto per la firma)
**Aritmetica**: riga 1 = 600/600. Lo scarico della clausola S-168 (` · KILL A REGISTRO: S-168 … sanata qui`, 128 caratteri) lascia 472 ⇒ spazio 128.
Il testo di Klabnik R12 (247 caratteri) **NON entra** nemmeno dopo lo scarico: il team lo accorcia; lettere emendate e barriera APERTA vanno nel corpo del
NEXT (file a 7.853/8.000: il corpo si riscrive a saldo in chiudi-sessione). **Scarico**: SÌ, deliberato per nome da QUESTO concilio (REGOLE §8: un kill resta
«fino al concilio seguente»; S-186 bis e S-189 sono concili seguenti a S-168): la clausola S-168 migra in `gaps/VETI.md` con riga datata 2026-11-03
«scaricata dalla riga 1 per delibera S-189». **ROTTA B MORTA resta intatta e verbatim** (01/11, <3 sessioni, «nessun altro riesame»): comprimerla sarebbe la
riscrittura che §8 punisce. Riga 1 proposta (**598 caratteri**, verificata con Python; `next-budget.sh` resta PASS, file 7.851):
```
# NEXT_SESSION — phpr: ORIZZONTE parità (≥1×) con l'oracle · TAPPA D (delibera utente S-186, 2026-10-22, verifica al concilio ~S-192): WP ≤1,5 (coppia notturna, mediana) · ORM ≤5 (coppia E5, al netto del pavimento sqlite3 comune) · micro ≤3× 6/6 · **ROTTA B MORTA (S-186 bis, 01/11, rc=4): mutante MUTO anche vs gemello Z (ΔM +2,18 s IN finestra [1,66;2,25]; builtin 67 % < 75 %) ⇒ strumento non provato, kill definitivo (criterio p.9 + firma (b): nessun altro riesame)** · **delibera S-189, 2026-11-03: `--axum` a CODA CONDIVISA (= accept `php -S`); W=1 stalla; gate-loopback rosso sul pin s188**.
```
Marcatore «delibera S-189, 2026-11-03» presente. Un kill S-189 scattato entra in riga 1 (§8) SOLO scaricando per nome un'altra clausola con firma.

## ORDINE OBBLIGATO DEI PASSI
(0) Firma dell'utente su `COUNCIL_WP189_REVIEWS.md` + riga 1 + scarico S-168 → (1) C0 criterio/docs → (2) C1 gate rosso-giusto sul pin s188, oracle
bilaterale (`WORKERS=2` / 1 processo), new = tree ROSSO → (3) C2 `dispatch.rs` + test coda → (4) C3 cablaggio + dente unitario (rosso su C2 provato) + tutti
i gate VERDI → (5) C4 lettere/parser/`GIUDIZIO_C_AXUM`/sintesi → (6) riga 1 NEXT + `gaps/VETI.md` → (7) C5 `pin-server.sh` s189 + `PIN_REGISTRY` + guardia AX
→ (8) promozione = pin s189 registrato e `PIN_ATTESO` aggiornati; **merge a `main` resta decisione separata dell'utente**. Apparato oltre ½ sessione a
qualunque passo ⇒ incidente §1: si chiude al commit verde raggiunto.
**Se un kill scatta, cosa resta in branch**: KS-SK-189-1 ⇒ fermi a C1, nessun `.rs` scritto, fixture da rifare; KS-SK-189-6 ⇒ C2-C4 restano in `s186-firma`
NON promossi, pin e `PIN_ATTESO` restano s188, §3.9 (h) «cura in branch, non promossa», kill in riga 1; KS-SK-189-8 (HALT) ⇒ revert C3+C2 insieme,
gate-loopback di nuovo rosso come prova, docs C0/C1 restano (il criterio sopravvive al codice).

## DISSENSI A REGISTRO
- **Stogov — barriera `__census_self`**: accettata con R7; il team la rinvia per nome (Klabnik R9). Stogov non si oppone; R7 resta il criterio ereditato.
- **Klabnik — R8 «unbounded come il backlog del kernel»**: corretta da Stogov R8 (somaxconn); il team adotta il testo di Stogov, rettifica a nome di Klabnik.
- **Stogov «advisory» vs Klabnik «fail-closed»** (Q2): assorbito, registrato per trasparenza (R13). **Klabnik R12 riga 1 (247 car.)**: refutata dall'aritmetica.

## KILL-SWITCH DEL TEAM
| condizione (meccanica) | atto | sedia |
|---|---|---|
| controllo positivo sul pin con hash == `PIN_ATTESO` NON «A ERR28 ∈ [2500;4000] ∧ B OK < 1000» in 3/3, o hash ≠ atteso | gate VOID, nessuna promozione né merge | Klabnik (KS-SK-189-1) |
| braccio W=1 (phpr o oracle) NON stalla, o risposta esterna oracle ≠ 200; oracle `WORKERS=3` presentato come «3 worker» | HALT / verdetto oracle VOID | Stogov (KS-DS-189-1/2) |
| dente unitario assente da C3 o senza prova del rosso sul tree C2 nel `.out` | C3 non si committa; se in `main`: incidente | Klabnik (KS-SK-189-2) |
| ripetizione difforme, boot con richieste ≠ 4 promosso, o ripetizione senza restart | FAIL del gate / prova VOID, mai «2 su 3» | Klabnik (189-3) · Stogov (189-5) |
| diff a `main.rs` 420-440 con timeout/`select` sul `rx.await`, o 503/504 di coda/attesa | REJECT d'ufficio | Stogov (KS-DS-189-3/8) |
| static-counter G-APERTURA-2 stampa ≠1 dopo la cura | FATAL, nessuna promozione | Stogov (KS-DS-189-4) |
| edit a verdict/test census oltre la riga W>1, o ghost A-PP37 «aggiornato» invece che cancellato, senza voce | incidente (dossier KS-189-3) | Klabnik (KS-SK-189-4) |
| riga 1 NEXT riscritta per far posto senza scarico deliberato, o senza «delibera S-189» a fine sessione | incidente KS-K186-1 | Klabnik (KS-SK-189-5) |
| mediana AX C=4 o C=8 < 0,97× t1 su una URL, o spread > 5 % | cura NON promossa (resta in branch), pin s188 | Klabnik (KS-SK-189-6) |
| barriera `__census_self` o knob `rr` costruiti in S-189; barriera futura senza scadenza o fuori `mem-census` | incidente §1, revert prima del pin / REJECT | Klabnik (189-7) · Stogov (189-6) |
| §3.39 senza «curl non coinvolto», o sua cura a prerequisito del gate; `battery61-axum` < 6/6, G-APERTURA-2 o lever-pins/capture-order FAIL dopo C3 | emendare prima della firma / HALT | Stogov (189-7) · Klabnik (189-8) |
