# COUNCIL_WP81_REVIEWS.md — Concilio a 9 sedie su S-79.0 (pre-lever hardening + design79) + programma WP-80 (implementazione A-BB6)

**Data**: 2026-07-31 (chiusura S-79.0)
**Oggetto**: revisione di S-79.0 (commit ff22a2e…36d62df: ordine WP-80 8/8 +
design79.md) e giudizio del programma WP-80 (misura R≥3 + DR-1 +
implementazione leva A-BB6 + A/B).
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto
NEXT_SESSION, WP_SESSION_79, design79, COUNCIL_WP80 (il proprio ordine) e il
CODICE/i gate del proprio perimetro; Gregg ha campionato i raw.
**Status**: verbali VINCOLANTI per WP-80 (blocco ⚖️ in NEXT_SESSION).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO CON EMENDAMENTI — nessuna opposizione.**
L'esecuzione S-79.0 è giudicata reale e verificata NEL CODICE da ogni sedia
sul proprio ordine (8/8 punti onorati; A-DS1 chiuso; A-MS1/2/4 chiusi;
A-SK1..8 mordenti; A-AH10..13 eseguiti; A-BB10..15 onorati; A-PP1..6
implementati). I colpi duri sono tre: **l'identità del binario è ancora
bucata sull'asse GIT/tree** (3 sedie), **il §1 di design79 cita una cifra
senza run tracciato** (Gregg, bloccante — KG-78.D esteso ai design file), e
**il falsificatore anti-wrap è vacuo in release** (Hoare, dimostrazione
aritmetica: il test passa CON la regressione installata — in dissenso con la
lettura "sound" di Matsakis/Klabnik: prevale la dimostrazione).

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **Identità GIT/tree non asserita** (Gregg A-BG18 — buco ATTIVO: driver a
   git 3a73be9 accetterebbe matrix di 36d62df; Klabnik A-SK11 — matrix-log
   SOVRASCRITTO a ogni run: la quaterna è ri-auditabile solo per l'ultima
   build, match awk a prefisso; Hejlsberg A-AH14 — tree sporco non rilevato):
   il driver deve confrontare `git=` del matrix col git corrente E rifiutare
   tree sporco; matrix archiviato per-run [KG-81-2, KS-AH-81-1, KS-SK-81-2].
