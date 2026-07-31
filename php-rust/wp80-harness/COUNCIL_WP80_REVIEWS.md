# COUNCIL_WP80_REVIEWS.md — Concilio a 9 sedie su S-78.1 (hardening+misura) + programma WP-79 (leva A-BB6)

**Data**: 2026-07-31 (chiusura S-78.1)
**Oggetto**: revisione di S-78.1 (commit 146a4c1…fedf497: ordine WP-79 8/8 + fase misura design78 + leak KS-DS-78-4 trovato/chiuso) e giudizio del programma proposto WP-79 (leva A-BB6 cache Module).
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto NEXT_SESSION, WP_SESSION_78_1, MEASURE78_RESULTS/design78, i gate, e il CODICE del proprio perimetro; Gregg ha campionato i raw log.
**Status**: verbali VINCOLANTI per WP-79 (blocco ⚖️ in NEXT_SESSION).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO CON EMENDAMENTI — nessuna opposizione.**
La sanatoria S-78.1 è giudicata reale su tutti i punti verificati sul codice
(fix RetainSet per-richiesta Zend-fedele — Stogov; regola capture/reset
rispettata — Pedersen; cifre = raw ×campione — Gregg; rigore dei gate
mantenuto — Klabnik). Il colpo duro è duplice: **un bug reale nell'osservabile
depth** (3 sedie indipendenti) e **il "99% fase a" NON è ancora il dato che
autorizza il design A-BB6** (3 sedie: va decomposto prelude/main/include).

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **Contatore depth FAIL-OPEN: inc DOPO send** (Hoare A-TH1, Bak A-BB13,
   Matsakis A-MS1): il worker può decrementare prima dell'incremento ⇒
   underflow usize e watermark che può SOTTO-contare — l'osservabile di
   KH78-2 può mancare la violazione che deve rilevare. Ogni claim depth-based
   (incluso il "depth_max=1 ovunque" di MEASURE78 §4) è ADVISORY finché il
   fix (inc-before-send + test anti-wrap) non è dentro
   [KH80-4, KB-80-4, KS-MS-80-3].
2. **Il "99% fase a" è un LETTO, non un canale** (Hoare A-TH2, Bak A-BB10,
   Klabnik A-SK5): su hello.php ~30 byte gli 80k call/13MB sono con ogni
   probabilità il PRELUDE, non il main; e la percentuale è artefatto della
   fixture minima (fase b banalmente vuota). Obbligo pre-design A-BB6:
   split fase a in (a1 prelude, a2 main, a3 include) + seconda fixture
   include-pesante/compute + citare ASSOLUTI, mai percentuali
   [KH80-2, KB-80-1, KS-SK-80-2].
3. **Chiave cache = FINGERPRINT del contenuto, MAI (path,mtime)** (Hoare
   A-TH3, Hejlsberg, Bak A-BB14, Pedersen A-PP6, Matsakis A-MS5, Stogov
   A-DS5): mtime a granularità grossolana = staleness servita in silenzio;
   la unit cache TL già usa fingerprint. **Forma fedele (Stogov A-DS5): NON
   una seconda cache — la STESSA unit cache TL estesa al main** (canonicalize
   + stat di meta.path; il contratto M-68.5 "main can never reach this put"
   va superato deliberatamente, audit M-67.2 rifatto)
   [KH80-1, KB-80-2, KS-DS-80-2].
4. **Verdetto A-BB6 = COPPIA (churn risparmiato, retained aggiunto), mai
   sola-churn** (Leijen A-DL1/KL-80-1, Hoare A-TH7, Gregg KG-80-2, Hejlsberg
   KS-AH-80-3, Matsakis A-MS5d): il 13MB/req è churn LORDO (realloc a taglia
   piena, dealloc non nettate — etichettare); la cache è thread-local ⇒
   retained ×W; obbligo: taglia retained profonda di un Rc<Module> misurata
   PRIMA, bound/eviction con contatore len osservabile, peak a W=num_cpus
   nell'A/B (lo spread 12% del picco W=10 È già metrica di verdetto per
   questa leva — Leijen A-DL4) [KL-80-1/2, KS-AH-80-3, KG-80-2].
5. **La build census è FUORI dal perimetro di identità** (Hejlsberg A-AH10/
   A-AH11, Klabnik A-SK7, Gregg A-BG15): la quarta configurazione non è nel
   feature-matrix né in CI, l'hash census 8f4146db non è nel log (le cifre §4
   auto-esentate dal proprio kill-switch KS-AH-78-1), il gemello strumentato
   non ha gate di equivalenza funzionale, e il driver stampa ma non ENFORCE
   (mismatch hash / depth-violation devono uscire ≠0)
   [KS-AH-80-1, KS-SK-80-4].
6. **Canale c senza controllo positivo** (Bak A-BB12, Gregg A-BG14): c=0 è
   indistinguibile da un contatore cieco finché una fixture teardown-allocante
   non lo muove (`c_calls>0`); inoltre i fatal di compile ritornano PRIMA dei
   contatori — righe census mancanti in silenzio (Pedersen A-PP5): asserire
   righe-attese==N [KB-80-5].
