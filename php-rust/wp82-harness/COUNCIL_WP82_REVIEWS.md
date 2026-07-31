# COUNCIL_WP82_REVIEWS.md — Concilio a 9 sedie su S-80.0 (identity & channel repair + misura R≥3 + DR-1) + programma WP-81 (implementazione leva A-BB6)

**Data**: 2026-07-31 (chiusura S-80.0)
**Oggetto**: revisione di S-80.0 (commit a1dee58…7ddb6bc: punti 1-8 del
Concilio WP-81, misura di riferimento R=3, DR-1 macchina, design79
emendato) e giudizio del programma WP-81 (implementazione A-BB6 + fixture +
A/B).
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto
NEXT_SESSION, WP_SESSION_80, design79 emendato, MEASURE80_RESULTS, il
proprio ordine WP-81 e il CODICE/i raw del proprio perimetro; Gregg ha
ricomputato le cifre dai raw in proprio.
**Status**: verbali VINCOLANTI per WP-81 (blocco ⚖️ in NEXT_SESSION).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO CON EMENDAMENTI — nessuna opposizione.**
L'esecuzione S-80.0 è giudicata reale e verificata NEL CODICE e NEI RAW da
ogni sedia sul proprio ordine (A-TH9..14, A-MS7..12, A-SK9..13, A-AH14..20,
A-BB16..21, A-PP10..13, A-DL6..10, A-DS7..11, A-BG17..21 e TUTTI i
kill-switch WP-81: onorati). La misura R=3 regge al ricomputo indipendente
(md5 r1==r2==r3, cifre ricomputate riga per riga) con UNA eccezione: un
errore di trascrizione ×10 (b_calls include_heavy 2.798 → **27.982**,
Gregg A-BG22 BLOCCANTE) trovato dal ricomputo e CORRETTO in chiusura di
sessione con nota di provenienza. Il colpo strutturale della tornata è di
Stogov+Hoare sul DR-1: l'epoch chiude la classe "stato IC", NON la classe
"id run-scoped COTTI nei payload immutabili del Module cached" (A-DS12) —
e la leva stessa cambia la classe del wrap u32 dell'epoch da irraggiungibile
a silenzioso-a-50-giorni (A-TH16).

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **DR-1 copre le celle, non i payload** (Stogov A-DS12 — un ClassId
   cotto in un op immutabile è immutabile E stantio: il token-scan è cieco
   per costruzione; "il main ricompila deterministico" è frase del mondo
   pre-leva; Hoare A-TH16 — il wrap u32 dell'epoch diventa classe ATTIVA
   col main cached process-lifetime: u64 o bound macchina NELLO STESSO
   COMMIT del put): §5 si riscrive in DUE classi con enumerazione per-campo
   dei payload id-portanti + fixture F13 (ordine include variato) con
   controllo positivo [KS-DS-82-1, KS-DS-82-2, KH82-1].
2. **Il protocollo non è certificato dall'identità git** (Hejlsberg A-AH21
   — un edit degli script harness tra matrix e misura produce cifre sotto
   identità che non descrive lo script eseguito, e l'episodio del sommario
   idle lo DIMOSTRA; Klabnik A-SK14 — parser posizionali del driver senza
   pin di cardinalità né self-test; Gregg A-BG24 — il sommario driver non
   sta in alcun log committato): driver_sha nell'header run + porcelain
   esteso agli script + pin ==4 righe idle + self-test sintetico
   [KS-AH-82-1, KS-SK-82-1, KG-82-3].
3. **La trascrizione a mano è il buco che KG-81-1 non copre** (Gregg
   A-BG22: cifra rintracciabile ma SBAGLIATA ×10; convergente col principio
   di Hoare A-TH18 — claim senza comando): ogni tabella derivata nasce da
   uno SCRIPT raw→tabella committato; equivalenze tra commit citate col
   diff-vuoto comandato [KG-82-1, KH82-2].
4. **Il retained walker non è ancora uno strumento** (Leijen A-DL12 —
   visited-set per identità di puntatore sull'INTERO walk, controllo
   positivo a DUE moduli con sotto-grafo condiviso, etichetta "≥, Const
   esclusi"; Matsakis — budget ×W NULLO finché walker Program + regola
   Rc + controllo positivo non esistono; ×W è ESATTA perché Rc è !Send):
   [KL-82-2, KS-MS-82-2, KS-AH-82-4 sestetto mem-census].
5. **Le fixture della leva possono passare VACUE** (Klabnik A-SK16/17 —
   F-oneshot con campo assente ⇒ "==0" per assenza, serve il positivo
   gemello main_probe==nreq; "battery su HIT" senza pin main_hit può girare
   tutta su MISS; Pedersen A-PP17 — F8c body-only è quasi-vacua:
   link_fatal_check gira per-richiesta ⇒ 500 comunque, il discriminatore
   sono i contatori main_put==0/main_hit==0): [KS-SK-82-2, KS-SK-82-3,
   KS-PP-82-2].
6. **Il churn del fatal sparisce dal denominatore** (Pedersen A-PP14 — il
   drain scarta senza accumulatore: Δglobale=Σ(a+b+c+resid) vale solo
   fatal-free e il commento "the denominator reconciles" è ora falso
   incondizionato): accumulatori drained_* + identità condizionale
   dichiarata [KS-PP-82-1]. E il difetto mod-tests-non-ancorato appena
   corretto in check_marker_position è REINTRODOTTO in count_postend_reads
   (A-PP15) — stessa classe, stesso commit.
7. **bare.php si contraddice** (Bak A-BB22 — "a2(bare) = costo che nessun
   HIT può rimuovere" è FALSO sotto A-BB6 di 30×; il floor vero ex-post =
   a_calls(bare, HIT)): commento riscritto same-commit della leva + floor
   misurato bare-HIT nel verdetto [KB-82-1/2/3].
8. **"Idle drift 0" va scopato** (Leijen A-DL11 — il contatore è
   process-global e copre i worker parcheggiati, MA è cieco a dealloc, FFI
   malloc, arene mimalloc, page-dirtying; mimalloc non ha purge thread:
   con PURGE_DELAY=0 e zero traffico non scatta nulla): la frase è "0
   allocazioni Rust-allocator, tutti i thread" [KL-82-1].
