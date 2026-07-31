# COUNCIL_WP84_REVIEWS.md — Concilio a 9 sedie su S-82.0 (honest instruments + misure residue A-BB6) + programma WP-83

**Data**: 2026-08-01 (chiusura S-82.0)
**Oggetto**: revisione di S-82.0 (commit 2a883e8…5b668d0: 8 passi Concilio
WP-83 + campagna measure82 + verdict82) e giudizio del programma WP-83
(slope CPU, verdetto peak ×W, mitigazioni).
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto
NEXT_SESSION, WP_SESSION_82, MEASURE82_RESULTS, verdict82, il proprio
ordine WP-83 e il CODICE/raw del proprio perimetro; Bak e Gregg hanno
ricomputato le cifre dai raw in proprio.
**Status**: verbali VINCOLANTI per WP-83 (blocco ⚖️ in NEXT_SESSION).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO CON EMENDAMENTI — nessuna opposizione.**
Le MISURE sono confermate a ricomputo indipendente (Gregg: V2, peak, slope,
witness — tutte ESATTE dai raw; Bak: slope riconfermata dai .log). Lo
strumento retained è VIVO e controllato (Leijen). La quarantena, il fold
A-TH21, i denti u64/A-PP20/A-DS19 e la chiusura KB-83-2 sono sottoscritti.
I colpi della tornata:

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **Il programma slope "N-doppio" è aritmeticamente INCAPACE di chiudere
   KB-83-3** (Bak: con rumore moltiplicativo f, spread(slope) ≈
   f·S·(N₂+N₁)/(N₂−N₁) e il fattore resta 3 sulla coppia 2000/4000; floor
   asintotico f·S≈352 > 125,7 — nessun N chiude; Gregg: lo spread base 970
   µs/req è 48× il passo di quantizzazione di time -l, rumore ∝ runtime).
   Chiude solo cambiando FORMA: per-request timestamps nella finestra di
   regime (A-BG32) e/o estimatore min-of-R / R≥9 (A-BB31), con identità
   dell'arm base (A-TH26 rustc+lock sha; A-BG31 pin measure78 nel
   base_run; A-AH31/32 helper+lock-cmp): [KB-84-1, KG-84-2/3, KH84-2,
   KS-AH-84-1].
2. **Il budget retained 20,65MB×W è VIZIATO da doppio conteggio** (Leijen:
   rw_bytes contiene già il floor shallow dei Program via add_program, che
   viene risommato col net deep; ri-etichettato UPPER; Pedersen: la cifra
   è dell'arm CLI-SERVER, per axum serve witness o bound VP-derivato):
   [KL-84-1, KS-PP-84-3]. Il net FLAT ~1,85MB/main per 4 fixture minuscole
   puzza di residuo per-lower non posseduto dalla entry: per-entry in-band
   obbligatorio (A-DL20) e scomposizione PRIMA della delibera peak
   (A-DL22): [KL-84-2].
3. **Mitigazione peak ×W: forma DELIBERATA** (Stogov: main ESENTE da ways
   — vittima = più vecchia NON-main, un solo fp main per key: la forma
   opcache-fedele "una persistent_script per path"; ways-bump REFUTATO;
   Matsakis: SOLO come partizione-per-TIPO dello slot
   `UnitSlot{main, includes}`, MAI skip-per-tag): [KS-MS-84-3,
   KS-DS-84-3/4]. Registry condivisa read-only: stima Bak 25-31MB
   recuperabili (A-BB35 budget ex-ante) MA tocca le IC nel Module ⇒
   riapre KH81-3, non lo aggira (Hoare): [KH84-4].
4. **Conteggi al posto di NOMI, di nuovo** (Klabnik: `grep -cE '^OK'`
   ammette un gate sempre-FAIL non nominato; Hejlsberg: equivalenza
   KH82-2 senza diff computato dallo script e mai transitiva; Gregg:
   header void «162+23»/«46 REFUSAL» MAI ricomputato dai manifest —
   BOCCIATO e retro-corretto in MEASURE82: 45 slot-run / 185 file / 15
   refusal): [KS-SK-84-1, KS-AH-84-3, KG-84-1]. Cifre da micro-bench solo
   con raw committato (A-BG30 — scan retro-corretto: 3 ns/key da
   m82.scanbound.raw).
5. **Pin di SPELLING, non di classe** (Hoare: DECIMO sito lower_source in
   phpt-runner:287 fuori da ogni pin; il ==9 conflaziona 2 prod + 7
   selftest e ammette swap compensativi; troncatura `as u32` sull'epoch
   invisibile al dente 1w; Stogov: a_ds17 è CIECO per costruzione —
   FxHash a seed fisso non può divergere su ordine cotto; il grep-gate
   non vede `for … in &mappa`/drain/retain né mappe fuori suffisso):
   [KH84-1, KH84-3, KS-DS-84-2].
6. **A-MS17 è una VIOLAZIONE d'ordine** (Matsakis: la condizione "al
   prossimo tocco" era innescata da S-82.0 stesso; rinvio rotolante =
   classe M-68.5): atterra nel PRIMO commit WP-83 che tocca vm/mod.rs o
   worker_pool.rs, same-commit: [KS-MS-84-1]. `lower_net` trasloca in
   MainPublishTicket (A-MS22): [KS-MS-84-4].
