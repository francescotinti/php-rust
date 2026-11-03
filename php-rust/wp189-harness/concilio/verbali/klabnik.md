# Verbale sedia KLABNIK — S-189 (bozza indipendente, FASE 1)
Generato con assistenza AI (Claude Fable 5). Lente: chiarezza, gate, testabilità, migrazione incrementale, processo. Mandato: refutare.
Fonti lette: dossier, REGOLE (25 righe), NEXT 1-25, §3.9 (h), revisione S-188 t2, `gate-lever-pins.sh` 870-890, `gate-capture-order.sh` 190-240,
`run-gate.sh` 40-90/260-280, `verdict90.sh` 210-230, `measure90-campaign.sh` 290-305, `COUNCIL_WP186_REVIEWS.md` (FORMA, ordine obbligato),
`worker_pool.rs` 330-360/500-560/650-760/1796-1845, `main.rs` 380-404, `loc_dente.rs` 25-140, `ci.yml` 82-150, `next-budget.sh`, `pin-server.sh`.

## VERDETTI
**Q1 ROTTA: CONCORDO CON EMENDAMENTI su (A).** (B) è refutata di principio: «primo worker LIBERO» richiede un flag occupato letto fuori dal lock della
coda — fra la lettura e la send il worker si occupa, e sotto saturazione ricade nel cieco; non è la semantica dell'accept condiviso (§3). (C) è refutata
di processo: la batteria 61 NON esercita il loopback bloccante (revisione S-188 #2), quindi «6/6» certificherebbe un difetto noto di produzione —
un gate che non copre il caso che sa di non coprire è teatro (REGOLE §5). Emendamenti: R1-R4 (gate), R5 (dente unitario in CI), R9 (barriera NON ora).
**Q2 CONTRATTO: CONCORDO CON EMENDAMENTI su (i), con (iii) per `__census_self`.** Σcount==nreq e insieme ⊆ {0..W-1}; a W>1 la mappa non è «advisory»,
è NON VERDICT-GRADE per nome (fail-closed, come A-PP28 fa già per la slope). Knob `rr` (ii): MI OPPONGO (due vie al cap, riga di matrice cfg, e un
knob di misura che cambia il dispatch è esattamente il contrasto fra binari diversi che §3 vieta). Barriera: MI OPPONGO alla sua costruzione in S-189
(R9): ogni campagna census è a W=1 (KB-78-3, A-PP28); a W>1 `__census_self` deve RIFIUTARE in-band, la barriera resta apertura per NOME.
**Q3 FORMA E GATE: CONCORDO CON EMENDAMENTI.** Il dente `gate-loopback` basta come ACCETTAZIONE solo con R1-R4; NON basta come guardia nel tempo
(non è in CI: R5). Throughput: sì, SOLO-REGRESSIONE sul solo braccio AX (R7), soglia pre-registrata 0,97×, R=3, C∈{4,8}; «A/P=1,00 per costruzione»
è refutato (il dispatch cambia: Mutex+Condvar al posto di mpsc), ma CPU/req non si rimisura (costo ≪ 1e-4 del req; la guardia è il throughput a C=8).
«Bounded + 503» WP-77.5: SUPERATO per nome, con residuo nuovo §3.9 (l): phpr accetta senza limite dove `php -S` ha il backlog del kernel (R8).
Timeout nel `rx.await`: NO (fedeltà; `php -S` non ne ha; §3.38 resta aperta per nome).

## ROTTA (una riga)
GO su (A) coda condivisa, criterio PRIMA (C0), dente unitario in CI rosso→verde, gate-loopback bilaterale col controllo positivo sul pin s188 per hash,
lettere emendate per nome nello stesso arco, barriera e throughput-lettera FUORI dalla sessione se l'apparato supera ½ (REGOLE §1).

## REFUTAZIONI ED EMENDAMENTI
**R1 — La coreografia del gate assume ciò che il gate deve provare.** Il controllo positivo (dossier §4: A→k, B→k+1, int-B→k+2, int-A→k+3≡k) regge SOLO
se (a) `next_worker` parte da 0 a OGNI ripetizione ⇒ restart del server per ripetizione, obbligatorio, non «con restart» per cortesia; (b) le richieste
di prontezza (`battery61-axum.sh:47`: fino a 40 curl di attesa) CONSUMANO slot round-robin ⇒ il gate deve usare UN probe di prontezza su un path
statico (non passa dal pool: `main.rs:399-401`) o contare i probe e fasarli; (c) il divario di lancio g fra A e B deve stare in (0; d_B): con `usleep`
in MICROsecondi d=600/300 sono 0,6/0,3 ms, sotto il jitter di spawn di curl (5-20 ms) ⇒ la fixture prende `d` in ms e lo moltiplica, g dichiarato
(es. 100 ms) e provato dal timestamp di arrivo nel log del server; (d) la porta va passata in query string, non letta da `SERVER_PORT` (un'altra cosa
non pinnata). Emenda: `gate-loopback.sh` stampa per ripetizione `rep=N old|new|oracle w=W tA= tB= A=OK|ERR28 ms= B=… reqs=4` e il PASS porta
`old=<hash> new=<hash> git=<rev>` come `run-gate.sh:280`.
**R2 — «3 ripetizioni identiche» è gate solo se fail-closed e con prova d'igiene.** Qualsiasi ripetizione difforme ⇒ FAIL (mai 2 su 3); boot con
conteggio richieste nel `server.log` ≠ 4 (2 esterne + 2 interne) ⇒ boot VOID e ripetuto (max 3) come `s188-criterio-server-pair.md` p.11 (a); porta
≠ 8080 (la siteurl che la scheda browser ricarica: incidente t1) scelta libera dal kernel; docroot = sola cartella delle due fixture. Senza queste tre
righe le «3 ripetizioni» sono teatro: un run verde può esserlo per un probe estraneo che ha spostato il contatore.
**R3 — Controllo positivo sul pin vecchio: SÌ, nello STESSO script, per hash, e rosso PER LA RAGIONE GIUSTA.** `PIN_ATTESO=97b58fd225ca7289` come
`s188-wp-server-pair.sh:38`; hash ≠ atteso ⇒ VOID (mai skip). Rosso giusto = A `ERR errno=28` con ms ∈ [2500;4000] E B `OK` < 1000 ms: «entrambe ERR»
è un server fermo o W=1 e NON convalida il dente. Il controllo convalida anche la fixture (che `curl_errno` di phpr renda 28 sul timeout di ureq non è
provato altrove). Bracci oracle (`php -S` W=3 OK / W=1 ERR28) = fixture bilaterale (§5), obbligatori nello stesso `.out`.
**R4 — Il dente deve essere ROSSO PRIMA sul tree, non solo sul pin.** Ordine: C1 esegue il gate col braccio «new» = build dall'artefatto cargo (JSON,
`run-gate.sh:58-80`) del tree PRIMA del cablaggio ⇒ FAIL registrato con git rev; C3 lo ripete ⇒ PASS. Il verdetto sta fuori repo, ma script, fixture
e le due righe PASS/FAIL con hash entrano nel `.out` citato nel verbale (GATE AUDITABILE §5: `.out` da solo = VOID, qui è script+hash+rev).
**R5 — Il gate shell NON è in CI: serve il dente unitario che morde a ogni commit.** `ci.yml:98/105` eseguono `cargo test -p php-server` (axum-server,
census); `gate-loopback.sh` (php oracle, pin vecchio, porte) non ci entrerà mai. Dente in `worker_pool.rs` tests (W=2, pool fresco): dispatch A =
`usleep(1500000)`, B = breve, C = breve; attesa: la risposta di C arriva PRIMA di quella di A. Round-robin da 0: C→worker 0 dietro A (≥1,5 s) ⇒ ROSSO;
coda condivisa: worker 1 prende C appena finita B ⇒ VERDE. Nome `no_idle_worker_while_queue_nonempty`; prova del rosso = un run sul tree C2 nel `.out`.
Questo è il guardiano meccanico contro l'«aggiramento» (R10): un ritorno al round-robin fa rosso in CI, non in un verbale.
**R6 — Gate e cap toccati, dichiarati per percorso.** `loc_dente.rs:109` worker_pool.rs cap 2074: saldo ≈ −4 ⇒ invariato (slack 200 non scatta);
`dispatch.rs` nuovo sotto `CAP_NUOVI` 2000; il pavimento «≥1874» del criterio S-188 p.4 resta vero. `gate-lever-pins.sh:878-885` (A-PP23: `execute_request
(&reg, &task.meta)` prima di `response_tx.send(`) INVARIATO ma RIESEGUITO nel `.out` di C3; aggiungere una riga di ASSENZA: `next_worker` deve essere
assente da worker_pool.rs (forma di «nessun `vm_new(` in axum_web.rs»). `gate-capture-order.sh:196-207` `POSTEND_PIN_WORKER=2` invariato; `dispatch.rs`
è scansionato (ricorsivo) con atteso 0. `verdict90.sh:216-228`: NON riscrivere la logica di mappa — una riga prima: `[ "$W" = 1 ] || { emit "FAIL VDISP:
w=$W>1 non verdict-grade (S-189)"; … }`; stessa riga nei lettori `verdict86.sh:66`, `verdict87.sh:69`, `verdict88.sh:109`, `verdict89.sh:163`,
`va-census-template.sh:17` (copia dichiarata, elenco nel verbale). `measure90-campaign.sh:296-303` invariato (W=1). Test census 1801/1841/1911/1919/2029:
INVARIATI (API pubblica, W=1; verificato: nessun test tocca `senders`/`next_worker`). Il commento ghost A-PP37 (`worker_pool.rs:724-729`) si CANCELLA,
non si aggiorna: non esiste più il contatore.
**R7 — Throughput: guardia, non misura.** Solo braccio AX (O4 non è cambiato: rilanciarlo è apparato), copia dichiarata di `s188-throughput.sh` con
`PIN_ATTESO` nuovo per nome, confronto con le celle AX di `s188-throughput-verdetto-t1.out` (C=4: 12,6 · 10,8 · 23,6 · 32,9 · 20,9; C=8: 12,7 · 10,8 ·
23,6 · 33,1 · 20,9), soglia 0,97×, spread ≤5 %, R=3, criterio scritto in C0. È contrasto fra binari diversi ⇒ SOLO direzione/regressione, mai cifra (§3).
**R8 — Lettera WP-77.5 «bounded + 503»: superata per nome, non in silenzio.** Testo per `COUNCIL_WP77.5_REVIEWS.md` (nota a piè): «SUPERATA S-189: la
coda è una, unbounded, come il backlog del kernel di `php -S`; residuo §3.9 (l): sotto flood phpr accetta e accoda senza limite dove il kernel
rifiuta oltre SOMAXCONN — dichiarato, non curato».
**R9 — La barriera `__census_self` è apparato senza committente.** Nessuna campagna a W>1 esiste né è in ordine del giorno; costruirla (OnceLock<Barrier>,
handler che concatena W corpi) è codice vivo in `main.rs`/`worker_pool.rs` per un modo che il contratto stesso dichiara non verdict-grade. Emenda:
a W>1 il ramo self risponde `census-self VOID w=<W>` (fail-closed in-band, il verdetto già rifiuta w≠1) e la barriera entra in «Aperture per NOME».
Timebox §1: concilio + gate + dente + lettere + guardia AX è già vicino a ½ sessione; la barriera lo sfonda per certo.
**R10 — Come questo concilio verrà aggirato (S-168 insegna) e la guardia.** S-168: delibera scritta nel NEXT (075b6c63) e SPARITA nella riscrittura
S-171 (7fbe896f). Qui il rischio è doppio: (a) la riga 1 del NEXT è a **600/600 caratteri** e il file a 7.824/8.000: NON c'è spazio per la delibera
S-189 senza scaricare qualcosa — e REGOLE §8 dice che un kill resta in riga 1 «fino al concilio seguente»: QUESTO è il concilio seguente per S-168
(R1/R4) ⇒ il concilio DELIBERA per nome lo scarico di «KILL A REGISTRO: S-168 … sanata qui» in `gaps/VETI.md` (riga datata), ROTTA B MORTA resta
(01/11, <3 sessioni); senza questa delibera esplicita la riga 1 si riscrive «per far posto» = incidente KS-K186-1. (b) Una sessione futura ripristina
il round-robin «per la misura»: lo impediscono R5 (dente in CI) e la riga di assenza in `gate-lever-pins.sh` (R6), non il verbale.
**R11 — Ordine dei commit (criterio prima del codice, §3) con gate per commit; rollback.**
C0 docs: `wp189-harness/s189-criterio-loopback.md` (≤10 righe: segno, soglie R1-R3, R=3, giudice = lo script), criterio guardia AX (R7), sintesi del
concilio con lettere emendate (testi in R12), §3.9 (h) riscritta + (l) nuova · gate: `next-budget.sh` (NEXT a saldo), nessun `.rs`.
C1 fixture + `gate-loopback.sh` · gate: run con bracci old (ROSSO giusto) + oracle W=3/W=1 + new=tree (ROSSO: il dente è rosso prima); `.out` con hash.
C2 `dispatch.rs` + test unitari della coda (produttori/consumatori, drena-poi-esce, push dopo close, close sveglia) · gate: `cargo test -p php-server`
×3 feature (ci.yml 98/105/143), `loc_dente`, corsie `-D warnings` (82-85, 128).
C3 cablaggio in `worker_pool.rs` + `__census_self` fail-closed + dente R5 (prova del rosso sul tree C2 nel `.out`) · gate: C2 + `gate-lever-pins.sh`
(A-PP23 + assenza `next_worker`) + `gate-capture-order.sh` + `gate-doc-purge.sh` + `gate-loopback.sh` VERDE 3/3 + `run-gate.sh` G-APERTURA-2 +
`battery61-axum.sh` 6/6 (body).
C4 lettere/parsers: la riga W>1 nei sei verdict (R6), `gate-lever-pins.sh` emendato, `COUNCIL_WP189_REVIEWS.md`; riga 1 NEXT SOLO con firma.
C5 `scripts/pin-server.sh` → s189 (2 commit dello script) → guardia AX (R7) ⇒ KS-SK-189-6; `PIN_ATTESO` dei harness S-188 aggiornato per nome.
Rollback: `git revert` di C3 **e** C2 insieme (C2 solo = `dispatch.rs` morto ⇒ `-D warnings` rosso); la prova del rollback è `gate-loopback.sh` di
nuovo rosso sul tree; pin s188 resta in stash e in `PIN_ATTESO` fino a C5.
**R12 — Testi sostitutivi delle lettere (entrano in C4, firma per la riga 1).**
A-PP36 (`wp87…:323`): «dispatch-count per-thread in-band; verdict fail-closed su Σcount ≠ nreq; dal S-189 (coda condivisa) la ripartizione per thread
è definita SOLO a W=1 — ogni cifra per-thread esige `w=1` in-band (A-PP28), a W>1 il run è non verdict-grade».
KS-PP-87-3 (`:161`): «cifra net da run senza dispatch-count in-band con Σ==atteso, o con w≠1 ⇒ riga = envelope, mai attribuzione».
A-PP37 (`wp88…:131`): «verdict esige Σ della mappa == attesa e thr-set ⊆ {0..W-1}; a W=1 mappa == {0:nreq}. Il ghost `next_worker` è ABROGATO S-189:
non esiste contatore di dispatch, una send fallita non lascia scheletri (restano i ghost delle watermark A-MS16, dichiarati)».
A-PP42/KS-PP-89-2 (`wp90…:154`, `design89.md:64,81`): «VDISP = thr-set ⊆ {0..W-1} e Σcount == nreq (+1 self sulla slope); count per-thr == nreq/W
vale SOLO a W=1; w>1 ⇒ FAIL VDISP non verdict-grade. “Mai assumere il round-robin in silenzio” diventa “il round-robin non esiste (S-189)”».
§3.9 (h) (`PHPR_DIVERGENCES…:697-705`): «(h) CURATA S-189: pool a coda condivisa (semantica dell'accept condiviso di `php -S`/php-fpm); dente
`wp189-harness/gate-loopback/` bilaterale con controllo positivo sul pin s188; a W=1 il loopback bloccante stalla fino al timeout del client COME
l'oracle (conservato per nome); nessun timeout nel handler (fedeltà). (l) coda unbounded senza il backlog del kernel: dichiarata».
Riga 1 NEXT, testo da aggiungere (≤300, a saldo dopo lo scarico R10a): «· **S-189 (delibera utente 03/11): pool `--axum` a CODA CONDIVISA = accept
di `php -S`; gate-loopback con controllo positivo rosso sul pin s188; A-PP36/37/42, KS-PP-87-3/89-2 emendate per nome; barriera `__census_self`
APERTA non costruita; W=1 unico modo census**» (247 caratteri).
**R13 — Uso del concilio (REGOLE §7/§8): corretto nella convocazione, a rischio nell'esito.** Richiesta esplicita + cambio di rotta: titolo valido. Ma il
dossier §4 porta già nome di file, API, elenco test e lettere «da emendare»: una fase 2 che li ricopia ha benedetto. La sintesi deve registrare per
ogni sedia almeno una refutazione ACCOLTA con modifica del §4, o dichiarare «nessuna» col nome (dissenso a registro, come S-186).

## KILL-SWITCH
| condizione | atto |
|---|---|
| KS-SK-189-1: sul pin con hash == `PIN_ATTESO` il controllo positivo NON dà «A ERR28 ∈ [2500;4000] ∧ B OK < 1000» in 3/3, o hash ≠ atteso | gate VOID, nessuna promozione, nessun merge |
| KS-SK-189-2: dente unitario R5 assente da C3, o senza prova del rosso sul tree C2 nel `.out` | C3 non si committa; se già in `main`: incidente |
| KS-SK-189-3: ripetizione difforme (qualsiasi), o boot con richieste ≠ 4 promosso invece che VOID+ripetuto | FAIL del gate, mai «2 su 3» |
| KS-SK-189-4: edit a un verdict/test census oltre la riga W>1 dichiarata, o commento A-PP37 «aggiornato» invece che cancellato, senza voce nel verbale | incidente (dossier KS-189-3) |
| KS-SK-189-5: riga 1 del NEXT riscritta per far posto senza delibera di scarico per nome (R10a), o senza «S-189 coda condivisa» a fine sessione | incidente KS-K186-1 |
| KS-SK-189-6: mediana AX C=4 o C=8 < 0,97× t1 su una URL, o spread > 5 % | cura NON promossa (resta in branch), pin resta s188 |
| KS-SK-189-7: barriera `__census_self` o knob `rr` costruiti in S-189 | apparato oltre mandato: incidente §1, revert prima del pin |
| KS-SK-189-8: `battery61-axum` < 6/6, G-APERTURA-2 FAIL, `gate-lever-pins`/`gate-capture-order` FAIL dopo C3 | HALT (dossier KS-189-4) |