2. **design79 §1 = cifra da chat** (Gregg A-BG17 BLOCCANTE; Klabnik f la
   giudica lecita-se-dichiarata ma converge sull'obbligo di ribase): i
   numeri citati (83.834/13,36MB, bin 37f630d6) non stavano in NESSUN raw
   del repo né in alcun matrix-log. §1 è ADVISORY; ri-derivazione R≥3 con
   driver ENFORCE corretto PRIMA di DR-1 e del design freeze. [Nota di
   cancelleria: il raw dello smoke, vissuto in /tmp, è stato preservato in
   `wp79-harness/censuscli-smoke-RAW-ADVISORY.log` DOPO il finding — resta
   ADVISORY perché il binario non è in alcun matrix-log dell'epoca.]
3. **Canale a1 sporco sui path fatal** (Pedersen A-PP11 — gli early-return
   di lower-fatal/Unsupported NON drenano take_split e LAST_S3: il residuo
   a1 del compile fallito inquina la riga successiva, ed è ESATTAMENTE la
   sequenza della fixture F8; Hoare A-TH12 — assert a1≤a mancante, il
   saturating_sub maschererebbe; Matsakis A-MS11 — provider guard + a1
   senza RAII perde la finestra su early-return): drain su OGNI uscita
   (forma del CLI arm) + assert + tripwire [KS-PP-81-1, KS-MS-81-2].
4. **Il pin del design copre il Module, NON il Program** (Pedersen A-PP13 —
   il RetainSet è FrozenVec<Rc<Module>>: `main_program` non è parcheggiabile
   lì; Leijen A-DL7 — il walk del Program NON esiste e module_census_bytes
   salta le entry Rc-shared: la taglia retained del main cached uscirebbe
   sottostimata proprio dei ~10,8MB del prelude condiviso; Matsakis
   implicito su ownership): KS-PP-80-2 esteso alla coppia (Module, Program);
   DR-1 audita ANCHE il grafo Program [KS-PP-81-2, KL-81-2].
5. **KS-AH-80-4 malformata** (Klabnik A-SK13, Gregg A-BG21): "a_calls +
   a1_calls" doppia-conta (a1⊆a) — ridefinire su UNA quantità (`a_bytes`
   steady, o a_calls) PRIMA della baseline R≥3 [KS-SK-81-4].
6. **Depth: semantica CODA, non in-flight + falsificatore vacuo** (Hoare
   A-TH9/A-TH10): il dec al pickup misura la coda — una richiesta IN
   ESECUZIONE + una prelevata dà watermark 1 (overlap pipelined invisibile
   a KH78-2); e il test anti-wrap passa CON la regressione inc-dopo-send in
   release (wrap modulare si auto-annulla). Tripwire `d==0 ⇒ abort` in
   note_depth + test negativo + contatore IN_FLIGHT o declassamento
   dichiarato [KH81-1, KH81-2]. (Dissenso registrato: Matsakis/Klabnik lo
   giudicavano sound — la dimostrazione aritmetica di Hoare prevale.)
7. **Idle window mono-braccio** (Leijen A-DL8, Gregg A-BG20, Hejlsberg
   A-AH20): la probe idle esiste solo su MODE=census — censuscli (il
   braccio che rende A-BB1 giudicabile) non ce l'ha; più una run idle LUNGA
   (≥60s) per campagna; finestra dichiarata churn-only [KL-81-3].
8. **Pin dei marker a somma aggregata** (Hejlsberg A-AH18 — grep di
   STRINGHE, non di attributi cfg: un commento entra nel conto; Klabnik
   A-SK10/KS-SK-81-3 — sostituzione intra-crate invisibile): pin per-FILE
   con siti nominati, match sulla forma `cfg(feature = ...)`.
9. **Off-by-one righe==N** (Gregg A-BG19, bug REALE del driver nuovo): la
   boot-probe curl-a la fixture ⇒ 1 riga census extra ⇒ il check nuovo
   VOID-erebbe ogni run legittima (census.r1 storico: 111 righe = 1+10+100).
   Boot-probe fuori canale o EXPECTED=W+M+1 dichiarato.
10. **A-MS3 ancora aperto — bloccante pre-leva** (Matsakis A-MS8/KS-MS-81-3;
    correlato Hoare A-TH14 — "il corpus resta il giudice" è errore di
    categoria: il one-shot esercita solo MISS; il probe on/off sia UN
    parametro grep-gated): `execute_with_retain` pub + `&RetainSet` non
    sigillato — col main parcheggiato diventa KS-DS-78-4 aggravato.
11. **Census cross-crate mai sotto -D warnings** (Hejlsberg A-AH15; +
    A-AH16 quinta config axum+census-no-cli non compilata da nessuno:
    pinnarla o bandirla [KS-AH-81-4]).
12. **F6 contraddice KS-MS-80-2** (Matsakis A-MS10): include doppio
    non-once ⇒ park-eventi > "unit distinte + main" — riformulare su
    park-EVENTI o F6 fa scattare il FATAL su comportamento corretto.
13. **Fixture mancanti dalla paura opcache** (Stogov A-DS8: F10
    early-binding [KS-DS-81-1], F11 __FILE__/__DIR__ [KS-DS-81-2], F12
    strict_types; Klabnik A-SK12: F-probe-fail + pin `main_probe==0` sul
    one-shot; Pedersen: F8b fatal-stessa-richiesta + caso link-fatal).
14. **SAFETY del catch_unwind resa FALSA dal punto 5 della stessa sessione**
    (Hoare A-TH11): col hook che aborta prima dell'unwind, "the unwind
    still runs drops" non è più vero in modalità axum — correggere +
    pattern doc-purge (la mina A-DS1 in altra veste).
15. **Split definito SOLO a workers=1** (Bak A-BB17/KB-81-1): a W>1 snap()
    globale + accumulatori thread-local ⇒ righe non definite; il driver le
    rifiuta. Più: floor non-compile ex-ante (A-BB16/KB-81-2), bound
    autoload-run in a3 (A-BB18/KB-81-3), CPU a slope due-N (A-BB21/KB-81-5),
    include_heavy = fixture-b non proxy-WP (A-BB20/KB-81-4).

### Refutazioni respinte dal collegio (registrate)

- "Il doppio pass rustc/build non prova la warning-freeness del binario
  hashato" — RESPINTA da Hejlsberg (a): pass-1 prova il sorgente, pass-2
  hasha lo stesso sorgente; il buco vero è il tree non asserito.
- "Il supersede-per-path è più aggressivo di opcache" — RESPINTA da Stogov
  (a): opcache è single-version per path; blue-green via symlink produce
  path canonici distinti che il supersede non incrocia. Il costo vero è la
  scan O(N²) di warmup (A-DS9: misurare su corpus WP).
- "Le soglie del gate overlap sono numeri magici" — RESPINTA da Klabnik (a):
  W=1 < 3,0s è fisicamente impossibile; ma il claim va ridotto a "≥2 worker"
  [KS-SK-81-1] e l'esca del ramo FAIL manca (A-SK9).

### Ordine vincolante di apertura WP-80 (S-80.0 "identity & channel repair", poi misura, poi leva)

1. **Identità**: driver confronta `git=` del matrix col git corrente + tree
   pulito obbligatorio nel matrix gate + archivio matrix per-run + match
   esatto della riga bin (A-BG18/A-AH14/A-SK11) [KG-81-2, KS-AH-81-1,
   KS-SK-81-2].
2. **Driver**: fix off-by-one righe==N (boot-probe fuori canale o
   EXPECTED=W+M+1, ricontare i raw storici — A-BG19); righe split rifiutate
   a W>1 (A-BB17); probe idle anche su censuscli + una run idle ≥60s
   (A-DL8/A-BG20/A-AH20); etichetta gross nella riga census (A-DL10).
3. **Canale**: drain dello split su OGNI uscita di execute_with_retain
   (forma CLI arm) + assert a1≤a FATAL + A3Window !Send/token LIFO/tripwire
   + a1 RAII + provider guard (A-PP11/A-TH12/A-MS7/A-MS11) [KS-PP-81-1,
   KS-MS-81-2]; tripwire depth `d==0⇒abort` + test negativo + IN_FLIGHT o
   declassamento dichiarato (A-TH9/A-TH10) [KH81-1/2].
4. **Sigillo A-MS3**: RetainSet costruito dentro l'entry-point o newtype
   one-shot; in subordine grep-gate call-site (A-MS8) [KS-MS-81-3].
5. **-D warnings cross-crate census** (php-runtime, php-cli) in gate+CI +
   quinta config pinnata o bandita (A-AH15/A-AH16) [KS-AH-81-4]; pin marker
   per-FILE con siti nominati (A-AH18/A-SK10) [KS-SK-81-3]; SAFETY
   catch_unwind corretta + pattern doc-purge (A-TH11); near-miss dispatcher
   in Phase B (A-PP12); esca ramo FAIL + verifica body W=1 nel
   gate-concurrent, verdict "overlap≥2" (A-SK9) [KS-SK-81-1]; censimento
   letture post-request_end pinnato (A-PP10) [KS-PP-81-3].
6. **Misura di riferimento R≥3** coi driver corretti (census E censuscli,
   hello + include_gate + include_heavy): SOSTITUISCE il §1 ADVISORY di
   design79 (A-BG17); floor non-compile ex-ante (A-BB16) [KB-81-2, KG-81-1].
7. **design79 emendato** (stesso commit della misura): MAIN_CHAIN_FP
   COMPUTATO dal contenuto del prelude a init + input enumerati
   (ini/autoload-mid-compile — A-AH19/A-DS7) [KS-AH-81-2]; pin coppia
   (Module, Program) + DR-1 esteso al grafo Program (A-PP13) [KS-PP-81-2];
   fixture F10-F12 + F-probe-fail + F8b + link-fatal + pin main_probe==0
   one-shot (A-DS8/A-SK12/Pedersen) [KS-DS-81-1/2]; KS-AH-80-4 ridefinita su
   una quantità (A-SK13/A-BG21) [KS-SK-81-4]; predizioni resid+b a caldo
   (A-BG21) [KG-81-3]; KS-MS-80-2/F6 su park-eventi (A-MS10); strumento
   retained dichiarato con regola Rc-shared + controllo positivo di taglia
   nota (A-DL7) [KL-81-2]; budget entries/retained ×W ex-ante (A-MS12);
   supersede: contatore bytes + probe residente twin (A-DL6) [KL-81-1];
   filtro reg_mode nel supersede + costo scan nel braccio CPU (A-MS9/A-DS9).
8. **DR-1** (Module E Program): esiti ammessi = IC fuori dal Module
   (run_time_cache per-Vm) o reset-on-hit con contatore, MAI solo-F7/prosa
   (A-TH13) [KH81-3]; audit come check macchina.
9. **SOLO POI** implementazione leva + fixture verdi + A/B (design79 §11
   emendato: CPU slope due-N, peak W=num_cpus, floor numerici — A-BB21/
   A-DL9) [KB-81-5, KL-81-3, KS-DS-80-3 invariato: body≠oracolo ⇒ REVERT].

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH81-1 | claim sequenzialità/overlap dal watermark con dec-at-pickup | ADVISORY (misura la coda) |
| KH81-2 | anti-wrap senza tripwire d==0⇒abort + test negativo | KH80-4 resta APERTO |
| KH81-3 | DR-1 chiuso con solo audit-prosa o solo F7 | leva non autorizzata |
| KS-MS-81-1 | guard/pin thread-affine Send/Sync per auto-derive | REJECT (assert compile-time) |
| KS-MS-81-2 | tripwire a3-stack >0 in una run | cifre a1/a2/a3/resid VOID |
| KS-MS-81-3 | leva con execute_with_retain pub+&RetainSet non sigillato | leva RESPINTA |
| KS-SK-81-1 | claim concorrenza W>2 senza osservabile che separi W da W−1 | ridotto a "≥2" |
| KS-SK-81-2 | cifra con hash solo in matrix-log sovrascritto | NULLA |
| KS-SK-81-3 | pin di censimento aggregato dove esiste la forma per-file | gate non conforme |
| KS-SK-81-4 | soglia ex-ante su canali non disgiunti | predizione VOID |
| KS-AH-81-1 | matrix-log su tree sporco o senza assert tree==rev | log NULLO, misure VOID |
| KS-AH-81-2 | MAIN_CHAIN_FP letterale hand-maintained | leva respinta |
| KS-AH-81-3 | bump marker census senza lista siti nel commit | gate non conforme |
| KS-AH-81-4 | config feature compilabile fuori da matrice+CI usata in un run | cifra NULLA |
| KB-81-1 | cifra split a W>1 | VOID |
| KB-81-2 | predizione §10 senza floor non-compile ex-ante | non falsificabile, VOID |
| KB-81-3 | "a3 = ciò che il HIT salta" senza bound autoload-run | ADVISORY |
| KB-81-4 | estrapolazione % WP da include_heavy | respinta |
| KB-81-5 | CPU da totale singolo senza slope due-N | ADVISORY |
| KS-PP-81-1 | riga census con residui non drenati di richiesta fallita | run census VOID |
| KS-PP-81-2 | Program non pinnato per la richiesta o DR-1 senza grafo Program | reject della leva |
| KS-PP-81-3 | lettura post-request_end nuova senza bump del censimento | gate FAIL |
| KL-81-1 | claim "supersede libera memoria" senza bytes + probe twin con floor | respinto |
| KL-81-2 | cifra retained da walker senza controllo positivo/regola Rc-shared | NULLA; A/B VOID |
| KL-81-3 | verdetto A-BB1 da censuscli senza idle window su quell'arm | NON GIUDICABILE |
| KS-DS-81-1 | HIT del main col binding vecchio dopo edit lib (F10) | leva RESPINTA |
| KS-DS-81-2 | __FILE__/__DIR__ ≠ oracolo su HIT (F11) | REVERT immediato |
| KS-DS-81-3 | confronto censuscli↔axum senza dichiarazione asimmetria superglobali | cifra respinta |
| KG-81-1 | cifra in un design senza raw nel repo E binario nel matrix | VOID d'ufficio |
| KG-81-2 | run con git del matrix ≠ git del tree | VOID |
| KG-81-3 | verdetto A-BB6 senza resid a caldo vs predizione | NON giudicabile |

---

## VERBALI INTEGRALI

### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

Verifiche eseguite sul codice, non sulla prosa: fix depth in `worker_pool.rs:303-318` (inc-before-send, undo su Err a r.313-315) ✓; `reset_depth_stats` cfg(test) r.129-132 + `DEPTH_TEST_LOCK` r.567 ✓; panic-hook abort solo nel ramo `use_axum` (`main.rs:398-403`), CLI e cargo-test esclusi ✓; trigger dispatcher `ends_with("/__phpr_panic_dispatcher")` (`main.rs:157-159`) ✓; bracket a1 in `lower/mod.rs:301-308` (seed None) e `compile/mod.rs:258-268, 348-357, 391-394` ✓; A3 RAII annidabile netto (`alloc_census.rs:97-115`) ✓; riga census-cli su `php-cli/src/server.rs:626-647` ✓. L'ordine WP-80 è onorato NEL CODICE su 8/8 punti. Ma refuto quattro sostanze.

**VERDETTO: CONCORDO CON EMENDAMENTI**

**A-TH9 — Il depth fix chiude l'underflow, NON il fail-open semantico.** Il decremento avviene al PICKUP (`worker_pool.rs:248`), non a fine richiesta: QUEUE_DEPTH conta i task in coda, non gli in-flight. Una richiesta IN ESECUZIONE più una appena accodata-e-prelevata dà watermark 1: l'overlap pipelined che KH78-2 deve rilevare passa invisibile. Il commento r.96-98 ("the watermark cannot under-count") è FALSO nella semantica che conta. Fix: dec dopo il send della risposta, o contatore IN_FLIGHT separato (inc al pickup, dec post-send).

**A-TH10 — Il test anti-wrap NON può fallire in release: falsificatore vacuo.** Sotto la regressione inc-dopo-send, il wrap è aritmetica modulare: dec su 0 → MAX, inc successivo → 0; inc/dec bilanciati ⇒ drain finale ==0 SEMPRE; e `note_depth` riceve `fetch_add()+1` = MAX+1 → 0 in release (overflow-checks off) ⇒ watermark intatto, `peak ∈ 1..=N` soddisfatto. Entrambe le assert di `census_queue_depth_no_underflow_inc_before_send` (r.957-966) passano CON la regressione installata; il commento r.936-937 promette il contrario. La regola "cargo test sempre --release" rende il panic da debug-overflow irraggiungibile. Rimedio: tripwire nel canale — in `note_depth`, `d==0` è impossibile senza wrap (fetch_add ritorna ≥0 ⇒ d≥1): `d==0 ⇒ abort` nel census build, più test negativo che lo veda mordere.

**A-TH11 — SAFETY comment reso falso dal punto 5 della stessa sessione.** `worker_pool.rs:186-189`: "The unwind itself still runs drops … before abort()". Col hook globale che aborta PRIMA dell'unwind (main.rs:398-403), in modalità axum nessun drop gira. Doc falsa sulla superficie panic = la mina A-DS1 in altra veste: correggere + pattern nel doc-purge.

**A-TH12 — Classificazione a1: incoerenza mascherata da saturating_sub.** La condizione `prelude.is_empty() && link.is_none()` è giusta per il main; ma se un compile a RUNTIME la soddisfacesse (eval/deferred con seed anomalo), a1 accumulerebbe nella finestra b e l'emettitore (`worker_pool.rs:533-534`) saturerebbe a2 a 0 in silenzio. Serve assert `a1 ≤ a` (violazione = FATAL census) + controllo positivo che a1 resti fermo tra s1 e s3.

**A-TH13 — DR-1: l'esito (b) "determinismo via F7" NON basta.** F7 è un campione a 3 richieste; il determinismo degli id è proprietà globale che una fixture non stabilisce (lezione WP-28). Forme ammesse: IC fuori dal Module (analogo run_time_cache di Zend, per-Vm) oppure reset-on-hit con contatore `ic_reset` visto muoversi + F7 come tripwire. L'audit deve atterrare come check macchina (grep-gate Cell/RefCell nei tipi raggiungibili da Module), mai prosa — la mia lezione WP-78 vale intera.

**A-TH14 — §2: "il corpus per NOME resta il giudice" è un errore di categoria.** Il one-shot non ingaggia il probe ⇒ il corpus esercita SOLO il path MISS: sul path HIT il giudice sono esclusivamente F1-F9 + battery su HIT (già in §9 — dichiararlo in §2). Il probe on/off sia UN parametro al confine SAPI, grep-gated: due call-site con logica propria = secondo path in attesa.

**Kill-switch:**
- **KH81-1**: claim di sequenzialità/overlap dal watermark con dec-at-pickup = ADVISORY (misura solo la coda).
- **KH81-2**: senza tripwire `d==0⇒abort` + test negativo, il test anti-wrap non vale come falsificatore; KH80-4 resta APERTO.
- **KH81-3**: DR-1 chiuso con solo audit-prosa o solo F7 = leva non autorizzata.

### 2. Matsakis — ownership/aliasing/Send-Sync — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI — con UN punto bloccante (A-MS3 non sigillato)**

**Verificato nel codice** (non su prosa): A-MS1 CHIUSO bene — inc-PRIMA-di-send con undo su send fallita (`worker_pool.rs` dispatch), `DEPTH_TEST_LOCK` + `reset_depth_stats` presenti ⇒ KS-MS-80-3 soddisfatto per i run futuri. A-MS2 ONORATO: tutti i test di `worker_pool.rs` sono a forma worker_loop (RetainSet fresco per richiesta, verificato riga per riga; il cli-server passa per `run_source_with_ini`, RetainSet interno). A-MS4 in forma forte: panic-hook globale che aborta (main.rs:398-403) + SAFETY inline VERO sul catch_unwind. A-MS5a/b/c mappati in design79 §2-3. KS-MS-80-1: nessun `unsafe impl`. KS-MS-80-2: aggiornato correttamente — design79 §4 dichiara PER NOME il bump dell'atteso A-DS8 da 1 a 2.

**(a) A3Window su unwind `?`: contabilità CORRETTA.** Drop LIFO di scope: l'inner libro `own = gross − nested` e somma `gross` al parent; il parent sottrae ⇒ né doppio né sotto-conteggio, anche su PhpError. REFUTO invece la robustezza del guard: (1) è una unit struct ⇒ **auto `Send+Sync`** — un drop cross-thread poppa lo stack del thread SBAGLIATO e l'`else { return }` su pop-vuoto inghiotte la corruzione in silenzio; (2) nessun token LIFO: due guard mossi e droppati fuori ordine si scambiano i delta senza rumore; (3) `SNAPSHOT_FN` installato mid-window ⇒ `gross` = storia intera del processo dentro a3; (4) a1 è open/close MANUALE: un early-return tra i due perde a1 e gonfia a2 (derivato) — misattribuzione sui path fatal.

**(b) Supersede-per-path: aliasing ok** (collect in Vec, poi remove; un solo `borrow_mut`; `uc_stat`/`uc_log` non rientrano nella cache). reg_mode è process-constant (OnceLock in `reg_lower::enabled`) quindi OGGI il filtro path-only non può droppare chiavi vive — ma il "by construction" del commento poggia su un invariante NON locale: aggiungere `k.reg_mode == key.reg_mode` costa zero. Costo: scan O(entries) per ogni put a chiave nuova ⇒ O(N²) di warmup su N path distinti — trascurabile vs compile, ma va citato nel braccio CPU dell'A/B (WP: centinaia di unit × W worker).

**(c)** Nessun test riusa RetainSet: non diventano leak-shaped con A-BB6.

**(d) A-MS3 ANCORA APERTO — BLOCCANTE per WP-80.** `execute_with_retain` resta `pub` con `&RetainSet`; nessun newtype, nessun grep-gate nei harness, design79 non lo nomina, la lista WP-80 di NEXT_SESSION nemmeno. Con A-BB6 il main parcheggiato è l'oggetto PIÙ GRANDE (Module+Program): un caller che riusa il RetainSet accumula un main per richiesta = KS-DS-78-4 in forma aggravata. Va sigillato PRIMA della leva.

**(e)** KS-MS-80-2 aggiornato sì, ma **F6 lo contraddice**: include doppio non-`_once` ⇒ parking duplicato ⇒ `len` > "unit distinte + main". L'invariante va riformulato su park-EVENTI, o F6 fa scattare il FATAL su comportamento corretto.

## Emendamenti

- **A-MS7**: `A3Window` reso `!Send` (PhantomData<*const ()>) + token di profondità con assert LIFO + contatore tripwire su pop-vuoto/mismatch.
- **A-MS8** (bloccante pre-A-BB6): sigillo A-MS3 — RetainSet costruito DENTRO l'entry-point o newtype one-shot consumato; in subordine grep-gate sui call-site nella battery. Aggiungere a design79 §4.
- **A-MS9**: filtro supersede esteso a `reg_mode`; costo O(N)/put dichiarato nel braccio CPU A/B.
- **A-MS10**: KS-MS-80-2 riformulato: `len == park-eventi include + 1 main`; F6 dichiara il numero esatto.
- **A-MS11**: guardia provider: se `SNAPSHOT_FN` non era installato all'open, il delta al close si scarta e si conta (tripwire); a1 → bracket RAII o test che il segmento non abbia early-return.
- **A-MS12**: budget numerico entries/retained per-worker (main ×W) dichiarato PRIMA dell'A/B (chiude A-MS5d).

## Kill-switch

- **KS-MS-81-1**: guard/pin thread-affine che risulti `Send`/`Sync` per auto-derive = REJECT (assert_not_impl_any su A3Window, come RetainSet).
- **KS-MS-81-2**: tripwire a3-stack (underflow/mismatch) > 0 in una run ⇒ TUTTE le cifre a1/a2/a3/resid di quella run VOID.
- **KS-MS-81-3**: A-BB6 atterrata con `execute_with_retain` ancora pub+`&RetainSet` non sigillato/gateato = leva RESPINTA (riedizione S-78.0: l'invariante va nel type system).

### 3. Klabnik — spec/testabilità/gate — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.**

Verifica nel codice/gate dei miei A-SK1..8 e KS-SK-80-1..4: **tutti implementati e mordenti**. A-SK1: overlap observable reale (gate-concurrent.sh:126-155,171-195) col discriminatore W=1. A-SK2: verbi estesi (assegnamento, write!, swap/take, clear), tandem-di-SCRITTURA, NSITES_PIN=6 con siti nominati nel commento. A-SK4: RC≠134=FAIL, "all 2 workers joined" pinnato, Phase C dispatcher. A-SK7: measure78.sh rifiuta su hash∉matrix (exit 1), depth>1⇒RC=1, righe==N. A-SK8: `reset_depth_stats()` + `DEPTH_TEST_LOCK` (worker_pool.rs:567,905,941). Fix depth inc-before-send con undo su send fallita e test anti-wrap: sound.

**(a)** Le soglie NON sono magiche a metà: W=1 sotto 3,0s è **fisicamente impossibile** (8×400ms seriali ≥3,2s; usleep non sotto-dorme) — nessun sistema veloce può far passare W=1. Bonus non dichiarato: il discriminatore pinna anche la DURATA della fixture (chi accorcia usleep uccide il gate rumorosamente). Ma 1,7s prova solo **≥2 worker** (floor W=2 = 1,6s, margine 0,1s!): un pool degenerato a 2/4 passa. Su macOS carico il false-FAIL è possibile ma fail-closed: accettabile. **(c)** L'esca del ramo FAIL manca: il W=1 esercita il ramo ≥3,0, mai il ramo <1,7-che-fallisce — e i BODY-FAIL del volley W=1 sono appesi al log e **mai verificati** (riga 186: append senza ricontrollo).

**(b)** NSITES 4→6: bump giustificato (i 2 siti lifecycle che l'alfabeto vecchio non vedeva), siti nominati, commit 83c8dea — auditabile. Ma i pin census 18/11/2 sono **somme per-crate senza nomi**: una sostituzione intra-crate (−1 sito, +1 sito) è invisibile. E l'alfabeto verbi resta chiuso: `.stdout.truncate/drain/resize` non matcherebbero né il detect NÉ il count — il pin non mitiga i verbi non listati.

**(d)** F1-F9 copre i miei A-SK6 (same-second via fp in F2, symlink F3, negativa-stato via gate_stateful/A-DS9 su path HIT, corpus per NOME §11.5). Il fp hashato su `meta.source` letto QUESTA richiesta chiude anche la race TOCTOU del canonicalize — proprietà giusta, da dichiarare esplicita. Mancano: **F10 probe-fail** (canonicalize/stat che fallisce: esito da pinnare = MISS senza put, contatore `main_probe_fail` in §8) e il pin della proprietà "CLI one-shot non ingaggia il probe" (`main_probe==0`, oggi solo affermata).

**(e)** L'awk regge sul log attuale, ma: (i) `index(row)==1` è match a PREFISSO — "bin[census]" matcherebbe un futuro "bin[census-x]"; (ii) **feature-matrix.log è sovrascritto a ogni run** (`: > "$LOG"`): l'identità dello smoke 37f630d6 sopravvive solo in un verdict superseded — la quaterna è ri-auditabile solo per l'ULTIMA build; (iii) il driver cita il GIT_REV dell'albero corrente senza confrontarlo col `git=` del matrix log: binario vecchio + tree nuovo = cifra attribuita al commit sbagliato con hash formalmente valido.

**(f)** Smoke R=1: uso **lecito** — dichiarato tale, dimensionamento su assoluti, ribase R≥3 obbligata pre-A/B (KG-78.A). Ma la soglia KS-AH-80-4 è **malformata**: "a_calls+a1_calls" doppia-conta a1 (a1⊆a: a_calls=s1−s0 include il prelude). Su hello passa comunque (~97%), su include_heavy il doppio peso sposta il confine. Definirla su UNA quantità.

**Emendamenti**: **A-SK9** gate-concurrent: soglie derivate dalle costanti e stampate; esca ramo FAIL (applicare <1,7 all'elapsed W=1 e pretenderne il fallimento); verificare i BODY-FAIL W=1; verdict dice "overlap≥2", mai "W=4". **A-SK10** pin per-FILE con siti nominati (census-twin come stdout-tandem); censire ogni menzione `.stdout` con allowlist letture. **A-SK11** ENFORCE: match esatto "] ", archivio per-run del matrix log, cross-check git=+tree pulito. **A-SK12** F10 probe-fail + pin main_probe==0 one-shot. **A-SK13** ridefinire KS-AH-80-4 su `a_calls` steady, PRIMA della baseline R≥3.

**Kill-switch**: **KS-SK-81-1** claim di concorrenza W>2 senza osservabile che separi W da W−1 = ridotto d'ufficio a "≥2". **KS-SK-81-2** cifra il cui hash vive solo in un matrix-log sovrascritto = NULLA. **KS-SK-81-3** pin di censimento a somma aggregata dove esiste la forma per-file = gate non conforme. **KS-SK-81-4** soglia ex-ante su canali non disgiunti = predizione VOID, ridefinizione prima della baseline.

### 4. Hejlsberg — compilazione incrementale/feature-gating/igiene di build — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.** I miei A-AH10..13 sono eseguiti e verificati sul codice: quarta riga census nel gate e in CI (build E test), gate-census-twin operante (marker 18/11/2 + run-gate full-body sul binario census), driver ENFORCE (hash==riga matrix per modo, righe==N, depth>1⇒exit≠0, idle-probe), KS-AH-80-1..4 tutti mappati nel design79 (§7 eviction+len, §9 HIT vs oracolo, §10 soglia ≥90% ex-ante). Ma refuto e trovo buchi.

**(a) REFUTO il claim "la matrice non prova più la warning-freeness del binario hashato".** Il doppio pass è logicamente valido: pass-1 (`cargo rustc -- -D warnings`) e pass-2 (`cargo build`) compilano lo STESSO sorgente nella STESSA configurazione; se pass-1 passa, il sorgente non genera warning, dunque il binario hashato proviene da sorgente warning-free. Il -D non "nasconde" nulla: fallirebbe. Il buco VERO è un altro: nulla asserisce che l'albero sia identico tra i due pass e coincida col `git=$GIT_REV` loggato — un working tree SPORCO produce un matrix-log che certifica un rev che non è il sorgente compilato. **A-AH14**: `git status --porcelain` vuoto obbligatorio in testa a gate-feature-matrix.sh (o flag `dirty` nel log che rende ADVISORY ogni riga). Secondo buco: i bracket census vivono ora in php-runtime e php-cli, ma il perimetro -D resta php-server-only — il codice census cross-crate non passa MAI sotto -D warnings. **A-AH15**: aggiungere `cargo rustc -p php-runtime --features census-instrumentation -- -D warnings` (e php-cli) al gate e a CI.

**(b) Matrice weak-dep incompleta ma misure salve.** Nella build census (default attive) `php-cli?/census-instrumentation` si accende ✓. La quinta configurazione `--no-default-features --features census-instrumentation` (axum+census senza cli, la weak-dep esiste apposta) non è compilata da NESSUNO — né gate né CI: può bit-rottare in silenzio. Il driver la rifiuterebbe (hash assente dal log), quindi nessuna cifra può uscirne: il danno è di build, non di misura. **A-AH16**: o quinta riga compile-check in CI, o bandirla dichiaratamente.

**(c) CI: non è duplicazione, è il twin-check — ma va scritto.** `cargo test --features census-instrumentation` implica axum-server: gli stessi test axum girano due volte (plain e sotto census). Corretto — è l'equivalenza funzionale del gemello sui test — ma il commento CI non lo dichiara. Buco residuo: i test target della config axum-only (`--no-default-features`) non compilano mai da nessuna parte. **A-AH17**: dichiarare il doppio-run come twin-check + `--no-run` sulla config axum-only.

**(d) Il bump del marker census NON è auditabile.** `grep -c 'census-instrumentation'` conta STRINGHE, non attributi cfg: un commento che menziona la feature entra nel conto; aggiungere un sito reale rimuovendo un commento lascia il totale invariato — invarianza silenziosa; e il bump è un numero nudo, il gate non sa QUALI siti sono nuovi. **A-AH18**: matchare la forma `cfg(feature = "census-instrumentation")`, pinnare la lista per file (non il totale per crate), bump = diff enumerabile.

**(e) MAIN_CHAIN_FP: refuto il rischio catastrofico, non il rot.** La unit cache è in-memory per-processo: nessuna staleness cross-build è possibile (binario nuovo = cache vuota). Ma una "costante derivata da reg_mode+prelude" mantenuta a mano è doc-fiction: dopo tre edit del prelude nessuno la bumpa e diventa rumore semantico. **A-AH19**: fp COMPUTATO a init dal contenuto reale del prelude (hash dell'`include_str!`/forma lowered), mai letterale. **A-AH20**: censuscli non ha probe idle (solo MODE=census) — dichiarare il rumore o aggiungere la probe.

**Kill-switch:**
- **KS-AH-81-1**: matrix-log prodotto su tree sporco o senza assert tree==rev ⇒ log NULLO, misure a valle VOID.
- **KS-AH-81-2**: MAIN_CHAIN_FP letterale hand-maintained (non computato dal contenuto a init) ⇒ leva respinta.
- **KS-AH-81-3**: bump marker census senza lista dei siti nel medesimo commit ⇒ gate non conforme.
- **KS-AH-81-4**: configurazione feature compilabile assente da matrice+CI usata in qualunque run ⇒ cifra NULLA; entro WP-81 pinnarla o `compile_error!`.

### 5. Bak — alloc-rate/code-cache/path caldi — CONCORDO CON EMENDAMENTI

Verifica sul codice eseguita: `alloc_census.rs` (finestre a1/a3, stack nested), `compile/mod.rs:253-393` (prefisso contiguo fn/classi), `lower/mod.rs:298-307` (a1 su lower_prelude + note_prelude_counts), `vm/mod.rs:6787-6962` (A3Window RAII, chiusa prima di run_linked), `worker_pool.rs:372-543` (s1 PRIMA di vm_new: fase a = solo lower+compile; resid = s0(now)−s3(prev) su LAST_S3 thread-local), `server.rs:637-649` (riga census-cli), fixture include_heavy + heavy_lib1.

**VERDETTO: CONCORDO CON EMENDAMENTI.**

I miei A-BB10..15 sono ONORATI e verificati sul codice: seconda fixture reale (include_heavy, b non vuota); riga estesa con resid (A-BB11); canale c con controllo positivo teardown-allocante (KB-80-5); depth inc-before-send + anti-wrap (A-BB13/KB-80-4); chiave = fingerprint contenuto, gratis sul path server (A-BB14/KB-80-2); CPU nell'A/B §11.4 (A-BB15). Lo split ha fatto il suo mestiere: 81% prelude, KB-80-1 rispettato. Il design è il migliore del filone. Refuto però su sei punti.

**(a) a1 regge, ma a2 non è "main proper" puro.** Il prefisso contiguo è corretto (finestre chiuse al boundary idx==prelude_count, tail per programmi tutto-prelude). Però: le CLOSURES compilano tra le due finestre → a2; se il prelude contiene closure literals finiscono misattribuite ad a2 (da verificare, non asserito da nulla); `Vec::with_capacity(program.functions.len())` sta DENTRO a1 (riga 260) — slot utente billati al prelude. Piccolo, ma il canale su cui si predice va pulito o dichiarato.

**(b) a3 sovraconta il RUN degli autoload mid-lower.** La finestra nested netta solo il COMPILE nested; il run_linked dell'unit autoload-ata avviene dentro la finestra esterna e fuori dalla nested → billato ad a3 "own". Su fixture attuali ≈0 (class-decl run = poche centinaia di call); su WP con Composer files-array (codice eseguito al load) NON è epsilon. Quantificare con fixture autoload-che-alloca-al-run.

**(c) Il telescopio resid regge SOLO a workers=1.** `snap()` è GLOBALE, accumulatori e LAST_S3 thread-local: a W>1 la finestra a1 di un worker ingloba il traffico degli altri e resid copre richieste intere altrui. A W=1 il denominatore torna (Σ da req≥2 = s3(N)−s3(1); resid contiene fs::read del main, WorkerTask, oneshot, response-send, eprintln del req precedente — per costruzione). Req 1: resid=0 ⇒ init di processo NON contato: dichiararlo.

**(d) include_heavy è fixture-b, non proxy WP.** Steady (a3=0, unit cache): a1 ~74k vs b stimato 15-40k call ⇒ compile-share ~65-75%. Sufficiente per giudicare su ASSOLUTI; insufficiente per estrapolare percentuali a WP.

**(e) La predizione <4.000 è difendibile** — verificato che s1 sta PRIMA di vm_new: su HIT il path salta lower_source+compile_program interi, resta probe+stat+hash. MA il floor non-compile non è mai stato misurato: se il fingerprint hasha il source per-richiesta, il costo sta nel floor. Misurarlo ex-ante (bracket probe o fixture bare `<?php`).

**(f) CPU: totale singolo non basta.** `time -l` ingloba startup; verdetto CPU = slope a due N (100/200 req), stile WP-59.

**Emendamenti:**
- **A-BB16**: floor non-compile della fase a su HIT misurato ex-ante, PRIMA di dichiarare <4.000.
- **A-BB17**: righe split/resid DEFINITE solo a workers=1; il driver le rifiuta a W>1.
- **A-BB18**: bound di contaminazione autoload-run in a3, con fixture positiva.
- **A-BB19**: audit closures-nel-prelude + dichiarare resid(req1)=0 e with_capacity in a1.
- **A-BB20**: include_heavy etichettata fixture-b; vietato estrapolare % a WP.
- **A-BB21**: CPU A/B a due punti N (slope per-request).

**Kill-switch:**
- **KB-81-1**: cifra split a W>1 = VOID.
- **KB-81-2**: verdetto §10 senza floor ex-ante = predizione non falsificabile, VOID.
- **KB-81-3**: "a3 = ciò che il HIT salta" senza bound autoload-run = ADVISORY.
- **KB-81-4**: estrapolazione WP del rapporto compile/run da include_heavy = respinta.
- **KB-81-5**: CPU da totale singolo senza slope = ADVISORY.

### 6. Pedersen — disciplina di confine per-richiesta, lifecycle — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.** I miei A-PP1..A-PP6 e KS-PP-80-1..3 sono implementati e verificati sul codice: marker pinnato per forma+censimento+POSIZIONE (gate-capture-order con self-test bad_position); trigger dispatcher + Phase C reali (panic-hook globale in main.rs, abort 134 gateato); A-PP5 chiuso via rows==N nel driver; A-PP6 mappato in design79 §3/§4/§6/§9. La riga census-cli rispetta la permanent rule: legge SOLO contatori dopo `run_source_with_ini` (la capture vive dentro, prima del reset interno) e drena `take_split` incondizionatamente, anche su Err/panic. Il probe `/__census_global` non sporca alcun contatore per-richiesta (REQUESTS, QUEUE_DEPTH, LAST_S3/LAST_SPLIT intatti). Ma refuto quattro punti.

**A-PP10 — Il perimetro del gate è di nuovo indietro: il blocco census è una regione post-reset NON censita.** Le letture post-`request_end` in worker_pool.rs (r.501-543: take_split, LAST_S3, `retain.len()`, `vm.live_objects()`) sono lecite (non-capture), ma il gate pinna solo `.rendered/.stdout`: la regione è cresciuta da 2 a ~5 letture senza disciplina. Estendere il gate con un secondo censimento pinnato (letture `vm.`/`retain.` post-request_end in codice non-test == K, bump-in-commit). Inoltre il check posizione è debole: `in_test` è monotono per file (un `#[cfg(test)]` in commento/stringa lo arma per sempre) — ancorarlo alla regione reale.

**A-PP11 — Asimmetria di drain: gli early-return fatal dell'arm axum NON drenano.** Lower-fatal e compile-Unsupported ritornano PRIMA del blocco census (r.392/401/410): `take_split` non drenato e LAST_S3 stantio ⇒ la riga census della richiesta successiva ingloba il residuo a1 del compile fallito (il prefisso prelude di compile_program è già bracketato) e il resid assorbe l'intera richiesta fallita. rows==N intercetta la riga mancante ma NON il residuo — e la fixture F8 (fatal→fix→200) produrrà esattamente questa sequenza. Il CLI arm è già corretto: replicarne la forma (drain su ogni uscita, scope-exit) + test Unsupported→hello con bound su a1.

**A-PP12 — Negativa dispatcher incompleta.** L'ends_with è asserito in commento, non gateato: aggiungere in Phase B la near-miss ARMATA (`/__phpr_panic_dispatcherX` e `/x__phpr_panic` → lookup ordinario, server vivo). Nota positiva verificata: il trigger worker richiede il fixture su disco (fs::read precede dispatch) e il gate lo crea; l'assenza fallirebbe rumorosamente.

**A-PP13 — Il pin di design79 §4 copre il Module ma NON il Program.** Il RetainSet è `FrozenVec<Rc<Module>>`: non può parcheggiare `main_program: Rc<Program>`. Se il worker borrowa la entry invece di clonare l'Rc sullo stack, una eviction/supersede mid-request lascia la cache unico owner del Program in-flight — la stessa classe di KS-PP-80-2, fuori dal suo enunciato. Pinnare la forma clone-on-stack ed estendere KS-PP-80-2 alla coppia (Module, Program); DR-1 deve auditare ANCHE il grafo Program (HIR riusato cross-request), non solo Module. Su §6: F8 è falsificabile ma monca — manca F8b (fatal→STESSA richiesta→500 byte-identico, prova diretta del "mai pubblicato") e il caso LINK-fatal (post-compile-OK: dichiarare se il put sta prima o dopo `link_fatal_check`). Advisory: A-PP2 (scope scan = solo php-server) poggia su un audit dei caller di request_end pre-S-79.0 — rifarlo ora che il census tocca php-cli.

**Kill-switch:**
- **KS-PP-81-1**: riga census il cui canale include residui non drenati di una richiesta fallita precedente (early-return senza drain) ⇒ run census VOID.
- **KS-PP-81-2**: leva A-BB6 con Program non pinnato per la durata della richiesta, o DR-1 chiuso senza il grafo Program ⇒ reject della leva.
- **KS-PP-81-3**: nuova lettura post-request_end nel blocco census senza bump del censimento del gate nello stesso commit ⇒ gate FAIL.

### 7. Leijen — allocatore mimalloc/footprint fisico — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.** Recepimento dei miei punti verificato sui file: A-DL1 dichiarato (design78 §Identità S-79.0.4 + design79 §1 "churn LORDO, upper bound"); A-DL4 promosso a endpoint A/B (design79 §11.3); A-DL5 implementato (measure78.sh righe 121-126, probe self-cost + drift); KL-80-1/2 nel protocollo §11, KL-80-3 nella mappa. Ma quattro punti non reggono al mandato:

**A-DL6 — il supersede rimuove CHIAVI, non prova che liberi BYTE.** `unit_cache_put` (vm/mod.rs:15594-15611) fa `cache.remove` e incrementa `stranded_entries_dropped`: contatore di ENTRY, mai di byte. Tre buchi: (i) se la richiesta corrente ha parcheggiato il clone nel RetainSet, il free slitta a fine richiesta — bounded, ma va dichiarato; (ii) se il grafo Module acquisisce un ciclo Rc (lezione WP-71: i teardown Rc non raccolgono i cicli), il drop non libera NULLA e il contatore continua a salire — evidenza FALSA di pulizia; (iii) nessun osservabile in byte. Pretendo: `stranded_bytes_dropped` (module_census_bytes al sito di drop, build census) + probe sul gemello NON strumentato (workload di edit, V-prima/V-dopo, floor dichiarato, R≥3).

**A-DL7 — la misura retained di §11.1 non ha ancora lo strumento.** `module_census_bytes` (vm/mod.rs:16539) è feature **`mem-census`**, non `census-instrumentation`: la quaterna d'identità non ha una riga per il binario che produrrebbe la cifra. E il walker: (1) SALTA le entry Rc-shared (`strong_count > 1` ⇒ 0) — per il main cached il prelude È shared, cioè proprio i ~10,8MB dominanti: la cifra ×W può uscire sottostimata di 4/5; peggio, il put in cache ALZA strong_count e fa sparire byte genuinamente retained; (2) esclude i Const string bytes per costruzione; (3) il "walk del Program" NON esiste. Pretendo: strumento dichiarato (build + riga matrix), regola di ownership esplicita per il caso cache-as-second-owner, walker Program con controllo positivo su modulo sintetico di taglia nota.

**A-DL8 — finestra idle: interpretabile, ma per il canale giusto e su un solo arm.** Le probe `/__census_global` misurano CONTATORI (churn lordo): `MIMALLOC_PURGE_DELAY=0` è irrilevante per quei numeri — come bound del rumore cross-thread dei diff di fase la finestra è corretta. Ma NON dice nulla sul drift RESIDENTE idle (crescita arene), e il modo `censuscli` — quello che rende A-BB1 giudicabile — NON ha probe idle (measure78.sh: bracket solo su `MODE=census`). Dichiarare: (i) finestra idle = churn-only; (ii) probe idle anche su censuscli, o dichiarazione dei thread attivi su quell'arm; (iii) ogni claim di residenza idle esige twin union + vmmap + floor.

**A-DL9 — KL-80-3 recepito solo nella mappa: §11 non fissa il floor.** Il retained atteso (~13MB×W) sta sopra qualunque floor; il no-leak sul path HIT no: V2(N)=V2(2N) con vmmap a 0,1MB su 100 req = floor ~1KB/req. §11 deve citare il floor NUMERICO accanto a ogni claim e scalare N finché floor ≤ 1/10 dell'effetto cercato — vale anche per le probe V2 a W=num_cpus (KL-80-2), dove il floor è per-worker.

**A-DL10 — semantica lorda: in doc, non nell'emesso.** Il realloc a taglia piena (main.rs:82-86) è dichiarato in design78/79 ma né la riga census né il commento di `census_alloc` lo dicono. Ogni cifra nel report va etichettata "lorda/upper-bound"; consigliato tag `gross=1` nella riga.

**Kill-switch:**
- **KL-81-1**: claim "supersede libera memoria" senza `stranded_bytes_dropped` + probe residente sul twin con floor dichiarato = respinto.
- **KL-81-2**: cifra retained del main cached da walker senza controllo positivo di taglia nota o senza regola dichiarata per le entry Rc-shared = NULLA; A/B che la usa = VOID.
- **KL-81-3**: verdetto A-BB1 da righe censuscli senza finestra idle su quell'arm (o dichiarazione dei thread concorrenti) = NON GIUDICABILE.

### 8. Stogov — semantica Zend engine — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI** (serie A-DS di WP-81). Ho verificato NEL CODICE, non nel report.

**(b) A-DS1 CHIUSO.** `vm/mod.rs:3430-3434` ora recita: *"the RetainSet is the request-lifetime pin … The one cross-request survivor is the THREAD-LOCAL unit cache (WP-63/66 … the opcache analog)"*. La frase falsa è morta; la nuova è quella che avevo dettato.

**(a) Supersede-per-path** (`unit_cache_put` 15590-15612; test 16850 sul path REALE, 3 edit size-distinti, contatore che discrimina): KS-DS-80-1 chiuso. **REFUTO l'obiezione "più aggressivo di opcache"**: opcache è path-keyed a versione corrente UNICA — su alternanza A/B same-path ricompila a OGNI flip esattamente come il supersede; le entry vecchie diventano wasted, MAI più servibili (il restart le sotterra, non le resuscita). Il supersede rinuncia solo alla multi-versione (mtime,size) che opcache non ha mai avuto — ed era precisamente il leak. Blue-green via symlink: canonicalize ⇒ path canonici DISTINTI ⇒ il supersede non li incrocia, entrambi gli alberi restano caldi — MEGLIO di revalidate, non peggio. Il costo vero è altrove: la scan è O(entries) per ogni chiave nuova ⇒ cold-start O(N²); su WP (centinaia di path) si misura, non si presume.

**(c) Forma fedele §2-3: rispettata** (stessa cache TL, canonicalize+stat, fp del contenuto via `meta.source`, policy `pure`, M-68.5 a `vm/mod.rs:6939` ancora intatto — coerente, la leva non è scritta). **MA `MAIN_CHAIN_FP` costante è sottospecificato**: (i) `run_source_with_ini` accetta un vettore ini (oggi `&[]`) — se mai non-vuoto e compile-rilevante, il fp costante serve stale; (ii) autoload/include risolti DURANTE lower/compile del main incorporano nel Module decisioni prese contro una lib poi editata — il fp "stato vergine" non li vede. I due vhost: NON-problema (canonicalize distingue i docroot), ma per la ragione giusta, non per merito della chiave.

**(d) F1-F9: necessarie, non sufficienti.** Opcache m'ha insegnato tre paure che qui mancano: early-binding, __FILE__ cotto, strict_types.

**(e) censuscli** (`server.rs:622-651`): finestra = intero `run_source_with_ini` ⇒ confrontabile con axum solo su TOTALE+a1/a3 (manca a2/b/c). E il braccio CLI semina superglobali web REALI (`set_web_request`), il braccio axum no (A-BB4 deferito): su fixture che toccano `$_GET`/`$_SERVER` sono pere con mele; su hello il delta è ~0 solo SE il seeding è lazy — dichiararlo, non presumerlo.

## Emendamenti (serie WP-81)

- **A-DS7 — MAIN_CHAIN_FP enumerato.** Il design dichiara TUTTI gli input compile-rilevanti del main: fold del vettore ini (o assert-vuoto eseguibile) + copertura autoload-mid-compile (le unit toccate durante il compile del main entrano nel fp dello slot, oppure F10 deve falsificarlo).
- **A-DS8 — Tre fixture nuove dentro KS-DS-80-2**: **F10** early-binding (main `extends P` con P da lib; edit della lib ⇒ mai il binding vecchio) · **F11** `__FILE__`/`__DIR__` (costanti nel Module: due spelling che canonicalizzano allo stesso file ⇒ byte-parity con l'oracolo su HIT) · **F12** `declare(strict_types=1)` nel main cached (la coercion che deve fatal-are resta identica su HIT).
- **A-DS9 — Scan supersede**: misurare il cold-start O(N²) su corpus WP; oltre soglia dichiarata ⇒ indice per-path.
- **A-DS10 — censuscli**: dichiarare l'asimmetria superglobali nel verbale di misura; claim solo sui canali esistenti (totale, a1, a3) o aggiungere bracket a2.
- **A-DS11 — M-68.5**: riscrittura del commento 6939 NELLO STESSO commit del put del main + pattern doc-purge nuovo che lo esiga.

## Kill-switch

- **KS-DS-81-1**: HIT del main col binding VECCHIO dopo edit di una lib (F10) = leva RESPINTA — lo stale semantico è peggio dello stale testuale.
- **KS-DS-81-2**: `__FILE__`/`__DIR__` ≠ oracolo su HIT (F11) = REVERT immediato (riedizione mirata di KS-DS-80-3).
- **KS-DS-81-3**: confronto censuscli↔axum citato come verdetto SENZA la dichiarazione d'asimmetria superglobali = cifra respinta.

### 9. Gregg — metodologia di misura/attribuzione — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI — uno bloccante prima del design freeze.**

Ho campionato i raw. Esiti:

**(a) Smoke §1 = cifra da chat, non R=1 dichiarato — FINDING PRINCIPALE.** Non esiste NESSUN raw `census-cli:` nel repo (`grep -rl "census-cli:"` su measure-out, gate-axum/out, wp79-harness: vuoto; in measure-out non c'è alcun file `censuscli.*`). Il binario citato in design79 §1 (**37f630d6**) non compare in alcun feature-matrix.log né raw — solo in design79.md e nel session file. I totali citati (**83.834 call / 13,36MB**) non stanno in alcun log: l'unico raw con a1=74.288/10.825.612 è `wp78-harness/gate-axum/out/server.log` — righe **`census:` (braccio AXUM del twin-gate, binario 80f88ea7)**, con finestra 80.464-80.634 call / 13,0-13,09MB, NON 83.834/13,36. Solo a3 COLD (2.156/198.569, req=7) coincide al byte. KG-78.D è netto: cifra senza run tracciato = respinta. Il §1 va marcato **ADVISORY** e ri-derivato con driver ENFORCE + raw citato PRIMA del freeze; DR-1 non parte prima.

**(b) Idle: sottrazione dichiarata ma finestra corta e mono-braccio.** Il label `idle_window(+self)` e il commento dichiarano correttamente che p3−p2 include un self-cost — bene. Ma: probe SOLO in modo `census` (il braccio `censuscli`, quello dello smoke, NON ha idle window); IDLE_SECS=10s default non vede drift lenti. Nota: census.r1.log storico ha ZERO probe `census-global:` — la baseline S-78.1 è senza idle, dichiararlo.

**(c) ENFORCE bucabile — CONFERMATO IL BUCO GIT.** Il tail -1 su bin[row] è corretto (prende l'ultima riga). Ma il matrix log registra `git=36d62df` in testa e il driver **non lo confronta mai** col git corrente (**oggi 3a73be9 ≠ 36d62df**): il driver stampa `git=$GIT_REV` del TREE CORRENTE accanto all'hash di un binario di un tree precedente — misattribuzione d'identità attiva, non solo possibile. Fix obbligatorio: parse di `git=` dal matrix e refuse su mismatch.

**(d) Righe==N: OFF-BY-ONE REALE.** Il probe di boot curl-a la FIXTURE stessa: la prima richiesta riuscita produce una riga census. `census.r1.census` ha **111 righe** = 1(boot)+10+100; il check nuovo esige 110 ⇒ **VOID su ogni run legittima** (o peggio: "aggiustamenti" a mano). Fix: boot-probe su endpoint senza riga census, o EXPECTED=W+M+1 dichiarato.

**(e) Predizioni §10 incomplete.** Mancano **resid a caldo** (il canale di riconciliazione A-BB11 — su HIT la composizione cambia radicalmente; nei raw resid_bytes oscilla 0→552MB→66MB, canale instabile: senza predizione un leak ci si nasconde dentro) e **b a caldo** (~750 call/96KB nei raw). Inoltre la formula KS-AH-80-4 "a_calls+a1_calls" è ambigua: a1⊂a, doppio conteggio — riscrivere come "a_bytes E a1_bytes calano ≥90%".

**EMENDAMENTI:**
- **A-BG17 (bloccante)**: design79 §1 marcato ADVISORY; ri-derivazione R≥3 con driver ENFORCE, raw citati per path, binario dal matrix — prima di DR-1.
- **A-BG18**: driver verifica `git=` del feature-matrix.log == git corrente; mismatch ⇒ exit 1.
- **A-BG19**: boot-probe fuori dal canale census (o EXPECTED=W+M+1 esplicito); ricontare i raw storici con la regola nuova.
- **A-BG20**: idle window anche sul braccio censuscli + una run idle LUNGA (≥60s) per campagna.
- **A-BG21**: §10 esteso con predizione resid e b a caldo; formula KS-AH-80-4 riscritta senza doppio conteggio.

**KILL-SWITCH:**
- **KG-81-1**: cifra citata in un design senza raw log rintracciabile nel repo E binario nel matrix = cifra VOID d'ufficio (estende KG-78.D ai design file).
- **KG-81-2**: run di misura con git del matrix ≠ git del tree = VOID.
- **KG-81-3**: verdetto A-BB6 che non cita resid a caldo vs predizione = INCOMPLETO, non giudicabile.