7. **Twin-pair/A-PP20: PASS che reggono, chiusure da armare** (Pedersen:
   il probe canonicalizzato non vede un inseritore a key divergente —
   classe RETENTION: dente di enumerazione A-PP22; il MARK wc -l è
   disciplina non macchina: flush-before-response a macchina + controllo
   positivo del head-segment A-PP23; Klabnik: sentinella IN-BAND +
   eventi-fuori-trio==0 + union_hash verificato contro matrix A-SK26):
   [KS-PP-84-1/2].
8. **VA +56 = upper bound COMPOSITO, non attribuzione** (Bak: contiene
   Δ_register+Δ_include-HIT+Δ_body; Klabnik: mele/pere non dichiarato):
   terza fixture register-senza-trigger (A-BB33/A-SK28) + etichetta
   "upper bound vs opcache inheritance-cache" (A-DS25): [KB-84-3].
9. **F14 verde ma senza pin-HIT** (Stogov: un re-MISS per-URL la rende
   vacua rispetto alla leva — F14b con main_put==1/main_hit≥4; Klabnik:
   il positivo deve pinnare l'ATTESO C2, non la mera disuguaglianza
   A-SK31): [KS-DS-84-1, KS-SK-84-4].
10. **Identità degli strumenti nel tempo** (Hejlsberg: alloc_id semantica
    in-band per le cifre mem-census cross-campagna A-AH33; Leijen: ogni
    ns da build ad allocatore contante è upper-bound, mai discriminante
    A/B A-DL23): [KS-AH-84-2, KL-84-3].

### Ordine vincolante di apertura WP-83 (S-83.0 "denti di verbale", poi misure)

1. **Debiti di sigillo immediati**: A-MS17 token ZST al PRIMO commit
   (KS-MS-84-1) · A-MS22 lower_net→ticket · A-TH23 (pin lower di CLASSE,
   phpt-runner:287 nominato/foldato, prod==2 e selftest==7 separati) ·
   A-TH24 (dente cast restringente epoch) · A-DS23 (grep-gate iterazioni
   esteso + inventario mappe per NOME) · A-TH25 (drain-sync doc).
2. **Onestà di verdetto/equivalenza**: A-SK30/A-AH34 (equivalenza battery
   per NOME, mai conteggio, mai transitiva, matrix/.done pinnati a rev) ·
   A-SK29 (supplementi normati: lista chiusa, precondizioni, ledger,
   max 1) · A-SK26/A-PP23 (twin-pair: sentinella in-band, head-segment
   positivo, union_hash vs matrix, flush a macchina) · A-PP22 (dente
   enumerazione entries main) · A-BG29/30 GIÀ retro-applicati in chiusura
   S-82.0 (header void riconciliato, scan da raw committato).
3. **Strumento retained SCOMPONIBILE prima della delibera peak**: A-DL20
   (net per-entry in-band + ordinale put; smaschera il flat) · A-DL21
   (budget = rw_bytes − program_floor_share + rw_main_net, share emesso;
   il 20,65 resta UPPER) · A-DL22 (quadratura hello-only W=1 vs
   3,44MB/worker + frazione condivisibile misurata) · A-AH33 (alloc_id
   in-band) · A-PP25 (cifra per-ARM; witness axum o bound VP-derivato).
4. **Fixture nuove**: F14b (pin-HIT + matrice (o,c) vs oracolo per
   braccio, A-DS21/A-SK31) · F15b interleavata (discrimina
   FIFO/LRU/esenzione, A-DS24) · path≥384B (KB-83-1/KB-84-2) ·
   register-senza-trigger (A-BB33/A-SK28) · A-DS22 (a_ds17: thread
   inquinato + ordine permutato + mutation del comparatore).
5. **Slope CPU in forma NUOVA** (mai "solo N-doppio", KB-84-1/KG-84-3):
   per-request timestamps nella finestra di regime (A-BG32) e/o
   estimatore min-of-R / regressione multi-N R≥9 (A-BB31) · f dichiarata
   ex-ante · base-arm-build.sh (A-AH31) + lock-cmp (A-AH32) + pin
   measure78 nel base_run (A-BG31) · rustc -V + lock sha nell'header,
   arm interleaved stessa-sera (A-TH26) · scope del claim nominato.
6. **Delibera peak ×W** (solo DOPO il punto 3): mitigazione = main ESENTE
   da ways nella forma partizione-per-TIPO (A-MS24/A-DS24) con F15b verde
   e, post-mitigazione, main_evicted>0 = VOID d'ufficio (KS-DS-84-4);
   registry condivisa SOLO con budget ex-ante A-BB35 (25-31MB attesi),
   costo sync misurato sul path HIT, e riapertura ESPLICITA di KH81-3
   (KH84-4); peak lever 232±1MB = pin d'identità W=10 (A-BB34).
7. **A-PP18** (ledger drained esatto) calendarizzata PRIMA di ogni
   riconciliazione Δglobal a W>1 (A-PP24).
8. Poi: ripresa ROADMAP da [[php-rust-todo-master]]. Revert policy
   KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH84-1 | conteggio unico prod+selftest sul pin lower (swap compensativo possibile) | pin ADVISORY finché non splittato |
