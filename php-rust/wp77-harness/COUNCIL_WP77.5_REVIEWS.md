# ⚖️ CONCILIO WP-77.5 — revisione di WP-77.4.2 + programma WP-77.5

**Data**: 2026-07-30 · **Collegio**: 9 sedie + co-optata web-runtime (tema Axum)
· **Mandato**: REFUTARE report WP-77.4.2 e programma WP-77.5 · **Esito: 4 MI
OPPONGO (Matsakis, Pedersen, Leijen, web-runtime) + 6 CON EMENDAMENTI — il
programma WP-77.5 come scritto ("Vm reale in task_local + M3 + M4") è
RESPINTO; WP-77.5 si apre con la rinegoziazione del design (M9 → worker
pool) e la rietichettatura dei verdetti 77.4.2.**

## SINTESI DI CONVERGENZA (vincolante per design77.5)

1. **RIETICHETTATURA OBBLIGATORIA (Pedersen R1/R2 — FATTO verificato sul
   tree: `request_end()` ha ZERO chiamanti; il cli-server usa Vm fresco +
   mass-teardown WP-72)**: i PASS KM-77-2 e KS-M1-Gate-Upgraded valgono per
   il confine **cli-server/fresh-Vm** (baseline preziosa), NON per il
   confine reset-e-riusa di M1. Il kill-switch sul confine reuse RESTA
   APERTO. KS-M1-Complete di 77.4.1 (`object_id==1`) è trivialmente vero su
   Vm fresco: non discrimina. (Concordi: Hoare, Stogov S-77.5.1, web-runtime
   W-77.5.5, Klabnik KS-K1.)
2. **M9/task_local È MORTO COME SCRITTO** (Matsakis a/b/c, Hoare H-77.5.1/2,
   web-runtime 1/2/3, Hejlsberg R3): `task_local!` esige `'static` — un
   `Vm<'m>` che borrowa Module non può entrarci (type error); per-task =
   per-request ⇒ NESSUN riuso (M1 sarebbe decorativo); Vm è `!Send`
   (Rc-pervasivo) ⇒ handler axum multi-thread non compila, e task_local +
   spawn_blocking non si compongono (blocking pool = 512 thread effimeri =
   fino a 512 Vm). **Architettura convergente: worker-attore stile php-fpm**
   — N thread OS dedicati e longevi, ognuno PROPRIETARIO della sua Vm
   (`thread_local` lì è legittimo: non sono worker tokio), dispatch via
   canale mpsc bounded + oneshot, 503 su canale pieno; ownership round-trip
   `FnOnce(Vm)->Vm`, zero borrow attraverso confini. M9 va emendato dal
   concilio, non aggirato.
3. **request_end() HA BUCHI DI FEDELTÀ RSHUTDOWN** (Stogov, Hoare H-77.5.3,
   Pedersen R3/R5): non resetta static di funzione/classe (e la spec M1
   li dichiara "preserved" — contraddizione col fixture km77 che esige il
   restart a 0); `shutdown_fns/ob_stack/frames` SCARTATI senza eseguire
   (Zend li ESEGUE: shutdown fns → OB flush con handler → dtors in ordine);
   bypassa il mass-teardown S-72.4 (ogni ciclo Rc rileakerebbe — C4
   riaperta); `next_object_id=1` sicuro solo a store vuoto (tripwire);
   `error_level=0` non è il default configurato; `uncaught_throwable`
   preservato senza consumatore = veleno per la richiesta N+1. Serve
   `request_shutdown()` ordinato Zend + `request_start()` simmetrico.
4. **CHIUSURA PER COSTRUZIONE** (Pedersen P-77.5.4, Hejlsberg E-77.5.2):
   registro PRESERVE/RESET committato + grep-gate sui campi di `struct Vm`
   (ogni campo o nel reset o in allowlist motivata, pattern BORROW-OK);
   campo non classificato = FAIL.
5. **CACHE CALDE: clear ≠ fedeltà** (Bak, Stogov nota di merito):
   `preg_cache` in Zend è per-processo (persistente); `iof_cache` e affini
   vogliono modello epoch, non clear per-request. Costo del clear =
   refill spalmato sulla richiesta successiva: va CONTATO (eventi×ns) e
   A/B-ato prima di accettarlo.
6. **M4 NON È AVVIATA** (Bak KS-B77-1, Gregg, Leijen): nessuna colonna
   oracle (352 ms è un numero senza denominatore); wall di curl ≠ user CPU
   (regola di casa); n=10 con coda 4× non citabile; "350-367 MB stabile su
   ~46 req" RETROCESSO a "non falsificato" (banda 17 MB > qualsiasi leak
   plausibile a 46 req; mimalloc MADV maschera; serve ladder K∈{1,10,50}
   sotto census used_n/used_b + coppia oracle stessa-sera); "≤±2%" non è un
   gate finché non dichiara metro/stimatore/finestra/workload; budget
   footprint(N) predetto PRIMA del pool (KL-77.5-1: ΔVm marginale >25% del
   footprint N=1 ⇒ STOP share-more).