7. **gate-concurrent non PROVA la sovrapposizione** (Klabnik A-SK1; correlato
   Pedersen A-PP4): 24 richieste smaltite da UN worker darebbero lo stesso
   PASS — serve osservabile (depth>1 nel log o ≥2 worker con served≥1); e il
   panic nel task axum lato dispatcher è catturato da tokio (richiesta appesa,
   processo vivo = il "silently dead" di KS-PP-6 sull'altro lato)
   [KS-SK-80-1].
8. **Mina doc superstite + leak gemello** (Stogov A-DS1/A-DS2): la doc di
   request_end dice ancora "the only cross-request survivor is the caller-held
   RetainSet arena" (falsa post S-78.1.5 — il survivor è la unit cache TL) e
   il pattern non è nel gate; le STRANDED KEYS della unit cache (re-key su
   edit, TODO(port) noto) diventano crescita unbounded sul worker long-lived
   e WP-79 le moltiplica [KS-DS-80-1].
9. **Delta A/B +5,7MB peak = ADVISORY** (Gregg A-BG11): usa il peak del
   braccio cli-server che KG-79.B esclude dalle metriche exit-based —
   verdict-grade è il **+4,8M residente**; e lo spread W=4 (5,97%) andava
   flaggato come quello W=10; census.inc è R=1 non dichiarato (A-BG13)
   [KG-80-3].
10. **I test in-cargo normalizzano la forma bandita** (Matsakis A-MS2, Hoare
    A-TH5): RetainSet riusato tra richieste — innocuo oggi, leak-shaped con
    A-BB6 (il main verrà parcheggiato): migrarli a forma worker_loop PRIMA
    della leva.
11. **Detector lessicali da chiudere a censimento esatto** (Klabnik A-SK2/
    A-SK3, Pedersen A-PP1/A-PP3): NSITES==4 pinnato (≠0 non basta), verbi
    completi (assegnamento, write!, mem::swap/take), tandem che esige
    SCRITTURA su rendered, marker pinnato per POSIZIONE (cfg(test) +
    is_empty()), scan esteso ai crate che chiamano request_end quando A-TH4
    atterra [KS-SK-80-3, KS-PP-80-1].

### Ordine vincolante di apertura WP-79 (S-79.0 "pre-lever hardening", poi design A-BB6)

1. **Fix depth**: incremento PRIMA di send + test che il wrap sia impossibile
   (A-TH1/A-BB13/A-MS1) [KH80-4]. Reset del watermark nei test (A-SK8).
2. **Doc purge**: frase request_end + pattern nuovi nel gate-doc-purge
   (A-DS1); SAFETY inline su AssertUnwindSafe (A-MS4).
3. **Census hardening**: split a1/a2/a3 + seconda fixture include-pesante +
   riga estesa col residuo Δglobale−(a+b+c) (A-BB10/A-BB11) + controllo
   positivo canale c (A-BB12/A-BG14) + righe-attese==N (A-PP5) + finestra
   idle per il rumore cross-thread (A-AH13/A-DL5) + census.inc a R≥3
   (A-BG13).
4. **Identità quaterna**: census build nel feature-matrix e in CI con -D
   warnings, hash nel log (A-AH10); gate di equivalenza funzionale del
   gemello (run-gate sul binario census + grep-gate sui blocchi cfg census —
   A-AH11); driver ENFORCE (EXPECTED_HASH ∈ matrix-log, exit≠0 su
   depth-violation — A-SK7/A-BG15/A-AH12).
5. **Gate hardening**: concurrent con osservabile di sovrapposizione (A-SK1);
   stdout-tandem NSITES==4+verbi+tandem-scrittura (A-SK2/A-PP3); marker per
   posizione (A-PP1); worker-panic RC!=134 ⇒ FAIL + "all N workers joined"
   pinnato (A-SK4); trigger panic anche nel handler axum con verdetto
   dichiarato (A-PP4).
6. **Test in-cargo a forma worker_loop** (A-MS2/A-TH5) + emissione riga
   census sul path cli-server per rendere A-BB1 giudicabile (A-BG16,
   promosso da residuo a prerequisito).
7. **Stranded keys unit cache**: supersede-per-path alla put o cap, con
   contatore osservabile (A-DS2) [KS-DS-80-1].
8. **SOLO POI design A-BB6** — vincoli consolidati: forma fedele = unit cache
   TL estesa al main (A-DS5, canonicalize+stat; niente seconda cache);
   fingerprint (mai mtime); Module cached pinnato nel RetainSet della
   richiesta in esecuzione (A-PP6.2/KS-PP-80-2); immutabilità del Module
   condiviso asserita nel type system o gateata (A-TH4-Hoare); niente cache
   di esiti fatal/parse (A-PP6.4); predizione ex-ante dei contatori a caldo
   (KG-80-1); fixture A-DS6 (edit, same-size, symlink/canonicalize pinnato,
   include condizionale, hit-counter positivo, doppio include non-once);
   A/B: coppia churn/retained + peak W=num_cpus + CPU (A-BB15/KL-80-2) +
   corpus per NOME; su qualunque body ≠ oracolo ⇒ REVERT, mai fix-forward
   (KS-DS-80-3).

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH80-1 / KB-80-2 | cache HIT con contenuto su disco cambiato (o chiave sola-mtime) | leva RESPINTA |
| KH80-2 / KB-80-1 / KS-SK-80-2 | design/verdetto A-BB6 senza split a1/a2/a3 o fondato su "99%" senza seconda fixture | respinto d'ufficio |
| KH80-3 / KL-80-1 | verdetto footprint leva senza coppia (churn, retained-per-worker a W>1) | VOID |
| KH80-4 / KB-80-4 / KS-MS-80-3 | claim depth-based con ordering inc-dopo-send | ADVISORY / VOID |
| KS-MS-80-1 | unsafe impl Send/Sync o rimozione static assert senza delibera | REJECT, build rossa |
| KS-MS-80-2 | post-A-BB6: retain.len()/req > unit distinte + main | FATAL (riedizione KS-DS-78-4) |
| KS-SK-80-1 | PASS concurrent senza osservabile di sovrapposizione | vacuo, census bloccato |
| KS-SK-80-3 | detector lessicale senza census esatto dei siti | gate non conforme |
| KS-SK-80-4 / KS-AH-80-1 | cifra da binario con hash assente dal matrix-log (quaterna) | NULLA |
| KS-AH-80-2 | warm-hit verificato a sola auto-uguaglianza | VUOTO (full-body vs oracolo) |
| KS-AH-80-3 | cache senza eviction + len osservabile | leva respinta per costruzione |
| KS-AH-80-4 | fase a steady post-leva non cala della soglia dichiarata ex-ante | leva VOID, niente credito parziale |
| KB-80-3 | census senza riconciliazione del residuo | denominatore VOID |
| KB-80-5 | "c=0" citato senza controllo positivo del canale c | contatore presunto cieco |
| KS-PP-80-1 | marker allow fuori cfg(test) o senza is_empty() | gate FAIL |
| KS-PP-80-2 | cache unico owner di un Module durante richiesta in-flight | reject della leva |
| KS-PP-80-3 / KS-AH-80-2 | verdetto su cache-HIT senza cmp full-body vs oracolo | run VOID |
| KL-80-2 | A/B leva senza peak a W=num_cpus, o picco peggiorato >2% passato in silenzio | leva non passa |
| KL-80-3 | claim "nessun leak" senza floor di risoluzione dichiarato | respinto |
| KS-DS-80-1 | superseded_entries unit cache monotono crescente su workload di edit | FAIL |
| KS-DS-80-2 | leva senza le 6 fixture A-DS6 verdi | leva nulla |
| KS-DS-80-3 | body ≠ oracolo su stateful/include post-cache-main | REVERT, mai fix-forward |
| KG-80-1 | A/B leva senza predizione ex-ante dei contatori a caldo | VOID |
| KG-80-3 | cifra peak da braccio senza uscita pulita conforme | ADVISORY d'ufficio |

---

## VERBALI INTEGRALI

### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI**

Verifica sul codice eseguita: il fix RetainSet per-richiesta È in `worker_loop` (worker_pool.rs:224, fresh per iterazione); il FAIL-FAST È `catch_unwind`+`abort` (righe 155-166) con trigger armato solo da env; il body = solo `rendered` (riga 447, prima di `request_end`). I claim strutturali del verbale corrispondono al codice. Le dichiarazioni di NON-fatto (A-BB1, decomposizione +5,7MB, spread W=10) sono oneste. Ma refuto quattro punti sostanziali:

**A-TH1 — Il contatore depth è fail-open: l'osservabile di KH78-2 può sotto-contare.** In `dispatch` l'incremento avviene DOPO `send` (righe 262-271); il worker decrementa al pickup (riga 214). Il worker può decrementare PRIMA dell'incremento: `fetch_sub` su 0 ⇒ underflow a `usize::MAX`, il successivo `fetch_add` rientra wrappando e `note_depth` può registrare un valore che maschera la profondità reale (in build debug: panic da overflow). Un kill-switch il cui sensore può mancare la violazione non è un kill-switch. Fix: incremento PRIMA di `send`; il "depth_max=1 ovunque" di MEASURE78 resta plausibile (meccanismo curl sequenziale dichiarato) ma il ✓ su KH78-2 va declassato a corroborato-dal-meccanismo finché il contatore non è sound.

**A-TH2 — Il census NON sblocca il design A-BB6 come proposto: la fase a è un letto, non un canale.** `census_s0→s1` ingloba lower+compile di TUTTO (MEASURE78 stesso scrive "prelude+script"). `hello.php` è ~30 byte: 80.476 call/13MB non sono il main script, sono con ogni probabilità il PRELUDE. La proposta WP-79 dichiara la leva "riuso del Module del MAIN script" sul 99% — ma il 99% potrebbe appartenere quasi tutto al prelude, e la cache del main varrebbe epsilon. Obbligo pre-design: split della fase a in (a1 prelude, a2 main, a3 include), con predizione-misurata sul sotto-canale che la leva attacca (lezione WP-71: falsificare la famiglia nel letto).

**A-TH3 — `(path,mtime)` è una chiave più debole del fingerprint già in uso.** La unit cache TL è fingerprinted; mtime ha granularità grossolana (edit nello stesso secondo ⇒ staleness servita). O si usa il fingerprint come la macchineria esistente, o la fixture di invalidazione deve includere il caso edit-same-second.

**A-TH4 — L'immutabilità del Module condiviso cross-request non è nel type system.** L'isolamento regge perché statics/props vivono nel Vm (A-DS2, verificato) — ma un `Rc<Module>` riusato N volte richiede che Module sia privo di interior mutability, oggi non asserito da nulla. Lezione WP-78: l'invariante va nel compilatore, non nel commento.

Emendamenti minori:
**A-TH5** — I gate in-cargo (gate_apertura2, stateful, fatal, method-statics) girano su RetainSet RIUSATO tra richieste: forma che in produzione non esiste più. Accettabili come stress più severo, ma va dichiarato; almeno uno per famiglia in forma-worker_loop (oggi solo `include_units_pinned…` lo è).
**A-TH6** — "Epilogo ESATTO del main": condiviso è solo `link_fatal_check`; il match su `run_result` (execute_with_retain:407-432) è duplicazione a specchio non gateata strutturalmente — convergere sull'estrazione A-TH4 deferita, non accumulare drift.
**A-TH7** — "k dominante = copia Registry" è un ATTESO, non un misurato; e una cache Module thread-local moltiplica il retained ×W: la leva scambia churn per retained — il costo retained per worker della cache va predetto e dichiarato PRIMA dell'A/B.
**A-TH8** — `std::fs::read` bloccante nel task async (main.rs:135): wart tollerabile oggi, da sanare prima di qualunque misura di concorrenza verdict-grade.

Kill-switch:
- **KH80-1**: cache HIT che serve un Module quando il contenuto su disco è cambiato (fingerprint o fixture edit) = leva RESPINTA, non "da raffinare".
- **KH80-2**: design A-BB6 presentato senza split a1/a2/a3 del census = respinto d'ufficio (il 99% aggregato non è un canale).
- **KH80-3**: verdetto footprint della leva senza cifra retained-per-worker della cache a W>1 = VOID.
- **KH80-4**: qualunque futuro claim KH78-2 basato sul contatore depth nell'ordine attuale (inc-dopo-send) = ADVISORY finché il fix A-TH1 non è dentro con test che il watermark non possa sotto-contare.

### 2. Matsakis — ownership/aliasing/Send-Sync — CONCORDO CON EMENDAMENTI

**Verifiche eseguite sul codice.** `assert_not_impl_any!(RetainSet: Send, Sync)` presente (vm/mod.rs:435) con commento Council-only; `assert_impl_all!(WorkerTask: Send)` presente (worker_pool.rs:84). Il fix per-richiesta è reale: `RetainSet::new()` dentro il `while let` di `worker_loop` (riga 224), Vm che lo prende in prestito (`vm_new(retain, …)`) ⇒ drop order Vm→RetainSet garantito dal borrowck, non da prosa. `park` = `FrozenVec::push_get` su `Rc<Module>` (StableDeref), zero `unsafe`. `shutdown()` consuma il pool, droppa i sender e joina davvero. Nessun `panic = "abort"` nei profili (verificato: `catch_unwind` è operante). Il claim S-78.1.5 regge nel perimetro ownership.

**VERDETTO: CONCORDO CON EMENDAMENTI.**

**A-MS1 — Protocollo del contatore depth RACY (finding).** In `dispatch` l'incremento di `QUEUE_DEPTH` avviene **DOPO** `send(task)` (worker_pool.rs:262-271); il decremento avviene al pickup (riga 214). Il worker può decrementare **prima** dell'incremento del dispatcher: underflow `usize` (wrap a MAX) e, peggio, il watermark può **sottocontare** una profondità reale ≥2. KH78-2 usa `depth_max=1` come prova di sequenzialità: un osservabile che può sottocontare è un VOID-detector cieco. Fix obbligato: inc **prima** di send, dec dopo recv.

**A-MS2 — I test in-cargo normalizzano la forma bandita.** `gate_apertura2`, `stateful_static_counter`, `fatal_maps…`, `exit_…`, `method_…` riusano **UN** RetainSet su 2-4 richieste — esattamente la shape che KS-DS-78-4 ha dichiarato leak. Oggi innocuo (len resta 0: nessun include), ma con A-BB6 il main verrà **parcheggiato** ⇒ quei test diventano leak-shaped e le loro aspettative cambiano silenziosamente. Portarli alla forma worker_loop (fresh per richiesta) **PRIMA** della leva A-BB6.

**A-MS3 — Il pin request-lifetime è disciplina del caller, non type system.** `execute_with_retain` è `pub` e prende `&RetainSet`: qualunque futuro SAPI può tenerne uno a vita lunga e il compilatore non obietta — la lezione S-78.0 ("l'invariante va nel type system") non è chiusa. Sigillare: costruzione del RetainSet **dentro** la funzione di esecuzione (come `run_module_with_hir`), oppure newtype non riusabile; in subordine grep-gate sui call-site.

**A-MS4 — AssertUnwindSafe senza SAFETY inline.** La giustificazione implicita (Err ⇒ `abort()`, nessuno osserva stato rotto) è valida ma non scritta **sul punto d'uso**; e l'abort arriva DOPO l'unwind: i distruttori di Vm/RetainSet della richiesta in corso girano su invarianti potenzialmente rotti prima dell'abort (panic-in-drop, blocchi). Fail-fast più forte: panic-hook che aborta (niente unwind). Minimo: SAFETY comment + feature-matrix che pinna l'assenza di `panic="abort"` (oggi vera, non gateata).

**A-MS5 — Programma A-BB6, vincoli di ownership.** (a) Proprietario = cache **thread-local Rc-owned**: `Rc<Module>` è `!Send`, il compilatore lo forza — giusto così, mai Arc. (b) Invalidazione per **content fingerprint** (il meccanismo della unit cache), non `(path,mtime)`: la granularità mtime buca le riscritture same-second e la fixture proposta non le vede. (c) Il main deve transitare `park_module` — M-67.1 lo dichiara "unique constructor" e il main "the one exception": A-BB6 abroga l'eccezione, l'audit M-67.2 va rifatto, niente secondo path. (d) Bound/eviction per-worker (W cache moltiplicano il k=12-13M/worker). (e) `gate_stateful` e A-DS9 devono girare sul path **cached**.

**A-MS6 — Registry condivisa (deferita):** `Arc<Registry>` esige `Send+Sync` **veri**; vietato ogni `unsafe impl`.

**Kill-switch:**
- **KS-MS-80-1**: qualsiasi `unsafe impl Send/Sync` o rimozione degli assert compile-time senza deliberazione = REJECT, build rossa.
- **KS-MS-80-2**: post-A-BB6, `retain.len()` per richiesta > (unit distinte + 1 main) ⇒ FATAL (riedizione KS-DS-78-4); aggiornamento del gate A-DS8 dichiarato per NOME, mai silenzioso.
- **KS-MS-80-3**: cifra di misura il cui `depth_max` proviene dal protocollo inc-dopo-send = VOID finché A-MS1 non è chiuso.

### 3. Klabnik — spec/testabilità/gate — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.**

L'opposizione WP-78 è stata onorata: i gate ora eseguono, mordono (leak RetainSet trovato al primo colpo, mina doc n.12), portano comando+hash+git nei verdetti, hanno esche positive e assert assoluti (gate_apertura2 pinna `b"hello axum\n"`, fatal full-body, retain.len()==1 esatto). Il rigore è rimasto. Ma ho trovato buchi reali:

**A-SK1 — gate-concurrent non PROVA la concorrenza.** Le 24 curl partono in background, ma nulla verifica che siano state IN VOLO insieme né che ≥2 worker abbiano servito: un singolo worker che smaltisce 24 richieste sequenziali produce lo stesso PASS (tutti i body == oracolo). Il contratto A-SK6 chiede "in flight concurrently": serve un osservabile — depth watermark >1 registrato nel log server, o contatori per-worker served≥1 su ≥2 worker, o fixture con sleep che forza la sovrapposizione.

**A-SK2 — gate-stdout-tandem tre volte aggirabile.** (a) La lista verbi è chiusa: l'assegnamento diretto `vm.stdout = …`, `mem::swap/take` sfuggono; (b) il "tandem" è soddisfatto da QUALUNQUE `.rendered.` entro ±3 righe, anche una LETTURA (`.rendered.len()` benedice un sito senza mirror); (c) il conteggio siti verifica solo ≠0, non il census esatto — a differenza di capture-order (ALLOW_CENSUS=1 pinnato). Pinnare NSITES=4 con bump same-commit.

**A-SK3 — gate-capture-order eludibile via wrapper.** Un metodo in php-runtime che legge `rendered` internamente (`fn take_output(&mut self)`) chiamato post-reset in php-server evade il gate lessicale; inoltre lo scope è solo `crates/php-server/src`, mentre il braccio `--cli-server` e futuri SAPI chiamano `request_end` altrove. Emendo: censire i call-site di `request_end(` cross-crate e scannare ogni crate che ne contiene.

**A-SK4 — gate-worker-panic ammorbidito.** Exit≠134 è declassato a WARN (basta la riga di log); e `grep -q "workers joined"` non pinna N — "all 0 workers joined" passerebbe. RC≠134 = FAIL; pinnare "all 2 workers joined".

**A-SK5 — il "99% fase a" NON è ancora il dato che sblocca A-BB6.** 13MB/80k call per un hello di 221 byte significa che il canale è dominato da prelude/stdlib, non dal main script. Il census attribuisce a una FASE, non al bersaglio della leva: "cache del Module del MAIN" potrebbe intercettare una frazione minima. Prima del design: decomposizione fase a in prelude / main / include (predizione-misurata, WP-45).

**A-SK6 — matrice fixture WP-79 punto 2 incompleta.** Oltre a mtime-invalidation e include condizionale: (a) modifica same-second (granularità mtime batte la chiave (path,mtime) — la chiave deve includere size/hash); (b) identità path (symlink/relativo → doppio Module?); (c) fixture negativa: il Module riusato non deve trattenere stato per-richiesta (statics/define/`__FILE__`); (d) corpus per NOME obbligatorio (la leva tocca il compile path).

**A-SK7 — measure78.sh stampa ma non ENFORCE.** Il driver cita l'hash (e l'incidente Tier-0 dimostra che serve), ma non verifica che i verdict-file dei gate portino lo STESSO bin-hash del run; la violazione depth>1 stampa "VOID" ed esce 0. Il driver deve rifiutarsi di partire su mismatch e uscire ≠0 su depth-violation.