| KH84-2 | claim slope con lock non byte-identici tra arm o rustc diverso | NULLO, mai ADVISORY |
| KH84-3 | cast restringente sull'epoch osservato senza dente | «u64 chiuso» retrocede ad ADVISORY |
| KH84-4 | mitigazione ×W che tocca Module/IC senza riaprire KH81-3 | mitigazione respinta |
| KS-MS-84-1 | commit su vm/mod.rs o worker_pool.rs senza token A-MS17 | gate-lever-pins ADVISORY finché il sigillo non atterra |
| KS-MS-84-2 | side-effect DENTRO lo scope borrow di UNIT_CACHE | STOP, re-audit closure sul cache-path |
| KS-MS-84-3 | mitigazione spedita come skip-per-tag | reject |
| KS-MS-84-4 | lettura di lower_net fuori da main_publish_ticket pre A-MS22 | cifra VOID |
| KS-SK-84-1 | equivalenza battery a conteggio senza nome del gate mancante | precondition FAIL, campagna VOID |
| KS-SK-84-2 | supplemento senza precondizioni di campagna o fuori ledger | cifra VOID (quarantena, mai rm) |
| KS-SK-84-3 | claim sotto-floor con spread scalato > floor | VF declassato a NULL |
| KS-SK-84-4 | controllo positivo soddisfatto da corpo arbitrario senza atteso pinnato | controllo non conta, fixture incompleta |
| KS-AH-84-1 | base-arm senza lock-cmp verde | arm base VOID, delta A/B nullo |
| KS-AH-84-2 | confronto retained cross-campagna senza alloc_id identico | VOID |
| KS-AH-84-3 | equivalenza a conteggio (≥N) o transitiva | campagna VOID, battery integrale da rieseguire |
| KB-84-1 | campagna slope su coppia a fattore 3 con f≥1% dichiarata "chiudibile per N" | VOID |
| KB-84-2 | claim "floor universale" senza fixture ≥384B | VOID |
| KB-84-3 | cifra include-HIT da confronto cross-fixture senza fixture di controllo | VOID |
| KB-84-4 | peak lever post-registry non deterministico (spread >1%) | STOP, attribuzione prima del merge |
| KS-PP-84-1 | segmento twin-pair senza controllo positivo del head | taglio non provato → VOID |
| KS-PP-84-2 | claim "mai pubblicato" su fatal senza delta di enumerazione (post-A-PP22) | insufficiente |
| KS-PP-84-3 | cifra retained ×W citata per axum senza witness d'arm né bound VP-derivato | VOID |
| KL-84-1 | budget ×W citato come rw_bytes+rw_main_net senza scorporo A-DL21 | budget respinto |
| KL-84-2 | delibera mitigazione peak senza scomposizione A-DL22 | rinviata d'ufficio |
| KL-84-3 | cifra ns da binario ad allocatore contante usata come discriminante A/B | claim VOID |
| KS-DS-84-1 | A/B che cita la classe 2 con F14 senza pin-HIT | F14 vacua, classe 2 RIAPERTA |
| KS-DS-84-2 | claim "compile_program pura" appoggiato ad a_ds17 senza A-DS22 | claim NULLO |
| KS-DS-84-3 | mitigazione ways spedita senza F15b verde | mitigazione RESPINTA |
| KS-DS-84-4 | post-mitigazione, run con main_evicted>0 | campagna VOID d'ufficio |
| KG-84-1 | numero d'header non ricomputabile ESATTO dai manifest | header VOID, documento non pubblicabile |
| KG-84-2 | braccio base senza refusal driver_sha/HEAD | confronto A/B VOID |
| KG-84-3 | claim slope da time -l senza forma per-request su run <30s user | NULL automatico, mai stessa campagna ripetuta |

---

## VERBALI INTEGRALI

### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

**Q1 — fold A-TH21.** Nel merito è completo: `run_source_with`/`with_ini`/`with_argv` delegano tutte a `run_source_probed(…, false)` (vm/mod.rs:247/262/350), argv passante, `acquire_oneshot` emesso sull'arm non-probed (:15911) — F-oneshot non è più vacua. **MA il pin è di SPELLING e di FILE**: conta solo `crate::lower_source(` in vm/mod.rs. Eluso da: (i) `lower_source(` nudo dopo un `use`; (ii) `php_runtime::lower_source(` in un ALTRO crate; (iii) `lower_source_seeded`, non pinnato affatto; (iv) `compile_program` pub re-esportato senza sweep workspace. E **un decimo sito esiste già fuori pin**: `crates/phpt-runner/src/lib.rs:287` possiede un `lower_source` di produzione (pre-check — lower duplicato, non compile+run, ma fuori da ogni conteggio). Il ==9 conflaziona 2 produzione + 7 selftest: uno swap compensativo (un lower di selftest tolto, uno produttivo aggiunto) resta a 9 e passa. Sì: troppo permissivo.

**Q2 — dente u64 + mutation.** Il mutant sed morde la DICHIARAZIONE, e un revert reale (incluso `ic_epoch() -> u64`) non può evitare di toccarla: rappresentativo, e il rename di IC_EPOCH fallisce comunque (ep vuoto ⇒ FAIL, fail-closed). Ma la superficie vulnerabile vera è la **troncatura all'uso**: reintrodurre `as u32` sull'epoch ai fill-site mantiene `Cell<u64>` e passa 1w + tutti i test — wrap nel dominio dei valori, invisibile. Il «nessun `as u32`» del mio verbale WP-83 era verifica a mano, mai gate.

**Q3 — drain-sync.** Polarità CORRETTA: lo spin elimina solo il falso-positivo della finestra send→dec; un overlap reale lascia il watermark a 2 e l'assert morde comunque. Il flake residuo è un falso FAIL, mai un falso PASS: fail-closed, accettabile. Residuo: 10M è un timeout in iterazioni senza giustificazione; su runner monocore lo spin-yield può auto-affamare il worker.