7. **MATRICE GATE DA ESTENDERE** (Klabnik): probe di overlap per KM-77-2
   (oggi non discrimina simultaneità da serializzazione); controllo
   positivo SUL percorso HTTP; workload B è self-consistency non parity
   (pin hash committati + confronto una-tantum vs oracle + matrice volatili
   wp-login/POST/404); pin ORM diventa non-tautologico alla prossima run;
   fixture km77 estesa (static di classe, ridichiarazione classe,
   include_once); caso nuovo di 77.5 = eccezione-poi-richiesta-pulita.

**Gate d'apertura WP-77.5 (bloccanti, dalla convergenza):**
- G-APERTURA-1: spike 20 righe che istanzia il Vm REALE nello storage
  scelto e compila senza unsafe/leak (KS-M-77.5-A, KH-77.5-A).
- G-APERTURA-2: gate due-richieste-STESSO-Vm con request_shutdown+end in
  mezzo; byte-parity richiesta-2 vs Vm fresco (KS-P77-A, S-77.5.2/3/4).
- G-APERTURA-3: tripla+amp K=10 sul path reuse (KH-77.5-B = C4).
- G-APERTURA-4: coppia oracle stessa-sera su front/post/feed (CPU+footprint)
  prima di qualsiasi cifra M4 pubblicata.

---

## VERBALE HOARE — CON EMENDAMENTI

(vincolanti; senza H-77.5.1–3 mi opporrei al via del codice WP-77.5)

Ho letto NEXT_SESSION, il verbale 77.4.2, WP77_4_2_RESULTS, `php-server/src/main.rs` e `Vm::request_end()` (vm/mod.rs:3331).

**Cosa il report spaccia per validato e non lo è.** I due kill-switch sono dichiarati "CHIUSI ✅" nello stato-gate, ma sono chiusi *sul percorso cli-server*. `with_vm_lifecycle` (main.rs:50-62) non crea un Vm, non chiama `request_end()`, prende `FnOnce() -> String` e ritorna un literal: la "integrazione M1" è un commento. Il `task_local!` contiene `RefCell<u64>` (un epoch), non un Vm: l'amendment M9 è soddisfatto solo cerimonialmente. Un kill-switch definito per l'architettura axum non può risultare CHIUSO con evidenza raccolta altrove — lezione WP-72: il gate deve mordere sulla superficie appena attivata.

**H-77.5.1 — `Vm` è `!Send`: conflitto frontale con i bound di axum.** Il Vm è pervaso di `Rc`/`RefCell`; gli handler axum su runtime multi-thread esigono future `Send`. `tokio::task_local!` NON risolve questo: il future resta `!Send` e non compila. Il design DEVE scegliere *prima del codice* tra: (a) runtime `current_thread` / `LocalSet` per worker con `spawn_local`; (b) esecuzione PHP sincrona dentro `spawn_blocking` + `thread_local!` sul pool bloccante. Nota che (b) contraddice M9 alla lettera — il concilio deve emendare M9, non aggirarlo. Vietato per RULEBOOK §0 il terzo esito prevedibile sotto pressione di deadline: `unsafe impl Send`.

**H-77.5.2 — `Vm<'m>` non entra in uno storage `'static`.** `task_local!`/`thread_local!` esigono valori `'static`; un Vm che borrowa `Module` e `RetainSet` non-`'static` non può viverci in safe Rust, e il pattern N=1 *riuso* cross-request lo richiede. Decisione esplicita richiesta: `Module` immortale (`OnceLock`/`Box::leak` — accettabile: il module È per-process) e Vm ricostruito o riusato con `'m = 'static`. Nessuna struttura self-referential.

**H-77.5.3 — `request_end()` bypassa il mass-teardown S-72.4.** La funzione azzera `frames`, `created`, `teardown_weaks`, `gc_*` con `.clear()`: nessun reverse-symtab a fixpoint, nessun dtor-walk in ordine di creazione, nessun BREAK dei cicli via Weak. Sul path axum i distruttori non girano e ogni ciclo Rc utente rileaka per-richiesta — esattamente il leak chiuso in WP-72, che rientrerebbe dalla porta nuova. `request_end()` deve *invocare* la macchineria di teardown validata, poi azzerare; e la tripla (used_n/used_b/amp K=10) va rieseguita sul path axum.

**H-77.5.4 — borrow di `RefCell` attraverso `.await` = panic runtime, non errore di compilazione.** La "Rule" in commento (main.rs:39) non è un meccanismo. Struttura obbligata: il borrow vive solo dentro una closure sincrona (`with(|vm| …)`); nessun `.await` nel suo scope, verificato da gate.

**H-77.5.5 — Ri-armare i gate col nome nuovo**: KM-77-2-axum e KS-M1-Upgraded-axum (stessi runner, target axum con Vm reale) diventano BLOCCANTI di WP-77.5; i PASS attuali restano validi solo come baseline cli.

**Kill-switch**: KH-77.5-A (compilazione richiede unsafe/`Box::leak` per-request/wrapper Send fasullo ⇒ STOP, concilio); KH-77.5-B (tripla su path axum, amp K=10: used_n ≠ 0,000 o amp > +0,2 ⇒ STOP, è la C4 riaperta); KH-77.5-C (fixture concorrente con await forzato: `BorrowMutError` ⇒ FAIL; grep-gate zero `.await` nello scope di `with(...)`, zero `unsafe` in php-server).