9. **ORM/hk vanno rilanciati su ef90cb19 PRIMA del commit leva**
   (Hejlsberg A-AH23 — i fix warning toccano `calls`; se divergono DOPO la
   leva l'attribuzione è persa) [KS-AH-82-3].
10. **La porta vm_new resta aperta** (Matsakis A-MS13 — il sigillo copre il
    SAPI ma `vm_new(&RetainSet)`/`RetainSet::new` sono pub nel runtime):
    allowlist call-site nel commit leva [KS-MS-82-1].

### Ordine vincolante di apertura WP-81 (S-81.0 "seals & instruments", poi leva, poi A/B)

1. **Fix meccanici pre-leva**: A-PP15 (mod-tests ancorato + self-test
   negativo) · A-SK14/A-BG24 (pin cardinalità .idle==4 + self-test
   report_idle su log sintetico + sommario nel log committato) · A-MS14
   (TRIP_TEST_LOCK o clean-assert tollerante) · A-DL14 (gross su
   census-global) · A-AH25 (doc gate/driver coerente) · A-TH15 (commento
   OUTSTANDING ri-scoped: watermark>1 = run VOID, non "refutes") · A-TH17
   (DR-1 conta occorrenze via gsub, non righe) · A-TH18/KH82-2 (equivalenze
   tra commit col diff-vuoto comandato).
2. **A-AH23**: ORM 3E/13F + hk 1665 0E/0F ri-eseguiti su phpr ef90cb19
   PRIMA del commit leva.
3. **A-AH21**: driver emette driver_sha= (measure78.sh + gate) nell'header
   e nel raw; porcelain esteso a wp7*-harness/*.sh|*.pl.
4. **Nel commit leva (same-commit, pena i KS)**: A-TH16 epoch→u64 o bound
   macchina [KH82-1] · A-DS12 enumerazione payload id-portanti + A-DS14
   scelte §3 pinnate a una sola · A-AH22 MAIN_CHAIN_FP single-binding +
   falsificatore prelude-mutato [KS-AH-82-2] · A-MS13 allowlist vm_new
   [KS-MS-82-1] · A-BB22 bare.php riscritta + doc-purge [KB-82-1] ·
   A-PP16 pin SplitDrain-primo-return + A-PP17 F8c a contatori
   [KS-PP-82-2/3] · M-68.5/A-DS11 · contatori §8 con F-oneshot a tre denti
   [KS-SK-82-2].
5. **Strumenti §11 PRIMA dell'A/B**: A-DL12 walker con visited-set +
   controllo positivo a sotto-grafo condiviso [KL-82-2] · riga matrix
   mem-census = SESTETTO in gate+CI [KS-AH-82-4] · budget ×W SOLO dopo
   [KS-MS-82-2] · A-PP14 drained_* observable [KS-PP-82-1].
6. **Fixture**: F1-F12 + F13 (A-DS13, con controllo positivo) + F-probe +
   F-oneshot (tre denti) + battery-su-HIT col pin main_hit [KS-SK-82-3];
   deviazioni di fixture dichiarate (lezione goto-vs-Unsupported).
7. **A/B**: A-BB23/24/25 (floor bare-HIT, decomposizione se >5× floor,
   "bound su allocazioni non CPU") · A-DL13 (risoluzione CPU slope, soglia
   scan A-DS9, floor twin supersede ≥10×) · A-SK18/KG-82-2 (spread
   ri-derivato in-campagna, R≥3 obbligatorio — il determinismo WP-80 non è
   ereditabile) · KB-82-5 (autoload-run fixture prima dell'A/B o "HIT
   salta a3" resta ADVISORY) · KG-82-1 (tabelle da script raw→tabella).
8. Revert policy KS-DS-80-3 invariata: un body ≠ oracolo ⇒ git revert.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH82-1 | leva con epoch u32 senza bound macchina same-commit | DR-1 RIAPERTO, leva non autorizzata |
| KH82-2 | claim equivalenza-sorgenti tra commit senza diff-vuoto comandato | claim ADVISORY, cifre a valle degradano |
| KS-MS-82-1 | call-site nuovo vm_new/&RetainSet fuori allowlist nel commit leva | leva RESPINTA |
| KS-MS-82-2 | budget ×W senza walker Program + regola Rc + controllo positivo | budget NULLO, A/B VOID |
| KS-MS-82-3 | dedup in park_module con atteso F6 adattato al verde | gate non conforme |
| KS-SK-82-1 | sommario da parser posizionale senza pin di cardinalità | ADVISORY, mai verdict-grade |
| KS-SK-82-2 | fixture il cui PASS è compatibile con contatore mai wired | PASS vacuo, fixture non conforme |
| KS-SK-82-3 | claim "battery su HIT" senza pin main_hit per richiesta | ridotto a "path misto" |
| KS-SK-82-4 | R=1 sul braccio leva citando il determinismo WP-80 | cifra VOID |
| KS-AH-82-1 | sommario da script diverso da quello committato al rev citato | sommario NULLO, ricomputo dai raw |
| KS-AH-82-2 | MAIN_CHAIN_FP senza falsificatore A-AH22 nel commit leva | leva respinta |
| KS-AH-82-3 | A/B che cita ORM/hk con baseline non build-adiacente pre-leva | attribuzione nulla, A/B VOID |
| KS-AH-82-4 | config mem-census usata in un run senza riga gate+CI same-commit | cifra retained NULLA |
| KB-82-1 | commento bare.php contraddittorio al commit leva | floor claim VOID |
| KB-82-2 | verdetto <4.000 senza floor bare-HIT misurato | puntuale ADVISORY |
| KB-82-3 | a_calls(bare,HIT) > 200 | floor bucato: rideriva per nome, verdetti che lo citano VOID |
| KB-82-4 | claim CPU derivato dal floor alloc | respinto |
| KB-82-5 | fixture autoload-run dopo l'A/B o assente | "HIT salta a3" resta ADVISORY |
| KS-PP-82-1 | cifra census da run con fatal non marcata VOID | run VOID |
| KS-PP-82-2 | F8c chiusa body-only senza assert contatori | fixture VOID |
| KS-PP-82-3 | early-return prima della costruzione di SplitDrain | canale census VOID, gate FAIL |
| KL-82-1 | "idle drift 0" senza enumerazione dei canali ciechi | claim respinto |
| KL-82-2 | retained senza visited-set per puntatore o controllo positivo mono-modulo | cifra NULLA, A/B VOID |
| KL-82-3 | verdetto CPU/scan senza risoluzione numerica ex-ante | ADVISORY d'ufficio |
| KS-DS-82-1 | F13 body ≠ oracolo su HIT; o "stessi id per costruzione" senza enumerazione A-DS12 | leva RESPINTA / claim NULLO |
| KS-DS-82-2 | IC/fill nuovo non epoch-guarded o pin bump ≠1 senza ri-audit DR-1 | A/B VOID |
| KS-DS-82-3 | confronto censuscli↔axum su fixture $_GET/$_SERVER pre-A-BB4 | cifra respinta |
| KG-82-1 | cifra che diverge dal ricomputo scriptato dei raw | documento VOID finché riconciliato |
| KG-82-2 | banda di tolleranza senza spread osservato della propria campagna | NON giudicabile |
| KG-82-3 | numero derivato senza raw committato ricomputabile | ADVISORY d'ufficio |

---

## VERBALI INTEGRALI
### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

Verifiche sul codice, non sulla prosa: OUTSTANDING inc-prima-di-send (`worker_pool.rs:381`) e dec-DOPO-il-send della risposta (r.317→323) ✓; tripwire d==0 in `note_depth`/`note_outstanding` (r.129-148) + dec-su-0 al pickup (r.298-306) ✓; anti-wrap ri-scoped a "exercise rig" con rimando al vero falsificatore (r.1076-1086) e i due `#[should_panic]` mordono ✓; SAFETY catch_unwind corretta nei DUE modi (r.225-235) + pattern doc-purge "unwind itself still runs drops" bandito (gate-doc-purge r.41) ✓; assert a1≤a FATAL prima del saturating_sub (r.654-659) ✓; SplitDrain su ogni uscita + falsificatore diretto `fatal_early_return_drains_split_residue` ✓; A1Window/A3Window RAII `!Send` compile-time (assert_not_impl_any, alloc_census.rs:215-216), token LIFO con trip su drop fuori ordine, provider-guard mid-window ✓; sigillo A-MS3 (execute_request costruisce dentro, `execute_with_retain` privato) ✓; DR-1 a verdetto macchina con decoy self-test, allowlist, test epoch eseguiti sotto cargo, wiring pinnato ==1 ✓; raw committati (312 file `*.80.*`, matrix-archive, corpus80/refl80.fails) ✓. Risposte alle domande obbligate: sì, il dec-after-send dà watermark 2 su una-in-esecuzione-più-una-accodata (entrambe contate tra inc-dispatch e dec-post-send); sì, d==0 è raggiungibile solo via wrap (fetch_add ritorna ≥0 ⇒ d=old+1≥1 in aritmetica vera; d==0 ⇔ old==MAX ⇔ wrap pregresso; il dec-su-0 al pickup coglie direttamente la regressione inc-after-send); sì, la SAFETY è ora vera e la frase vecchia è irripetibile per gate. KH81-1, KH81-2, KH81-3 sono ONORATI NEL CODICE.

**VERDETTO: CONCORDO CON EMENDAMENTI**

**A-TH15 — Il commento di OUTSTANDING sovra-promette.** r.111-115 recita "watermark >1 REFUTES the mechanism": falso in senso stretto. Il dec è *sequenced-after* il send della oneshot ma non atomico con esso: un worker deschedulato tra r.317 e r.323 lascia il client chiudere il giro e il dispatch successivo osservare 2 su un client GENUINAMENTE closed-sequential. L'errore è fail-closed (invalida una run buona, mai benedice l'overlap) — direzione giusta, asimmetria accettabile — ma il commento va ri-scoped: "watermark>1 = run VOID (overlap O finestra send→dec)"; il giudice resta l'enforcement inflight_max≤1 del driver.

**A-TH16 — Il wrap u32 dell'epoch NON resta ADVISORY sotto la leva: è la leva a cambiarne la classe.** Oggi le celle IC muoiono col Module per-richiesta: il wrap è irraggiungibile. A-BB6 rende le celle del main cached PROCESS-LIFETIME: 2³²−1 bump ≈ 4,3e9 richieste/thread — a 1.000 req/s/worker ~50 giorni, e l'esito è un hit stantio SILENZIOSO (slot di una classe morta). Il residuo "recorded, not gated" è corretto per il tree di OGGI e scorretto per il tree post-leva. Ordine: epoch → u64 (costo: 8 byte/cella, zero atomics) oppure bound macchina dichiarato, NELLO STESSO COMMIT del put del main; il residuo DR-1 si ri-deriva lì.

**A-TH17 — Il census DR-1 conta RIGHE, non token.** Un secondo token di mutabilità contrabbandato su una riga già allowlisted (contiene "PropIc") passa sia il conteggio ==3 sia l'allowlist. Contare le occorrenze (gsub) o pinnare il corpo esatto delle 3 righe.

**A-TH18 — Un claim d'identità senza comando.** "Battery PASS a 1305327, stesse sorgenti crates di 6910767" è asserito, non comandato: il report citi `git diff --stat 1305327 6910767 -- crates/` == vuoto (comando che può fallire). Tutto il resto del report è comandato.

**Kill-switch:**
- **KH82-1**: leva A-BB6 atterrata con epoch u32 senza bound macchina nello stesso commit = DR-1 RIAPERTO, leva non autorizzata.
- **KH82-2**: claim di equivalenza-sorgenti tra commit senza il diff-vuoto citato = claim ADVISORY, ogni cifra che vi poggia degrada con esso.
### 2. Matsakis — ownership/aliasing/Send-Sync — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI — nessun bloccante residuo; i miei A-MS7..A-MS12 e KS-MS-81-1/2/3 sono ONORATI NEL CODICE.**

**Risposte obbligate (verificate su `alloc_census.rs` e `worker_pool.rs`, non su prosa):**

1. **A-MS3/KS-MS-81-3: SODDISFATTO al confine SAPI, più forte del richiesto.** `execute_request` (riga 409) costruisce il RetainSet DENTRO; `execute_with_retain` (465) è `fn` module-private. Di più: `mod implementation` è privato e il re-export (1218) espone solo WorkerPool/Context/Task/Meta — nemmeno `execute_request` esce dal file; l'unico ingresso di produzione è `worker_loop`. I test in-modulo usano tutti la forma worker_loop (RetainSet fresco, verificato riga per riga). **Residuo dichiarato**: `php_runtime::vm_new(&RetainSet, …)` e `RetainSet::new` restano API pub del runtime — il sigillo copre il SAPI, non la porta secondaria vm_new (→ A-MS13).

2. **Token LIFO A3Window: contabilità corretta.** Drop non-top ⇒ `trip()` + `truncate(depth−1)`; le finestre interne scartate NON entrano nei dati ma **ogni loro drop successivo trippa a sua volta** (il test `a3_window_non_lifo_drop_trips` lo asserisce sul secondo drop). Caso outer-first: truncate scarta anche i frame vivi sopra, i cui guard trippano al drop (len≠depth). Nessuna misattribuzione può sopravvivere: KS-MS-81-2 voida l'intera run al primo trip — fail-loud, mai assorbito.

3. **Provider guard: SÌ nei DUE tipi** (A1 in `close()` 125-128; A3 nel Drop 192-196). In A3 il guard sta DOPO il pop — corretto: lo stack resta coerente, e il parent che perderebbe il `gross` dell'inner trippa anch'esso al proprio drop (OnceLock monotono ⇒ `installed` false su tutta la catena). Sound.

4. **SPLIT_TRIP cross-processo: il timore è INFONDATO.** È uno static per-processo: i tripwire test vivono nel test-harness binary, la misura nel binario php-server — processi distinti, contatore che parte da 0. Un trip in-cargo NON può voidare una run di misura. **Dentro** il test-process invece `a1_window_close_is_idempotent_and_clean` asserisce delta==0 su contatore process-wide con test paralleli: un tripper concorrente lo fa fallire flaky (fail-closed, ma rumore — A-MS14).

5. **F6==3 vs FrozenVec: COERENTE.** `len` = numero di push = park-EVENTI per costruzione (FrozenVec non deduplica): l'invariante su eventi è esattamente la forma che una struttura append-only può garantire; due cloni dello stesso `Rc<Module>` muoiono entrambi col RetainSet. Rischio residuo: un dedup futuro in `park_module` farebbe passare F6 col numero sbagliato (→ KS-MS-82-3).

6. **Budget ×W (A-MS12): NON dichiarabile OGGI.** Il walker del Program non esiste; il walker Module azzera le entry Rc-shared e il put in cache ALZA strong_count del prelude ⇒ sottostima sistematica proprio del pezzo grosso; manca il controllo positivo a taglia nota. La sequenza WP-81 (tool §11 al passo 3, PRIMA dell'A/B) è ordinata correttamente — va vincolato che il numero preceda la PRIMA lettura del braccio leva (→ KS-MS-82-2). Inoltre la cache è thread-local: "×W" esige entries PER WORKER o la dichiarazione del warm-up asimmetrico.

**Emendamenti:**
- **A-MS13**: allowlist/grep-gate dei call-site `vm_new(&RetainSet, …)` fuori dai path SAPI sigillati, nello stesso commit della leva — la porta vm_new resta l'unico modo di riusare un RetainSet.
- **A-MS14**: tripwire test serializzati (TRIP_TEST_LOCK, come DEPTH_TEST_LOCK) o clean-assert tollerante ai delta concorrenti.
- **A-MS15**: A1Window non-nesting è prosa — flag thread-local open-while-open ⇒ trip (stesso canale SPLIT_TRIP).
- **A-MS16**: dispatch fallita lascia il watermark gonfiato di un fantasma (note_* pre-send, undo solo sui contatori): dichiarare che una run con dispatch Err è VOID (rows==N già la becca — scriverlo).

**Kill-switch:**
- **KS-MS-82-1**: call-site nuovo di `vm_new`/`&RetainSet` fuori allowlist nel commit leva ⇒ leva RESPINTA.
- **KS-MS-82-2**: budget ×W citato senza walker Program eseguito + regola Rc cache-as-owner + controllo positivo a taglia nota ⇒ budget NULLO, A/B VOID.
- **KS-MS-82-3**: dedup introdotto in `park_module` con atteso F6 adattato al verde (anziché ricalcolato PER NOME nel design) ⇒ gate non conforme.
### 3. Klabnik — spec/testabilità/gate — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.**

Verifica dei miei A-SK9..13 e KS-SK-81-1..4 sul codice: **tutti onorati nella sostanza**. A-SK9: soglie derivate e stampate (gate-concurrent.sh:133-139), esca ramo FAIL (r.222-227), body W=1 su log proprio e ricontrollati (r.205-211), verdict "overlap>=2" (r.235). A-SK11: match esatto `^bin\[row\] ` con bracket+spazio (measure78.sh:107), archivio per-run (r.118), cross-check `git=`↔HEAD (r.90-96), source-porcelain (r.100-105). A-SK12 in design79 §9 (F-probe/F-oneshot). A-SK13: KS-AH-80-4 v2 su `a_calls` sola, same-commit della baseline — ex-ante rispetto al braccio leva, accettabile. KS-SK-81-2/3 nel driver e nel gate per-FILE con decoy positivi e unpinned-file=FAIL.

**Risposte obbligate.** (1) **Pin per-FILE aggirabile**: sì — sostituzione intra-file a conteggio pari è invisibile (il pin è un COUNT, non i siti nominati che A-SK10 chiedeva; il bump-rule copre solo i bump). Il backstop è il dente 2 (battery full-body sul binario census), ma il residuo non è DICHIARATO nel gate: né accettato né chiuso = buco di spec. (2) **Esca ramo FAIL**: esercita il PREDICATO condiviso (`overlap_check`) su dato seriale e ne pretende il fallimento — sostanzialmente valida; non esercita il call-site shell del check principale (un `if` invertito lì passerebbe). Da dichiarare. (3) **Fix tail-3 NON pinnato**: nessun pin di cardinalità sulle righe census-global (attese ==4: boot+3 probe), nessun self-test su log sintetico. È un parser posizionale che sostituisce un parser posizionale — la lezione S-80.0 ⭐⭐1 applicata a metà: può regredire in silenzio esattamente come l'originale. (4) **Spread 0,0%**: dichiarato apertamente (Scoperta 3) e il protocollo TIENE R≥3 (design79 §11.2) — corretto; ma nessuna regola vieta il futuro shortcut R=1 "perché WP-80 era deterministico". Il determinismo è proprietà PER-BUILD: canonicalize/stat/hash della leva sono sorgenti nuove. (5) **F-oneshot pre-leva**: NON testabile prima (il contatore non esiste); §9 la colloca correttamente post-implementazione/pre-A/B, MA com'è scritta può passare VACUA: campo assente ⇒ "==0" per assenza, e nessuna fixture pinna `main_probe>0` da nessuna parte (F5 pinna main_hit, F-probe pinna main_probe_fail). (6) **Fixture §4**: F1/F3-F8c/F10-F12/F-probe falsificabili; F2 pinna solo il body (serve l'atteso in contatori); F9 "si muove"≥1 accettabile; **"battery rilanciata su HIT" non ha prova d'essere su HIT** — senza pin main_hit per richiesta può girare tutta su MISS e benedire il nulla.

**Emendamenti:**
- **A-SK14**: driver — pin cardinalità `$RUN.idle` (==4 su arm census) + self-test su log sintetico (4 righe note ⇒ cifre attese; 5 ⇒ FAIL). Ogni parser posizionale del driver ottiene il suo pin.
- **A-SK15**: gate-census-twin — dichiarare nel header il residuo same-count-substitution col backstop dente-2, OPPURE pinnare l'output `grep -n` per file. Terza via non esiste.
- **A-SK16**: F-oneshot a tre denti: campo `main_probe` PRESENTE (assente≠zero, coerenza KS-MS-81-2), ==0 sul one-shot, positivo gemello `main_probe==nreq` sull'arm server nella stessa campagna.
- **A-SK17**: "battery su HIT" = pin `main_hit` atteso per richiesta durante la battery; F2 con atteso in contatori (`main_put`+1), non solo body.
- **A-SK18**: R≥3 con check spread==0 in-protocollo OBBLIGATORIO sul braccio leva; il determinismo WP-80 non è ereditabile. Dichiarare che l'esca A-SK9b copre il predicato, non il call-site.

**Kill-switch:**
- **KS-SK-82-1**: sommario derivato da parser posizionale senza pin di cardinalità sul canale = ADVISORY, mai verdict-grade.
- **KS-SK-82-2**: fixture il cui PASS è compatibile con un contatore mai wired (campo assente o zero-sempre senza positivo gemello) = PASS vacuo, fixture non conforme.
- **KS-SK-82-3**: claim "battery su HIT" senza pin main_hit per richiesta = ridotto d'ufficio a "battery su path misto".
- **KS-SK-82-4**: R=1 sul braccio leva citando il determinismo WP-80 = cifra VOID; il determinismo si ri-dimostra per build.
### 4. Hejlsberg — compilazione incrementale/feature-gating/igiene di build — CONCORDO CON EMENDAMENTI

## VERDETTO: CONCORDO CON EMENDAMENTI

Emendamenti A-AH14..A-AH20 e KS-AH-81-1..4: **onorati nei fatti, verificati sui file**. A-AH14: clean-tree FULL-porcelain in testa a gate-feature-matrix.sh con poison del log live (`NULL: matrix run refused…`) — fail-CLOSED. A-AH15: `lint_crate` php-runtime/php-cli nel gate E in ci.yml, più i due step test census cross-crate. A-AH16/KS-AH-81-4: quinta config `census-axum-only` costruita e hashata (4c6264…) nel gate e compilata `-D warnings` in CI. A-AH17: doppio-run dichiarato come twin-check + axum-only `--no-run`. A-AH18/KS-AH-81-3: gate-census-twin pinna la FORMA cfg per-FILE (39 siti/7 file, somma verificata), raw==form, sweep unpinned, decoy positivi. A-AH19: design79 §3 emendato. A-AH20: idle su entrambi gli arm, `report_idle` tail-3. KG-81-2: match `git=` matrix↔HEAD nel driver, riga esatta `^bin\[row\] `, archivio per-run. I due archivi (1305327/6910767, quintetto hash IDENTICO) provano empiricamente che i commit harness-only non muovono i bit: igiene reale, non dichiarata.

**Risposte alle domande obbligate:**

1. **Matrix mid-campaign**: nessun falso-verde. Un ri-run su tree sporco avvelena il log live → il driver trova `git=` vuoto → refuse. Fallisce chiuso. Residuo COSMETICO: il commento del gate ("raws tracked or gitignored by design") contraddice il driver ("expected untracked mid-campaign") — sanare la doc (ADVISORY).

2. **Script harness fuori dal source-clean: BUCO REALE.** Il driver è il PROTOCOLLO (EXPECTED lines, tripwire, tail-3): un edit di measure78.sh tra matrix e misura produce cifre sotto identità git che non descrive lo script eseguito. L'episodio S-80.0.6 lo dimostra: raw giusti, sommario derivato sbagliato — dallo script, non dal binario. I raw committati permettono la ricomputazione, ma il sommario citato nei RESULTS è output dello script. → A-AH21.

3. **MAIN_CHAIN_FP**: la forma NON basta. "hash della forma lowered/include_str!" ammette due input diversi e non esige che l'input hashato sia LO STESSO binding che lower_prelude consuma: un hash di una COPIA costante del prelude soddisfa la lettera ed è il letterale mascherato con passaggio in più. Manca il falsificatore. → A-AH22.

4. **Quinta config senza test**: per la MISURA basta (il driver non seleziona mai quella riga: garanzia di build, non d'identità). Per la BUILD no: i suoi test-target non compilano da nessuna parte — stessa classe già sanata per axum-only con A-AH17. Asimmetria ingiustificata. → A-AH24.

5. **A-AH15 → bit di phpr mossi (ef90cb19): la ri-validazione NON basta.** Corpus 1418+refl 290+workspace coprono molto, ma la regola ORM-gate è MANDATORY per cambi ref/arg/reflection e i fix toccano `calls`. "Il corpus copre" è asserito, non dimostrato. Peggio: se ORM/hk girano solo DOPO la leva e divergono, l'attribuzione warning-fix-vs-leva è persa — esattamente il debito che una baseline incrementale non può portarsi dietro. → A-AH23, bloccante PRIMA del commit leva.

## Emendamenti (serie A-AH)

- **A-AH21**: il driver emette `driver_sha=` (shasum di measure78.sh + gate invocati) nell'header run e nel raw; porcelain-check esteso a `wp7*-harness/*.sh|*.pl` (gli script, NON measure-out).
- **A-AH22**: MAIN_CHAIN_FP = hash del MEDESIMO binding in memoria consumato da lower_prelude (single-binding, niente copie); falsificatore obbligatorio: prelude iniettato test-only mutato ⇒ fp cambia; reg_mode nel fp se il lowering ne dipende.
- **A-AH23**: ORM 3E/13F + hk 1665 0E/0F ri-eseguiti su phpr ef90cb19 PRIMA del commit leva.
- **A-AH24**: `cargo test -p php-server --no-default-features --features census-instrumentation --no-run` in CI (simmetria A-AH17).
- **A-AH25** (ADVISORY): sanare la contraddizione doc gate/driver sui raw untracked.

## Kill-switch (serie KS-AH-82)

- **KS-AH-82-1**: sommario derivato da uno script diverso da quello committato al rev citato ⇒ sommario NULLO; si ricomputa dai raw.
- **KS-AH-82-2**: MAIN_CHAIN_FP senza falsificatore A-AH22 nel commit leva ⇒ letterale mascherato, leva respinta (estende KS-AH-81-2).
- **KS-AH-82-3**: A/B WP-81 che cita verdetti ORM/hk con baseline diversa dal binario pre-leva build-adiacente ⇒ attribuzione nulla, A/B VOID.
- **KS-AH-82-4**: la riga matrix `mem-census` promessa in design79 §11.1 allarga il quintetto a SESTETTO: config usata in un run senza riga nel gate+CI stesso-commit ⇒ cifra retained NULLA (estende KS-AH-81-4).
### 5. Bak — alloc-rate/code-cache/path caldi — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.**

Verifica sul codice eseguita: `worker_pool.rs:372-560` (s0 prima di lower, s1 prima di vm_new, SplitDrain su ogni uscita, OUTSTANDING inc-before-send/dec-post-send), `measure78.sh:44-128` (census a W>1 RIFIUTATO — KB-81-1 ✓), `analyze80.pl` (medie steady, max su depth/inflight/trip, gross=1 propagato), `bare.php`, MEASURE80, design79 emendato.

**I miei A-BB16..21 e KB-81-1..5 sono ONORATI**: floor ex-ante dichiarato prima della leva (A-BB16); driver rifiuta split a W>1 (A-BB17, verificato alle righe 126-128); fixture autoload-run ordinata al posto giusto — NEXT_SESSION passo 3 "strumenti §11 PRIMA dell'A/B" scioglie l'ambiguità posizionale del §11.8, e KB-81-3 resta correttamente ADVISORY finché la fixture positiva non esiste (a3=0 steady su TUTTE le fixture: il canale non ha ancora un controllo positivo run-side); closures-audit + resid(req1)=0 + with_capacity dichiarati (A-BB19); include_heavy etichettata fixture-b senza estrapolazioni WP — b=2.798/1,87MB è dichiarato su ASSOLUTI e basta (A-BB20; noto onestamente: la mia stima 15-40k call era 10× sopra il misurato — il compile-share reale sulle fixture è ~96% dei call, ragione in più per KB-81-4); CPU a slope due-N in §11.4 (A-BB21). La misura R=3 spread 0,0% con inflight_max=1 su 2.860 righe è verdict-grade.

**Sul floor 18,5 call/unit: la derivazione REGGE.** b(include_gate)−b(hello)=37/2 include probe+park+relocation dell'include-HIT; il main-HIT avrà park ma NON relocation ⇒ la cifra è upper bound nella direzione giusta. Il bookkeeping (~41-44) vive nel canale resid, disgiunto da a: sommarlo nel floor è doppio-conservativo, accettabile per un bound. Caveat obbligatorio: il census conta ALLOCAZIONI — l'hash di `meta.source` è alloc-invisibile, quindi il floor alloc non dice nulla sul floor CPU (per quello esiste solo KB-81-5).

**REFUTO su bare.php**: il commento dichiara «a2(bare)=6.162 = the fixed per-request cost that no cache HIT can remove». FALSO sotto A-BB6: la leva cacha la UNIT MAIN, su HIT a2 va via INSIEME ad a1 (MEASURE80 lo dice: «la quota che la leva DEVE rimuovere: a1+a2»). Il contratto si contraddice di 30× (6.162 vs ≤200) nel file che il floor cita come proxy. Doc-fiction da purgare.

**Su <4.000 vs floor 200**: falsificabile sì (KB-81-2 soddisfatto), ma tra 200 e 4.000 ci sono 19× di spazio dove un clone per-richiesta da ~3.000 alloc passerebbe «verificato». Il vincolo VOID vero è KS-AH-80-4 v2 (<8.048); la puntuale <4.000 va accompagnata da rendicontazione contro il floor MISURATO — e bare su HIT È il floor misurato gratis.

**Emendamenti (serie A-BB):**
- **A-BB22**: commento bare.php riscritto NELLO STESSO COMMIT della leva (a2(bare) = costo che il HIT RIMUOVE; floor = probe+lookup) + pattern doc-purge sulla frase vecchia.
- **A-BB23**: floor ex-post = `a_calls(bare, HIT)` misurato; predizione ≤200; delta hello−bare su HIT dichiarato ≈0.
- **A-BB24**: se `a_calls(HIT)` > 5× floor (>1.000), decomposizione per-componente (canonicalize/stat/lookup/park/bookkeeping) obbligatoria nel verdetto.
- **A-BB25**: ogni citazione del floor dichiara «bound su allocazioni, non CPU»; il costo hash/stat alloc-invisibile è giudicato SOLO dalla slope due-N.
- **A-BB26**: se la forma del probe main cambia (es. fs::read aggiuntivo), il floor si RIDERIVA per nome, mai ereditato.

**Kill-switch (serie KB-82):**
- **KB-82-1**: commento bare.php contraddittorio ancora presente al commit leva ⇒ floor claim VOID.
- **KB-82-2**: verdetto «<4.000 PASS» senza floor misurato bare-HIT e rapporto a_calls/floor ⇒ puntuale ADVISORY (resta VOID-gate solo la v2 ≥90%).
- **KB-82-3**: `a_calls(bare,HIT)` > 200 ⇒ floor ex-ante bucato: rideriva per nome, verdetti che citano il vecchio floor VOID.
- **KB-82-4**: claim CPU derivato dal floor alloc ⇒ respinto.
- **KB-82-5**: fixture autoload-run eseguita DOPO l'A/B o assente ⇒ ogni enunciato «il HIT salta a3» resta ADVISORY e il verdetto lo marca.
### 6. Pedersen — disciplina di confine per-richiesta, lifecycle — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.**

**Emendamenti WP-81 onorati**: A-PP12 pienamente (near-miss ARMATE suffisso+prefisso in Phase A, server-alive verificato). A-PP13 pienamente (design79 §4 coppia clone-on-stack, DR-1 estende al grafo Program, F8b/F8c in §6/§9). A-PP11 sostanzialmente: drain RAII dichiarato a r.507, PRIMA di tutti e tre gli early-return (lower-Fatal r.521, parse-error r.530, compile-Unsupported r.539) — copertura per costruzione; ma il test prescritto era Unsupported→hello, consegnato goto-fatal (deviazione ragionevole: validate_goto post-seeding GARANTISCE residuo a1 pendente; va DICHIARATA, non taciuta). A-PP10 parzialmente: censimento pinnato ==2 e `cfg(test)` ancorato in check_marker_position — ma vedi A-PP15.

**Risposte obbligate.**

1. *LAST_S3 al drop*: la posizione è GIUSTA — snapshot a scope-exit, dopo il format! del body d'errore (traffico del fatal nel suo perimetro), prima del send (che cade nel resid successivo come sul path normale: simmetrico). Ma sì, **il churn del fatal SPARISCE dal denominatore**: nessuna riga lo riporta, il drop scarta la coppia senza accumulatore. Δglobale=Σ(a+b+c+resid) vale solo su run fatal-free; rows==N lo segnala SOLO sotto driver ENFORCE, e il commento r.641 ("the denominator reconciles") è ora falso incondizionato. L'alternativa (LAST_S3 stantio) era peggiore — inquinava resid violando KG-81-3. Scelta giusta, contabilità incompleta.

2. *Pattern `(vm|retain)\.`*: `self.vm.` matcherebbe (regex non ancorata); un rebinding (`let m = &vm`) elude — accettabile come tripwire lessicale in un file solo, NON come prova; è la stessa classe di debolezza del ban capture, accettata da WP-78. **NON accettabile invece**: `count_postend_reads` arma `in_tests` su `/mod tests/` NON ancorato — un commento o stringa contenente "mod tests" acceca il censimento per il resto del file. È ESATTAMENTE il difetto che A-PP10 ha appena fatto correggere in check_marker_position, reintrodotto nell'awk nuovo dello stesso commit.

3. *Copertura drain*: RAII copre i tre rami per costruzione (guard prima di ogni return); controllo positivo solo lower-fatal. Sul parse-error il residuo può essere 0 (drain no-op innocuo). Rischio residuo: nessun pin impedisce a un futuro early-return di infilarsi tra census_s0 (r.483) e la costruzione del guard (r.507) — finestra oggi senza return, ma un commento non è un gate.

4. *F8c ex-ante*: **NO, la posizione del put NON è gateabile prima che il put esista** — e F8c body-only (500→200) resta quasi-vacua anche DOPO: un put mal posizionato cacherebbe il Module compile-OK, ma link_fatal_check gira per-richiesta sul Vm nuovo → 500 comunque. Il discriminatore vero è sui CONTATORI §8: link-fatal richiesto DUE volte ⇒ `main_put==0`, `main_hit==0`, entrambi 500 byte-identici; fix ⇒ 200 con `main_put==1`. Gateabile SOLO same-commit del put (contatori inclusi) + pin strutturale del call-site.

**Emendamenti (serie A-PP).**

- **A-PP14** — Il churn drenato diventi OSSERVABILE: accumulatori globali `drained_calls/bytes` (build census) riportati a teardown, così Δglobale=Σrighe+drained riconcilia anche con fatal; commento r.641 annotato "fatal-free only"; MEASURE-doc dichiara l'identità condizionale.
- **A-PP15** — Ancorare `/mod tests/` alla forma reale di riga (`^[[:space:]]*mod tests`) + self-test negativo: snippet con "mod tests" in commento NON deve accecare il conteggio.
- **A-PP16** — Pin strutturale: costruzione SplitDrain lessicalmente prima del primo `return` di execute_with_retain (awk nel gate); deviazione goto-vs-Unsupported dichiarata nel verbale di sessione; LAST_SPLIT resta STANTIO dopo un drop — documentare o sentinellare (trappola per controlli in-cargo futuri).
- **A-PP17** — F8c in forma contatori (doppio link-fatal ⇒ put==0/hit==0, 500 byte-identici; fix ⇒ 200, put==1), armata NELLO STESSO COMMIT del put, con pin grep put-dopo-link_fatal_check.

**Kill-switch (serie KS-PP-82).**

- **KS-PP-82-1**: cifra census citata da una run contenente un fatal (rows≠N o drain scattato) senza marcarla VOID ⇒ run VOID.
- **KS-PP-82-2**: F8c chiusa body-only, senza assert sui contatori main_put/main_hit ⇒ fixture VOID, la leva non può appoggiarvisi.
- **KS-PP-82-3**: nuovo early-return in execute_with_retain prima della costruzione del guard SplitDrain ⇒ canale census VOID, gate FAIL.
### 7. Leijen — allocatore mimalloc/footprint fisico — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI — nessun bloccante sul S-80.0; tre precisazioni obbligatorie prima dell'A/B.**

Recepimento verificato SUI FILE: A-DL6 → design79 §7 (`stranded_bytes_dropped` + probe twin, impl. deferita e DICHIARATA in WP_SESSION_80 §Residui — corretto); A-DL7 → §11.1 (riga matrix per mem-census, regola Rc-shared, walk Program da scrivere, controllo positivo); A-DL8/A-BG20 → measure78.sh:183-188 (bracket su ENTRAMBI gli arm census, commento "CHURN-ONLY… says nothing about resident idle growth") + MEASURE80 §Idle intestato "churn-only"; A-DL9 → §11.3 (floor vmmap 0,1MB ⇒ ~1KB/req su 100, N scalato a floor ≤ 1/10, per-worker su W=num_cpus); A-DL10 → `gross=1` in-band (worker_pool.rs:668-676). KL-81-1/2 vivono come obblighi in §7/§11.1; KL-81-3 soddisfatto. Il sommario idle off-by-one (boot-probe → 4 righe census-global, awk head-anchored) è stato trovato e fixato tail-3 nello stesso commit: esattamente la classe della mia lezione WP-64 sui probe come API — bene.

**A-DL11 — "idle drift VERO = 0" è impreciso.** Il contatore è PROCESS-GLOBAL (main.rs:63-88): copre anche i worker parcheggiati in `blocking_recv` — su questo la finestra è più forte di quanto temevo, e va DETTO. Ma è cieco per costruzione a: (i) **dealloc — non incrementa nulla** (main.rs:75-77): un thread che solo LIBERA in idle dà drift 0; (ii) malloc FFI (zlib/gd/tidy fuori dal GlobalAlloc); (iii) metadata/arene interne di mimalloc (mmap diretto); (iv) page-dirtying di memoria già allocata. Nota d'autore: mimalloc NON ha un purge thread dedicato — il purge è in-band sulle operazioni heap, con PURGE_DELAY=0 e zero traffico non scatta nulla: il canale churn non lo vedrebbe comunque. MEASURE80 §Idle deve recitare "**0 allocazioni Rust-allocator, tutti i thread**", con i quattro canali ciechi enumerati.

**A-DL12 — la regola ownership §11.1 non è ancora operativa.** "Cache-as-owner conta la entry UNA volta" non dice il MECCANISMO: serve **visited-set per identità di puntatore sull'INTERO walk** (root = cache TL), non per-entry — senza dedup cross-entry, i nodi condivisi tra entry main e entry include si double-contano. Il controllo positivo a modulo sintetico SINGOLO non falsifica il double-count: servono DUE moduli che condividono un sotto-grafo di taglia nota (atteso = S1+S2−Sshared). La mia domanda "owner su thread diversi" si chiude in positivo: Rc è `!Send`, il cross-thread è impossibile per costruzione ⇒ la moltiplicazione ×W è ESATTA — dichiararlo accanto al budget A-MS12. L'esclusione Const-string è una sottostima SISTEMATICA: ogni cifra retained va etichettata "≥, Const esclusi".

**A-DL13 — floor NON propagati a tutto il protocollo.** §11.3 (V2/peak) ✓; churn ✓ (deterministico, spread 0,0%). Mancano: **CPU slope due-N** (§11.4) senza risoluzione dichiarata (spread coppia build-adiacente ±1,5-2%, WP-65 + granularità time) — sotto la minima differenza giudicabile è ADVISORY; **soglia A-DS9** (scan supersede) senza numero ex-ante; **probe twin del supersede** (§7): "floor dichiarato" senza cifra — fissare vmmap 0,1MB e dimensionare il workload di edit a ≥10× (supersede cumulativo atteso ≥1MB).

**A-DL14 (minore)** — la riga `census-global:` non porta `gross=1`: stessi byte lordi, stesso tag.

**Kill-switch:**
- **KL-82-1**: "idle drift 0" citato come assenza di attività di MEMORIA senza l'enumerazione dei canali ciechi (dealloc, FFI, arene mimalloc, dirtying) = claim respinto; residenza idle esige twin union + vmmap + floor (estende KL-81-3).
- **KL-82-2**: cifra retained da walk senza visited-set per puntatore o con controllo positivo a modulo singolo (niente sotto-grafo condiviso) = NULLA; A/B che la usa = VOID.
- **KL-82-3**: verdetto CPU slope o costo scan supersede senza risoluzione/soglia numerica ex-ante = ADVISORY d'ufficio, mai verdetto.
### 8. Stogov — semantica Zend engine — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.** Ho verificato nel codice e nei raw, non nella prosa.

**Onorati (A-DS7..11, KS-DS-81-1/2/3):** A-DS7 ✓ (design79 §3: ini `&[]` con assert-vuoto eseguibile O fold nel fp; autoload-mid-compile nel fp dello slot O falsificato da F10). A-DS8 ✓ (F10/F11/F12 in §9, con i miei kill-switch attaccati; F11 nella forma giusta: due spelling → stesso canonico → byte-parity con l'ORACOLO su HIT, non con se stessa). A-DS9 ✓ (§7+§11.4: scan supersede nel braccio CPU). A-DS10 ✓ e **nel posto giusto**: la dichiarazione d'asimmetria sta nel verbale di misura (MEASURE80, sezione arm cli), il confronto si limita a totale/a1/a3, e il delta +0,4% è correttamente NON promosso a verdetto A-BB1. A-DS11 ✓ (NEXT_SESSION §WP-81.2 e design79 §11.7: M-68.5 same-commit + doc-purge). KS-DS-81-1/2/3 tutti nella tabella attiva.

**Il punto che REFUTO — §5 non distingue ancora le due classi di stale.** Ho letto `bytecode.rs:151-285`: l'epoch è corretto per la SUA classe — ogni `fill` embedda l'epoch, ogni `get` lo confronta, `bump_ic_epoch()` in `Vm::new` (pin ==1 nel gate) invalida in O(1); il commento a riga 152-157 dice perfino la frase giusta ("same numeric id can name a DIFFERENT class"). Ma l'epoch guarda le CELLE, non i PAYLOAD: un `ClassId` cotto in un op immutabile è perfettamente immutabile E perfettamente stantio, e il token-scan di DR-1 è cieco su di lui **per costruzione** (cerca mutabilità interna; qui il pericolo è dato immutabile sbagliato). §5 liquida la classe con "il main ricompila deterministico, quindi gli id coincidono per costruzione" — frase del mondo VECCHIO: con il main cached la garanzia degrada a "stesso Module ⇒ stessi id", che regge SOLO se l'immagine d'id a valle del vergine-fp è deterministica richiesta-per-richiesta. F7 non basta come unico giudice: il suo meccanismo-di-registro sono le IC (che l'epoch già copre), quindi un F7 verde non discrimina la classe baked-id.

**Emendamenti (serie WP-82):**
- **A-DS12** — §5 si riscrive in DUE classi: (1) stato IC = epoch-guarded, CHIUSA; (2) id run-scoped COTTI nei payload degli op del Module cached (ClassId/FuncId/indici const): ENUMERARE ogni campo payload che porta un id run-scoped, con giustificazione per campo ("Module-relative + remap" oppure "deterministico dal vergine-fp"). Enumerazione assente = §5 non chiuso.
- **A-DS13** — **F13**: main cached su HIT con ORDINE di include variato tra richieste (include condizionale A-poi-B vs B-poi-A), `new`/`instanceof`/`static::` su classi di entrambe le lib, body == oracolo su ≥3 richieste; con controllo POSITIVO (una variante che rompe deliberatamente il determinismo deve far mordere la fixture — auto-uguaglianza vietata, KS-AH-80-2).
- **A-DS14** — i due "oppure" di §3 (ini: assert-vuoto vs fold; autoload-mid-compile: fp vs F10) si PINNANO a una scelta sola PRIMA che il put atterri; un put con la scelta non documentata nel commit = design violato.
- **A-DS15** (ADVISORY) — re-entrancy VM (call_method_sync, lezione bcmath): dichiarare se `Vm::new` può eseguire MID-request; il bump è safe-direction (solo invalida), ma il pin "call-site ==1" deve dire quale.

**Kill-switch (serie KS-DS-82-\*):**
- **KS-DS-82-1**: F13 body ≠ oracolo su HIT = leva RESPINTA; e QUALUNQUE citazione di "stessi id per costruzione" senza l'enumerazione A-DS12 = claim NULLO.
- **KS-DS-82-2**: nuova IC o nuovo sito di fill non epoch-guarded, o pin `bump_ic_epoch` ≠1, senza ri-audit DR-1 PRIMA dell'A/B = A/B VOID.
- **KS-DS-82-3**: confronto censuscli↔axum su fixture che tocca `$_GET`/`$_SERVER` prima di A-BB4 = cifra respinta a prescindere dalla dichiarazione (l'asimmetria lì non è più rumore, è il segnale).

M-68.5/A-DS11 è nel programma; il resto del programma WP-81 (ordine 1-6, revert KS-DS-80-3) lo sottoscrivo.
### 9. Gregg — metodologia di misura/attribuzione — CONCORDO CON EMENDAMENTI

Ho campionato i raw, non il report. Ricomputi eseguiti in proprio.

**VERDETTO: CONCORDO CON EMENDAMENTI — un errore di trascrizione ×10 trovato nei raw, il resto regge.**

**Verificato (raw committati, commit 7ddb6bc, git status pulito):**
- **KG-81-1**: ogni cifra della tabella axum ricomputata da `census.80.<fx>.r1.census` righe 11-110 combacia: hello a=80.476/13.015.960, a1=74.288/10.825.612 costante su OGNI fixture e run, resid 44,1/43.463, retain 0/0/2/5, cli 81.579/81.613/81.687/109.415. a3 cold: include_heavy req=1 ha a3=8.717 (controllo positivo MISS), steady=0.
- **Spread 0,0% = confronto REALE, non asserito**: md5 r1/r2/r3 identici per tutti gli 8 (arm×fixture). L'ho rifatto io.
- **Identità**: `.matrix` per-run md5-identici all'archivio `feature-matrix.6910767.20260731-151411.log`; git=6910767, bin[census]=5c9c6eec ✓; driver ENFORCE git=+porcelain (measure78.sh:90-100). A-BG18/KG-81-2 onorati.
- **Idle**: `.idle` = 4 righe (boot-probe prima); tail-3 ricomputato: axum 38 call/41.524B, cli 49/9.639, **drift vero 0/0 a 10s E 60s** ✓. Il bug è documentato onestamente (WP_SESSION_80 §🐛4: "raw corretti, sommario ricomputato"). A-BG19 (110 righe esatte, ENFORCE presente), A-BG20 (idle cli + idle60), A-BG21 (resid+b in §10, KS-AH-80-4 v2 mono-quantità) onorati.
- **Floor ≤200**: DERIVATO, non inventato — b(include_gate)−b(hello)=767−730=37/2 unit=18,5 verificato nei raw; resid 41-44; margine ×2 dichiarato. L'analogia probe-unit→probe-main resta assunzione, ma è ex-ante lecita.

**REFUTAZIONI:**
1. **b_calls include_heavy = 27.982, NON 2.798.** Tutte le 100 righe steady del raw dicono `b_calls=27982` (b_bytes 1.867.510 giusto). L'errore ×10 sta in MEASURE80 §arm-axum E propagato nel contratto design79 §10 ("b invariato ±5%… 2.798"): il check post-leva confronterebbe contro una cifra falsa di un ordine di grandezza. Trascrizione a mano = il buco che KG-81-1 non copre: la cifra È rintracciabile, ma SBAGLIATA.
2. **Finestra steady contiene un transiente**: req=11 ha resid=157 (poi 44, poi 98×43). Il regime parte a req=12; la media 44,1 lo ingloba; il picco 157 sta FUORI dalla banda 44±15 se letta per-riga. La predizione deve dichiararsi sulla MEDIA.
3. **R=3 deterministico = zero informazione di varianza**: le tolleranze ±15/±5KB/±5% sono concessioni dichiarate, non statistica — vero e ammesso implicitamente, ma nessun documento lo dice esplicitamente; la varianza vera si vedrà solo quando la leva romperà il determinismo.
4. **Nessun self-test pinna report_idle**: il fix tail-3 vive solo nel commento; il sommario emesso dal driver non sta in NESSUN .log committato (verificato: assente da census.80.hello.r1.log) — i numeri idle di MEASURE80 sono ricomputabili solo perché i .idle sono committati.

**EMENDAMENTI serie A-BG:**
- **A-BG22 (BLOCCANTE pre-leva)**: correggere 2.798→27.982 in MEASURE80 e design79 §10; ogni cifra trascritta ri-auditata da uno SCRIPT raw→tabella committato, mai a mano. [ESEGUITO in chiusura S-80.0: entrambe le cifre corrette con nota di provenienza]
- **A-BG23**: steady dichiarato 12-110 (o transiente req=11 documentato); predizioni resid sulla media, picco per-riga 157 dichiarato nella baseline. [nota aggiunta in MEASURE80 §predizioni]
- **A-BG24**: self-test di report_idle su fixture sintetica 4-righe con delta noti, eseguito nel preflight del driver; sommario driver archiviato nel .log committato.
- **A-BG25**: etichettare esplicitamente "spread 0,0% = determinismo, non base statistica"; prima campagna post-leva ri-deriva lo spread osservato. [nota aggiunta]

**KILL-SWITCH serie KG-82:**
- **KG-82-1**: cifra in MEASURE/design che diverge dal ricomputo scriptato dei raw = documento VOID finché riconciliato (estende KG-81-1: rintracciabile non basta, deve COMBACIARE).
- **KG-82-2**: verdetto A/B che cita una banda di tolleranza senza lo spread osservato della PROPRIA campagna = NON giudicabile.
- **KG-82-3**: numero derivato (idle/self-cost) senza raw committato da cui ricomputarlo = ADVISORY d'ufficio.