**Q4 — identità slope WP-83.** `--offline` + lock copiato pinna le versioni SOLO se il lock è byte-identico tra gli arm e rustc identico — oggi dichiarato, non gateato. Con segnale ~×47 il rischio non è la banda ma l'identità TERMICA: run base lunghe ⇒ throttling asimmetrico su macOS; servono arm interleaved stessa-sera, R≥3, banda ex-ante. E lo scope va NOMINATO: il claim vale per il working-set misurato, mai «CPU ×47» del sistema.

**Nota per il deliberato VP ×W**: la mitigazione «registry condivisa read-only» implica Arc+Sync sui grafi; Module contiene le IC (`Cell`, !Sync) ⇒ richiede la forma IC-out-of-Module che KH81-3 dichiarò non necessaria. Se si sceglie la mitigazione, quel verdetto si RIAPRE, non si aggira.

**Emendamenti:** **A-TH23** pin lower di CLASSE (sweep workspace su `lower_source(`/`lower_source_seeded(`/`compile_program(`, allowlist per file NOMINATA — phpt-runner:287 nominato o foldato; in vm/mod.rs conte SEPARATE prod==2/selftest==7) · **A-TH24** dente troncatura (`as u32|as u16` ==0 sulle righe epoch/IC, o sigillo `Epoch(u64)`) · **A-TH25** drain-sync: bound=timeout dichiarato, tempo nel fail, polarità documentata · **A-TH26** header slope: rustc -V + sha dei DUE lock (uguali o arm VOID), arm interleaved, scope nominato.

**Kill-switch:** **KH84-1** conteggio unico prod+selftest ⇒ pin ADVISORY finché non splittato · **KH84-2** slope con lock non identici o rustc diverso ⇒ NULLO · **KH84-3** cast restringente sull'epoch senza dente ⇒ «u64 chiuso» ADVISORY · **KH84-4** mitigazione ×W che tocca Module/IC senza riaprire KH81-3 ⇒ respinta.

*Tony Hoare*

### 2. Matsakis — ownership/aliasing/borrowck — CONCORDO CON EMENDAMENTI

Nessun difetto di soundness in S-82.0; UNA violazione d'ordine (A-MS17) e due superfici di proprietà da chiudere per costruzione.

**Q1 — filter A-MS18: NON rientrante, verificato.** `unit_cache_get` chiude il borrow di `UNIT_CACHE` dentro il `with` e ritorna Option POSSEDUTA (`.cloned()`); il `.filter()` gira DOPO su valore owned; `uc_stat`/`uc_log` borrowano RefCell DISTINTI: nessun double-borrow oggi. MA la salvezza è POSIZIONALE: l'ottimizzazione ovvia (rifiutare il main DENTRO il find per risparmiare i due Rc::clone) porterebbe i side-effect dentro lo scope del borrow. ⇒ A-MS21.

**Q2 — lever() è UNA volta per acquisizione.** Un secondo `lever()` è inerte per il publish (take ⇒ None) ma NON per `lower_net` (u64 Copy, viaggia su ogni lever successivo): un lever orfano con lower_net>0 e publish None. Il campo è proprietà del PUT pendente: va dentro `MainPublishTicket`, così il take muove ticket+net ATOMICAMENTE. ⇒ A-MS22.

**Q3 — A-MS17: VIOLAZIONE, dichiarata ma reale.** La condizione "al prossimo tocco" era INNESCATA (S-82.0 ha editato vm/mod.rs in almeno 4 punti del perimetro della porta e worker_pool.rs). Attenuante: l'ordine 8-passi non la listava. Ma un rinvio la cui condizione scatta e viene ri-rinviato con la STESSA clausola è chiusura prose-guarded — classe M-68.5. ⇒ A-MS23.

**Q4 — ownership PULITA al sito di eviction** (victim per valore, lettura owned, mai unico-owner). Sulla mitigazione REFUTO in anticipo lo skip-per-tag ("eviction salta i victim con main_program"): discriminazione per TAG in un Vec omogeneo. Forma proprietaria: **partizione per TIPO dello slot** — `UnitSlot { main: Option<CachedMain>, includes: Vec<CachedInclude> }`, ways SOLO sugli include, main singolo per key. Dividendo: main_evicted impossibile per costruzione (resta tripwire ==0 con trigger test-only) e il filter A-MS18 diventa strutturalmente morto ⇒ declassabile a debug_assert.

**Emendamenti:** **A-MS21** reject A-MS18 su Option OWNED fuori dallo scope RefCell, pinnato; preferito match esplicito · **A-MS22** lower_net → `MainPublishTicket{key,fp,lower_net}` · **A-MS23** A-MS17 al PRIMO commit WP-83 che tocca vm/mod.rs o worker_pool.rs, same-commit, commit nominato · **A-MS24** mitigazione SOLO partizione-per-tipo; F15 bumpata per NOME.

**Kill-switch:** **KS-MS-84-1** commit sui due file senza token A-MS17 ⇒ lever-pins ADVISORY · **KS-MS-84-2** side-effect dentro lo scope borrow di UNIT_CACHE ⇒ STOP+re-audit · **KS-MS-84-3** mitigazione come skip-per-tag ⇒ reject · **KS-MS-84-4** lettura di lower_net fuori dal put pre-A-MS22 ⇒ cifra VOID.

*Niko Matsakis*

### 3. Klabnik — spec/testabilità/gate — CON EMENDAMENTI

verdict82 è il miglior strumento della serie (fail-closed reale, NULL non promosso, DECLARED-WORSE mai silenzioso). Quattro vacui residui e un precedente pericoloso.

