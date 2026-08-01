# COUNCIL_WP85_REVIEWS.md — Concilio a 9 sedie su S-83.0 (denti di verbale + misure in forma nuova + partizione-per-TIPO) + programma WP-84

**Convocato**: 2026-08-01, chiusura S-83.0 (regola utente 2026-07-26: 9 sedie
dopo OGNI sessione, mandato REFUTARE). Atti: NEXT_SESSION_WORDPRESS.md,
sessions/WP_SESSION_83.md, wp83-harness/MEASURE83_RESULTS.md + verdict83.out,
wp84-harness/COUNCIL_WP84_REVIEWS.md (l'ordine eseguito), codice e raw dei
rispettivi perimetri. Verbali VINCOLANTI per il design WP-84.

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO/CONFERMATO CON EMENDAMENTI — nessuna
opposizione.** Le MISURE sono confermate a ricomputo indipendente (Gregg:
VC 36/36 valore per valore, VCR n=2000 esatti, void 21/108 esatti; Bak: VC
al centesimo + cross-check di linearità; Leijen: TUTTA l'algebra retained
ricomputata esatta dai raw). KS-MS-84-1 verificata a macchina (git log:
050c891 È il primo commit sui file della porta). L'emendamento lock-cmp
PRUNE-ONLY è **ACCETTATO CON RAFFORZAMENTO** dal suo autore (Hejlsberg) e
approvato da Hoare/Bak/Pedersen. I colpi della tornata:

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **Il sigillo VmGate regge sui mint 1-2, NON sul mint 3** (Hoare: il buco
   (d) alias si riapre via `use vm_gate_probe as x` — pub nudo ri-esportato,
   lì il giudice è di nuovo l'awk; Matsakis: ZST owned senza lifetime =
   capability-ESISTENZA senza binding temporale, bancabile oltre l'acquire;
   il claim "rustc è il giudice" si scrive "rustc per i mint 1-2 + belt"):
   [A-TH27/A-TH28, A-MS25/A-MS26, KH85-1, KS-MS-85-1/2].
2. **La macchina di equivalenza ha tre elusioni** (Klabnik: OUT mai legato a
   BREV — il pin ".done a rev" è nel commento, non nel codice; ledger
   parametrico touch-creato = storia anti-transitiva cancellabile; FAIL
   smerciabile come "assente con motivo"; Hejlsberg: la (iv) non copre gli
   oggetti TRASCLUSI dei gate — run-gate.sh/fixtures): [A-SK32/A-SK33,
   A-AH36, KS-SK-85-1/2, KS-AH-85-2]. La catena di S-83.0 resta LEGALE nel
   merito (Q4 Klabnik: due equivalenze su battery-rev distinte, mai
   transitive).
3. **PRUNE-ONLY accettato, da rafforzare su PROVENIENZA e resolver**
   (Hejlsberg: il buco vero è il provenance-strip — [patch]/path-dep
   cancella solo source=/checksum= mantenendo (nome,versione): A-AH35;
   KH84-2 riformulata in A-AH37, VC di campagna-2 valido retroattivamente;
   Hoare: pin resolver==2 nei due manifest + pruned-set atteso NOMINATO +
   diff integrale su file, mai head-6: A-TH30; campagna-2 SANA anche sotto
   la forma rafforzata — il diff archiviato è un edge dev-dep puro).
4. **VC PASS CONFERMATO, con due onestà in più** (Gregg: il conflitto era
   nella LETTERA di KG-84-3 vs il deliberato "e/o" dello stesso verbale —
   riscritto in KG-85-1, VC passa per la via (b); Bak: lo spread lever è
   1 tick del timer — res_eff col tick-floor A-BB36, il "~63×" si quota
   **~55–75×** A-BB37). **REFUTATO «interleaved»** (Gregg: i mtime provano
   blocchi adiacenti stessa sera, non alternanza — retro-correzione di
   MEASURE83 ordinata A-BG34, eseguita in chiusura): [KG-85-1/2, KB-85-1].
5. **La cifra VA (+4 register / ≤+52 include-HIT) è VOID d'ufficio**
   (Stogov Q4: il body di autoload83_reg non è MAI stato osservato —
   measure78 scarta ogni body e il full-body oracle copre solo la fase R;
   un fatal nella fixture produrrebbe cifre plausibili): [A-DS29,
   KS-DS-85-1 — rimisura in WP-84 con oracle per-fixture ledgerato]. Il
   PASS a3==0 resta come vitalità, non come attribuzione.
6. **Il residuo one-time apre la domanda che DECIDE il ×W** (Leijen: 7,35MB
   attribuito "per-processo" ma UNIT_CACHE è thread-local e il marginale
   3,44 < 7,35 — se è per-thread il budget ×W regge, se è per-processo il
   budget diventa residuo + W×2,79MB: la misura A-DL24 hello-only W=2
   per-thread DECIDE; delibera peak RINVIATA d'ufficio senza: KL-85-1) +
   (Bak KB-85-2: nessuna cifra peak ×W prima del rerun A-BB34
   post-partizione). Etichette: frazione condivisibile DOPPIA
   (walk 69,1% / footprint 66,1%, mai "⇒" tra sorgenti diverse: A-DL25);
   bytes-first su ogni cifra (la trappola 20.648.477 B ↔ "20,65": A-DL26).
7. **main_evicted è ora un dente che non può mordere** (Stogov: classe
   A-DS19 — ordina il test in-cargo a INIEZIONE A-DS26; Matsakis
   KS-MS-85-3: "mai evitto per costruzione" si scrive "per disciplina di
   routing + tripwire" finché A-MS27 non atterra; Hoare A-TH31: pin del
   sito produttore di main_program:Some ==1).
8. **sig() di a_ds17 ancora scoperto** (Stogov: props defaults, class
   consts, parent/interfaces confrontano UGUALE oggi — A-DS28 con secondo
   mutante; KS-DS-85-3 lega ogni campo nuovo del Module all'estensione
   same-commit).
9. **REQ_NS senza w= in-band** (Pedersen: a W>1 i campioni si interleavano
   nel Mutex globale e il verdict non può rifiutarli — A-PP28, KS-PP-85-1)
   + il base merita la forma per-request via driver curl -w (Gregg A-BG33)
   + "regime" definito nel verdict (A-BG35).
10. **Onestà di forma** (Klabnik: NAMED-DEVIATION a due denti — misura
    invalida ⇒ FAIL, mai conflata con la falsificazione del modello:
    A-SK34; F14b pinna occorrenze(C1)==1: A-SK35; Matsakis A-MS28: put-path
    raccogli-poi-emetti o dente che uc_log/uc_stat non toccano UNIT_CACHE;
    Pedersen A-PP26: delta entries-TOTALI in a_pp20; A-PP27 fixture
    head-segment PRIMA del prossimo twin-pair; A-PP29 commento UnitSlot →
    H-67.4; Hoare A-TH29: check_class splittato per regione su TUTTA la
    classe; Bak A-BB38/39: modello VW piecewise solo dopo sito nominato +
    bisezione, dente di linearità sui due-N).

### Ordine vincolante di apertura WP-84 (S-84.0, non rinegoziare)

1. **Retro-correzioni immediate** (eseguite in chiusura S-83.0 dove
   ordinato): A-BG34 (× interleaved → blocchi adiacenti) · A-BB37 (~63× →
   ~55–75× quotato) · VA derivate marcate VOID (KS-DS-85-1) in MEASURE83.
2. **Sigillo VmGate v2**: A-MS25/A-TH27 (lifetime-bound, same-commit sui
   mint) · A-MS26/A-TH28 (probe dietro cfg(test/feature), belt anti-alias
   ==0) · A-TH31/KH85-3 (pin sito produttore).
3. **Equivalenza/battery onesta**: A-SK32 (OUT↔BREV legati) · A-SK33
   (ledger canonico tracked) · A-AH36 (manifest per-gate degli oggetti
   trasclusi).
4. **Denti**: A-DS26 (iniezione main_evicted, il contatore morde una volta
   in vita) · A-DS28 (sig esteso + 2° mutante props) · A-SK34/A-SK35 ·
   A-MS28 · A-TH29 · base-arm rafforzato A-AH35+A-TH30.
5. **Misure decisive per la delibera peak (che resta APERTA con DUE
   precondizioni)**: A-DL24 (hello-only W=2, per-thread vs per-processo —
   DECIDE la formula del budget ×W; KL-85-1) · rerun pin A-BB34 232±1MB
   post-partizione (KB-85-2) · rimisura VA con A-DS29 (oracle per-fixture
   ledgerato) · prossima slope con A-PP28 (w= in-band) + A-BG33 (base
   per-request via driver) + A-BG35.
6. **A-PP18** (ereditata WP-84 §1): PRIMA di ogni riconciliazione Δglobal
   a W>1 (A-PP24).
7. Poi: **ripresa ROADMAP da [[php-rust-todo-master]]**. Revert policy
   KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH85-1 | "rustc è il giudice" citato con mint pub non-cfg-test raggiungibile | sigillo declassato ad ADVISORY (belt-only) |
| KH85-2 | arm base con prune non attribuito o resolver non pinnato | arm VOID |
| KH85-3 | put con main_program:Some da sito ≠ main_publish_ticket | partizione de-certificata, campagna VOID |
| KS-MS-85-1 | VmGate conservato oltre lo stack-frame dell'acquire | sigillo ADVISORY finché A-MS25 non atterra |
| KS-MS-85-2 | call-site vm_gate_probe fuori test senza feature dichiarata | build VOID per campagne |
| KS-MS-85-3 | "main mai evitto per COSTRUZIONE" citato pre-A-MS27 | declassato a "per disciplina di routing + tripwire" |
| KS-MS-85-4 | Drop con side-effect su tipi in CachedUnit | STOP, re-audit drop dentro borrow_mut |
| KS-SK-85-1 | equivalenza senza legame OUT↔rev o FAIL smerciato come assente | equivalenza VOID, campagna VOID |
| KS-SK-85-2 | ledger non canonico/untracked/rigenerato | TUTTE le equivalenze della catena VOID |
| KS-SK-85-3 | NAMED-DEVIATION su misura invalida | declassato a FAIL, verdetto campagna FAIL |
| KS-AH-85-1 | diff lock con source=/checksum=/version= cancellata e package superstite | arm base VOID |
| KS-AH-85-2 | equivalenza con helper/fixture di gate cambiati fuori manifest | equivalenza NULLA, battery da rieseguire |
| KS-AH-85-3 | raw cross-campagna senza alloc_id in-band sulla RIGA citata | cifra VOID |
| KB-85-1 | res_eff (tick-floor) ≥ banda/3 in qualunque cella | VC NULL, mai ADVISORY |
| KB-85-2 | cifra peak ×W pubblicata prima del rerun A-BB34 post-partizione | claim VOID, verdetto sessione FAIL |
| KB-85-3 | bisezione VW con a_calls ∉ {2,4} o a_bytes ≠ modello | modello RESPINTO → NAMED-DEVIATION |
| KS-PP-85-1 | slope per-request da __reqns senza w=1 in-band | campioni VOID |
| KS-PP-85-2 | claim "mai retained" post-fatal senza delta entries-TOTALI | insufficiente |
| KS-PP-85-3 | segmento twin-pair senza fixture head-segment preesistente | segmento VOID |
| KL-85-1 | budget ×W citato senza esito A-DL24 | delibera peak RINVIATA d'ufficio |
| KL-85-2 | cifra memoria in MB senza byte esatti nel corpus | documento non pubblicabile |
| KL-85-3 | frazione condivisibile senza sorgente (walk/footprint) | input delibera VOID |
| KS-DS-85-1 | cifra per-fixture senza full-body oracle di QUELLA fixture | cifra VOID |
| KS-DS-85-2 | main_evicted==0 citato come prova senza A-DS26 verde | pin ADVISORY |
| KS-DS-85-3 | campo nuovo del Module senza estensione sig()+mutante same-commit | purezza a_ds17 NULLA |
| KG-85-1 | (riscrive KG-84-3) slope <30s user senza via (a) per-request O via (b) min-of-R R≥9+f+res | NULL automatico |
| KG-85-2 | "interleaved" senza prova di alternanza nei timestamp | scope VOID d'ufficio |

---

## VERBALI INTEGRALI

---

# COUNCIL_WP85 — Sedia 1: Hoare (design linguaggio/runtime, safe-only) — su S-83.0

**VERDETTO: CONCORDO CON EMENDAMENTI.**

**Q1 — VmGate: il sigillo regge sui mint 1-2, NON sul mint 3.** Verificato: `VmGate(())` a campo privato, niente `derive(Copy/Clone)` (vm/mod.rs:494) — nessun clone del token, transmute fuori dal perimetro safe-only (ma noto: nessun `#![forbid(unsafe_code)]` in testa a lib.rs — il "safe-only" del sigillo è policy, non lint). Le vie a-d del mio verbale WP-83 sono chiuse per il **costruttore**; MA `vm_gate_probe` è `pub`, ri-esportato in lib.rs:64: qualunque codice non-test del workspace (o downstream) minta un token e per QUELLA via il giudice non è rustc — è l'awk, cioè un pin di spelling. `use php_runtime::vm_gate_probe as x; x()` elude il regex `vm_gate_probe[(]` (la riga `use` non ha parentesi, la call è `x()`): **il buco (d) — alias — si riapre esattamente attraverso il mint 3**. Il doc-claim "Closes Q3 holes a–d" è sovra-dichiarato. Secondo buco: `MainUnit::vm_gate()` ritorna una ZST `'static` NON brandizzata — mintabile durante un acquire, accumulabile in un `Vec<VmGate>`, usabile DOPO il drop del MainUnit: "enterable only from inside the sealed acquire lifecycle" è prosa, non tipo.

**Q2 — split prod==2/selftest==7: ancora sana, split POSIZIONALE.** L'anchor ==1 è fail-loud (bene). Ma la regione decide l'attribuzione per POSIZIONE: un lower produttivo aggiunto lessicalmente dopo `retained_walk_selftest` conta selftest. Peggio: `check_class "$VMMOD" 14` è un conteggio UNICO prod+selftest sull'intera CLASSRE — la conflazione KH84-1 sopravvive al livello di classe (un compile selftest tolto + un seeded produttivo aggiunto = 14 invariato, e lo split 2/7 non li vede perché conta solo `crate::lower_source(`).

**Q3 — lock-cmp PRUNE-ONLY: forma onesta, con due condizioni.** Il dente byte-esatto era strutturalmente insoddisfacibile (classe WP-72, correttamente riconosciuta); prune-only + subset (nome,versione) cattura re-resolve e drift (added>0 ⇒ FAIL; un cambio di versione produce righe `<`+`>` ⇒ FAIL). L'innocuità del prune per il binario release però POGGIA su `resolver = "2"` (verificato, Cargo.toml:2): sotto resolver v1 gli edge dev-dep unificano le feature anche nel build normale — e lo script non pinna il resolver. Inoltre lo script accetta QUALSIASI cancellazione: il prune di un edge di dipendenza RELEASE (divergenza di feature-unification tra manifest) passerebbe in silenzio ed è esattamente un vettore di bias CPU. E l'header archivia `head -6` del diff — evidenza troncata. **Emendato, l'amendment è APPROVATO.**

**Q4 — UnitSlot: bound ways+1 regge in tutti i path letti.** Main 0/1; includes: replace-in-place / remove(0)+push a cap / push sotto cap; supersede rimuove chiavi intere. Il REPLACE su fp diverso è corretto: la cache lascia solo il suo Rc; il get ritorna cloni owned e park_main pinna il modulo per la richiesta — nessuna entry in uso liberabile (Rc + RetainSet). Residuo NOMINATO: la corsia è decisa dal TAG del valore (`main_program.is_some()`), non dall'identità del chiamante — oggi l'unico produttore di `Some` è `main_publish_ticket`, ma niente lo pinna.

**Emendamenti:**
- **A-TH27** — VmGate brandizzato a lifetime (`VmGate<'a>` legato al MainUnit/RetainSet) + mint 3 dietro `#[cfg(any(test, feature))]`, mai pub nudo ri-esportato; finché non atterra, "rustc giudice" si scrive "per i mint 1-2".
- **A-TH28** — belt anti-alias: `use .*vm_gate_probe` ==0 non-test nel workspace (muore col cfg di A-TH27).
- **A-TH29** — `check_class` di vm/mod.rs SPLITTATO per regione sull'intera CLASSRE (chiude KH84-1 a livello di classe).
- **A-TH30** — base-arm: pin `resolver=="2"` in ENTRAMBI i manifest + pruned-set atteso NOMINATO per campagna + diff integrale su file.
- **A-TH31** — pin: costruzione `main_program: Some` ==1 sito (main_publish_ticket).

**Kill-switch:**
- **KH85-1** — "rustc è il giudice" citato mentre esiste un mint pub non-cfg-test raggiungibile da codice non-test ⇒ sigillo declassato ad ADVISORY (belt-only).
- **KH85-2** — arm base con prune non attribuito al delta di manifest o resolver non pinnato ⇒ arm VOID.
- **KH85-3** — put con `main_program: Some` da sito diverso da main_publish_ticket ⇒ partizione de-certificata, campagna VOID.

*Tony Hoare*

---

**VERBALE — Matsakis, sedia 2, Concilio WP-85 su S-83.0. Perimetro: ownership/borrow/sigilli. VERDETTO: CONCORDO CON EMENDAMENTI — nessun difetto di soundness; un claim sovrastimato (rustc-giudice), un dividendo ordinato e non incassato.**

**Q1 — KS-MS-84-1 ONORATA.** `git log` filtrato su vm/mod.rs+worker_pool.rs nel range 050c891..857da59: il PRIMO commit è **050c891**, che È A-MS17 (VmGate ZST + A-MS22 same-commit, nominato nel messaggio). Nessun commit del range precede il sigillo sui due file. A-MS22: `lower_net` vive in `MainPublishTicket` (mod.rs:15946-15954), letto in `main_publish_ticket` (:16155); il travaso in `CachedUnit.main_program_net` è trasferimento di proprietà al publish (A-DL20), non aliasing — legittimo.

**Q2 — buco REALE, classe nominata: capability-ESISTENZA senza binding temporale.** `vm_gate(&self) -> VmGate` (:15978) ritorna ZST **owned, senza lifetime**: mintabile N volte da un solo acquire, bancabile (ZST di `()` è Send+Sync auto) in static/struct e riusabile su richieste successive — il token prova "un main FU acquisito", non "QUESTO request tiene un main acquisito" (time-of-mint ≠ time-of-use). Aggravante: `vm_gate_probe` (:500) è `pub #[doc(hidden)]` — un crate esterno minta in produzione e il giudice lì è solo l'awk-belt, non rustc. Oggi accettabile (5 call-site auditati, belt vivo), ma il claim "rustc è il giudice" va declassato a "rustc + belt" finché A-MS25/26 non atterrano.

**Q3 — partizione-per-CORSIA, non per-TIPO.** KS-MS-84-3 **non violata**: il loop di evizione (:16263-16281) è lane-puro, il main non entra mai nella selezione vittima — non è uno skip-per-tag. MA entrambe le corsie portano `CachedUnit` col tag runtime `main_program: Option<…>`; il routing al put è `if cu.main_program.is_some()` (:16247) e rustc **non vieta** un main in corsia include — il tripwire :16270 esiste proprio perché il tipo non lo esclude. Il dividendo ordinato (reject strutturalmente morto) non è incassato: `unit_cache_get` (:15918) itera main+includes unificato e il reject in acquire (:16042) resta load-bearing. Replace su fp diverso: ownership PULITA — `slot.main = Some(cu)` droppa il vecchio, ma park_main (design79 §4) + clone-on-stack in `MainUnit` garantiscono che la cache non sia mai unico owner mid-request; cache thread-local ⇒ chi rimpiazza possiede già i nuovi Rc. Residuo: il drop del vecchio avviene DENTRO `borrow_mut`.

**Q4 — dump PULITO, KS-MS-84-2 non innescata lì.** Il `with` chiude a :1504; `census_line` delle righe per-entry sta a :1506-1508, FUORI; UC_STATS letto PRIMA (:1440); dentro il borrow solo costruzione (format!/Rc::clone/walk). MA `unit_cache_put` tiene `uc_stat`/`uc_log` (I/O) e i drop di supersede DENTRO `borrow_mut` di UNIT_CACHE (:16206-16285): RefCell distinti, nessun double-borrow oggi — salvezza POSIZIONALE (classe A-MS21). Ordino re-audit leggero del put-path, non STOP: la lettera della KS mirava al path di lettura, e lì è onorata.

**Emendamenti:**
- **A-MS25**: VmGate lifetime-bound — `VmGate<'a>(PhantomData<&'a ()>)`, `vm_gate(&self) -> VmGate<'_>`, porta su `&VmGate<'_>`: il banking muore a compile-time; mint in-module e probe adattati same-commit.
- **A-MS26**: `vm_gate_probe` dietro `#[cfg(any(test, feature = "lifecycle-probe"))]`, feature mai nei build di parity/campagna.
- **A-MS27**: partizione-per-TIPO vera (`CachedMain`/`CachedInclude`, tag Option morto, get per-corsia); se il churn è giudicato alto, backlog NOMINATO — mai chiusura silente.
- **A-MS28**: put-path in forma raccogli-poi-emetti (uc_log/uc_stat fuori dal borrow_mut) O pin+dente che uc_log/uc_stat non toccano UNIT_CACHE.

**Kill-switch:**
- **KS-MS-85-1**: VmGate conservato oltre lo stack-frame dell'acquire (static/struct/thread_local) ⇒ sigillo ADVISORY finché A-MS25 non atterra.
- **KS-MS-85-2**: call-site di `vm_gate_probe` fuori test senza feature dichiarata ⇒ build VOID per campagne.
- **KS-MS-85-3**: "main mai evitto per COSTRUZIONE" citato in verdetto pre-A-MS27 ⇒ declassato a "per disciplina di routing + tripwire".
- **KS-MS-85-4**: Drop con side-effect su tipi contenuti in CachedUnit ⇒ STOP, re-audit dei drop dentro borrow_mut.

*Niko Matsakis*

---

**VERBALE KLABNIK — sedia 3, Concilio WP-85 — CON EMENDAMENTI**

**Q1 — battery-equivalence.sh: TRE elusioni concrete.** (a) *OUT non legato a BREV*: battery-83pre.sh stampa `git=$GIT_REV` nel summary e `rev=$GIT_REV` nel `.done` (righe 16/45) — ma il checker non li legge MAI. Qualunque battery.out (vecchio o contraffatto con 15 righe `OK <name>`) più un BREV a delta-crates vuoto passa: il pin ".done a rev" di A-SK30 è ordinato nel commento e assente nel codice. (b) *Ledger arbitrario*: `LEDGER` è parametro del chiamante e `touch`-creato (r.90-91): un path fresco svuota la (iii); un ledger rigenerato cancella la storia anti-transitiva. La catena è onesta solo finché il ledger è quello canonico committato — nulla lo impone. (c) *FAIL ≠ assente*: `.done` viene scritto anche a battery FAIL; il checker cerca solo `^OK name` — un gate FALLITO fuori da NEVER_ABSENT è smerciabile come "assente con motivo". L'anchoring del grep (`^OK[[:space:]]+name( |$)`) e la (iv) whole-tree sono invece corretti.

**Q2 — F14b: NON tautologia, con un residuo.** W14N deriva da W14, ma W14 è corpo dell'ORACOLO (r.275), non del comparando phpr: B5/B6 sono confrontati con una stringa oracle-derivata con delta NOMINATO e guardia `W14N != W14`. Un fatal non passa: A-SK31 soddisfatta in sostanza. Residuo: `${W14/C1/C2}` sostituisce la PRIMA occorrenza — se la fixture echeggiasse C1 due volte, il pin diverrebbe silenziosamente sbagliato. Sul secondo punto: verificato — nessun verdetto VH e nessun claim twin-pair in verdict83/MEASURE83; KS-PP-84-1 non violato (nulla claimato senza A-SK26).

**Q3 — VW: esito onesto, FORMA bucata.** NAMED-DEVIATION senza fail è legittimo per KB-84-2 (falsificazione nominata, aritmetica 2len+2(len+1) ricomputabile). MA il ramo (verdict83.sh r.189-192) manda in NAMED-DEVIATION anche `steady_n != 30`: una finestra rotta verrebbe etichettata "modello falsificato" invece che FAIL — deviazione e strumento-guasto conflati. Qui sn=30, esito valido; la forma va corretta. Inoltre KB-84-2 esige il ritiro del modello VECCHIO in ogni citazione normativa: se un pin/gate codifica ancora `2×len`, va bumpato same-commit del modello nuovo.

**Q4 — catena LEGALE.** Ledger: `7b90614→ac35d24`, poi battery VERA 15/15 a 925da3b, poi `925da3b→e5f2a5e`. Battery-rev distinti, 925da3b non è `head=` di equivalenza precedente: niente transitività, una equivalenza per battery-run. La riga ac35d24 (campagna-1 VOID) resta nel ledger e blocca il riuso di 7b90614 — corretto. Caveat: la legalità sostanziale ("la battery RAN a B") oggi la garantisce la narrazione, non la macchina (buco Q1a).

**Emendamenti:**
- **A-SK32**: battery-equivalence LEGA OUT a BREV: match obbligatorio di `git=$BREV` nel summary E `rev=$BREV` nel `.done`; refuse se in OUT compare `^FAIL <name>` per qualunque gate del set.
- **A-SK33**: ledger CANONICO: path pinnato nello script (non parametro), tracked in git; refuse se untracked o se HEAD ne contiene una versione divergente; append committato nella campagna.
- **A-SK34**: NAMED-DEVIATION a due denti: misura invalida (steady_n≠30, parse anomalo) ⇒ FAIL; la deviazione è raggiungibile SOLO a misura valida. Ogni citazione normativa del modello 2×len marcata RITIRATA; gate che lo codificano bumpati same-commit del pin nuovo.
- **A-SK35**: F14b pinna anche `occorrenze(C1 in W14)==1`; la semantica prima-occorrenza dichiarata nel commento.

**Kill-switch:**
- **KS-SK-85-1**: equivalenza accettata senza legame OUT↔rev (o con gate FAIL smerciato come assente) ⇒ equivalenza VOID, campagna VOID.
- **KS-SK-85-2**: ledger non canonico, untracked o rigenerato ⇒ TUTTE le equivalenze della catena VOID, battery da rieseguire.
- **KS-SK-85-3**: NAMED-DEVIATION emesso su misura invalida ⇒ declassato a FAIL, verdetto di campagna FAIL.

*— Klabnik, sedia 3.*

---

# VERBALE — Hejlsberg, sedia 4, Concilio WP-85

**VERDETTO: CONCORDO CON EMENDAMENTI. L'emendamento PRUNE-ONLY ad A-AH32 è ACCETTATO CON RAFFORZAMENTO.**

**Q1 — PRUNE-ONLY.** La diagnosi della sessione è corretta nel meccanismo cargo: `--offline` con lock copiato non ri-risolve; alla build cargo **normalizza** il lock al manifest, potando edge/pacchetti irraggiungibili dal manifest 7593d8e. Un re-resolve versionale produce SEMPRE almeno una riga `>` (la versione nuova va scritta): `added_lines==0` lo cattura. E il byte-cmp letterale era un dente che non può mai passare tra manifest divergenti per dev-dep — classe WP-72, la lezione 4 di S-83.0 è esatta. Sul buco temuto (feature-unification): il Cargo.lock **non registra feature** — l'unificazione deriva dai manifest a build-time, e col resolver ≥2 i dev-dep non unificano su `cargo build`; righe cancellate non possono cambiarla, il manifest divergente È l'oggetto A/B. **MA il buco reale esiste ed è un altro: provenance-strip.** Il passaggio di un dep a `[patch]`/path-dep cancella SOLO le righe `source =`/`checksum =` mantenendo (nome, versione): subset-versioni PASSA, added==0 PASSA, codice DIVERSO. Il diff archiviato di campagna-2 (`1909d1908`, `<  "mimalloc",` — una edge in lista dependencies) è conforme anche alla forma rafforzata: campagna-2 SANA.

**Q2 — ALLOC_ID.** Semantica sufficiente: memcensus.rs:1010-1016 definisce bump = same-commit, nominato, a ogni cambio della superficie di conteggio (tre superfici nominate) + storia v1/v2. Pin sorgente in census-twin (§1b, fail-closed). E sì, l'in-band nei raw comparati **c'è**: verdict83.sh:125 falsifica la riga VR (`alloc_id != memcount-v2-s82 ⇒ FAIL`) e :135 ogni per-entry. Il literal duplicato in due script è accettabile: un bump dimenticato in uno dei due FALLISCE, mai passa.

**Q3 — battery-equivalence.** I vincoli (i)-(iv) sono implementati fedeli al mio verbale; il `-ge 14` è morto; il ledger anti-transitivo (:92-97) è corretto. Il doppio confronto path (`php-rust/$script` e nudo) REGGE: `git -C <subdir> diff --name-only` emette path relativi alla radice GIT (php-rust-experiment) ⇒ il ramo prefissato è quello operativo, il nudo è fallback inerte. **Buco nominato**: la (iv) copre solo lo SCRIPT del gate, non gli oggetti trasclusi — census-twin invoca `gate-axum/run-gate.sh` e le sue fixtures; un cambio lì elude la (iv) e la (i).

**Q4 — Header A-TH26: PRESENTE.** `m83.base.83c.n1000.r1.log` righe 22-28: rustc identico sui due arm (1.96.0 ac68faa20), sha di ENTRAMBI i lock, `lock_cmp=PRUNE-ONLY pruned=1 added=0 drift=0`, diff grezzo archiviato, driver_sha+head. **Però i lock sha differiscono per costruzione ⇒ la LETTERA del mio KH84-2 («lock non byte-identici ⇒ NULLO») annullerebbe VC.** Lo riformulo io: l'intento era identità delle versioni risolte, che il prune-only certificato preserva.

**Emendamenti:**
- **A-AH35** (rafforzamento prune-only): le righe cancellate sono legali SOLO come (a) edge `"pkg",` dentro liste `dependencies`, o (b) blocchi `[[package]]` INTERI il cui nome è assente dal lock post-build; ogni `version =`/`source =`/`checksum =` cancellata con package superstite ⇒ arm VOID. Resolver (≥2) dichiarato nell'header.
- **A-AH36**: oggetto di gate per la (iv) = script + helper/fixture NOMINATI in manifest per-gate; census-twin dichiara run-gate.sh + fixtures/.
- **A-AH37**: KH84-2 riformulata: «lock byte-identici O prune-only certificato (A-AH32+A-AH35), rustc identico»; VC di campagna-2 valido retroattivamente sul diff archiviato.

**Kill-switch:**
- **KS-AH-85-1**: diff lock con `source=`/`checksum=`/`version=` cancellata e package superstite ⇒ arm base VOID, delta A/B nullo.
- **KS-AH-85-2**: equivalenza concessa con helper/fixture di gate cambiati fuori dal manifest per-gate ⇒ equivalenza NULLA, battery integrale da rieseguire.
- **KS-AH-85-3**: raw citato in confronto cross-campagna la cui riga citata non porta alloc_id in-band ⇒ cifra VOID (estende KS-AH-84-2 dal build alla RIGA).

*Anders Hejlsberg — sedia 4. Il dente ha morso, la forma emendata distingue il prune legittimo dal re-resolve; ora distingua anche la provenienza.*

---

# VERBALE — Bak, sedia 5, Concilio WP-85 (vincolante per WP-84)

Ho ricomputato in proprio dai raw (`wp78-harness/measure-out/`): tutte le 36 righe user-CPU di VC, i census 83a/83w, verdict83.out.

**Q1 — CONFERMATO al centesimo.** I miei estratti: lever N=1000 min=0,13 (9 run: 0,13–0,15), N=2000 min=0,24; base N=1000 min=6,91 (range 6,91–7,10), N=2000 min=13,83. Slope_lever=(0,24−0,13)/1000=**110 µs/req**, slope_base=(13,83−6,91)/1000=**6920 µs/req**, bound 7291, res max 20 < banda/3=123,7. Cifre del verdetto ESATTE. Cross-check di linearità che il verdetto NON fa: base 6,91s/1000=6910 µs/req ≈ slope 6920 ⇒ intercetta ~0, il costo base è puramente per-request — la retta due-punti è onesta.

**Q2 — regge, con UNA falla nominata: il tick.** Il rumore osservato è one-sided e sotto l'f=5% ex-ante (base full-range 2,7%): il min è robusto e (3°min−min) è lo spacing giusto per stimare il bias del minimo con R=9. MA lo spread lever (10 e 5 µs/req) è ESATTAMENTE 1 tick di `/usr/bin/time` (10ms): lì la risoluzione è timer-limited, non noise-limited — la statistica sta misurando il quantizzatore, non il rumore. Conseguenza: il PASS vs bound è intoccabile (110 ≪ 7291), ma la cifra 110 standalone porta ±15–20 µs/req e il "~63×" va quotato **~55–75×**. La coppia due-N basta per QUESTO claim; la regressione multi-N R≥9 resta la via se mai servisse la slope lever come cifra propria.

**Q3 — modello nuovo, con controllo GIÀ nei raw.** La fixture bare corta ESISTE nella stessa campagna: `census.83a.hello.census`, len=98, a_calls=2,0, a_bytes=196=2×98 — il modello-floor vive sotto soglia. Modello proposto (piecewise):
`a_calls = 2 + 2·[len≥THR]`, `a_bytes = 2·len + 2·(len+1)·[len≥THR]`
(1662 = 2×415 + 2×416 ✓; le due copie extra sono NUL-terminate ⇒ candidato: CString/realpath nel canonicalize, spill del buffer inline a THR≈384). Pin WP-84: (a) NOMINARE il sito nel codice, (b) bisezione a len=THR−1/THR, (c) pin piecewise con a_calls esatto per regime.

**Q4 — rinvio accettato, ma BLOCCANTE per i claim.** Il lavoro di partizione può procedere; nessuna cifra peak ×W è pubblicabile prima della riesecuzione del pin identità 232±1MB post-partizione. Non è calendario: è precondizione di validità.

**Nota lock-cmp PRUNE-ONLY**: dalla mia sedia, accettabile SOLO perché il diff grezzo è archiviato in-header e il set (nome,versione) è identico; il dente ha morso, non è stato limato.

## Emendamenti

- **A-BB36 (tick floor)**: la regola di risoluzione diventa `res_eff = max(spread3low, 2·tick/N·1e6)` con tick dichiarato dal timer (10ms); confronto con banda/3 su res_eff.
- **A-BB37 (errore propagato)**: slope citata standalone e rapporti (63×) portano l'errore propagato ±2·res_eff per braccio; il rapporto si pubblica come intervallo (~55–75×).
- **A-BB38 (pin VW)**: modello piecewise sopra, pinnato SOLO dopo sito nominato + bisezione; controllo = hello len=98 stessa campagna.
- **A-BB39 (dente di linearità)**: per ogni claim due-N, verificare min(N)/N ≈ slope entro res_eff (qui: base ✓, lever entro 20).

## Kill-switch

- **KB-85-1**: res_eff (con tick floor) ≥ banda/3 in QUALUNQUE cella ⇒ VC NULL, nessun claim CPU (eredità KB-83-3, mai ADVISORY).
- **KB-85-2**: qualunque cifra peak ×W pubblicata prima del rerun del pin A-BB34 post-partizione ⇒ claim VOID e verdetto di sessione FAIL.
- **KB-85-3**: se la bisezione VW dà a_calls ∉ {2,4} o a_bytes ≠ modello a uno dei due lati di THR ⇒ modello RESPINTO, si torna a NAMED-DEVIATION.

**VERDETTO: CONFERMATO CON EMENDAMENTI** — VC/VW ricomputati identici ai raw; 4 emendamenti, 3 kill-switch.

*Lars Bak*

---

**VERBALE — Concilio WP-85, sedia 6 (Pedersen, confine per-richiesta/lifecycle). Mandato: REFUTARE.**

**VERDETTO: CON EMENDAMENTI (A-PP22/23/25 CONFERMATE con residui nominati; Q3 REFUTATA come disciplina-sola).**

**Q1 — A-PP22: CONFERMATA, con due cecità residue nominate.** L'enumeratore è key-blind come ordinato (`vm/mod.rs:16145`: iterazione diretta su `UNIT_CACHE`, `values().flat_map(ways)`, filtro `main_program.is_some()` — nessuna canonicalizzazione: un'entry main sotto chiave con `reg_mode` divergente È contata, il caso che avevo nominato è coperto). Il positivo esercita il path giusto (`worker_pool.rs:1057-1076`: +1 esatto su main pulito, stesso thread, KG-79.A onorata). MA: (a) il filtro `main_program.is_some()` è cieco a un inserter divergente che pubblicasse il main fatale in **forma-include** (`main_program=None`) — il modulo resterebbe retained, invisibile all'enumeratore; (b) l'enumeratore è **thread-affine**: nel test la precondizione vale, ma non è scritta nel doc del probe. → A-PP26.

**Q2 — A-PP23: ACCETTO il non-innesco.** I pin lessicali sono la metà macchina ordinata (`gate-lever-pins.sh` §7: region-anchored `==1` in `request_shutdown`, send-after-execute per numero di riga, decoy che MORDE — righe 494-533; verificato che `request_shutdown` a `vm/mod.rs:3594` è la regione ancorata). KS-PP-84-1 vige su segmenti NUOVI e in campagna-2 non ce ne sono stati: non-innesco legittimo. Ma la lezione WP-81 (harness = identità del protocollo; script mid-campaign ⇒ campagna rifatta) impone che la fixture esista PRIMA. → A-PP27, KS-PP-85-3.

**Q3 — REQ_NS: REFUTO la cintura di solo commento.** `worker_pool.rs:94` dichiara "the slope campaign runs W=1" — disciplina, non macchina. La riga drain (`main.rs:220`) porta `arm=axum-worker` e `unit=ns` ma **nessuna etichetta `w=`**: a W>1 i campioni di worker diversi si interleavano nel Mutex globale e il verdetto slope non ha modo meccanico di rifiutarli. Ordino il dente in-band: `w=<W>` nella riga `reqns:` + rifiuto fail-closed nel verdict (non refuse-at-arming: il probe può servire altri usi a W>1, ma la SLOPE no). → A-PP28, KS-PP-85-1.

**Q4 — A-PP25: CONFERMATA.** La cifra 19,69MB porta `arm=cli-server` in-band (`MEASURE83_RESULTS.md:52`); la trasposizione axum passa dal bound VP-derivato (ratio 2,95, condivisibile ≈69% — KS-PP-84-3 onorata, mai cifra nuda). Lifecycle del replace: catena VERIFICATA — K-M67.2 (la cache clona sul get, i caller possiedono l'Rc), H-67.4 (`vm/mod.rs:5897-5906`: il parked clone sopravvive a fp-replace/ways-eviction mid-request), `park_main` sul HIT (riga 838, porta `VmGate`), RetainSet droppato DOPO la Vm (P-67.5). Il replace del main è same-thread e serializzato per costruzione: nessun drop-in-uso. Residuo minore: il commento di `UnitSlot` (15403-15410) non rinvia a H-67.4 — il prossimo lettore non vede la catena. → A-PP29.

**EMENDAMENTI:**
- **A-PP26**: estendere il pin a_pp20 a **delta==0 su ENTRIES TOTALI** (main+include) attraverso le richieste fatali — chiude la forma-include; documentare la precondizione same-thread di `uc_main_entry_count`.
- **A-PP27**: fixture head-segment positivo **committata nell'harness PRIMA** di ammettere il prossimo segmento twin-pair.
- **A-PP28**: riga `__reqns` porta `w=<W>` in-band; il verdict slope rifiuta `w≠1` fail-closed.
- **A-PP29**: commento `UnitSlot` rinvia esplicitamente a H-67.4/K-M67.2 (replace-in-uso coperto dal park).

**KILL-SWITCH:**
- **KS-PP-85-1**: slope per-request citata da `__reqns` senza `w=1` in-band ⇒ campioni VOID.
- **KS-PP-85-2**: claim "mai retained" post-fatal senza delta entries-TOTALI (post-A-PP26) ⇒ insufficiente.
- **KS-PP-85-3**: segmento twin-pair aperto senza fixture head-segment preesistente ⇒ segmento VOID (rafforza KS-PP-84-1, che resta armata).

Su lock-cmp PRUNE-ONLY (delega WP-85): perimetro altrui per l'arbitrato pieno; dal lato lifecycle nessuna obiezione — la forma che distingue prune legittimo da re-resolve è quella onesta.

— Pedersen, sedia 6.

---

# VERBALE WP-85 — Sedia 7, Daan Leijen (mimalloc/footprint/contabilità)

**VERDETTO: CONCORDO CON EMENDAMENTI.** I miei ordini A-DL20/21/22/23 sono eseguiti nella LETTERA e la scomposizione è onesta nell'algebra. Due derivate hanno etichette che REFUTO.

**Q1 — Ricomputo ESATTO dai raw (m83.r1.fourfix, ultimo gruppo, righe 4570-4574).** Σnet = 7.349.977+8.208+45.392+4.233 = **7.407.810 == rw_main_net** ✓. Σfloor_inc = 997.878+1.272+9.634+576 = **1.009.360 == program_floor_share** ✓. rw_budget = 14.250.027−1.009.360+7.407.810 = **20.648.477** ✓, /1.048.576 = **19,69 MiB** ✓. Residuo ord1−median = 7.341.769 B = 7,00 MiB = 35,55% ✓. r2 hello-only: 4.283.308−997.878+7.349.977 = 10.635.407 = 10,14 MiB ✓, ratio 10,14/3,44 = 2,95 ✓. Identico su tutti i gruppi dump, alloc_id=memcount-v2-s82 in-band ✓. L'attribuzione floor "il primo put paga il prelude" è la forma che ordinai: deterministica in put-order, `floor_rule` in-band (vm/mod.rs:1479-1498). Residuo minore: floor_inc per-entry è ammissibile SOLO sommato (pfs) — mai citarlo come proprietà della entry.

**Q2 — Il flat è DISSOLTO, ma la lettura nuova apre la domanda che decide il ×W.** Il residuo 7,35MB è attribuito "per-PROCESS" sulla sola base di net_window=process-counters. MA `UNIT_CACHE` è thread-local e il prelude è Rc (!Send): se ogni worker THREAD ripaga il primo lower, il budget ×W lo conta W volte GIUSTAMENTE — però allora il marginale fisico dovrebbe essere ~10MB/worker, non 3,44. Il 3,44 < 7,35 suggerisce massa una-volta-per-processo (interner/statics globali) che il net window cattura ma i thread successivi non ripagano. **ORDINO la misura che decide (A-DL24)**: hello-only W=2, una richiesta per worker distinto, righe unitcache_main_entry PER THREAD: net(ord1) del secondo thread ≈7,35MB ⇒ per-thread, budget ×W regge; ≈8KB ⇒ per-processo, budget diventa `residuo + W×(budget−residuo)` ≈ 7,35+W×2,79MB.

**Q3 — REFUTO l'etichetta, non la cifra.** Il 69,1% è net(ord1)/rw_budget: walk-bytes su walk-bytes, commensurabile e onesto COME STIMA WALK. Ma la frase del documento («ratio 2,95; ⇒ frazione ~69%») lo incatena al marginale vmmap: il lettore crede che segua dal ratio. La frazione footprint-derivata è 1−1/2,95 = **66,1%** — tre punti sotto, e il gap È l'informazione (overhead heap mimalloc + page-rounding nel marginale fisico). Conflazione di PRESENTAZIONE, non di calcolo: emettere ENTRAMBE con sorgente nominata (A-DL25).

**Q4 — Distinti nel testo, TRAPPOLA nel numero.** Il documento tiene UPPER e scorporato separati (KL-84-1 soddisfatta, scorporo 1.009.360 B ≈ 0,96MB ✓). Ma **20.648.477 B letto in MB decimali = "20,65"** — collide ESATTAMENTE con l'UPPER 20,65 MiB (21.657.837 B). Chi cita rw_budget senza unità produce la citazione incrociata che KL-84-1 voleva impedire. Bytes-first obbligatorio (A-DL26).

**Emendamenti:**
- **A-DL24**: misura hello-only W=2 con righe per-entry PER THREAD prima di ogni delibera ×W; l'esito riscrive la formula del budget nel verso misurato.
- **A-DL25**: frazione condivisibile emessa DOPPIA (walk-based 69,1% + footprint-based 66,1%), sorgente in-band; mai "⇒" tra metriche di sorgente diversa.
- **A-DL26**: ogni citazione = bytes PRIMA, MiB dopo («20.648.477 B = 19,69 MiB»); gate-measure-cifre estende il corpus alle DUE forme.
- **A-DL27**: gli addendi di rw_bytes−bytes_counted (= 3.057.613 B, ancora opachi) emessi in-band — resta il pezzo non scomposto del mio Q2 WP-84.

**Kill-switch:**
- **KL-85-1**: budget ×W citato senza esito A-DL24 (per-processo vs per-thread) ⇒ delibera peak RINVIATA d'ufficio.
- **KL-85-2**: cifra di memoria in MB senza byte esatti a fianco nel corpus gate ⇒ documento non pubblicabile.
- **KL-85-3**: frazione condivisibile citata senza sorgente (walk vs footprint) ⇒ input della delibera VOID.

*Daan Leijen — l'algebra è esatta; adesso che i numeri sono onesti, la domanda giusta è DI CHI è il residuo.*

---

## Verbale Stogov — sedia 8, Concilio WP-85 (perimetro Zend/opcache, cache di unit)

**VERDETTO: CONCORDO CON EMENDAMENTI.** La partizione-per-TIPO è la forma che ho deliberato, implementata nella lettera. Tre buchi di macchina, uno grave (Q4).

**Q1 — Fedeltà opcache: SÌ.** `UnitSlot{main: Option, includes: Vec}` (vm/mod.rs:15412) è "una persistent_script per path": il replace su fp diverso della main lane (:16258, `slot.main = Some(cu)` incondizionato) è la semantica revalidate corretta — il vecchio script non può più servire, il content-fp è il giudice; classificarlo supersede e MAI evizione è esatto. Il supersede-per-path (:16217-16245) droppa lo slot INTERO via `cache.remove(&sk)` — main E includes: giusto, le entry include sono compilazioni dello STESSO file sotto chain-fp diversi; il file editato le rende tutte irraggiungibili per costruzione. Nit per Leijen: il replace same-key/fp-diverso del main droppa i byte del vecchio Program SENZA ledger (il ramo supersede li ledgera, questo no).

**Q2 — F15b da sola NON discrimina, la COPPIA sì: accettabile, da dichiarare.** Una LRU a corsia mista passerebbe F15b (il main toccato è sempre recente) ma FALLIREBBE F15 (main mai ritoccato, 5 inserimenti lo evincono); una FIFO mista fallisce entrambe. Solo l'esenzione-per-corsia passa la coppia: il discriminante vive nell'UNIONE F15∧F15b, non nella singola fixture. L'esenzione del main ERA il punto; FIFO-vs-LRU sulla corsia include è bound, non semantica (opcache non ha ways) — fuori scope, dichiarato. Nessun pin extra: in F15b ogni include-fp è toccato una volta sola, FIFO≡LRU per costruzione lì.

**Q2b — main_evicted: il tripwire è ora un dente che non può mordere.** Post-partizione un main nella corsia include è strutturalmente impossibile (:16266-16275): lo zero del pin è vero per costruzione OGGI, ma se il wiring del contatore marcisce nessuno se ne accorge — classe A-DS19 identica ("a tripwire that cannot trip", stesso file :18857). NON basta. Ordino il test in-cargo.

**Q3 — A-DS22: il comparatore rafforzato NON copre tutto il Module.** sig() (:18900-18916) copre ops+consts di main/functions/metodi. Le closures passano se compilate in `m.functions` (da verificare per NOME nel test). SCOPERTI: property defaults (`public $p = "A"` vive nei metadati di classe, non negli ops di un metodo — una mutazione lì confronta UGUALE oggi), class constants, parent/interfaces, static_vars se campo separato di Func. Il mutante di :18984 muta un corpo di metodo: cieco a queste superfici.

**Q4 — REFUTO la copertura oracle di reg83.** Il campaign script verifica solo l'ESISTENZA di autoload83_reg.php (measure83-campaign.sh:80); Phase A (:185) passa da measure78.sh, che scarta OGNI body (`curl -o /dev/null`, :249/:277) e non ha response-parity. Il full-body vs oracle esiste SOLO in run_phase_r, che non copre reg83. Un fatal nella fixture (autoload rotto, typo) produrrebbe cifre VA "plausibili" mai distinguibili dal positivo: la cifra "+4 register" poggia su un body MAI osservato.

**Emendamenti:**
- **A-DS26**: test in-cargo che INIETTA artificialmente un CachedUnit con main_program nella corsia includes (accesso diretto a UNIT_CACHE, il path pubblico giustamente lo impedisce), forza l'evizione, pinna main_evicted +1 ed evento uc_log: il contatore deve MORDERE una volta in vita.
- **A-DS27**: header di F15/F15b dichiara la discriminazione PAIR-WISE (F15∧F15b) e lo scope escluso (policy include-lane).
- **A-DS28**: sig() esteso a props defaults + class consts + parent/interfaces (+static_vars se campo separato); secondo mutante = prop default mutato, deve mordere; closures verificate per NOME in m.functions.
- **A-DS29**: ogni fixture citata in una cifra per-fixture (Phase A/W) riceve full-body vs oracle alla stessa rev sull'arm union, ledgerato in campagna, PRIMA della finestra contata (fuori dal canale census).

**Kill-switch:**
- **KS-DS-85-1**: cifra per-fixture da campagna senza full-body oracle di QUELLA fixture alla stessa rev ⇒ cifra VOID.
- **KS-DS-85-2**: pin main_evicted==0 citato come prova d'esenzione senza A-DS26 verde ⇒ pin ADVISORY.
- **KS-DS-85-3**: nuova superficie emessa dal Module (campo nuovo in Func/Class) senza estensione sig()+mutante nello stesso commit ⇒ claim di purezza a_ds17 NULLO.

*Dmitry Stogov — la partizione è la mia forma; i denti intorno vanno ancora affilati.*

---

# VERBALE — B. Gregg, sedia 9, Concilio WP-85 (vincolante per WP-84)

## VERDETTO: CONFERMATO CON EMENDAMENTI — VC REGGE (con riscrittura del kill-switch), una dicitura REFUTATA

**Q1 — Riconciliazione VC: ESATTA (36/36).** Ricomputato dai raw `wp78-harness/measure-out/`: lever n1000 r1–r9 = 0.15/0.15/0.14/0.14/**0.13**/0.13/0.14/0.14/0.14; lever n2000 = **0.24**/0.28/0.27/0.25/0.26/0.26/0.24/0.29/0.29; base n1000 = 6.96/6.94/6.93/6.95/6.92/7.10/**6.91**(r7)/7.03/6.99; base n2000 = 13.89/13.88/13.86/13.91/**13.83**(r5)/13.86/13.87/13.87/13.88. Tutte le 4 liste di verdict83.out coincidono valore per valore. Slope ricomputate: (0.24−0.13)/1000 = **110,0 µs/req**; (13.83−6.91)/1000 = **6920,0 µs/req**; banda 0,05×6920+25 = 371,0; res 2×10µs = 20,0 < 123,7. Esatte. VCR r2: **n=2000 campioni contati esatti**; ricomputo full-set: min 184,167 / median 344,75 / p90 485,417 µs — coincide con verdict (344.8/184.2/485.4). La riga n=11 (parity+warmup) è separata e fuori finestra.

**Q2 — KG-84-3, GIUDIZIO ESPLICITO: VC NON è NULL.** Per la LETTERA del mio kill-switch il braccio base (13,9s user < 30s, nessuna forma per-request) cade nel NULL. Ma lo STESSO verbale WP-84, punto 5 del deliberato, ammette «per-request (A-BG32) **e/o** min-of-R R≥9 (A-BB31)»: il conflitto è nel mio testo, non nella campagna. La ratio del kill-switch era la risoluzione: qui res 20,0 µs/req contro un misurando di 6920 (rapporto 346:1), f=5% ex-ante, 3-low spread quantificato per cella, e la forma per-request esiste sul braccio lever e concorda in ordine di grandezza (min wall 167–184µs vs 110µs user). Dichiarare NULL sarebbe teatro della risoluzione. **VC PASS confermato**, perimetro: confronto A/B di pendenze compile-side; NESSUNA attribuzione per-request del braccio base è autorizzata da questo PASS. La sanatoria è KG-85-1 (sotto), non un'eccezione tacita.

**Q3 — Contabilità void: ESATTA.** Una quarantena (`20260801T012523Z-ac35d24-lockcmp-bite/`); MANIFEST = 108 righe sha256; su disco 108 file + MANIFEST; slot ricomputati: 18 lever fase C (9×n1000+9×n2000) + 3 CR = **21 slot × 5 file = 105, + 3 file m83.* = 108**. Nessun `rm`.

**Q4 — Parity: BASTA, ed è già per-R.** `base_run()` (measure83-campaign.sh:116–143) è invocato per OGNI slot (N,R): ogni run ha server fresco, refusal identità (driver_sha+head_unmoved, KG-84-2 in-header verificato su r7/r5: base_rev/base_hash/rustc/lock_live+base_sha presenti), parity vs oracle, poi 9 warmup + N. Il 1+9=10 pre-loop replica measure78. Non ordino nulla.

**REFUTATO: «interleaved».** I mtime dei raw: TUTTI i lever 03:36:27–03:41:06, CR fino 03:42:10, POI tutti i base 03:42:32–03:50:48. Blocchi adiacenti stessa sera (finestra totale ~14 min), **non interleaving**. La dicitura in MEASURE83 §Verdetti e nello scope VC è falsa alla lettera.

## Emendamenti
- **A-BG33**: forma per-request sul braccio BASE via driver (`curl -w %{time_total}`), senza probe nel binario: l'identità 7593d8e resta intatta. Obbligatoria alla prossima campagna A/B. «Il binario non ha il probe» non è più una scusa valida.
- **A-BG34**: «interleaved» si scrive solo se i timestamp dei raw provano l'alternanza per-R; retro-correzione di MEASURE83/scope VC a «blocchi adiacenti stessa sera».
- **A-BG35**: «regime» in VCR va definito nel verdict (qui = intera finestra N, warmup n=11 drenato fuori). Il nome di una finestra è API (lezione WP-64).

## Kill-switch
- **KG-85-1** (riscrive KG-84-3): claim slope su run <30s user = NULL salvo ALMENO UNA tra (a) forma per-request nella finestra; (b) min-of-R R≥9 con f ex-ante E res<banda/3 E 3-low spread quantificato. VC-83 passa per la via (b).
- **KG-85-2**: «interleaved» in scope/header senza prova di alternanza nei timestamp dei raw ⇒ scope VOID d'ufficio.

*— B. Gregg, sedia 9*