**A-SK8 — census_queue_depth_burst:** il watermark globale non è azzerato a inizio test — PASS spurio possibile da traffico di altri test. Reset esplicito.

**Kill-switch:**
- **KS-SK-80-1**: PASS di KH78-1/concurrent senza osservabile di sovrapposizione (depth>1 o ≥2 worker con served≥1) = vacuo, census bloccato.
- **KS-SK-80-2**: design o verdetto A-BB6 fondato su "99% fase a" senza decomposizione prelude/main/include = respinto d'ufficio.
- **KS-SK-80-3**: detector lessicale senza census esatto dei siti (≠0 non basta) = gate non conforme.
- **KS-SK-80-4**: cifra di misura il cui verdict-gate porta bin-hash diverso dal run = nulla (KS-AH-78-1 diventa enforcement, non convenzione).

### 4. Hejlsberg — compilazione incrementale/feature-gating/igiene di build — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI** (numerazione prosegue da A-AH9 per evitare collisioni).

S-78.1 è solida: la terna nel gate è la matrice vera del Cargo union-resolver, `-D warnings` copre anche i test target (A-AH6 ha già morso), il controllo positivo nm sull'union è corretto, l'identità del binario ha smascherato un run nullo (KS-AH-78-1 ha funzionato). Ma trovo QUATTRO buchi.