**Q1 — vacui.** (a) *VH/MARK*: i pin esatti probe==hit==nreq ∧ put==0 rendono uno shift del segmento FAIL rumoroso — bene. Ma il MARK wc -l è ancorato al tempo, non al contenuto; il regex del segmento ignora in silenzio ogni evento fuori dal trio; `union_hash` sta nel meta ma verdict82 non lo confronta MAI con la riga matrix. (b) *VF scaled-pair*: NESSUNO spread dichiarato per V2(1000)/V2(2000) — oscillazione 0,1MB = ordine del floor: claim giudicato alla risoluzione del rumore. (c) *VA 787−731*: mele/pere NON dichiarato (autoload82 differisce da hello in TUTTO il body); `$b_steady` letto e mai usato — codice morto.

**Q2 — supplemento fase-R: precedente pericoloso.** mem_hash forte, ma: non replica le precondizioni di campagna; cambia ARM vendendolo come re-run; supplement-1 muto senza manifest; nulla limita i re-run finché la cifra non piace. "Driver-independent" è auto-dichiarato, non un criterio.

**Q3 — battery equivalence: elusione da manuale.** `grep -cE '^OK'` conta righe, non gate: 15 meno UNO sempre-FAIL = 14 OK = PASS, mancante mai nominato — violazione di "gate per NOME, mai conteggio". Più: matrix-final non pinnato a rev==HEAD, `.done` senza rev.

**Q4 — F14 c=n**: prova PIÙ di "$_GET arriva" (un main cotto su c=y darebbe BX==W14 ⇒ morde: probe di staleness genuino) ma MENO del comparatore pieno: passa con QUALSIASI corpo ≠ W14, anche un fatal. L'atteso C2 non è mai pinnato.

**Emendamenti:** **A-SK26** sentinella IN-BAND nel uclog al posto di wc -l; pin eventi-fuori-vocabolario==0 nel segmento; union_hash confrontato con matrix · **A-SK27** coppia scalata con spread per-N stampato e pin spread ≤ floor, altrimenti VF=NULL · **A-SK28** twin non-autoload per isolare la quota register; finché manca, [derivata] dichiara "upper bound, fixture diverse"; rimuovere $b_steady · **A-SK29** supplementi = lista chiusa, precondizioni replicate, ledger void, max 1 per fase · **A-SK30** equivalenza per NOME (set nominato, mancante MAI in {parity-full, lever-fixtures, measure-cifre}; matrix-final e .done pinnati a rev) · **A-SK31** F14 positivo con atteso C2 PINNATO.

**Kill-switch:** **KS-SK-84-1** equivalenza a conteggio ⇒ precondition FAIL, campagna VOID · **KS-SK-84-2** supplemento fuori norma ⇒ cifra VOID · **KS-SK-84-3** claim sotto-floor con spread scalato > floor ⇒ VF NULL · **KS-SK-84-4** positivo soddisfatto da corpo arbitrario ⇒ non conta (estende KS-AH-80-2).

*— Klabnik, sedia 3.*

### 4. Hejlsberg — build/identità — CONCORDO CON EMENDAMENTI

S-82.0 è la sessione più onesta dell'arco sul mio perimetro (il --no-run A-AH29 ha MORSO; la quarantena ha pagato ×3). Tre claim refutati.