Punti positivi riconosciuti (non benedizione): fixture rese mordenti, caveat di perimetro onesto, `request_end()` safe-only. Ma il programma tratta WP-77.5 come "mettere il Vm nel handler"; è invece la collisione delle tre invarianti — `!Send`, `'static`, teardown — e va progettata, non incontrata.

---

## VERBALE MATSAKIS — MI OPPONGO

(al meccanismo di storage del programma WP-77.5 come scritto: "Module+RetainSet via tokio::task_local!"). Il report 77.4.2 in sé è onesto (caveat placeholder dichiarato); ma il programma successivo contiene **tre errori di ownership non ancora visti**, di cui uno non compila proprio.

**(a) `tokio::task_local!` esige `T: 'static`.** La macro genera una `static LocalKey<T>`: un `Vm<'m>` che tiene borrow su `Module` (il run_loop è `&'m Op` zero-clone, WP-29/31) **non può entrare** in task_local. Il piano "Vm con lifetime su Module e RetainSet dentro task_local" è un type error prima di essere un design. Le vie d'uscita sono strutturali: Vm che *possiede* `Arc<Module>`/`&'static` (e leak per-request = reintrodurre per costruzione il leak chiuso in WP-72), oppure separare `Vm` (owned, 'static) dal contesto d'esecuzione borrowing creato *dentro* la richiesta.

**(b) task-local = per-task = per-request ⇒ NESSUN riuso.** In axum ogni richiesta è un task nuovo: uno storage per-task muore col task. M1 `request_end()` ("reset per il *prossimo* request") è **incoerente** con task_local: non esiste un prossimo request in quel storage. O la persistenza vive FUORI dal task (pool con round-trip di ownership: `FnOnce(Vm) -> Vm` via canale — niente RefCell, niente borrow, il borrow checker lavora per noi), o M1 è un no-op decorativo. Nota: l'attuale `VM_EPOCH` non è mai `.scope()`-ato — un `.with()` panicherebbe al primo uso (main.rs:40-42, codice morto che morde).

**(c) Il runtime è `Rc`-pervasivo ⇒ `Vm`/`Zval`/`Module` sono `!Send`.** Conseguenze a catena non affrontate: (1) `axum::serve` su runtime multi-thread esige future `Send` — un handler che tocca un Vm `!Send` non compila; (2) `spawn_blocking` (l'unica via pulita per PHP sync CPU-bound in async) **non vede i task_local** e non accetta closure `!Send`: task_local e spawn_blocking si escludono a vicenda; (3) eseguire PHP inline nel handler blocca il worker tokio. La scelta obbligata va dichiarata: runtime `current_thread`/LocalSet-per-worker (SO_REUSEPORT), o pool owned per-thread — non è un dettaglio, è l'architettura.

**Emendamenti**: M-77.5.1 (spike 20 righe col Vm REALE nello storage scelto che compila, PRIMA di ogni handler; `Box::leak` per-request ⇒ STOP); M-77.5.2 (ownership round-trip `FnOnce(Vm) -> Vm` o pool via canale; zero `Ref`/`RefMut` che possano attraversare `.await`); M-77.5.3 (`with_vm_lifecycle` ⇒ `FnOnce(&mut Vm) -> Response` con `request_end()` dal wrapper sullo stesso borrow esclusivo + drop-guard su panic: Vm resettato o scartato, mai riusato sporco); M-77.5.4 (dichiarare il modello di esecuzione e l'implicazione su KM-77-2: con Rc `!Send` la concorrenza è per-worker, non per-task).

**Kill-switch**: KS-M-77.5-A (spike non compila senza unsafe/leak ⇒ halt, riconvoca); KS-M-77.5-B (controllo positivo del riuso: due richieste sequenziali stesso worker devono mostrare cache calda — se ogni richiesta è fredda il "N=1 reuse" è finzione ⇒ FAIL); KS-M-77.5-C (panic-in-handler: richiesta successiva pulita, no RefCell poisoned).

---

## VERBALE KLABNIK — CON EMENDAMENTI

(il lavoro di gate di 77.4.2 è onesto e ha corretto due false-verdi reali, ma la completezza dichiarata non regge: la matrice misura stabilità, non simultaneità né correttezza, e un pin è nato tautologico).

1. **KM-77-2 non discrimina la concorrenza.** Il runner lancia 2 curl parallele, ma nulla prova che il server le abbia servite *sovrapposte*: se il cli-server serializza, il gate degrada a isolamento *sequenziale* — già coperto da KS-M1-Complete. Nessun probe di overlap: niente timestamp in-fixture, niente contatore in-flight, niente endpoint lento che forzi la sovrapposizione. "30 marker unici" prova freschezza per-richiesta, non simultaneità.
2. **Il controllo positivo morde FUORI percorso.** `pc.php` gira via `phpr` CLI, non via HTTP: prova che la *fixture* morde in CLI, non che un leak sul percorso server emergerebbe nell'output HTTP.
3. **Workload B è self-consistency, non parity.** 7/7 byte-id confronta il server *con sé stesso*: una pagina d'errore stabile passerebbe. Nessun pin di contenuto né confronto vs oracle. E front/post/feed sono le tre pagine più statiche e anonime di WP: zero copertura di volatili (nonce, Set-Cookie, login, admin, $_SESSION). Per pagine con nonce il byte-id è l'oracle *sbagliato* (devono differire) — serve un oracle strutturale separato.
4. **Il pin ORM di questa sessione è una tautologia.** `orm7742-fails.txt` è stato creato dalla stessa run che il runner diffa: PASS per costruzione. L'àncora reale è solo il conteggio 3E/13F vs WP-72 e il match con le famiglie della recipe. Diventa un gate vero solo dalla prossima run.
5. **Fixture km77 incompleta sul modello di stato.** Mancano: static di *classe*, ridichiarazione di classe cross-request, include_once/require_once state, $_SERVER/$_GET contaminati.

**Emendamenti**: K-77.5.1 (probe di overlap: fixture `usleep` parametrico + wall-time round — 2 parallele ≈1× latenza = concorrenza; ≈2× = serializzazione; finché non misurato, KM-77-2 riclassificato "isolamento sequenziale via HTTP"); K-77.5.2 (controllo positivo SUL percorso HTTP); K-77.5.3 (pin hash front/post/feed committati + confronto una-tantum vs oracle stessa DB; guardia anche su status-code e header); K-77.5.4 (matrice volatili: wp-login, POST commento, 404, ?s= con oracle strutturale); K-77.5.5 (ORM ri-run in WP-77.5 contro il pin = prima verifica non-tautologica + hash log nel verdict-file); K-77.5.6 (fixture km77 estesa: static di classe, classe ridichiarata, include_once); K-77.5.7 (matrice WP-77.5: multi-endpoint interleaved, eccezione-non-catturata-poi-richiesta-byte-id — è IL caso nuovo, richiesta abortita a metà output); K-77.5.8 (protocollo M4 definito PRIMA: la banda attuale 350–367 è già ±2,4% e fallirebbe il proprio target).

**Kill-switch**: KS-77.5-K1 (senza overlap dimostrato, VIETATO citare KM-77-2 come prova di isolamento concorrente); KS-77.5-K2 (caso eccezione→richiesta-pulita FAIL ⇒ M3 non spedibile); KS-77.5-K3 (mismatch vs oracle su K-77.5.3 ⇒ le tre pagine decadono a evidenza di sola stabilità, claim corretto retroattivamente).

---

## VERBALE HEJLSBERG — CON EMENDAMENTI

(il PASS 77.4.2 è onesto nel suo perimetro, ma il perimetro non contiene il mio: nessun gate morde sul compile-side, e il programma WP-77.5 non ha ancora deciso il modello di condivisione).

**R1 — "statics preservate" è una divergenza semantica non gateata.** In PHP `static $n` di funzione e le proprietà statiche di classe sono stato PER-RICHIESTA: su richiesta fresca danno 1, sempre. Il gate B (WP front 7/7 byte-id) NON può mordere: byte-parity su contenuto immutato è cieca alla stalezza. È la lezione del contatore-tutto-zero applicata al compile-side.

**R2 — class-table preservata ≠ opcache.** Opcache cacha gli opcode compilati (Unit immutabile); la tabella delle classi DICHIARATE è ricostruita a ogni richiesta. Preservarla cambia gli osservabili: `if(!class_exists('X')) class X{}` con corpo diverso per route, `get_declared_classes()`, la trait table FQN (S-72.6) popolata lazy in ordini diversi per route diverse. Nessuna fixture 77.4.2 tocca dichiarazioni condizionali.

**R3 — il modello di sharing Axum è indeciso, e la M4 non lo attribuisce.** M9 ⇒ un Vm per task; il Vm vive su `Rc` (non `Send`): il Module/unit-cache NON è condivisibile tra task così com'è. Tre alternative non scelte: (a) compile-side `Arc` immutabile condiviso + runtime per-Vm; (b) N task = N copie ⇒ 4 task = 1,4 GB, peggio del CGI; (c) N=1 serializzato (concorrenza solo I/O). "350–367 MB stabile" è UN Vm sequenziale: non dice nulla sulla (b). Con N>1 publisher la publish condizionata a `pure` (WP-66) diventa una race di wrong-result sulla trait table FQN senza single-writer o publish idempotente verificata.

**R4 — reset non chiuso per costruzione.** In `request_end()` non vedo lo stack `spl_autoload_register` né la tabella include-once: `autoload_cursors` sì, il registro handler no. La lezione WP-70 vale qui.

**Emendamenti**: E-77.5.1 (fixture statiche differenziali vs oracle: static di funzione, static prop, stale-DB — UPDATE tra req1 e req2, req2 deve rifletterlo); E-77.5.2 (registro PRESERVE/RESET esplicito e committato + grep-gate campo-per-campo); E-77.5.3 (decisione di sharing PRIMA del codice handler: Unit-cache/Module → Arc immutabile o N=1 serializzato; duplicazione per-task VIETATA senza misura preventiva); E-77.5.4 (assert di uguaglianza su ri-publish — fatal in ogni build, stile tripwire WP-67 — per unit-cache e trait table FQN); E-77.5.5 (M4 attribuita: footprint scomposto in quota condivisibile vs per-Vm — è il numero che decide (a)/(b)/(c)).

**Kill-switch**: KS-77.5-STATIC (fixture static-counter ≠ oracle ⇒ stop, si progetta il reset statics prima del Vm nel handler); KS-77.5-FOOT (Δfootprint per task > 15% della quota condivisibile ⇒ modello (b) morto, obbligo (a) o (c)); KS-77.5-PUB (ri-publish non-identica in cache condivisa ⇒ FATAL, mai warning).

---

## VERBALE BAK — CON EMENDAMENTI

(vincolanti. Il lavoro dei gate 4-5 è onesto. Ma M4 **non è avviata**: 352 ms/req è un numero senza denominatore, e `request_end()` contiene clear di cache che in Zend **sopravvivono alla richiesta** — un handicap CPU strutturale spacciabile per fedeltà.)

**R1 — l'oracle manca dalla tabella.** Tre righe phpr e zero righe oracle. Non so se 352 ms sia 2× o 10× PHP 8.5.7: l'intervallo di incertezza copre l'intera domanda. Un "steady ≤±2%" senza denominatore certifica solo la stabilità di un numero che potrebbe essere 5-10× fuori.

**R2 — il clear per-request delle cache calde (vm/mod.rs:3404-3424).** `preg_cache.clear()`: in Zend la cache PCRE è **per-processo**; WP compila decine di regex a pagina — ricompilarle a ogni richiesta è puro handicap. Idem `iof_cache` (serve un epoch, non un clear), `enum_cache`, `reflect_method_info_cache`, `autoload_cursors`, `magic_guard`. Il costo del clear non è nel reset (µs): è **spalmato sulla richiesta successiva come refill**, invisibile a ogni cronometro del reset. Non c'è né il conto eventi/req né i ns/evento (regola WP-53).

**R3 — Vm-per-task moltiplica gli heap caldi.** 350-367 MB per **un** Vm; K task ⇒ ~K×350 MB se class-table/cache non sono condivise. Va pinnato prima del codice.

**Emendamenti**: B-77.5.1 (coppia oracle↔phpr sulle stesse tre pagine, stessa sera, mediana n≥10, stessa tabella; dichiarare se l'oracle ha opcache — precondizione per dichiarare M4 avviata); B-77.5.2 (censimento refill feature-gated: eventi/req e ns/evento per preg-compile, iof-miss, enum, reflect-info, autoload-scan su WP front; il costo si cita come Σ(eventi×ns)); B-77.5.3 (riclassificare `preg_cache` come persistente; `iof_cache` e affini a modello epoch; ogni cache resa persistente passa il gate di fedeltà completo); B-77.5.4 (A/B build-adiacente reset-pieno vs cache-persistenti, 50 req steady, Δ CPU mediana, fail-set identico); B-77.5.5 (pin anti-moltiplicazione: footprint a K=1/2/4 con Vm reale; footprint(K) ≈ base + K×Δreq con Δreq dichiarato; Module/class-table via Arc, mai clonati); B-77.5.6 (cronometrare `request_end()` stesso, n≥40; verificare che `stream_wrappers.clear()` non perda i wrapper built-in alla richiesta 2).

**Kill-switch**: KS-B77-1 (tabella M4 senza colonna oracle ⇒ M4 NON-avviata, stop ottimizzazioni, prima attribuzione owner-level); KS-B77-2 (clear cache calde >2% CPU su A/B ⇒ modello epoch obbligato); KS-B77-3 (footprint(K=4) − base > 4×80 MB ⇒ Vm-per-task BOCCIATO, ridisegno condivisione).

---

## VERBALE PEDERSEN — MI OPPONGO

(alla dichiarazione "kill-switch CHIUSI" come validazione del confine; i dati raccolti restano validi come baseline cli-server).

**R1 — `request_end()` ha ZERO chiamanti.** Verificato sul tree: l'unica occorrenza fuori da `vm/mod.rs:3331` è un **commento** in `php-server/src/main.rs:57` ("request_end() *would be called* here"). `with_vm_lifecycle` non crea nemmeno un Vm. Il reset di ~50 campi è **codice mai eseguito da alcun percorso**, gate compresi.

**R2 — KM-77-2 PASS valida un ALTRO meccanismo.** Il cli-server (`php-cli/src/server.rs:589 run_php`) chiama `run_source_with_ini` per richiesta: **Vm fresco + mass-teardown WP-72**. Il PASS 30/30 prova l'isolamento del confine fresh-Vm — già provato da WP-72 — non il confine reset-e-riusa che KM-77-2 doveva gateare. Il controllo positivo "morde" sul fixture, non sul meccanismo M1. Idem KS-M1-Complete (77.4.1): `object_id==1` è **trivialmente vero su un Vm fresco**.

**R3 — Contraddizione statics baked nella spec.** M1 dichiara "preserving persistent data (**statics**, module, classes)"; ma il fixture km77 (test 3) esige che `static $n` **riparta da 0** — semantica PHP corretta. Su Vm riusato con statics preservati, il fixture oggi verde diventerà FAIL alla prima coppia di richieste reali. Stessa famiglia: include_once table, static props, class-table utente (re-include WP ⇒ fatal redeclare o skip con statics stantii). In PHP **tutto lo stato utente è per-request**; solo l'opcode persiste.

**R4 — Lista DO-NOT-RESET per enumerazione, non per costruzione.** Nessun grep-gate lega i campi di `struct Vm` al reset. Assenti dalla lista: cwd, ini runtime, error_reporting runtime, timezone/locale, include-once state. Ogni campo futuro nasce non classificato.

**R5 — `uncaught_throwable` è veleno trans-confine.** Preservato (AMEND-2), ma M3 non esiste: nessuno lo consuma, nessuno lo azzera. Su Vm riusato la richiesta N+1 parte con l'eccezione della N armata. Inoltre: superglobali resettate a `Undef` senza un `request_start()` che le ricostruisca = $_SERVER indefinito alla N+1.

**Emendamenti**: P-77.5.1 (rietichettare: KM-77-2 = "PASS (confine cli-server/fresh-Vm)"; il kill-switch sul confine reset-e-riusa RESTA APERTO); P-77.5.2 (gate due richieste STESSO Vm con request_end in mezzo — km77 ×2 + WP front ×2; byte-parity richiesta-2 vs Vm fresco); P-77.5.3 (risolvere PRIMA la semantica "persistente": persiste solo module/opcode; statics/include_once/class-table utente per-request o evidenza oracle dell'equivalenza); P-77.5.4 (chiusura PER COSTRUZIONE: grep-gate sui campi di `struct Vm`, allowlist annotata stile BORROW-OK; campo non classificato = FAIL); P-77.5.5 (produttore/consumatore di `uncaught_throwable` con azzeramento post-M3; `request_start()` simmetrico per superglobali).

**Kill-switch**: KS-P77-A (richiesta-2 stesso-Vm non byte-id vs fresh-Vm ⇒ STOP riuso, fallback fresh-Vm-per-task già validato); KS-P77-B (campi Vm non classificati >0 ⇒ FAIL gate); KS-P77-C (`uncaught_throwable` Some al confine senza M3 ⇒ FATAL in ogni build, mai trasporto silenzioso).

---

## VERBALE LEIJEN — MI OPPONGO

(non ai gate KM-77-2/KS-M1 — ben costruiti — ma al claim **"350–367 MB STABILE su ~46 req (no growth)"** e all'avvio del codice pool WP-77.5 senza budget footprint predetto.)

**Refutazione "no growth"**: (1) 46 richieste non sono una prova per gli standard di QUESTO progetto — WP-72 ha chiuso il leak con tripla + amplificazione K=10 intera-esatta; qui c'è un'osservazione passiva vmmap. Un residuo di 100 KB/req = +4,6 MB su 46 req: **invisibile in una banda dichiarata di 17 MB** (350→367 = ±2,4%). La banda stessa è il rumore che maschera l'ipotesi. (2) mimalloc mente a questa scala: MADV_FREE/purge-delay rende il footprint oscillante per meccanica dell'allocatore; il metro giusto per il retention è il **census used_n/used_b per-richiesta** (phpr-memgc esiste, tag=teardown) — non usato. (3) 350 MB non è confrontato con nulla: nessun oracle sulla stessa pagina la stessa sera; potrebbe essere in banda 3,0-3,1× o fuori 2×: oggi è un numero orfano.

**Refutazione Vm-per-task**: N task ⇒ N heap Vm; costo marginale per-Vm non misurato né predetto (viola predizione-misurata WP-48 e il mio Tier-0 di WP-77.2). Aggravante mimalloc: i task migrano tra worker; le free cross-thread finiscono nelle deferred-lists dell'heap del thread proprietario ⇒ frammentazione per-thread che cresce con N e con la migrazione.

**Emendamenti**: L-77.5.1 (ladder per-richiesta amplificata PRIMA di "stabile": K∈{1,10,50} sotto census; accettazione = |Δused_n/req| = 0 spread ≤0,01, NON banda vmmap; il claim 77.4.2 RETROCESSO a "non falsificato" nei documenti); L-77.5.2 (coppia oracle stessa-pagina stessa-sera, footprint a regime R≥3; rapporto ×N in GAP_TREND; senza, 350 MB non entra in nessun report come "in banda"); L-77.5.3 (budget footprint(N) PRIMA del pool: ΔVm marginale N=1 vs N=2, poi predire footprint(4/8/16) = shared + N×marginale, misura post-pool che riconcilia — 100% predetto WP-48; dichiarare cosa è condiviso); L-77.5.4 (igiene mimalloc nel harness Axum: worker FISSATI, `MIMALLOC_PURGE_DELAY=0` per il retention, passata pinnati-vs-migranti prima di N>4); L-77.5.5 (KS-M4-Steady riformulato: "±2%" su 350 MB = ±7 MB non discriminante; sostituire con **≤64 KiB/req di slope vmmap su K=50** E census-slope zero).

**Kill-switch KL-77.5-1**: ΔVm marginale > 25% del footprint N=1 ⇒ il modello Vm-per-task si FERMA prima del codice, ritorno al concilio con l'opzione share-more.

Senza L-77.5.1–3 eseguiti in apertura di WP-77.5, la sedia considera qualunque "M4 PASS" non citabile.

---

## VERBALE STOGOV — CON EMENDAMENTI

(lavoro onesto, gate che mordono; ma il PASS valida il percorso sbagliato e `request_end()` ha buchi di fedeltà RINIT/RSHUTDOWN non gateizzati.)

KM-77-2 e km77 sono PASS **sul percorso cli-server**, dove il PHP gira con stato fresco per richiesta: gli static che "ripartono a 0" lì sono gratis. Ma WP-77.5 spedirà il percorso **Vm-riusato + request_end()**, e lì: (1) **non esiste alcun reset degli static di funzione né delle proprietà statiche di classe** — in Zend entrambi sono per-richiesta anche sotto opcache (map_ptr re-inizializzato a ogni RINIT); (2) `shutdown_fns.clear()`, `ob_stack.clear()`, `frames.clear()` **scartano senza eseguire** — Zend a RSHUTDOWN *esegue* le shutdown function, *flusha* gli OB invocando gli handler, distrugge la symtab *eseguendo i __destruct* (WordPress vive di `wp_ob_end_flush_all` e shutdown hook); (3) `next_object_id = 1` è Zend-fedele **solo a store vuoto**; con stato preservato, oggetto sopravvissuto + id riciclato = collisione `spl_object_id` silenziosa; (4) class table preservata: `class Foo{}` top-level alla richiesta N+1 — nessun gate copre already-declared; (5) `error_level = 0` non è il reset Zend (`zend_restore_ini_entries` riporta al **default configurato**); ini_set non toccato; (6) `constants.clear()` — se svuota le costanti engine senza ri-registrazione, la richiesta 2 è rotta.

**Emendamenti**: S-77.5.1 (ri-eseguire KM-77-2 + km77 + snapshot WP front **sul percorso Axum+Vm reale** appena esiste; il verdetto odierno etichettato "valido per cli-server" nei verdict-file); S-77.5.2 (reset static di funzione e di classe in `request_end()` — semantica map_ptr; fixture dedicata con FAIL atteso oggi sul path reuse come controllo positivo); S-77.5.3 (gate ridichiarazione: stesso span ⇒ no-op; span diverso ⇒ fatal come Zend); S-77.5.4 (**`request_shutdown()` nell'ordine Zend: shutdown_fns eseguite → OB flush con handler → mass-teardown S-72.4 → poi `request_end()`**; gate: shutdown function che stampa, presente nel body di OGNI richiesta); S-77.5.5 (`next_object_id = 1` dietro tripwire store-vuoto FATAL-in-ogni-build); S-77.5.6 (error_level/ini al default configurato; gate `error_reporting()` in richiesta 2); S-77.5.7 (gate costanti: PHP_VERSION + `define()` utente su 2 richieste — la prima sopravvive, la seconda no).

**Kill-switch**: KS-S77-A (finché S-77.5.2/3/4 non sono verdi sul path reuse, VIETATO dichiarare "M1 complete" o pinnare metriche M4 su quel path); KS-S77-B (tripwire store-non-vuoto su WP front = STOP e attribuzione, niente workaround); KS-S77-C (byte-id cross-path Axum-reuse vs cli-server su front/post/feed 7/7 prima di dichiarare la migrazione: qualsiasi delta = divergenza di confine, non rumore).

Nota di merito: `preg_cache.clear()` è più conservativo di Zend — lecito, ma costo CPU per-richiesta da dichiarare in M4.

---

## VERBALE GREGG — CON EMENDAMENTI

(Il lavoro sui gate è metodologicamente solido — controllo positivo, guardia size>0, pin per nome. Ma le metriche M4 **non sono citabili** e il target "≤±2%" **non è misurabile**. Refuto il protocollo, non i verdetti.)

(1) **"352 ms/req" è wall di curl, non user CPU del server** — include TCP, kernel, scheduling, curl stesso. Regola di casa: "fede solo allo user CPU"; nessun `time -l` né campionamento nel report. Declassato a INDICATIVO. (2) **n=10 con outlier 1,45 s = code larghe non caratterizzate**: serve n≥30, warm-up dichiarato PER PROTOCOLLO, mediana+IQR, mai un numero secco. (3) **"350→367→350 su ~46 req" non è un trend, è rumore**: tre letture vmmap sparse nella banda ±35 MB nota da WP-52; "~46" è un conteggio non controllato — il workload di un metro si dichiara. La prova di casa è il ladder K-amplificato + contatori per-richiesta. (4) **"variance ≤±2%" è uno slogan**: non dichiara metro, stimatore, finestra di regime rispetto agli eventi del workload (lezione gradino-cron WP-69), workload, n. Un gate non definito non può né passare né fallire. (5) **Manca la coppia oracle stessa-sera**: 352 ms senza referente non dice se siamo a 2× o 10×. (6) **Lo scarto della prima passata è stato a discrezione**: la precondizione macchina-scarica deve vivere NEL runner (guard pgrep ⇒ VOID), non nella memoria dell'operatore.

**Emendamenti**: G-77.5.1 (protocollo M4 riscritto PRIMA del codice: user CPU/req = Δ(user CPU `proc_pid_rusage`/`time -l`) su K richieste note / K; wall curl solo secondario, mediana+IQR, n≥30, warm-up W dichiarato); G-77.5.2 (steady-state = spread user-CPU/req ≤±2% su **R≥3 boot** × K richieste, finestra dichiarata rispetto agli eventi WP; footprint = ladder K vs 10K con banda pre-dichiarata + tripla used_n/used_b, mai vmmap sparse); G-77.5.3 (coppia oracle stessa-sera obbligatoria per ogni cifra CPU pubblicata); G-77.5.4 (guard di contaminazione NEL runner: pgrep processi pesanti ⇒ VOID automatico); G-77.5.5 (ogni cifra dichiara unità e processo misurato; tabella per-bin — "4,3 MB placeholder" e "350 MB phpr -S" non confrontabili né sommabili).

**Kill-switch**: KS-G-77.5-A (spread user-CPU/req su R≥3 > ±2% ⇒ M4 FAIL, no medie post-hoc); KS-G-77.5-B (ladder 10K−K fuori banda pre-dichiarata ⇒ leak, blocco merge Vm-in-handler); KS-G-77.5-C (run con guard scattata ⇒ VOID; due VOID consecutivi ⇒ ambiente non idoneo, niente cifre).

---

## VERBALE CO-OPTATA WEB-RUNTIME — MI OPPONGO

(al disegno di integrazione WP-77.5 come implicito in M9 + main.rs; i verdetti di gate 77.4.2 accettati con riserva di etichetta.)

**Refutazione centrale**: M9 (`tokio::task_local!`) non regge WP-77.5. (1) **Vm quasi certamente `!Send`** (Rc pervasivi): non attraversa `spawn_blocking`, non sta in un task_local che migra, non entra in `Arc<Mutex<Vec<Vm>>>`. Il task_local attuale funziona solo perché contiene un u64 finto. (2) **PHP = 352 ms sincroni CPU-bound**: eseguirli nel corpo async blocca il worker del reactor — con N core, N richieste WP congelano *tutto* il server (health check, accept, keep-alive). Il disegno non contiene né spawn_blocking né pool dedicato: questo è il buco che WP-77.5 scriverebbe nel codice. (3) **task_local + spawn_blocking non si compongono**: la closure gira su un thread del blocking pool fuori contesto task; e il blocking pool tokio ha default **512 thread effimeri**: thread_local Vm lì = fino a 512 Vm × ~350 MB creati/distrutti non-deterministicamente. Falsifica M4 e il footprint per costruzione.

**Architettura corretta (emendamento vincolante)**: modello **worker-attore stile php-fpm** — N thread OS dedicati e longevi, ognuno **proprietario** della sua Vm (thread_local legittimo lì: non sono worker tokio; M9 vietava thread_local *sui worker tokio*); richieste inoltrate via canale bounded, risposta via oneshot. La Vm non attraversa mai un confine di thread; il backpressure è il canale.

**Emendamenti**: W-77.5.1 (rinegoziare M9 PRIMA del codice: "worker pool dedicato, Vm confinata al thread proprietario, mpsc bounded + oneshot"; verifica preliminare: test compile-fail `Vm: !Send` che documenti il fatto); W-77.5.2 (concorrenza limitata per costruzione: N worker = core fisici default, canale bounded cap N o 2N come semaforo, 503 su pieno; MAI spawn_blocking per PHP; `block_in_place` vietato); W-77.5.3 (contratto di risposta: `ResponseParts { status, headers, body }` da `http_response_code()`/`header()`/OB → axum Response; WP-77.5 = solo body bufferizzato; streaming/`flush()` NON-GOAL con fixture pinnata); W-77.5.4 (`request_end()` chiamato SUL thread worker, path OK e errore; panic PHP ⇒ catch_unwind + Vm ricostruita su poisoning, senza uccidere il pool); W-77.5.5 (etichetta onesta: KM-77-2 = "PASS (cli-server); PIENO=pending WP-77.5" nel NEXT_SESSION — un kill-switch mezzo-chiuso dichiarato chiuso è gate-inflation); W-77.5.6 (M4: la riga "axum 0,3 ms" non entra in nessun trend; baseline vera = post-Vm-nel-worker, in coppia con cli-server stessa-sera).

**Kill-switch**: KS-W77-A (N=4 worker, 8 curl parallele su WP front: nessuna richiesta oltre ~2× la mediana sequenziale E hello risponde <5 ms DURANTE il carico — prova che il reactor non è bloccato; FAIL ⇒ stop design); KS-W77-B (test compile-fail `Vm: Send` — se Vm risultasse Send l'assunto cade e si riconvoca: in entrambe le direzioni il fatto va PROVATO); KS-W77-C (100 richieste: footprint ±5% e #thread costante — crescita thread = blocking-pool infiltrato ⇒ FAIL).