**A-AH10 — `census-instrumentation` è FUORI dalla matrice: è un BUCO, non una scelta.** La feature esiste in Cargo.toml ma né gate-feature-matrix.sh né ci.yml la compilano: la quarta configurazione (union+census) può bit-rottare in silenzio fino al giorno della misura, e non è mai passata sotto `-D warnings`. Peggio: KS-AH-78-1 dichiara nulla ogni misura senza feature-matrix.log, ma l'hash census 8f4146db NON è in quel log — le cifre §4 di MEASURE78 sono formalmente auto-esentate dal proprio kill-switch. Rimedio: quarta riga `build "census" --features census-instrumentation` nel gate e in CI, hash nel log.

**A-AH11 — Il gemello non ha un gate di equivalenza funzionale.** KB-78-5 vieta cifre dal census, ma nulla garantisce che il build strumentato sia funzionalmente il gemello: run-gate, gate-concurrent e capture-order sono girati SOLO su 5fdc9716. Un `cfg(census-instrumentation)` che altera un path (non solo conta) produrrebbe contatori di un programma diverso. Rimedio: run-gate full-body vs oracolo ANCHE sul binario census, più grep-gate sui blocchi `cfg(feature = "census-instrumentation")` (solo snapshot/counter/eprintln ammessi, censimento per marker come KS-PP-4). Dichiarare inoltre la semantica di CountingAlloc: `realloc` conteggia `n` lordo e `dealloc` non conta — è churn lordo, va scritto nel report.