**Q1 — Trappole void: due meritano il dente, una no.** *Chain-HEAD*: KG-81-2 ha già morso — non si duplica. *Worktree-subdir* e *--locked-vs-dev-dep*: vivono come check inline in UNO script, e WP-83 riuserà lo stesso base-arm — ripetizione certa. Forma minima: helper unico `base-arm-build.sh` (crate-root check, lock copiato, --offline, lock-cmp, hash+lock-sha nell'header), chiamato dalle campagne.

**Q2 — REFUTO che l'hash del binario basti.** L'hash è identità, non provenienza. `--offline` vieta la rete, NON il re-resolve: cargo può riscrivere il lock dalla cache in silenzio. Dente da una riga: `cmp` byte del lock POST-build contro il live; toccato ⇒ FAIL. Più sha del lock nell'header raw.

**Q3 — REFUTO la confrontabilità cross-campagna per hash.** mem_hash discrimina tutto e quindi niente: serve **allocator_id in-band** (const semantica, emessa nelle righe census, bump same-commit a ogni cambio della superficie di conteggio, pin nel census-twin).

**Q4 — KH82-2: confine BUCATO in due punti.** (a) `-ge 14` accetta un gate mancante senza nominarlo; (b) il precondition delega la certificazione del diff al matrix a HEAD, che certifica il build, non l'equivalenza. Confine formale: legale sse (i) diff battery-rev..HEAD su crates/Cargo* computato VUOTO dallo script; (ii) gate assenti = SET per NOME con motivazione; (iii) al più UNA equivalenza per catena; (iv) mai per un gate il cui oggetto è il file cambiato.

**Emendamenti:** **A-AH31** base-arm-build.sh unico · **A-AH32** lock-cmp post-build obbligatorio · **A-AH33** alloc_id in-band + pin census-twin · **A-AH34** KH82-2 riscritta coi vincoli (i)-(iv), il -ge 14 sparisce.

**Kill-switch:** **KS-AH-84-1** base-arm senza lock-cmp verde ⇒ arm VOID · **KS-AH-84-2** confronto retained cross-campagna senza alloc_id identico ⇒ VOID · **KS-AH-84-3** equivalenza a conteggio o transitiva ⇒ campagna VOID, battery da rieseguire.

*Anders Hejlsberg — sedia 4.*

### 5. Bak — alloc-rate/path caldi — CONFERMATO CON EMENDAMENTI

**Il programma slope WP-83 (N=2000/4000) è aritmeticamente INCAPACE di chiudere KB-83-3 e va riprogettato.**

**Q1 — Ricomputo dai raw.** Confermo: lever 120/170/160 → 150,0 µs/req; base 6520/7490/7120 → 7043,3; spread 970. Lo spread lever (50) è ~metà granularità time -l, ~metà rumore. Lo spread base 970 NON è granularità: rumore macchina moltiplicativo ~5% (r2 ha real 51,4s vs 38,6/40,6 — interferenza). **Il numero**: con rumore moltiplicativo f, spread(slope) ≈ f·S·(N₂+N₁)/(N₂−N₁). La coppia (2000,4000) ha fattore 3, IDENTICO a (1000,2000) ⇒ spread atteso ≈ 1050 µs/req — invariato. Floor asintotico f·S ≈ 352 > 125,7: **nessun N chiude la banda**. Chiude solo cambiando estimatore: min-of-R (min-slope base dai raw = 6840), o R≥9 con media (352/√9≈117 ≤ 125,7), o f ≤ 0,6% (macchina quiescente).

**Q2 — KB-83-1 aperto**: peso MODERATO ma bloccante per la generalizzazione — il pin a_bytes==2×len regge sui raw (hello 196=2×98, autoload 206=2×103) ma entrambi <384B: il modello è non-falsificato al suo confine. Fixture da un'ora, si fa in WP-83.

**Q3 — VA: derivata BOCCIATA come attribuzione, tenuta come upper bound.** Dai raw: +56 su b, ma anche c_calls 1 vs 0, resid +121, retain 2 vs 1. Il fixture differisce per il BODY: +56 = Δ_register + Δ_include-HIT confusi. Terza fixture register-senza-trigger: include-HIT = 56 − Δ_reg. "+56 ≥ costo include-HIT" è vero; "+56 = include-HIT" è falso per costruzione.

**Q4 — Peak W=10: due dinamiche confermate.** Base 189,7/202,0/201,1 (spread 6,2%, user 0,95-1,08s): MAX di overlap transiente di compile concorrenti — stocastico. Lever 231,9/232,0/231,9 (0,04%): somma deterministica dei retained ×W. Il ×3 esatto è la firma del retained-dominato. **Stima registry read-only**: condivisibile ≈ 3,44×(W−1) ≈ 31 MB − mutabile per-worker (10-20%) ⇒ recupero ~25-31 MB ⇒ lever ~201-207 MB, dentro banda. Non-gratis: sincronizzazione sul path HIT — costo da misurare contro i 150 µs/req.

**Emendamenti:** **A-BB31** estimatore ridefinito (min-of-R o multi-N R≥9; f ex-ante) · **A-BB32** mai coppie a fattore 3 spacciate per "risoluzione raddoppiata"; dichiarare f·S · **A-BB33** +56 declassato a UPPER BOUND; fixture register-senza-trigger obbligatoria · **A-BB34** peak lever 232±1 MB = pin d'identità W=10 · **A-BB35** registry: budget 25-31 MB ex-ante + costo sync misurato.

**Kill-switch:** **KB-84-1** campagna slope a fattore 3 con f≥1% "chiudibile per N" ⇒ VOID · **KB-84-2** "floor universale" senza fixture ≥384B ⇒ VOID · **KB-84-3** cifra include-HIT cross-fixture senza controllo ⇒ VOID · **KB-84-4** peak post-registry non deterministico (spread >1%) ⇒ STOP.

*Lars Bak*

### 6. Pedersen — confine per-richiesta/lifecycle — CONCORDO CON EMENDAMENTI

**Q1 — REFUTO la copertura della "seconda via".** `uc_main_key_in_cache` canonicalizza: il probe vede SOLO la key canonica — un inseritore a key divergente crea un'entry che il probe non vede; a_pp20 passerebbe col fatal IN cache. Distinguo: il serve-path è salvo (anche l'acquire canonicalizza ⇒ mai SERVITA); la classe scoperta è RETENTION (main fatale ritenuto per la vita del thread). Serve il dente sull'ENUMERAZIONE.

**Q2 — Il MARK è disciplina, non macchina — ma fail-closed in ogni modo realizzabile.** Il flush di fine-run precede send ⇒ curl ⇒ wc -l: l'ordine regge OGGI, garantito da commenti (stessa classe del binding output-capture, che rendemmo vincolante). Analisi dei modi di rottura: warm-line tardiva ⇒ put nel segmento ⇒ FAIL; riga persa al TERM ⇒ hit<nreq ⇒ FAIL. L'uguaglianza ESATTA per-path chiude ogni compensazione: nessun falso-PASS identificato — il PASS VH sta in piedi. Ma l'ordine va pinnato a macchina e il taglio testimoniato (head-segment con ESATTAMENTE 4 main_put warm).

**Q3 — A-PP18 resta APERTA** (drop booka ancora solo s0→drop). Nessuna violazione in S-82.0 (nessuna riconciliazione tentata; KS-PP-83-3 fail-closed). Priorità ALTA-condizionata: la delibera VP spingerà verso census a W=10 = esattamente la precondizione VOID. Chiuderla PRIMA.

