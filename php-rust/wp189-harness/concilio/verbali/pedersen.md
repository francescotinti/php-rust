# Verbale sedia PEDERSEN — S-189 (bozza indipendente, FASE 1)

Generato con assistenza AI (Claude Fable 5).
Lente: confine per-richiesta e lifecycle — RetainSet, census per-thread, mappa di dispatch, contatori in-band.
Letto: dossier §1-§8; codice `worker_pool.rs` 185-303 / 400-460 / 504-650 / 659-746, `main.rs` 283-440,
`memcensus.rs` 2148-2162 (`census_line`: una `write_all` O_APPEND per riga, atomica PER RIGA, non per sequenza);
lettere A-PP36/37/42, KS-PP-87-3, KS-PP-83-3, design89 §A-PP49, `verdict88.sh:105-120`, `verdict90.sh:205-230/375-400`,
`measure90-campaign.sh:285-310`. Nessun harness S-18x legge `worker_dispatch`/`VDISP`/`__census_self`: i consumatori
della mappa sono TUTTI archivio wp86–wp90 (pin ≤ s188, dispatcher round-robin).

## VERDETTI
- **Q1 ROTTA — (A) CONCORDO CON EMENDAMENTI.** La coda condivisa non tocca il confine per-richiesta: `WorkerTask` resta
  l'unico oggetto che attraversa thread (A-MS2/A-MS3 INVARIATE), RetainSet e Vm nascono e muoiono sul thread che fa `pop`
  (A-DS2 INVARIATA), OUTSTANDING è globale e indipendente dalla forma della coda (A-TH15/A-TH22/KS-PP-83-3 INVARIATE).
  (B) RIFIUTATA: il flag «libero» è una seconda fonte di verità accanto alla coda e sotto saturazione torna cieco (dossier §7).
  (C) RIFIUTATA: difetto di produzione (tabella §2, flussi B/C revertono modifiche e aggiornamenti).
- **Q2 CONTRATTO — (iii) ALTRO: «W=1 unico modo di census» ESTESO al self-census, stile A-PP28; niente barriera in S-189.**
  Le lettere 8/9/10 si emendano come in R1-R4; la lettera 11 NON si ri-disegna: `__census_self` a W>1 risponde
  rifiuto in-band fail-closed (R5). La barriera del relatore, così com'è, è insana (R5-R6): se la si vuole, nasce come
  design189 con le quattro condizioni di R6 e si attua SOLO nella sessione che ne consuma l'esito (precedente design89).
  La mappa per-thread a W>1 è ADVISORY per dichiarazione; a W=1 la FIFO condivisa è round-robin per identità: nessuna
  campagna già a registro cambia grado.