**A-AH12 — Identità cargo build vs cargo rustc non asserita.** Il matrix-gate compila con `cargo rustc … -D warnings`, run-gate col json artifact di `cargo build`: fingerprint diversi, l'uguaglianza degli hash oggi è fortuna riproducibile, non un assert. Rimedio: il driver di misura deve verificare `hash(BIN misurato) ∈ feature-matrix.log` automaticamente (grep sul log, FAIL se assente); inoltre lo step 4 del gate (cargo test) gira DOPO l'union build — asserire che l'hash su disco sia invariato a fine gate.

**A-AH13 — Rumore di fondo dei contatori globali.** I contatori sono globali Relaxed; a --workers 1 il runtime tokio alloca comunque fuori fase. Serve il controllo positivo inverso: finestra idle (nessuna richiesta) con drift dichiarato ~0, altrimenti la fase a assorbe rumore altrui.

**Su A-BB6 (WP-79):** il canale è giusto (99% = ricompilazione), ma **(path,mtime,size) è invalidazione DEBOLE e inutile**: il handler legge già `source` da disco a ogni richiesta (main.rs, `std::fs::read`) — la chiave corretta è il **fingerprint del contenuto**, che la unit cache TL già usa (WP-63/66). L'opzione "unit-cache anche per il main" è l'unica coerente; mtime dissolve, e con essa la fixture stessa-mtime-stesso-size-contenuto-diverso che altrimenti la falsificherebbe. Gates PRIMA della leva: (a) fixture contenuto-cambiato (via fingerprint, non mtime); (b) include condizionale alternante; (c) fixture stale-IC: se Module contiene interior mutability (PropIc/MethodIc, WP-29+), il riuso cross-request con id riassegnati (lezione WP-28 next_id) è la superficie vergine che morde; (d) bound: la cache DEVE avere eviction e un contatore len osservabile — un'arena senza lookup-bound è il leak KS-DS-78-4 in altra veste; (e) predizione DICHIARATA prima: fase a hello → ~0 call su richiesta ≥2.

**Kill-switch:**
- **KS-AH-80-1**: cifra (anche di contatori) da un binario il cui hash non è nel feature-matrix.log ESTESO alla quaterna = NULLA.
- **KS-AH-80-2**: warm-hit verificato solo per auto-uguaglianza col cold di phpr = VUOTO; il warm si confronta full-body vs ORACOLO.
- **KS-AH-80-3**: cache Module senza eviction + contatore len esposto = leva RESPINTA per costruzione.
- **KS-AH-80-4**: post-leva, se fase a steady non cala della soglia dichiarata ex-ante, leva VOID — niente credito parziale.

### 5. Bak — alloc-rate/code-cache/path caldi — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI** (numerazione da A-BB10: A-BB1..9 sono già assegnati).

La misura è la più pulita mai prodotta dal filone (righe intero-esatte, prova regina N/2N, identità binario). Ma il verdetto "99% = compile" contiene tre difetti di attribuzione e un bug nell'osservabile, e il programma A-BB6 va corretto prima del design.

**1. Il "99%" è un artefatto della fixture, il dato vero è l'ASSOLUTO.** Su hello.php la fase b è banalmente vuota (730 call): QUALSIASI costo fisso in a domina in percentuale. Gli 80k call/13MB sono il ricompile di prelude+main — costo fisso per-richiesta, reale e invariante. Ma su WordPress la fase b (esecuzione reale) sarà 10-100× quella di hello, e la quota compile potrebbe scendere al 10-50%. La leva A-BB6 resta giusta (è il classico compilation-cache miss che V8 chiuse col code cache: input identico ricompilato a ogni richiesta), ma va dimensionata sull'assoluto 80k/13MB, mai sul "99%".

**2. Il denominatore del census è bucato: a+b+c ≠ alloc/req.** Tra s3 e s0 successivo cadono, NON contati: `fs::read` del sorgente (worker_pool.rs riceve `meta.source` già allocata), `path.to_string`, `format!(file_path)`, WorkerTask, canale oneshot, invio risposta (main.rs:127-178). L'esclusione del self-traffic del log per costruzione (lezione WP-64, corretta) esclude anche traffico server REALE — che è esattamente il canale A-BB1 dichiarato non giudicabile. Riconciliazione obbligatoria: residuo = Δglobale/req − (a+b+c), riportato per riga.

**3. c=0 non ha controllo positivo di CANALE.** MEASURE78 §4 dichiara "il canale c è coperto dal controllo positivo dei contatori globali": FALSO come garanzia — i contatori globali provano che il wrapper conta, non che la finestra s2→s3 bracketta il codice inteso. Serve una fixture il cui teardown ALLOCA (`register_shutdown_function` che costruisce stringhe, `__destruct` che alloca) con assert `c_calls>0`. Nota: c=0 su hello è plausibile (teardown = free puri, dealloc non conteggiate), ma plausibile ≠ verificato.