**Q4 — Il moltiplicatore ×W trasferisce, la CIFRA no.** 20,65MB è dell'arm CLI-SERVER: dump mai osservato su axum; superglobali divergenti (KS-DS-82-3) toccano lowering/seeding; reg_mode è nella UnitKey. Il +3,44MB/worker su axum è consistenza, non identità. Forma onesta: cifra per-ARM; per axum witness o bound VP-derivato.

**Emendamenti:** **A-PP22** `uc_main_entry_count()` (entries con main_program), delta==0 nel test a_pp20 · **A-PP23** flush-before-response a MACCHINA + controllo positivo head-segment nel verdict · **A-PP24** A-PP18 calendarizzata PRIMA di ogni riconciliazione Δglobal W>1 · **A-PP25** cifra retained PER ARM; axum: witness fp-identity o bound VP-derivato.

**Kill-switch:** **KS-PP-84-1** segmento senza positivo del head ⇒ VOID · **KS-PP-84-2** "mai pubblicato" senza delta di enumerazione ⇒ insufficiente · **KS-PP-84-3** retained ×W per axum senza witness/bound ⇒ VOID.

*Michael Pedersen — il PASS di VH e VR regge; le chiusure emendate no, senza denti.*

### 7. Leijen — mimalloc/footprint — CONCORDO CON EMENDAMENTI

A-DL15/16 eseguiti sul serio; la scoperta dello strumento MORTO è la mia lezione incarnata: onore a chi l'ha trovata col controllo positivo. Ma REFUTO la composizione del budget e la lettura aggregata del net.

**Q1 — 7.407.810 B / 4 main: puzza nella FORMA.** Se il prelude (~1MB Rc-condiviso) non si rialloca a lower warm, l'atteso è ASIMMETRICO (primo put ≫ altri tre); 1,85MB/main FLAT implica massa ri-pagata a OGNI lower — residuo di PROCESSO che la entry non possiede. L'aggregato lo rende invisibile. GA_* sono contatori di processo: finestra inquinabile da altri thread — dichiararlo. ORDINO il per-entry; senza, la cifra è ammissibile solo come UPPER.

**Q2 — Ordinamento coerente, identità NON dimostrata.** 14,25 > 11,19 (rapporto 1,27, verso giusto), ma l'algebra esige: rw_bytes − bytes_counted = Σshared-once + Σprogram_floor = 3.057.613 B — nessun addendo stampato. Addendi in-band o resta un rapporto, non un'algebra.

**Q3 — +34,4MB quadra in prima battuta, e proprio per questo va scomposto.** Lever-vs-base entrambi a W=10 ⇒ l'overhead per-thread mimalloc si cancella: +3,44MB/worker è il marginale della leva — e la quadratura CORROBORA il flat di Q1 (residuo per-lower nel net). Prima di deliberare: (i) quadratura hello-only W=1 vs 3,44; (ii) i 4 net per-entry; (iii) frazione CONDIVISIBILE misurata (dimensiona la registry: risparmio = shared×(W−1)). **E il budget 20,65MB è VIZIATO**: rw_bytes+rw_main_net somma il floor shallow dei Program E il loro net deep — stessa massa due volte, più il prelude in entrambi i lati. Come UPPER per un DECLARED-WORSE erra nel verso sicuro; come "budget" è respinto.

**Q4 — MemCountingMi: i BYTE sono esatti, i NS no.** Due fetch_add su cache-line globali contro un fast-path small-alloc a scala ns ⇒ +20-50% per-alloc nelle finestre dense; a W>1 le line rimbalzano. Ogni ns da build contante è upper-bound, mai discriminante A/B.

**Emendamenti:** **A-DL20** rw_main_net PER-ENTRY in-band con ordinale di put; se flat, scorporare il residuo per-lower; finestra su contatori di processo dichiarata · **A-DL21** budget = rw_bytes − program_floor_share + rw_main_net, share emesso; il 20,65 = UPPER · **A-DL22** pre-delibera: quadratura hello-only W=1 + frazione condivisibile · **A-DL23** etichetta "alloc-instrumented, timing upper-bound" su ogni ns.

**Kill-switch:** **KL-84-1** budget senza scorporo A-DL21 ⇒ respinto · **KL-84-2** delibera senza scomposizione A-DL22 ⇒ rinviata d'ufficio · **KL-84-3** ns da build contante come discriminante A/B ⇒ VOID.

*Daan Leijen*

### 8. Stogov — semantica Zend — CONCORDO CON EMENDAMENTI

I miei 5 ordini eseguiti nella lettera; due hanno buchi di macchina. (Nota: la sede `Vm::enum_cache` è dichiarata correttamente in design79; il riferimento "~12708" del verbale precedente è stantio.)