- **Q3 FORMA — CONCORDO CON EMENDAMENTI.** Il dente gate-loopback basta per la SEMANTICA del dispatch solo se legge
  anche la riga `tag=worker_dispatch` del braccio union (R7: impronta esatta `{0:2,1:1,2:1}` sul pin vecchio, Σ==4 sul
  nuovo). Throughput: RIMISURARE (Mutex+Condvar al posto di W canali lock-free: a C=8 la contesa è un cambio reale;
  «A/P=1,00 per costruzione» vale per CPU/req a W=1, non per il wall a C=4/8). «Bounded+503» WP-77.5: SUPERATO per nome
  (l'oracle ha il backlog del kernel: pieno ⇒ SYN drop, mai 503) — apertura §3.9 (l) solo descrittiva. Timeout in
  `rx.await`: NO (fedeltà; §3.38 è la sede). Residuo DICHIARATO: a W=1 la richiesta interna in stallo viene ESEGUITA dopo
  il timeout del client PHP (zombie, come `php -S`): conta in `dispatched`, OUTSTANDING pareggia (send fallita, dec esegue).

## ROTTA (una riga)
A con census chiuso a W=1 anche per `__census_self` (rifiuto in-band a W>1), lettere 8/9/10 versionate «pin ≤ s188 round-robin»,
verdetti d'archivio INTATTI, dente gate-loopback con impronta di dispatch in-band, throughput rimisurato con KS-189-2.

## REFUTAZIONI ED EMENDAMENTI
- **R1 — Il dossier sbaglia la direzione dell'insieme: «⊆ {0..W-1}» è PIÙ DEBOLE del provabile.** La riga
  `tag=worker_dispatch thr=i count=n` è emessa UNA volta per worker al TEARDOWN (`worker_pool.rs:412-415` union,
  `447-451` census, A-PP39), anche con `count=0`. Sotto coda condivisa l'insieme dei `thr=` delle RIGHE resta ESATTO
  {0..W-1}: è il testimone «W worker vivi e joinati» (A-MS7), non una proprietà del dispatcher. Ciò che cade è solo
  `count == nreq/W` per thread (`verdict88.sh:116`, `verdict90.sh:222-227`). **A-PP42 / KS-PP-89-2 (nuovo testo)**:
  «VDISP esige W righe `worker_dispatch` con thr-set ESATTO {0..W-1} (unicità+completezza, invariata) e Σcount == nreq
  (+1 per `/__census_self` sullo slope); l'uguaglianza per-thread `count == nreq/W` vale SOLO per pin con dispatcher
  round-robin (≤ 97b58fd225ca7289) ed è ADVISORY altrove. Con Σ≠nreq ⇒ FAIL (phantom), set≠esatto ⇒ FAIL (worker perso)».
- **R2 — A-PP37 (`wp88…:131`): la clausola del ghost MUORE, la mappa degrada, A-MS16 resta.** `push` chiusa restituisce
  `Err(task)` senza consumare slot: nessun ghost di dispatch esiste. Il ghost dei watermark `note_*` (A-MS16, `:730-735`)
  è INVARIATO (inc prima della push, mai annullato). **Nuovo testo**: «il verdetto esige Σcount per-thread == atteso e set
  esatto; la MAPPA è attesa solo su pin round-robin; una run con dispatch Err resta VOID per A-MS16 e rows==N; la coda
  condivisa non ha slot ⇒ nessun ghost da dichiarare». Il commento `:724-729` si cancella a saldo righe: tenerlo dichiara
  un fenomeno impossibile (classe KS-PP-87-1, token-presenza).
- **R3 — A-PP36 / KS-PP-87-3 (`wp87…:161,323`): INVARIATE nel meccanismo, emendate nell'«atteso».** `dispatched`
  conta al pickup (`:543`), prima di ogni esito — sotto `pop_blocking` identico. **Testo**: «cifra net da run senza
  dispatch-count in-band con Σ==atteso ⇒ envelope»: l'atteso è la SOMMA; la ripartizione per-thread non è mai più
  «atteso» a W>1. Nessuna riga di runtime cambia.
- **R4 — KB-87-3 / KL-87-2 / design89 A-PP49: la coda condivisa rende IRRIPRODUCIBILE lo slope ×W a client sequenziale.**
  Con curl sequenziali la coda è vuota a ogni completamento e tutti i W worker dormono sul Condvar: `notify_one` sveglia
  un parcheggiato NON SPECIFICATO (può essere sempre lo stesso ⇒ `{0:nreq, 1:0, …}`). Il protocollo m88/m90 «un pad per
  thr», «hello+pad per worker» (design89 §2) e il budget b per-worker (KL-87-2) NON sono più ottenibili senza concorrenza
  C=W o gang. **Testo**: «le campagne ×W (m88, m90) e la composizione warm per-worker sono VALIDE SOLO per pin round-robin;
  ogni rimisura del budget ×W su pin ≥ s189 esige un protocollo nuovo (C=W concorrente, inflight_max==W DICHIARATO) o è
  VOID». Coerente con S-186 (census per-thread già declassato a calibrazione).
- **R5 — La barriera del relatore è INSANA in tre modi; la risposta di S-189 è il rifiuto in-band, non la barriera.**
  (a) Deadlock permanente: pool non quiescente + un worker occupato su una richiesta che fa loopback ⇒ W−1 worker fermi
  alla barriera, la richiesta interna in coda, il worker occupato non termina mai, il handler non ha timeout (`main.rs:432`)
  e `std::sync::Barrier` non ha `wait_timeout`: processo VIVO e MORTO (classe KS-PP-6). (b) `OnceLock<Barrier>`: una
  generazione parziale (worker abortito, timeout lato curl `-m 5`) corrompe tutte le chiamate successive. (c) CONFLITTO
  PER COSTRUZIONE con KS-PP-83-3 e col driver (`verdict81.sh:28,99,245`, `measure78.sh:318,404`): i W task gang passano
  per `OUTSTANDING` (devono: saltare l'inc farebbe scattare il tripwire del dec al pickup, `:549-556`) ⇒ `inflight_max==W`,
  watermark di PROCESSO mai azzerato ⇒ OGNI run con gang a W>1 è VOID dal giudice esistente. **Emendamento alla lettera 11**:
  `main.rs:396-397` — se `worker_count() > 1` il handler risponde `census-self: refused w=W (W=1 only, KB-78-3/A-PP28)`
  senza dispatch; il commento «W sequential calls cover the W workers under round-robin» (`:392-394`) si cancella;
  `measure90-campaign.sh:296-303` resta archivio INTATTO con nota «round-robin, pin ≤ s188».
- **R6 — Condizioni minime se un futuro concilio volesse la gang (design189, NON S-189).** (1) Pre-check in-band:
  `OUTSTANDING==0` (Acquire) IMMEDIATAMENTE prima dei W push, altrimenti rifiuto senza dispatch; (2) gang Mutex+Condvar
  propria con `wait_timeout` (≤ il `-m` del driver) e generazione azzerata su timeout, riga
  `tag=peak_self thr=i status=timeout` ⇒ VOID; (3) `WorkerTask.probe: bool` — inc/dec OUTSTANDING sì, `note_outstanding`
  NO (il watermark `inflight_max` non vede la sonda; la coppia `arr_pre/arr_post` esige `arr_post == arr_pre + W` esatto);
  (4) semantica righe: ogni worker prende ESATTAMENTE un task (un thread fermo in `wait` non può fare un secondo `pop`;
  W arrivi ⇒ W thread distinti) ⇒ `tag=peak_self thr=i` una per worker, heap distinti (VSELF `verdict90.sh:384-388`
  resta ESATTO, non ⊆). Avvertenza parser: `census_line` è atomica per RIGA, le sequenze `mi_bin_thread_rows` di W
  thread si INTERFOGLIANO: il parser a finestra `:392` (`inwin` dal primo `peak_self`) regge perché unisce gli `heap=`;
  ogni parser che assuma contiguità thr→righe è VOID.
- **R7 — Tripwire QUEUE_DEPTH con Condvar: REGGE, e diventa ESATTO.** Inc Relaxed sequenced-before `push` (lock+unlock
  = release), `pop` acquisisce lo stesso Mutex dopo: l'inc happens-before il dec al pickup, `old==0` è irraggiungibile
  senza wrap (KH81-2 invariato). In più la coda espone `len()`: con il dec spostato DENTRO la sezione critica di `pop`,
  `QUEUE_DEPTH == VecDeque::len()` è un'identità verificabile. Non tocca `:543-648`: va come test unitario in
  `dispatch.rs` (push N senza consumer ⇒ `len()==N==QUEUE_DEPTH`), non come modifica del worker_loop.
- **R8 — Dente gate-loopback: il controllo positivo deve provare la COLLISIONE, non solo l'ERR28.** Sul pin vecchio W=3 e
  server fresco (`next_worker=0`): A→0, B→1, interna B→2, interna A→0 ⇒ righe union `thr=0 count=2`, `thr=1 count=1`,
  `thr=2 count=1` (impronta ESATTA, deterministica). Sul nuovo: 3 righe, Σ==4, nessuna attesa per-thread. Un ERR28 senza
  `thr=0 count=2` è un ERR28 di altra origine (DNS, pong lento): dente nullo. Lo stderr union ha già la riga (A-PP39):
  costo zero, nessuna feature.
- **R9 — Test unitari dichiarati «INVARIATI» (dossier §4): veri, ma manca il controllo negativo.** 1801/1841/1911/1919/2029
  sono W=1 o indipendenti dalla forma: restano. MANCA un test W≥2 che provi la proprietà nuova: worker 0 bloccato su un
  task lungo, secondo task servito da un altro worker entro un bound dichiarato (rosso sul round-robin con probabilità
  1/W — quindi deterministico solo con W=2 e task lungo dispatchato per primo: round-robin manda il secondo a thr=1,
  passa anche lui!). Il falsificatore deterministico è a W=2 con TRE task: lungo, corto, corto — round-robin: terzo→thr=0
  attende il lungo; coda condivisa: terzo→thr=1. Bound = timeout dichiarato nel messaggio (A-TH25).

## KILL-SWITCH
| # | condizione (meccanica) | atto |
|---|---|---|
| KS-PP-189-1 | verdetto d'archivio (`verdict86-90.sh`, `va-census-template.sh`) EDITATO per far passare un raw di pin ≥ s189 | verdetto NULLO, incidente: l'archivio giudica i pin round-robin; i pin nuovi hanno template nuovo |
| KS-PP-189-2 | claim per-thread (mappa, composizione warm, budget b/worker) da raw con `w>1` su pin ≥ s189 senza protocollo C=W dichiarato | claim ADVISORY, mai attribuzione (estende KS-PP-87-3) |
| KS-PP-189-3 | `__census_self` servito a `w>1` (corpo `census-self thr=` senza rifiuto) su pin ≥ s189 senza design189 firmato | raw VOID, cifre self VOID |
| KS-PP-189-4 | raw con `inflight_max>1` e righe `peak_self` ⇒ gang passata per il watermark | run VOID (KS-PP-83-3 invariata, nessuna eccezione tacita) |
| KS-PP-189-5 | gate-loopback: braccio pin vecchio W=3 senza impronta `thr=0 count=2` o Σcount≠4 su un braccio qualsiasi | dente NULLO, nessuna promozione (rafforza KS-189-1) |
| KS-PP-189-6 | commento ghost `:724-729` sopravvive alla coda condivisa, o il commento `:392-394` (W sequenziali) sopravvive al rifiuto | token-presenza: review FAIL (classe KS-PP-87-1) |
| KS-PP-189-7 | `dispatch.rs` promosso senza il test negativo R9 (W=2, lungo+corto+corto, rosso sul pin vecchio per costruzione) | cura NON promossa: la proprietà nuova non è falsificata |

— Pedersen, sedia 6.