**4. Bug d'ordinamento nell'osservabile depth (worker_pool.rs:256-274 vs :212-214).** `dispatch` fa `send` PRIMA di `fetch_add`; il worker fa `fetch_sub` al pickup. Finestra: il worker decrementa prima che il dispatcher incrementi → underflow usize (wrap a MAX, che si auto-sana ma può corrompere o far mancare il watermark). KH78-2/KB-78-2 poggiano su uno strumento che può sotto-riportare burst reali. Fix: incremento PRIMA del send.

**5. CPU non misurata (R-G4 non commissionato).** La roadmap è CPU 2,06×; A-BB6 è motivata SOLO da churn alloc. 80k call sono ~ms, non è detto siano il collo CPU. Dichiararla leva-alloc con ipotesi CPU da verificare nell'A/B.

**6. Chiave cache: fingerprint, non (path,mtime).** Il sorgente è già letto per-richiesta nell'handler: hash del contenuto = chiave naturale, invalidazione esatta, zero race mtime (granularità/clock). È il meccanismo che la unit cache TL già usa — riusarlo.

Emendamenti: **A-BB10** census su SECONDA fixture include-pesante+compute (proxy WP) prima del design; solo assoluti. **A-BB11** riga census estesa col residuo gap. **A-BB12** controllo positivo canale c (fixture teardown-allocante, `c_calls>0`). **A-BB13** fix ordering QUEUE_DEPTH (inc-before-send) + test anti-wrap. **A-BB14** chiave = fingerprint; gate same-mtime-contenuto-diverso ⇒ MISS; gate stateful sul path HIT; retain_len bounded. **A-BB15** A/B di A-BB6 riporta anche CPU (time -l).

Kill-switch: **KB-80-1** verdetto A-BB6 che cita "99%" senza seconda fixture = VOID. **KB-80-2** cache a sola mtime = REJECT. **KB-80-3** census senza riconciliazione del residuo = denominatore VOID. **KB-80-4** claim depth-based con ordering non fixato = ADVISORY. **KB-80-5** "c=0" senza A-BB12 = contatore presunto cieco.

### 6. Pedersen — disciplina di confine per-richiesta, lifecycle — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI**

Verificato sul codice (`crates/php-server/src/worker_pool.rs`): l'epilogo rispetta la regola permanente — `mem::take(&mut vm.rendered)` (r.447) precede `request_end()` (r.451); epilogo fatal fedele al main (render_fatal dopo flush_diags, exit→200, exception-handler→200 senza banner); fix RetainSet per-richiesta corretto nel worker_loop; il gate A-DS8 ha morso su superficie vergine — lezione confermata. Ma REFUTO i seguenti punti.

**A-PP1 — Il census del marker pinna il CONTEGGIO, non la POSIZIONE.** Spostare l'unico marker sanzionato dalla riga del test (worker_pool.rs:571) a una capture reale di produzione mantiene NMARK==1: il gate passa sanzionando una violazione vera. Pinnare anche la forma della riga marcata (deve contenere `is_empty()`, sotto `#[cfg(test)]`).

**A-PP2 — Grep-gate aggirabile per INDIREZIONE.** Un helper `vm.take_output()` definito in php-runtime e chiamato dopo `request_end()` non contiene `.rendered` lessicale; idem un alias `let r = &mut vm.rendered` pre-reset letto post-reset. Verificato con Serena: `request_end` è definito solo in vm/mod.rs:3440 e chiamato solo in php-server — lo scope dello scan oggi è adeguato, ma A-TH4 (`request_end(self)→CapturedOutput`) lo romperà: KH79-1 copre la firma, NON lo scope del find. Prioritizzare A-TH4 (chiude l'aggiramento nel type system); al suo arrivo estendere lo scan a php-runtime nello stesso commit.

**A-PP3 — gate-stdout-tandem: tre buchi.** (a) Verbi non censiti: `.stdout = x` e `write!(self.stdout, …)` non matchano il pattern — oggi i 4 siti sono tutti `extend_from_slice` (vm/mod.rs:5390,5489, host.rs:6190, ini.rs:657), ma un quinto sito con verbo diverso è invisibile; (b) NSITES pinnato solo ≠0: pinnarlo ==4 con bump-in-commit come ALLOW_CENSUS; (c) il tandem accetta QUALSIASI `.rendered.` entro ±3 righe, anche una LETTURA (`.rendered.len()`): richiedere verbo di scrittura anche sul lato rendered.

**A-PP4 — FAIL-FAST ha edge non gateati.** `catch_unwind` copre SOLO worker_loop. Un panic nel task axum (handler HTTP, dispatch, parsing) è catturato dal runtime tokio: il task muore, il processo SOPRAVVIVE, la richiesta resta appesa — il "silently dead" che KS-PP-6 vieta, sul lato dispatcher. Gateare con trigger armabile nel handler axum (stessa env var, path distinto), verdetto dichiarato (abort o 500 pulito). Secondo edge: panic worker DURANTE `shutdown()` (tra drop dei sender e join) → abort corretto ma join line assente; gate-worker-panic Phase B non lo copre — almeno dichiararlo.

**A-PP5 — Buchi del census sul confine.** Il blocco census (r.456-474) legge `retain.len()`/`vm.live_objects()` post-reset: lecito (non è capture di output) ma non censito da nessun gate. Più grave: i fatal di lower/compile ritornano PRIMA dei contatori — REQUESTS non incrementa, la riga census manca in silenzio. Il protocollo misura deve asserire righe-census-attese == N.

**A-PP6 — WP-79 cache Module: proprietà da gateare PRIMA della leva.** (1) Invalidazione per FINGERPRINT del contenuto, MAI mtime (granularità 1s = stale silenzioso; la unit cache TL già usa fingerprint — stessa disciplina). (2) Il Module cached DEVE essere pinnato nel RetainSet della richiesta (regola 3): la ways-eviction durante una richiesta in-flight non deve mai lasciare la cache unico owner del Module in esecuzione. (3) Cache = solo bytecode/immutabile: rilanciare gate_stateful, A-DS9 e include_units_pinned sul percorso HIT. (4) MAI cachare esiti fatal/parse; fixture fatal→fix-del-file→200. (5) HIT confrontato col full-body dell'ORACOLO, mai solo HIT==MISS. (6) `vm_new` riceve anche `Some(&program)`: il design dichiari cosa si cacha (Module+program o refactor della firma).

**Kill-switch:**
- **KS-PP-80-1**: marker allow fuori da `#[cfg(test)]` o su riga senza `is_empty()` ⇒ gate FAIL.
- **KS-PP-80-2**: cache cross-request unico owner di un Module durante richiesta in-flight ⇒ reject della leva.
- **KS-PP-80-3**: verdetto su cache-HIT senza cmp full-body vs oracolo ⇒ run VOID.

### 7. Leijen — allocatore mimalloc/footprint fisico — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.**

Verifiche positive svolte: il gemello resta mimalloc puro (`main.rs:41-43`, `cfg(not(census-instrumentation))` → `MiMalloc` diretto; il wrapper contatore inoltra a `MiMalloc` senza cache proprie); `MIMALLOC_PURGE_DELAY=0` è applicato a TUTTI i modi dal driver (measure78.sh:52, unico punto di lancio); il peak `time -l` misura solo il processo server — i curl sono figli della shell del driver, non di `time`, e i worker sono thread, quindi inclusi. Protocollo del driver difendibile. La chiusura KS-DS-78-4 (RetainSet per-richiesta) è la decisione giusta: un'arena write-only ha come unica semantica il lifetime. Ma il verbale non è una benedizione: le cifre a supporto di WP-79 hanno quattro difetti.

**A-DL1 — churn ≠ retained (bloccante per il design A-BB6).** Il "13MB/req" è alloc LORDO: il wrapper non netta i dealloc e conta `realloc` a taglia piena (`main.rs:83-87`), quindi le catene di crescita (PhpStr growable, WP-57) sono contate più volte. Etichettare la cifra "gross churn, upper bound". Il beneficio della cache Module è churn−; il costo è **retained+ × W** (cache thread-local ⇒ duplicata per worker). Prima del design: misurare la taglia RETAINED profonda di un `Rc<Module>` (prelude+main) con contatore dedicato e controllo positivo. Su WordPress (centinaia di file × W worker) il retained può essere centinaia di MB: budget e eviction (ways) dimensionati su frequenza×taglia del corpus WP, non su hello.php.

**A-DL2 — la prova regina ha un floor di risoluzione non dichiarato.** V2(100)=V2(200)=18,0M esclude leak ≥~1KB/req (granularità vmmap 0,1M su 100 req), non leak sub-KB — che su WP (centinaia di include/req) si amplificano. Inoltre §3 non riporta V1 per N=200, e "assestamento che plateaua" è un'etichetta, non un meccanismo: nominarlo (crescita segmenti/arene mimalloc?) via diff MIMALLOC_SHOW_STATS o diff regioni vmmap V1↔V2.

**A-DL3 — k=12,4-13,1M/worker: attribuzione affermata, mai misurata.** "Dominante = copia Registry" è plausibile ma la unit cache TL persiste anch'essa per worker ed è candidata. Probe richieste: (i) V-idle(W) a 0 richieste post-spawn (separa stack+canali dall'init lazy); (ii) contatore census bytes-registry-init per worker con predizione W·k_reg confrontata al fit; (iii) watermark unit-cache per worker dopo 100 req.