**Q1 — F14: le tre superfici CI SONO; manca il pin-HIT.** Il controllo positivo NON è mero transito di $_GET (l'unico delta del braccio c=n è il CORPO della condizionale: un main cotto darebbe BX==W14 ⇒ morde). MA nulla prova che le richieste 2-4 siano HIT: un re-MISS silenzioso renderebbe F14 verde e VACUA rispetto alla leva. Ordino F14b.

**Q2 — a_ds17 è CIECO all'order-baking per costruzione.** Il commento "hash order nondeterministic across processes" è FALSO su questo tree: FxHashMap ha seed FISSO ⇒ stessa sequenza d'inserimento = stesso ordine, sempre — double-compile e thread vergine non possono MAI divergere su un ordine cotto. E il grep-gate non vede `for … in &mappa`, `.drain()`, `.retain()`, `.into_iter()`, né mappe fuori suffisso.

**Q3 — F15/WP: rischio reale BASSO, mitigazione dovuta.** WordPress: root index.php solo main; l'index.php del tema è un altro file. main_evicted==0 coerente. **DELIBERO: main ESENTE da ways** (vittima = più vecchia NON-main; un solo fp main per key ⇒ bound ways+1 per costruzione) — la forma opcache-fedele: una persistent_script per path, mai vittima per-chain. Refresh-on-hit ammesso come complemento LRU ma da solo NON salva F15. Ways-bump REFUTATO (sposta la soglia a 9 fp).

**Q4 — VA: semantica opcache CORRETTA.** Include cached = riuso della persistent_script, zero compilazione (a3==0 esatto); il costo per-richiesta è l'ESECUZIONE dell'op_array e vive legittimamente in b. Upper bound onesto: opcache con INHERITANCE_CACHE può saltare il re-link — leva futura a backlog NOMINATO.

**Emendamenti:** **A-DS21** F14b: pin-HIT in-fixture (main_put==1, main_hit≥4) + matrice (o,c) completa vs oracolo per-braccio · **A-DS22** a_ds17 bracci mancanti: thread INQUINATO + ordine PERMUTATO con confronto per-NOME + mutation-test del comparatore sig · **A-DS23** grep-gate esteso (for-in-&, into_iter, drain, retain) + inventario mappe per NOME con decoy · **A-DS24** mitigazione: main-esente + F15b interleavata (discrimina FIFO/LRU/esenzione); post-mitigazione F15 flip a ==0 col contatore VIVO · **A-DS25** +56 etichettato "upper bound vs opcache inheritance-cache".

**Kill-switch:** **KS-DS-84-1** classe 2 citata con F14 senza pin-HIT ⇒ F14 vacua, classe RIAPERTA · **KS-DS-84-2** "compile_program pura" su a_ds17 senza A-DS22 ⇒ NULLO · **KS-DS-84-3** mitigazione senza F15b verde ⇒ RESPINTA · **KS-DS-84-4** post-mitigazione, main_evicted>0 ⇒ campagna VOID d'ufficio.

*Dmitry Stogov — quarantena, strumento resuscitato e KB-83-2 ad aritmetica: sottoscritti.*

### 9. Gregg — metodologia di misura — MISURE CONFERMATE, HEADER BOCCIATO

Ho ricomputato tutto dai raw, in proprio.

**Q1 — Riconciliazione (tutte ESATTE dove c'è un raw):** V2 n100 20,2×3 / n200 20,0-20,1 → Δ −0,133 ✓; scalata 20,167/20,167 → 0,000 ✓. Peak lever 231,9/232,1/232,0 → 232,0 ✓ (probe interno concorda); base 189,65/202,04/201,14 → 197,6 ✓; Δ +34,4/+17,4% ✓ (spread base 6,3%, a un capello dalla banda). Slope ✓. Witness a_calls==2 ESATTO ×4 ✓. Derivate 3,44 e 20,65 (21.657.837 B) ✓. **NON riconciliata: scan 2 ns/key** — nessun raw (test *ignored* nel cargo-test.log): violazione KG-83-3. [RETRO-APPLICATO in chiusura: raw committato m82.scanbound.raw → 308/503 ns, 3 ns/key.]

**Q2 — void_runs: HEADER BOCCIATO.** I manifest contano 81+81+23 = **185 FILE** (non run); il "46 REFUSAL" non esiste (manifest-3: **15** file da 287 B). Nessuno dei due numeri era stato ricomputato. [RETRO-APPLICATO: header riconciliato per-manifest — 45 slot-run / 185 file / 15 refusal.]

**Q3 — base_run senza ENFORCE.** *Contaminano*: (1) niente refusal driver_sha/chain-HEAD — il guasto di campagna-1 può ripetersi in silenzio nel braccio base; (2) niente response-parity — N è "richieste inviate", errori costano meno CPU ⇒ bias sulla slope. *Non contaminano* (ma tolgono il cross-check): boot-probe e probe interno assenti (il lever ha interno==esterno 232,0 ✓; il base non ha controprova). Pin minimi in ordine: refusal identitario → response-parity → probe interno gemello → forma per-request.

**Q4 — granularità.** time -l user a 10 ms ⇒ quantizzazione ±20/±10 µs/req: NON è il collo — lo spread base 970 µs/req = 0,97 s è 48× il passo, rumore ∝ runtime (~5-13%). Raddoppiando N raddoppiano segnale E rumore ⇒ spread ~invariato; anche nell'ipotesi ottimistica 970/2=485 ≥ 125,7. **N=4000 da solo NON basta: serve la forma per-request** (timestamp monotonic nella finestra di regime).

**Emendamenti:** **A-BG29** contabilità void in RUN e FILE separate per-manifest, ricomputata da script (gate-measure-cifre esteso ai MANIFEST) · **A-BG30** cifre da micro-bench solo con raw committato · **A-BG31** base_run adotta i pin measure78 (identità → parity → probe) · **A-BG32** slope in forma per-request; il raddoppio di N non è mai, da solo, la risposta a uno spread ∝ runtime.

**Kill-switch:** **KG-84-1** numero d'header non ricomputabile dai manifest ⇒ header VOID, documento non pubblicabile · **KG-84-2** braccio base senza refusal driver_sha/HEAD ⇒ confronto A/B VOID · **KG-84-3** slope da time -l senza forma per-request su run <30 s user ⇒ NULL automatico.

*— B. Gregg, sedia 9*