**A-DL4 — spread 12% picco W=10 liquidato troppo in fretta.** Il picco multi-worker È la metrica fisica di produzione di questo progetto (il headline è peak FULL), e il transitorio di primo-compile concorrente è esattamente la superficie che la cache Module modifica: non "se diventa metrica di verdetto" — lo È già per WP-79, come endpoint A/B. Inoltre KB-78-4 dice "oltre 2% si indaga": qui non si è mediato (bene) ma nemmeno indagato.

**A-DL5 — bleed cross-thread nei diff di fase.** I contatori sono globali: il thread router alloca concorrentemente alle finestre a/b/c del worker (accept, parsing, oneshot). Con depth=1 il rumore è piccolo ma non zero: quantificarlo (census su pool idle / tier0) e dichiararlo accanto al 99%.

**Kill-switch:**
- **KL-80-1**: nessun verdetto sulla leva A-BB6 senza la COPPIA (churn risparmiato, retained aggiunto) misurata sullo stesso run; un verdetto sola-churn è nullo.
- **KL-80-2**: l'A/B di WP-79 include obbligatoriamente peak a W=num_cpus (transitorio primo-compile); picco peggiorato >2% ⇒ la leva non passa in silenzio.
- **KL-80-3**: ogni claim "nessun leak" cita il floor di risoluzione della probe (KB/req rilevabili); claim senza floor = respinto.

### 8. Stogov — semantica Zend engine — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI** (serie A-DS di WP-80).

Il gate che avevo preteso al c.5c ha morso al primo colpo: il RetainSet per-worker era un leak reale (arena append-only, write-only, 1 unit/include/richiesta). Il fix **è Zend-fedele**: la decomposizione ora rispecchia Zend — bytecode persistente = unit cache thread-local (l'analogo SHM di opcache), pin per-richiesta = RetainSet che muore con la richiesta, come le op_array delle unità incluse liberate a RSHUTDOWN quando non sono in SHM. Verificato nel codice (`worker_loop`, `park_module`, put solo su `pure`, ways-eviction). L'epilogo fatal in `execute_with_retain` rispecchia `run_module_with_hir` riga per riga. Ma refuto quanto segue.

**A-DS1 — Mina doc superstite (il purge non è chiuso).** `vm/mod.rs:3425-3426` (doc di `request_end`): *"The only cross-request survivor is the caller-held RetainSet arena"* — FALSA post S-78.1.5: nel worker il RetainSet è per-richiesta; il survivor è la unit cache TL. I 7 pattern di `gate-doc-purge.sh` non la coprono. Correggere la frase e aggiungere pattern (`cross-request survivor` + RetainSet). Una doc falsa su questa esatta superficie ha già autorizzato un refactor letale.

**A-DS2 — Stranded keys: il leak gemello è ancora aperto.** Un file editato re-keya (path, mtime, size) e **stranda le vecchie entry** nella HashMap — dichiarato TODO(port) a `vm/mod.rs:1324-1327` (Leijen R5). Nel CLI era innocuo; su worker long-lived ogni deploy/edit è crescita unbounded. Opcache lo bounda (`max_wasted_percentage` → restart); phpr no. WP-79 moltiplica i put col main: chiudere PRIMA (supersede-per-path alla put, o cap). **KS-DS-80-1**: `superseded_entries` monotono crescente su un workload di edit = FAIL.

**A-DS3 — Contratto fatal: fedele, ma registrare i confini.** exit()→200 è FPM-fedele (l'exit code non ha canale FCGI); uncaught consumato da `set_exception_handler`→200 senza banner è fedele (terminazione normale). Però fatal→500 vale in FPM solo sotto "headers not sent": con output flushato FPM serve 200+body parziale. Oggi php-server bufferizza tutto, quindi 500 è sempre lecito — ma è proprietà del SAPI, non del contratto: REGISTRARE (come il 3.9) insieme alla scelta display_errors=On implicita nel "body = stdout CLI" (default FPM production: Off).

**A-DS4 — Unit cache vs opcache reale.** (path canonico, mtime ns, size)+fp è PIÙ severo di opcache (mtime a secondi, `revalidate_freq=2`, `revalidate_path=0`): revalidazione a ogni include = `revalidate_freq=0`, accettabile. Mancano e vanno dichiarati: `include_path` ini (`resolve_include_path` prova solo dir-script e cwd), `opcache_invalidate`/`opcache_reset`, modalità `validate_timestamps=0`.

**A-DS5 — WP-79, forma fedele.** In Zend il main script È cacheato da opcache identicamente agli include. La forma fedele NON è una seconda cache `Rc<Module>`: è la STESSA unit cache TL, con chiave da canonicalize+metadata di `meta.path` (oggi il main non fa né stat né canonicalize) e stessa policy `pure`. Il contratto M-68.5 ("main can never reach this put", `vm/mod.rs:6922-6925`) va superato deliberatamente, non aggirato. Attenzione: un main impure (WP index.php) può dare hit-rate 0 legittimo — serve il controllo positivo.

**A-DS6 — Fixture PRIMA della leva** (**KS-DS-80-2**: leva senza queste fixture verdi = nulla): (1) main modificato tra richieste → body nuovo; (2) edit same-size (solo mtime); (3) symlink swap del docroot — canonicalize = `revalidate_path=1`: comportamento da PINNARE, non presumere; (4) include condizionale/impure; (5) hit-counter che DEVE muoversi (contatore fermo = void, KG-79.A); (6) stesso file incluso due volte non-`_once` nella stessa richiesta (parking duplicato: pinnarlo).

**KS-DS-80-3**: dopo la cache del main, un solo body ≠ oracolo su `gate_stateful`/`include_gate` ⇒ revert della leva, mai fix-forward.

### 9. Gregg — metodologia di misura/attribuzione — CONCORDO CON EMENDAMENTI

Verifica compiuta: ho ricontrollato ogni cifra campionabile del report contro i raw (`measure-out/`): tier0 10.665.984/10.731.520/10.633.216 ✓, axum peak 59.817.984/59.686.912/59.703.296 e V2 18,1/18,0/18,0M ✓, cliserver 53.985.280×3 al byte ✓, W4/W10 e n200 ✓, righe census intero-esatte req 108–111 identiche tra run ✓, join line presente su tutti i run axum ✓. Il sito di emissione dei contatori census è in `worker_pool.rs` (path pool-only), il che conferma — e circoscrive — la scusante su A-BB1.

**VERDETTO: CONCORDO CON EMENDAMENTI**

Nel WP-79 dichiarai il programma non eseguibile senza osservabili. La sessione li ha costruiti (contatori per-fase, retain_len, live_objs, depth) e la misura è tracciata: ho campionato i raw e **ogni cifra del report corrisponde al log citato**, R≥3 quasi ovunque, nessuna media su spread oltre soglia, depth_max=1 verificato, controllo positivo retain_len che DISCRIMINA (0 vs 2). Il protocollo è stato sostanzialmente onorato. Restano scarti puntuali:

**A-BG11 — Il peak cli-server viola KG-79.B per lettera.** L'emendamento che la sessione stessa ha scritto dice: braccio cli-server ⇒ SOLO vmmap V1/V2. Ma il delta A/B di testa («+5,7MB peak») usa il peak `time -l` di quel braccio — metrica exit-based su un braccio senza drain. Il peak è monotono e il 53,99MB×3 al byte lo rende credibile, ma la regola o si emenda esplicitamente (il peak non richiede join) o si rispetta: fino ad allora il delta verdict-grade è **+4,8M residente**; il +5,7MB è ADVISORY.

**A-BG12 — Spread flaggato solo quando è enorme.** Il picco W=10 (12%) è dichiarato e non mediato — corretto, e la spiegazione (transitori di primo-compile concorrenti) è plausibile ma NON verificata da alcun osservabile. Però il picco W=4 ha spread 5,97% (113,98/117,42/121,00MB), oltre il 2%, e il report non lo segnala. Regola: OGNI riga oltre soglia si flagga, non solo la peggiore. Dichiarare inoltre esplicitamente che il fit di linearità usa SOLO V2 (spread 0,3%) — è così, ed è ciò che lo salva.

**A-BG13 — `census.inc` è R=1.** Il protocollo dice R≥3; la discriminazione retain_len=2 poggia su un run singolo. L'esattezza intero-identica tra 100+ richieste mitiga, ma la deroga andava dichiarata nel report, non taciuta dietro «R3 + include». Replicare in WP-79.

**A-BG14 — Il canale c non ha controllo positivo.** c=0 ovunque; KG-79.A esige contatore visto muoversi o census VOID. «Coperto dai contatori globali» non è un controllo positivo di c: serve la fixture che DEVE muoverlo (destructor che alloca in teardown). Fino ad allora c=0 è indistinguibile da un contatore morto.

**A-BG15 — Incidente c21c2959: ben gestito, rimedio incompleto.** L'hash nel driver ha smascherato il binario sbagliato — esattamente il perché di KS-AH-78-1. Ma il rimedio è rimasto umano: il driver deve rifiutare da solo (`EXPECTED_HASH` pinnato, mismatch ⇒ exit 1). E il driver etichetta tier0 «ADVISORY (no join line!)» mentre il report dice «N/A per costruzione»: allineare con un caso tier0 esplicito.

**A-BG16 — A-BB1 «NON GIUDICABILE»: conclusione formalmente corretta, deferimento evitabile.** I contatori vivono in lower/compile/vm (codice condiviso); solo l'emissione è nel worker_pool, e la riga per-richiesta NON richiede exit pulita: bastava emetterla sul path cli. Promuovere il punto 4 di WP-79 da «residuo minore» a prerequisito del verdetto A-BB1. Il «delta somma non decomposta» è invece onesto: A-BG10 prevede esattamente quella dicitura.

**WP-79/A-BB6: ben posto** — census prima del design (99%/80k call/13MB è il dato frequenza×taglia giusto), gate prima della leva (mtime, include condizionale), A/B build-adiacente su coppia census+gemello. Kill-switch:

- **KG-80-1**: verdetto A-BB6 VOID se la predizione dei contatori post-cache (a_calls/req atteso a caldo) non è dichiarata PRIMA della run A/B.
- **KG-80-2**: cache Module per (path,mtime) senza contatore di taglia + fixture di eviction che DEVE scattare = leva respinta (lezione KS-DS-78-4: una struttura che cresce e nessuno legge è un leak in attesa; WordPress = centinaia di path).
- **KG-80-3**: cifra peak da braccio senza uscita pulita conforme = ADVISORY d'ufficio (formalizza A-BG11).
