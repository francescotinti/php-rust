# COUNCIL_WP88_REVIEWS.md — Concilio a 9 sedie su S-86.0 (contro-prova A-BB45 + ABBA purge + sigilli/battery v4) + programma WP-87(sessione)

**Convocato**: 2026-08-02, chiusura S-86.0 (regola utente 2026-07-26: 9 sedie
dopo OGNI sessione, mandato REFUTARE). Atti: NEXT_SESSION_WORDPRESS.md,
sessions/WP_SESSION_86.md, wp86-harness/MEASURE86_RESULTS.md + verdict86.out
+ verdict86.sh + measure86-campaign.sh + battery-86pre.sh,
wp87-harness/COUNCIL_WP87_REVIEWS.md (gli ordini eseguiti), codice e raw dei
rispettivi perimetri. Verbali VINCOLANTI per il design WP-87(sessione).

## ⚖️ SINTESI DI CONVERGENZA

(compilata in coda ai verbali — v. fondo file)

---

## VERBALI INTEGRALI

# VERBALE — Hoare, sedia 1, Concilio WP-88

**VERDETTO: CON EMENDAMENTI** — i miei ordini A-TH36/37/38/39 sono eseguiti nella lettera e KH87-1/2/3 non sono scattati; ma ho verificato nel codice quattro superfici residue. Il mandato è refutare: refuto.

**Q1 — A-TH36: le due finestre nominate sono chiuse; ne resta una TERZA, dei parametri.** Verificato: arm a vm/mod.rs:16511, subito dopo il solo check enabled (:16499); kpath (:16521), putord, superseded/victim/replaced (:16530-16532) sono dichiarati DOPO la guardia ⇒ in unwind droppano prima che disarmi; `cu` è mossa nella closure a :16536, quindi un panic DENTRO la mutazione droppa le catture ancora guardate. MA i **parametri** `key: UnitKey, cu: CachedUnit` (:16498) in Rust droppano DOPO tutti i locali: un panic fra :16511 e :16536 (es. alloc in `key.path.clone()` :16521) fa droppare `cu` a guardia GIÀ disarmata. Oggi innocuo (CachedUnit = Rc di dati piatti, :15376-15385), ma la garanzia è tornata posizionale. Il ramo early-return :16499 droppa `cu` non guardata (benigno, cache mai toccata).

**Q2 — A-TH37: l'accoppiamento marker↔grep è per CONVENZIONE, non pinnato.** Il payload `phpr_vm_gate_probe_tainted_a_th37` (vm/mod.rs:539) contiene la sottostringa `vm_gate_probe` che `probe_in` greppa (wp85-harness/gate-binary-noprobe.sh:34-38). Ma il selftest del gate sintetizza la PROPRIA stringa nei bin fittizi — prova il detector, mai il legame col marker sorgente; e gate-lever-pins.sh non contiene alcuna occorrenza di `VM_GATE_PROBE_TAINT`. Un rename del payload compila verde, selftest verde, KH87-2 (verificato UNA volta, S-86.0) mai ri-armato: dente svuotato in silenzio.

**Q3 — A-TH38/39: grafie che ancora sfuggono, PER NOME.** (1) UFCS `RetainSet::production_gate(&rs)` / `MainUnit::vm_gate(&u)` — i pin `[.]production_gate[(]`/`[.]vm_gate[(]` (pins:333-336) esigono il punto; (2) spaziatura `. vm_gate (` — adiacenza obbligata dalla regex; (3) `Self { …, ..o }` dentro `impl CachedUnit` — cu_functional_update aggancia solo `CachedUnit[[:space:]]*\{` (pins:389); (4) alias `type CU = CachedUnit; CU { …, ..o }` — stessa cecità; (5) `CachedUnit`⏎`{` multilinea — awk line-based non arma depth; (6) **la decisiva: carrier a VALORE INTERO** — `#[derive(Clone)]` (:15376) + `unit_cache_get` che ritorna `.cloned()` (:16053): un re-put di un hit clonato porta `main_program: Some` con ZERO token `main_program` nel sorgente — nessuna TH39RE può vederlo, solo A-MS27 chiude; (7) transmute multilinea o aliasato (`use std::mem::transmute as tm` + `type G<'a>=VmGate<'a>`) — sweep :360 line-bound e name-bound; (8) `impl Copy`⏎`for VmGate` e `impl Copy for G<'_>` via alias — stessa classe.

**Q4 — putord: avanza sempre (:16525-16529), corretto; tre disallineamenti.** (a) la riga `supersede entries` (:16600) NON porta putord — il claim doc «cross-put reorder detectable» (:16474-16477) vale solo per la corsia evict/main_evicted (:16619, :16626); (b) esistono DUE spazi ordinali non mappati: `UC_STATS.main_put` (:16421-16425, solo main) e `UC_PUT_ORD` (tutti i put) — un join fra `ord=` e `putord=` è indefinito; (c) il contatore è per-thread ma le righe uc_log non portano thr (:15873): a W>1 putord collide fra thread; (d) `o.get()+1` in release wrappa silente — irraggiungibile (2^64) ma non dichiarato.

**Emendamenti:**
- **A-TH40**: subito dopo arm, `let (key, cu) = (key, cu);` — i parametri diventano locali post-guardia e droppano guardati; commento aggiornato.
- **A-TH41**: pin di accoppiamento A-TH37 — il letterale payload ==1 in vm/mod.rs E lo stesso letterale presente in gate-binary-noprobe.sh (single-source), verificato dalla battery.
- **A-TH42**: sweep v5 — UFCS `::production_gate(`/`::vm_gate(` ==0 fuori sedi; regex spacing-tolerant; pin `= CachedUnit;`/`= VmGate` (alias) ==0; transmute/impl su testo tr-joined (multilinea).
- **A-TH43**: putord sulla riga supersede + thr in-band (o limite W=1 dichiarato) + nota doc sui due spazi ordinali.

**Kill-switch:**
- **KH88-1**: payload marker o token grep cambiati senza positive-control tainted nello STESSO commit ⇒ gate-binary-noprobe ADVISORY, campagne VOID.
- **KH88-2**: re-put di un CachedUnit clonato (carrier a valore) prima di A-MS27 ⇒ unicità del produttore (A-TH31) declassata, attribuzioni main-lane VOID.
- **KH88-3**: panic osservato nella finestra pre-closure di put con campagna proseguita ⇒ run VOID (estende KH87-3).
- **KH88-4**: putord citato cross-thread o joinato con main_put senza mapping dichiarato ⇒ claim d'ordine ADVISORY.

Firmato: T. Hoare, sedia 1.

---

# VERBALE — Hejlsberg, sedia 4, Concilio WP-88

**VERDETTO: CON EMENDAMENTI.** A-AH40 e A-AH41 sono implementati nella lettera e mordono nei due scenari nominati; ma A-AH40 vede solo le ADDIZIONI e solo a fine battery, A-AH41 è attribuzione senza comparatore, e il fallback bare-path di A-SK41 è "inerte" solo per contingenza, non per costruzione.

**Q1 — set-difference (battery-86pre.sh:60-88).** Due matrix-gate nella stessa battery ⇒ `NMTX=2` ⇒ FAIL, nessun `.done`; run standalone DURANTE la battery ⇒ idem. Entrambi fail-closed, e il porcelain in testa (A-SK42) forza il commit dell'archivio orfano prima del re-run: la catena regge. Tre residui per NOME: (a) `comm -13` conta solo addizioni — una DELEZIONE di archivio mid-battery è invisibile; (b) create+delete transiente (intruder che si ripulisce) sfugge, e con l'archivio del gate deletato l'intruder può farsi nominare nel `.done` (stessa-rev obbligata dal case-check a valle, quindi build deterministica identica — ma l'identità è per FORTUNA, non per catena); (c) il check spara solo a fine battery: un rebuild esterno di `$BIN` fra i gate 1 e 15 è fuori giurisdizione (lo coprono solo i gate che hash-ENFORCEano in proprio).

**Q2 — chain coerente; la bruciatura è la FEATURE che ha morso.** Ledger stamps: 3818edd/9d50b47/c259bc6, ognuno appeso dalla battery e committato (f463813, 59eaa2e, 45d3714). Equivalence line 3 `battery_rev=9d50b47 head=70824ea`: appesa a HEAD=70824ea, committata in 8429763; la campagna è poi ABORTITA (bash-3.2), raws VOID committati (c259bc6). La riga NON è morta per la macchina: brucia 9d50b47 (one-per-chain) e blocca 70824ea come futura BREV (anti-transitiva) — entrambi effetti conservativi, ed è esattamente questa bruciatura che ha FORZATO la battery fresca a c259bc6 invece di un'equivalenza transitiva. Il tooth (iii) legge la riga correttamente in ogni caso futuro. Due nei: l'append è finito in un commit MISTO col fix memrun86 (8429763) — "commit NOW" onorato in lettera, non in isolamento; e il ledger non distingue equivalenza CONSUMATA da NON-consumata (un lettore umano può inferire una campagna a 70824ea che non esiste — MEASURE86 §Identità lo dichiara, il ledger no).

**Q3 — A-AH41 a metà.** `rustc=`/`cargo=` sono in testa all'archivio (gate-feature-matrix.sh:77-78), parsabili (`sed 's/^rustc=//'`; valore con spazi, formato `rustc 1.x.y (hash data)`). Ma il grep sull'albero mostra ZERO consumer: nessun checker confronta il `rustc=` cross-archivio o contro l'ambiente della campagna. Il confronto resta MANUALE — l'attribuzione per NOME che KS-AH-87-2 esige al mismatch dipende dal fatto che qualcuno si ricordi di leggere la riga.

**Q4 — il fallback non è inerte per costruzione.** battery-equivalence.sh:137-139: `{ git show HEAD:php-rust/…; git show HEAD:…; } | grep -q`. Oggi il bare-path NON esiste a HEAD (verificato: fatal), quindi un solo emettitore. Ma se un twin tracked comparisse al git-root (ristrutturazione, copia accidentale), ENTRAMBE emettono e il grep passa sulla CONCATENAZIONE: uno stamp presente SOLO nel twin — file che il tooth (iii) non esamina — certifica. Canale di forgiatura latente, non attivo.

**Emendamenti:**
- **A-AH43**: risolvere il path UNA volta via `git rev-parse --show-prefix` e interrogare esattamente `HEAD:${PREFIX}wp83-harness/evidence/battery-stamps.ledger` — mai concatenare due sorgenti di prova.
- **A-AH44**: cablare il comparatore toolchain: equivalence/campagna leggono `rustc=` dalla matrix nominata nel `.done` e la confrontano con `rustc -V` corrente; divergenza ⇒ FAIL nominato (KS-AH-87-2 diventa meccanico).
- **A-AH45**: snapshot matrix-archive confrontato in ENTRAMBE le direzioni (`comm -23` per delezioni) e convenzione: un'equivalenza abortita si annota per NOME nel MEASURE della campagna che NON l'ha consumata (mai edit del ledger).

**Kill-switch:**
- **KS-AH-88-1**: stamp A-SK41 accettato da una sorgente `git show` diversa dal path canonico risolto col prefix ⇒ equivalenza VOID.
- **KS-AH-88-2**: `.done` che nomina un archivio matrix non prodotto dal matrix-gate della battery stessa (delezione/sostituzione mid-battery) ⇒ battery VOID.

*Anders Hejlsberg — sedia 4. La catena tiene perché la bruciatura ha morso; ora il toolchain deve avere il suo comparatore e il fallback la sua unica porta.*

---

# VERBALE WP-88 — Sedia 7, Daan Leijen

**VERDETTO: CONCORDO CON EMENDAMENTI.** I miei quattro ordini WP-87 sono eseguiti e MORDONO (A-DL32 fetch_max in testa al dump, memcensus.rs:841; A-DL33 burst declassificato dal dente, vm/mod.rs:16209 + VBURST PASS; A-DL34 semantica dichiarata e pinnata, vm/mod.rs:2293-2306 + controlli 17848-17886; VABBA ha refutato il MIO candidato purge — lo registro senza sconti). Ma ho trovato un buco residuo: **il fix atomicità copre `census_line`, NON `phys_window_dump`**, che scrive ancora `writeln!` per frammento (memcensus.rs:856/864/895) sullo stesso file O_APPEND — le righe mi_proc/ctx/mi_bin da thread concorrenti restano garblabili esattamente come il `pid=pid=` dello smoke.

**Q1 — È il design, non un artefatto di `mi_heap_of`.** Nel vendored v3 (3.3.02): `mi_heap_of` (heap.c:233) risolve pagina→heap via `mi_page_heap`; l'entità per-thread in v3 è la **theap** (types.h:14: "a thread local heap belonging to a specific heap"), e tutte le theap di default appartengono a `heap_main` (init.c:181-183, 332-333). Puntatore identico = verità misurata. API residue per lo split: (a) `mi_heap_stats_get` esiste (mimalloc-stats.h:139) ma sul heap CONDIVISO — inutile; `theap->stats` esiste (types.h:535) ma NON è esportata → servirebbe patch vendored. (b) `mi_heap_area_t` non espone `xthread_id` (che vive nel page header, types.h:383) → visitor patch vendored. (c) **Unica via pulita**: per-worker `mi_heap_new()` + `mi_heap_theap()` + `mi_theap_set_default()` (mimalloc.h:373-376) — heap distinto per worker, poi visit/stats per heap splittano davvero. Costo: cambia la superficie di conteggio ⇒ **bump ALLOC_ID (A-AH33) + ricalibrazione NET_H/NET_P**, e i free cross-thread finiscono in abandoned (visit_abandoned per heap c'è).

**Q2 — Sì, serve il tag.** `finish()` (vm/mod.rs:2305) restituisce UN u64: il clamp è dichiarato "indistinguishable by design", ma un bracket di produzione che RIDE il clamp (teardown differiti cross-richiesta) pubblica net=0 legale e MUTO — sulle calibrazioni il dente morde (NET_H≠0), su una riga NON-calibrata no. Il flag costa un confronto pre-clamp.

**Q3 — Decomposizione: fatto nominabile Δw2−Δw3 = 8.585.216 B ≈ 8,19 MiB una-tantum alla PRIMA transizione multi-thread** (runtime + strutture cross-thread first-touch); il resto ~14,3 MiB/worker = candidati: first-touch per-theap (≈30 size-class × pagine v3), eager-commit d'arena, stack toccato (2 MiB default Rust, non 8), retention (purge refutato come driver dello SPREAD, non della QUOTA). Protocollo GIUSTO (già nominato nei residui, lo formalizzo): steady-state, non peak d'avvio — è la forma della derivazione KL-85-2 → **A-DL38**.

**Q4 — Il limite NON è PIPE_BUF** (vale solo per pipe). Su XNU/APFS il vnode serializza `write(2)` (range-lock): un singolo write non interleava a qualunque taglia di riga census. Il failure-mode reale è il **partial write** (EINTR/ENOSPC/…): `write_all` allora emette un SECONDO write → garbling ritorna. Non c'è soglia da pinnare sulla taglia (le righe sono ≪4 KiB); si pinna la GRAMMATICA: una riga valida = un solo `^pid=\d+ ` e `\n` terminale.

**Emendamenti**
- **A-DL36**: `finish()` espone (net, clamped) o la riga census pubblica `da= df= clamped=` accanto a `net=`; verdict: riga clamped mai citabile come costo-zero.
- **A-DL37**: estendere il fix census_line a `phys_window_dump`: format-in-buffer + `write_all` singolo per OGNI riga (mi_proc/ctx/mi_bin).
- **A-DL38** (sostituisce la forma VW123): metà fisica = Δcommitted di processo fra W e W+1 a steady (N=100/worker), quiesce + `mi_theap_collect(force)` su ogni worker, snapshot win=0; R≥5 min-of-R; chiusura `7.343.135+own = Δcommitted + slack(committed−used) + untouched ±5%`; bracci `MIMALLOC_ARENA_EAGER_COMMIT` e stack-size pinnato in-band per sottrarre i candidati per NOME.
- **A-DL39**: design (solo design) della via `mi_heap_new` per-worker per lo split vero; gate: bump ALLOC_ID + ricalibrazione, o in alternativa export vendored di `theap->stats` — delibera del Concilio prima di toccare l'allocatore.

**Kill-switch**
- **KL-88-1**: cifra per-thread da heap-visit sul default heap condiviso ⇒ VOID (il dente `heap=` è il giudice).
- **KL-88-2**: righe multi-riga phys_window_dump da run W≥2 pre-A-DL37 citate senza passare la grammatica `^pid=\d+ …\n` a match singolo ⇒ escluse dal corpus.
- **KL-88-3**: net=0 di produzione citato come "costo zero" senza flag clamped ⇒ riclassificato UNKNOWN.

*Daan Leijen — il puntatore identico non era un bug: era l'allocatore che diceva la verità sul proprio design; ora tocca a noi scegliere un design che permetta la domanda.*

---

# VERBALE — Bak, sedia 5, Concilio WP-88

**VERDETTO: PASS CON TRE REFUTAZIONI DAI RAW.** VCAL/VINV/VBURST/VW500 ricomputati e confermati al byte. Ma VOVL è **FALSO-DAI-RAW** e VABBA/VW123 sono giudicati sulla metrica che questo progetto stesso ha bollato come bugiarda (max RSS, non peak footprint).

**Q1 — VINV ricomputato**: `m86.inv.memcensus`: pad ord1 net=7.803.281 ✓, hello ord2 net=6.842 ✓; algebra 7.349.977−7.803.281+460.146=6.842 ✓. Il confound NON ha morso per ragione **strutturale, non fortuita**: floor_inc di hello crolla da 997.878 (ord1, cal-h) a 1.040 (ord2) — Δ=**996.838 ESATTO**, il mio confound. La coppia è ANNIDATA (pad 377 fns ⊇ hello 317): chi corre primo paga l'intero set condiviso, in entrambi gli ordini — la simmetria è del NIDO. Su una coppia a overlap parziale morderebbe. → la promozione va SCOPED alla coppia annidata (A-BB53: contro-prova su coppia disgiunta).

**Q2 — VABBA ricomputato**: 18 peak dai `.log` combaciano; spread(A)=252.772.352−231.456.768=21.315.584 ✓, spread(B)=249.806.848−235.667.456=14.139.392 ✓; ratio 1,51 → INCONCLUSIVE per la mia soglia. **Ma**: (a) lo spread A è UN outlier (senza r4: 8,72 vs 5,87 MiB — max−min a R=8 è statistica fragile, colpa MIA); (b) sul **peak memory footprint** (metrica vincolante) i bracci si SEPARANO COMPLETAMENTE: A r2..r9 media 209.670.819, max 213.828.232 < B min 226.132.640, media 230.597.283 — purge=0 abbassa il fisico di **~20,9 MB con separazione 8/8**, e spread(A)=10.371.024 < spread(B)=14.155.800. Il "purge refutato, anzi spread maggiore" è un artefatto della metrica RSS. Protocollo WP-88 (A-BB51/52): (1) ri-giudizio su footprint; (2) diff PER-REGIONE dei vmmap V1/V2 già in archivio (MALLOC arena → first-touch; stack → ordine spawn); (3) braccio pre-touch (una scrittura per pagina committed) e braccio spawn-order deterministico (barrier), ABBA R≥16, giudice **Levene/varianza**, mai max−min.

**Q3 — VW123**: i peak 56.033.280/79.659.008/94.699.520 sono **max RSS**; sul footprint: 38.552.056/55.263.784/78.561.808 → Δ=16.711.728 (15,94 MiB) e 23.298.024 (22,22 MiB). Le due metriche INVERTONO il trend (RSS decresce, footprint cresce) ⇒ il Δpeak d'avvio non misura una quantità per-worker stabile: conflaziona transient di lowering concorrente (~7,3 MB/worker), commit del segmento heap (mi_bin_thr_sum committed≈16,2 MB), stack first-touch, retained (~3,6 MB). Sì, il protocollo era un ALTRO: **slope di committed steady-state** — W∈{1..4}, richieste SEQUENZIALI, drain+purge, snapshot win=0 a teardown, regressione; 3.605.572±5% vale solo lì.

**Q4 — A-BB47: il fixture nuovo NON serve, serve il fix del qualificatore.** Refutazione principale di questo verbale: ho ri-eseguito l'awk di qualificazione sui 10 raw — gli span si INTERSECANO in 10/10 (es. a2: h=[0,13756], p=[2,14395], tid distinti), hello si abbassa in **~13,8 ms, NON in µs** (i µs sono di inv ord2). Il qualificatore fallisce per **confronto STRINGA**: dopo `sub()` t0/t1 perdono lo status strnum e `"4" < "13300"` è FALSO come stringhe. E i net sotto overlap sono già agli atti: pad=15.153.408 = NET_H+NET_P+150 (la finestra process-counters ha inghiottito il lowering altrui INTERO), hello 57,1–57,4 MB (a8: 17,9 MB) — **il per-thread sotto concorrenza è già REFUTATO dai raw**, non CANDIDATO. Fixture-pair comunque ordinato (A-BB53): due pad DISGIUNTI (prefissi simbolo unici, 377 fns, ~14 ms), barrier-released; predizione ex-ante: con finestra per-thread net=cal al byte per lato; con process-window netX≈calX+calY (firma già osservata).

**Emendamenti**: **A-BB49** fix qualificatore (`t0=$i+0`) + re-run OVL sui fixture esistenti; **A-BB50** net window PER-THREAD (contatori thread-local); **A-BB51** VABBA ri-giudicato su footprint, vmmap per-regione; **A-BB52** VW123→slope steady-state; **A-BB53** coppia disgiunta per scoping della scomposizione.

**Kill-switch**: **KB-88-1** net citato da run con span intersecanti sotto net_window=process-counters ⇒ cifra VOID; **KB-88-2** peak/spread/pin citato senza NOME della metrica (footprint vincolante) ⇒ VOID; **KB-88-3** max−min con R<16 come giudice di attribuzione ⇒ VOID (varianza/Levene o diff per-regione).

— Bak, sedia 5

---

# VERBALE — Concilio WP-88, sedia 6 (Pedersen)

**VERDETTO: CON EMENDAMENTI** — A-PP33/34/35/36 eseguite nella lettera; il phantom che ho nominato in WP-87 è escluso nel suo caso esecutivo, ma il dente è **solo-somma**, vive **solo nell'arm mem-census**, e il tier A dello sweep ricade ESATTAMENTE nella classe KS-PP-87-1 un livello più su.

**Q1 — dente dispatch: il phantom nominato è escluso; due residui trovati.** Verificato: `dispatched+=1` al pickup, prima di ogni esito (worker_pool.rs:408), riga `tag=worker_dispatch thr= count=` nel teardown (:325-329); parse86 esige `dsum==ndispexp` (verdict86.sh:77-79). Un abort PRE-handler non consuma round-robin (dispatch è l'unico fetch_add su next_worker, main.rs:280→worker_pool.rs:504-507): il phantom "consuma senza righe" è ora strutturalmente impossibile; un abort a rx.await è contato ⇒ sum≠atteso ⇒ FAIL. Il retry-compensato non è costruibile da QUESTO driver (memrun86 non ritenta e `ndisp=$n` conta i TENTATIVI). **Residuo (a)**: `%disp` per-thread è raccolta ma MAI confrontata — il giudizio è la sola somma; l'identità thread è delegata alle righe unitcache (che sono per-ENTRY, non per-richiesta: una ri-esecuzione HIT non lascia seconda riga). **Residuo (b), il caso che ancora passa**: le fasi **W123 e ABBA girano sull'arm UNION, dove la riga dispatch non esiste** (emissione cfg mem-census; il contatore c'è, :393, incondizionato). VW123 interpreta "one hello per worker" via round-robin **senza alcun dente** — con PORT=8296 fisso, un client esterno sposta il mapping a due-su-uno e nulla lo vede; la NAMED-DEVIATION 22,53/14,34 MiB portata a WP-88 poggia su un mapping NON verificato. **(c)** su send-Err `next_worker` non è ripristinato (:527-541) — ghost dichiarato, marginale (solo teardown).

**Q2 — a_pp33: trasferisce per la SUPERFICIE, non per il PATH.** Il thread di test usa lo stesso `std::thread::spawn` default-stack della produzione (:283 vs :1255) e `registry()` in-thread; la cache è thread-local e l'anti-vacuità `uc_entry_count()==0` (:1264-1268) pinna la precondizione essenziale ⇒ il claim regge per la superficie `execute_with_retain`-come-req1. MA il test BYPASSA il pickup-path: un futuro inserter in `worker_loop` fra recv e execute_request (:407-440) resterebbe fuori copertura — "vera req1 del worker di produzione" è un'approssimazione DA DICHIARARE. La forma piena è gratis: `WorkerPool::new(ctx,1)` + dispatch del fatale come PRIMO task.

**Q3 — /__reqns: PULITO.** Entrambi i rami ritornano dal router (main.rs:214 unarmed, :234 armed); tocca solo `worker_count()` (= senders.len(), nessuna mutazione di next_worker). Nessun fallback dispatcherebbe: il dispatch vive a :280 DOPO un fs::read riuscito; un probe mal-ancorato (`/__reqns/`) cade nel 404 a :253, ancora senza dispatch (servirebbe un file omonimo nel docroot — assente nelle fixtures). Unico inverso nominato: `ends_with` OSCUREREBBE una fixture chiamata `__reqns` — hazard di shadowing, non di dispatch.

**Q4 — buco CONFERMATO.** gate-lever-pins.sh:896: `grep -q 'reqns-guard[.]pl'` — un commento `# TODO reqns-guard.pl` soddisfa il tier A senza mai invocarlo: token-presenza come conformità, la MIA classe KS-PP-87-1 recidiva. Nessun decoy copre il caso (:909-918 testano solo il digit-guard). Aggravante: lo scan copre solo `*.sh` in wp8[4-9]/wp9x — un parser in un `.pl` sfugge.

**Emendamenti:**
- **A-PP37**: verdict esige la MAPPA per-thread dispatch == attesa (non la sola somma); ghost next_worker su send-Err dichiarato in-comment.
- **A-PP38**: arm full-path per a_pp33 (pool W=1 reale, fatale come primo task dispatched); l'arm attuale resta come unit-surface con approssimazione dichiarata.
- **A-PP39**: riga `worker_dispatch` emessa ANCHE su union a teardown (contatore già incondizionato; stderr, costo zero) — chiude il buco W123/ABBA.
- **A-PP40**: tier A ESECUTIVO — `grep -qE '^[^#]*perl[^#]*reqns-guard[.]pl'` + decoy (c) citazione-in-commento nel self-test; scan esteso ai `.pl`.

**Kill-switch:**
- **KS-PP-88-1**: tier A soddisfatto da occorrenza non-invocativa di reqns-guard.pl ⇒ sweep VACUO, eredità non provata.
- **KS-PP-88-2**: attribuzione per-worker (Δpeak, one-request-per-worker) citata da arm SENZA riga dispatch in-band ⇒ claim = envelope/assunzione di protocollo, mai attribuzione — VW123 a WP-88 porta questo tag.
- **KS-PP-88-3**: claim "vera req1" esteso al pickup-path di worker_loop senza l'arm A-PP38 ⇒ declassato a superficie-dichiarata.

— Pedersen, sedia 6.

---

# VERBALE KLABNIK — sedia 3, Concilio WP-88

**VERDETTO: CON EMENDAMENTI — A-SK41/42/43/44/45 eseguiti nella lettera (verificati negli script), ma trovo un BYPASS del checker sul fast-path stessa-rev, un taint cross-blocco, un'evasione residua delle bande, e la sovrascrittura Q4 esige una classe nuova.**

**Q1 — La forge resta possibile, e il fast-path non chiama nemmeno il checker.** I due REFUSE auto-inflitti sono sanati nel codice: riga PASS terminale via echo-non-tee (battery-86pre.sh:101-104), sed ancorato `^rev=` (battery-equivalence.sh:119-122), path git-root duale (battery-equivalence.sh:134-141). Ma: (a) il ledger È appendibile a mano — il checker greppa solo `rev= sha256=` (:139), MAI i campi matrix della riga, né che l'archivio matrix nominato sia COMMITTED con sha combaciante: forge = OUT ricostruito (15 righe `OK <name>` + PASS ancorata) → sha → .done → append+commit. Il commit è l'unica traccia. (b) PEGGIO: measure86-campaign.sh:74-76 — sul path stessa-rev la campagna greppa la PASS ancorata + `rev==HEAD` e BYPASSA battery-equivalence: né sha256(OUT) ricomputato né stamp ledgerato verificati. La campagna S-86.0 ha usato ESATTAMENTE questo path ("nessuna equivalenza"): lo stamp c259bc6 è ledgerato (battery-stamps.ledger:10) ma nessuna macchina l'ha confrontato. → **A-SK46**.

**Q2 — A-SK44/45 eseguiti: tutti i 7 blocchi sono collect-then-emit con flush gated su bfail==0** (verdict86.sh:103,131,157,207,237,268,320; ordmap generalizzata :57-62; dente diretto anche in verdict85.sh:66-68). Nessun ramo stampa PASS col PROPRIO bfail>0. Tre buchi di spirito: (a) **taint cross-blocco**: il dente VACUOUS (:100) scatta DOPO l'assegnazione di NETH/NETP (:92-93); VINV/VBURST controllano solo defined-ness (:111,:139) — "VINV PASS" greppabile da verdetto globalmente FAIL. (b) `s/\0//g` (:52,:183) RIPARA in silenzio righe con NUL invece di rifiutarle — post-fix write_all un NUL è firma di binario pre-fix. (c) driver_sha first-match-wins (:287): 18 log, zero check di uniformità. → **A-SK47**.

**Q3 — Allowlist sana ma aggirabile per distacco d'unità.** gate-measure-cifre.sh:194 esige l'unità ADIACENTE: "pin 232 ± 16 (MiB)" non matcha né la banda né il check (3); "16" è 2-cifre (fuori scope :12) e "232" è in corpus (verdict85.out stampa "232±1 MiB") ⇒ passa TUTTO. La forma ASCII "+/-16 MiB" invece è morsa dal check (3). MiB-only (:175-177) vive solo dentro le coppie verificate — le nude cadono su (3), ok; `[Bb]` accetta "Mib" (bit spacciati per byte, cosmetico). Il corpus resta INDISCRIMINATO sui raw VOID (:114-116, KG-87-2 senza enforcement macchina). → **A-SK48**.

**Q4 — Lettera rispettata, spirito a metà: serve la classe.** Nessun rm: i raw abortiti vivono a c259bc6 ("committati come VOID… saranno soprascritti", dichiarato ex-ante) e la campagna reale (45d3714) li ha rimpiazzati via `: > "$MC"` (measure86-campaign.sh:126,178) e O_TRUNC sotto gli STESSI nomi (verificato: m86.burst.memcensus 90 vs 90 righe, byte diversi). Ma: (a) la preservazione fu VOLONTARIA — un relaunch pre-commit avrebbe distrutto i raw senza traccia (rm-equivalente); (b) il label è ora AMBIGUO: `src=m86.*` (KG-87-2) denota due generazioni; (c) `: > "$OVLED"` (:174) tronca il ledger tentativi KG-87-1 al relaunch. → **A-SK49**.

**Emendamenti:**
- **A-SK46**: nessun path di campagna consuma una battery senza i denti v4 — anche stessa-rev verifica sha256(OUT)↔.done + stamp ledgerato; il checker confronta TUTTI e 4 i campi della riga ledger e pretende l'archivio matrix COMMITTED con sha combaciante.
- **A-SK47**: flag `vcal_clean` richiesto dai blocchi consumatori (mai solo defined); NUL in raw parsato = REFUSE, mai strip; uniformità driver_sha su tutti i log della fase.
- **A-SK48**: banda a scope di RIGA — ogni '±' su riga con unità di memoria nei MEASURE8[4-9] deve risolversi in banda allowlisted.
- **A-SK49**: pre-flight campagna fail-closed su raw same-label non committati in measure-out; relaunch = label con suffisso attempt (legge A-BG39 estesa dai supplement alle campagne); ledger tentativi append-only.

**Kill-switch:**
- **KS-SK-88-1**: battery consumata via path che salta ledger/sha ⇒ precondizione VOID, campagna da ri-verificare.
- **KS-SK-88-2**: PASS di blocco a valle flushata con blocco-sorgente delle calibrazioni in fail ⇒ verdetto del blocco VOID.
- **KS-SK-88-3**: riga giudicata proveniente da raw con byte NUL strippati ⇒ riga VOID.
- **KS-SK-88-4**: raw sovrascritto same-label senza generazione precedente a HEAD, o citazione `src=` di label multi-generazione senza rev ⇒ raw/citazione VOID.

*— Klabnik, sedia 3.*

---

# VERBALE — Matsakis, sedia 2, Concilio WP-88

**VERDETTO: CON EMENDAMENTI.** A-MS32/33/35 eseguite a regola e verificate nel codice; A-MS34 eseguita nella lettera ma **REFUTATA A METÀ**: il flag è un bool globale e la finestra di falso-negativo ESISTE — provata dagli stessi atti di S-86.0. Un dente nuovo (Q2, Q4) è vivo ma non ancora fatto mordere. (Repo: `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust`.)

**Q1 — TIENE: la promotion è morta.** `vm_gate_probe(anchor: &mut ())` (vm/mod.rs:576); call-site con anchor locale `let mut probe_anchor = ()` (worker_pool.rs:974-975, 1441-1442). Costruzione mentale del token 'static: (a) `vm_gate_probe(&mut ())` → E0716 — la promotion di `&mut` a temporary non esiste in fn body (solo `&mut []` in contesti const/static, e non è `()`); (b) `static mut` / `&raw mut` → unsafe, fuori perimetro safe; (c) banking via `thread::spawn` → morto per E0277 (A-MS33, `PhantomData<(&'gate (), *mut ())>` a :525). Residuo irriducibile: la **famiglia leak** — ma il doc (:569) nomina solo `Box::leak`; `&mut Vec::leak(vec![()])[0]` è la stessa classe con grafia diversa, e il token leaked resta bancabile SAME-thread (thread_local). La belt pinna firma e decoy `&()` (gate-lever-pins.sh:290-304) ✓.

**Q2 — TIENE NEL PERIMETRO, ma il dente non è pinnato.** I doctest girano perché `VmGate` è ri-esportato pub (lib.rs:70) e non `#[doc(hidden)]`. Catena PER NOME: battery-86pre.sh:80 gate **`parity-full`** → gate-parity-83p1.sh:51 `cargo test --release` (workspace, senza `--lib` né filtro) → doc-target incluso. **Prova empirica dal log della battery S-86.0** (`/Volumes/Extreme Pro/Claude/wp83-battery-out/p1-parity/cargo-test.log`, mtime 2 ago 11:14): sezione `Doc-tests php_runtime`, `test … vm::gate::VmGate (line 512/520) - compile fail ... ok`, **2 passed**. MA: nessun gate CONTA i doctest — se il re-export cadesse o VmGate diventasse `doc(hidden)`, rustdoc collezionerebbe ZERO doctest e `parity-full` passerebbe con rc=0: il dente morirebbe in silenzio (stessa classe del "15/15 cablato" di Klabnik).

**Q3 — REFUTATA A METÀ.** Ordering: SeqCst basta e avanza — il hook (main.rs:484-497) gira SUL thread che panica, quindi per il panic del probe stesso la visibilità è di program-order (:317 store→:319 catch_unwind, nessun codice interposto). Il buco è il **TIPO, non l'ordering**: `CENSUS_PROBE_ACTIVE` è UN bool globale (worker_pool.rs:57) e i probe di teardown girano su OGNI worker thread concorrentemente — l'overlap è FATTO EMPIRICO di S-86.0 (scoperta 2: due census_line di teardown W=2 garblate MID-LINE). Sequenza: A store(true) → B store(true) → A store(false) → **il probe di B panica col flag false** → il hook stampa la riga WORKER per un panic dell'STRUMENTO. Speculare il falso-positivo: un panic del dispatcher durante una finestra probe legge true → un bug del misurando liquidato come strumento. KS-MS-87-3 è formalmente sollevata ma l'attribuzione "via flag" mente sotto W≥2.

**Q4 — TIENE.** Guardia in `unit_cache_key_present` (vm/mod.rs:16007-16011). Unico caller prod: :7083 (miss-taxonomy, corsia acquire, dopo un get — mai dentro put). La finestra armata (put :16511 → Drop :16493) emette solo uc_stat/uc_log/drop di CachedUnit a dati piatti: **nessun path odierno chiama key_present con guardia legittimamente armata dallo stesso thread** ⇒ nessun falso positivo. Ma: `a_ms35` come test **non esiste in crates/** — il bite a :19451 copre solo `unit_cache_get`; il dente key_present non è mai stato fatto mordere.

**EMENDAMENTI**
- **A-MS36**: flag probe **per-thread** (thread_local `Cell<bool>`; il hook gira sul thread panicante ⇒ attribuzione esatta), globale al più come belt; chiude falso-negativo E falso-positivo.
- **A-MS37**: `parity-full` pinna nel cargo-test.log `vm::gate::VmGate .* compile fail ... ok` **==2** (il dente doctest diventa contato, non presunto).
- **A-MS38**: bite test key_present sotto guardia armata, specchio di :19451-19459.
- **A-MS39**: doc :569 emendato — residuo = famiglia **leak** (Box/Vec/slice), non la sola grafia `Box::leak`; nominare il banking same-thread.

**KILL-SWITCH**
- **KS-MS-88-1**: riga di attribuzione probe/worker citata da run W≥2 con flag ancora bool globale ⇒ attribuzione VOID (estende KS-MS-87-3).
- **KS-MS-88-2**: parity-full PASS con conteggio doctest VmGate ≠2 (o assente) ⇒ sigillo !Send/!Sync ADVISORY, A-MS33 non provata.
- **KS-MS-88-3**: nuovo caller di key_present in fase di emissione senza delibera ⇒ classe A-TH35 riaperta, battery FAIL atteso dal bite A-MS38.

*Niko Matsakis — un bool condiviso fra N finestre è un testimone che ricorda solo l'ultima voce che ha sentito: il flag giusto abita il thread, come il panic.*

---

# VERBALE — B. Gregg, sedia 9, Concilio WP-88

**VERDETTO: cifre CONFERMATE a ricomputo indipendente; catena dell'evidenza DIFETTOSA in tre punti nominati. PASS CON RISERVE.**

**Q1 — Ricostruibile solo per archeologia git, e la narrativa NON coincide coi raw.** Le tre battery sono ledgerate pulite (3818edd→stamp f463813; 9d50b47→stamp 59eaa2e; c259bc6→stamp in 45d3714). La campagna abortita no: i raw VOID stanno a c259bc6 e 45d3714 li SOVRASCRIVE sotto filename IDENTICI — nel working tree la campagna abortita non esiste più; la VOID-ness vive solo nel commit message. In-band: zero marker (m86.cal-h/cal-p/inv.memcensus committati VUOTI, 0 B). Peggio: i summary VOID registrano `FAIL: feature-matrix git=9d50b47 != HEAD 70824ea (KG-81-2)` — la causa documentata nei raw è la matrix stale, non (solo) il bug bash-3.2 del commit message. E `m86.done` è presente ANCHE nel commit VOID (fail() incrementa senza uscire → touch finale comunque), mentre verdict86.sh non consuma affatto m86.done: marker decorativo e ambiguo. → **A-BG43**: ledger di campagna APPEND-only (`attempt=N git= esito= raw…`) + attempt nel filename; mai riusare nomi tra tentativi.

**Q2 — Due violazioni trovate.** Conformi: MEASURE86 Identità (r.22-24) ed envelope (r.87-92: entrambi gli sha + R=9/R=9). Violazioni: (a) **verdict86.out r.18 enuncia A-BG41 mentre la viola** — nomina solo 699db00a; 54717a9a è assente dall'INTERO file macchina, e la r.4 confronta "vs WP-85 record" senza sha né R su alcun lato; (b) MEASURE86 r.31-32 "terza campagna consecutiva sui record WP-83/84/85": i lati 83/84 non hanno sha in nessun documento (il lato-84 fe6983d8 sta solo nel mio verbale WP-87). Sano a ricomputo: il delta driver è il solo parametro purge in measure78.sh, fuori dalla finestra VCAL — inerzia favorevole, ma la forma va sanata.

**Q3 — Conforme alla LETTERA, violata la SOSTANZA.** Il ledger ha 10 righe `attempt=N … raw kept` ✓ (KG-87-1). Ma `: > "$OVLED"` (measure86-campaign.sh r.174) tronca, e il ledger VOID a c259bc6 è IDENTICO: stesse 10 righe, stessi filename a1..a10. Venti tentativi reali, dieci visibili; il "raw kept" del ledger VOID punta a file che oggi contengono ALTRI dati — "tutti tenuti" (A-BG39) è falso nel tree. Stesso morbo di Q1: il riuso dei nomi. Rientra in A-BG43.

**Q4 — Nessun drift, ma l'ordine NON è nel repo.** Sequenza reale (dai soli mtime — NON versionati: dopo un clone l'interleave è indimostrabile; i log/summary non hanno mono_ms/epoch): A1 B1 B2 A2 | A3 B3 B4 A4 | A5 B5 B6 A6 | A7 B7 B8 A8 | A9 B9 = [ABBA]×4+AB confermato. Bilanciamento: pos media A=9,44 vs B=9,56 — la coda AB pesa 0,11 posizioni, bias trascurabile. Ricomputo ordine↔peak: Pearson 0,032, Spearman −0,055 (per braccio: A −0,065, B +0,163); metà1 228,74 vs metà2 228,48 MiB. Spread riconciliati ESATTI (A r2..r9 = 21.315.584 B; B = 14.139.392 B). Caveat: campagna intera in **62 s** (cadenza 3,4-3,5 s/run) — "no drift" vale a QUESTA scala; un drift termico lento non aveva finestra per esistere. E il label in-band `idle_secs=10` è falso in modo axum: IDLE_SECS è consumato solo in census (measure78.sh r.279-282).

**Emendamenti**: **A-BG43** (sopra); **A-BG44** — ogni summary/log porta `seq=` globale + epoch in-band (l'ordine appartiene al contenuto versionato, non ai mtime) e il verdict.out nomina ENTRAMBI i lati d'ogni confronto; **A-BG45** — label di protocollo stampati solo se armati (`idle_secs=n/a` in axum).

**Kill-switch**: **KG-88-1** — raw sovrascritto sotto lo stesso nome da un tentativo successivo ⇒ ENTRAMBE le versioni VOID (il nome non identifica più). **KG-88-2** — claim d'interleave/ordine non ricostruibile dal contenuto versionato ⇒ claim VOID. **KG-88-3** — marker `.done` scritto con FAILS>0, o non consumato dal verdict ⇒ marker VOID e il verdict deve gate-are l'esito campagna.

*— B. Gregg, sedia 9*

---

# VERBALE STOGOV — sedia 8, Concilio WP-88 (Zend/opcache, semantica engine, unit cache)

**VERDETTO: PASS CON EMENDAMENTI — un pin monco (Q1), un'entry di catalogo con ricetta d'innesco REFUTATA sull'oracle vivo (Q2), un dente mancante nel template (Q4).**

**Q1 — putord NON basta: il pin morde sulla PRIMA coppia e nessuno legge putord in campagna.** Verificato: `UC_PUT_ORD` thread-local + putord in-band su `main_evicted`/`evict fp` (vm/mod.rs:16477, 16618, 16625); a_ds26 arma adiacenza+same-putord ma via `position()` sulla PRIMA riga (:19404-19423) e la fixture produce UNA sola coppia. Un flush batched che preserva la prima coppia e riordina dalla seconda passa a_ds26 E passa le campagne: nessun dente di verdict/battery parsa putord nei log armati — "detectable" (:15797) non è "detected". In più le righe `supersede` NON portano putord. → A-DS38.

**Q2 — QUARTO e QUINTO observable TROVATI, ma la ricetta d'innesco di §3.3-ter è REFUTATA.** Sull'oracle vivo (8.5.7, opcache caricato e verificato attivo): (a) `class C {}` semplice e parent-EARLIER sono early-bound in ENTRAMBI i bracci (class_exists pre-decl = true anche a opcache OFF) — §3.3-ter come scritto è falso per queste forme; (b) con solo `opcache.enable_cli=1` (SHM, prima esecuzione) i tre observable NON si invertono (parent-later: false/false; fatal LSP dopo l'output in entrambi); (c) l'inversione appare SOLO col persist pass — `opcache.file_cache_only=1`, già al run1: class_exists true, fatal pre-output. Il braccio che phpr riproduce è il braccio PERSIST (≡ richiesta calda FPM), non "enable_cli". Le fixture WP-87 non sono in repo: entry oracle-pinned SENZA ancora. Nuovi observable divergenti verificati (parent-later, phpr vs CLI-oracle): **4° = costante di classe pre-decl** (`C::K` → oracle Error "Class not found", phpr/persist → `7`), **5° = get_declared_classes() pre-decl** (false vs true). **Redeclare NON diverge**: fatal identico (messaggio+timing) su OFF/persist/phpr, exit 255. Confermato dal vivo §3.3-quater: la fixture covariance stampa "out" ed esce 0 su phpr, fatala su entrambi i bracci oracle.

**Q3 — perimetro A-DS35, fase 1 (spec, non idea).** DENTRO: metodi non-privati su extends/implements/abstract (costruttori solo vs interface/abstract-ctor): return covariance (aggiunta su parent untyped OK), param contravariance (widening/rimozione OK, aggiunta/narrowing fatal, by-ref esatto, required non cresce, extra solo optional, variadic assorbe), static/self/parent sostituiti, union/nullable subset, static-vs-instance, visibilità non riducibile, final non overridabile; property typed = INVARIANZA esatta + readonly. FUORI (per NOME, differite): costanti tipizzate covarianti 8.3, DNF/intersection oltre le catene nominali, enum edge. TIMING: fatal al BIND nella registry = braccio persist (pre-output, messaggio Zend byte-fedele). Fixture-oracle che inchiodano: tC (return, già viva) + un negativo per ciascuna regola + 4 positivi (covariant return, contravariant param, static return, union subset), tutti pinnati stdout+exit sul braccio persist; gate ORM/http-kernel OBBLIGATORIO (ricetta a memoria).

**Q4 — template VA: un dente mancante.** Presenti: identity, vitalità in-band (dispatch+main_probe), MEASURED ex-ante, collect-then-emit, dente ord (A-SK45), deflazione, burst, raws, cifre. MANCA il dente di quiescenza put-path: una finestra steady con `main_evicted>0`/`supersede>0` muove il retained a metà finestra senza dichiararlo, e il template non esige il pair-checker putord sui log armati (KS-DS-87-3 resta senza difesa runtime). → A-DS39.

**Emendamenti**: **A-DS38** — fixture a DUE evizioni con invariante su TUTTE le coppie (loop, non first-position) + checker condiviso stile reqns-guard (ogni main_evicted ⇒ riga successiva evict-fp same-putord) cablato in verdict/battery + putord sulle righe supersede. **A-DS39** — dente template VA: conteggio evizioni/supersede in-band dichiarato + pair-checker obbligatorio su log armati. **A-DS40** — emendare §3.3-ter: innesco = PERSIST (file_cache/FPM warm), non enable_cli; precisare che plain/parent-earlier hoistano in ENTRAMBI i bracci; aggiungere 4°/5° observable; committare le fixture in repo. **A-DS41** — la spec Q3 entra in [[php-rust-todo-master]] come contratto di A-DS35.

**Kill-switch**: **KS-DS-88-1** — log armato con main_evicted senza pass del pair-checker su TUTTE le coppie ⇒ ordine ADVISORY, raw non verdict-grade. **KS-DS-88-2** — entry di catalogo oracle-pinned senza fixture committata o citata con innesco `enable_cli` ⇒ entry UNANCHORED, non opponibile nei gate. **KS-DS-88-3** — merge di A-DS35 senza battery fixture-oracle persist-branch (negativi+positivi per NOME) e senza gate ORM/hk verde ⇒ REJECT.

---

## ⚖️ SINTESI DI CONVERGENZA (compilata a valle dei 9 verbali)

**Verdetto complessivo: 9× CONCORDO/PASS CON EMENDAMENTI — nessuna
opposizione.** Le CIFRE della campagna sono confermate a ricomputo
indipendente (Bak e Gregg al byte: VINV, VBURST, VW500, i 18 peak ABBA,
gli spread; Matsakis ha verificato i doctest NEL log della battery;
Stogov ha interrogato l'oracle vivo). I colpi della tornata:

### Refutazioni convergenti (≥2 sedie o dai raw)

1. **🔴 VOVL era FALSO-DAI-RAW (Bak, la refutazione capitale)**: il
   qualificatore overlap confrontava STRINGHE (awk `sub()` uccide lo
   status strnum: `"4"<"13300"` è falso) — gli span si INTERSECANO in
   10/10 tentativi (hello si abbassa in ~13,8 ms, non µs) e i net sotto
   overlap sono già agli atti: pad = NET_H+NET_P+150 (la finestra
   process-counters ha inghiottito il lowering altrui INTERO). **Il
   per-thread sotto concorrenza NON è OPEN: è REFUTATO dai raw** — il ×W
   resta sequenziale-only (KL-87-2 confermata nella direzione dura).
   → A-BB49 (fix numerico + re-run del giudizio sui raw ESISTENTI),
   A-BB50 (net window PER-THREAD), KB-88-1.
2. **VABBA: l'INCONCLUSIVE è un artefatto della METRICA (Bak)**: su max
   RSS i bracci non discriminano, ma sul **peak footprint** (la metrica
   che questo progetto ha giurato vincolante da WP-22) si SEPARANO 8/8:
   purge=0 abbassa il fisico di ~20,9 MB e stringe lo spread. Il
   giudizio va RIFATTO dai vmmap V1/V2 già in archivio → A-BB51,
   KB-88-2 (peak/spread senza NOME della metrica = VOID), KB-88-3
   (max−min con R<16 mai giudice; varianza/Levene o diff per-regione).
3. **Riuso dei NOMI = veleno della catena dell'evidenza (Klabnik+Gregg+
   Bak convergenti)**: i raw della campagna abortita sono stati
   sovrascritti sotto filename identici; il ledger OVL è troncato al
   relaunch; la VOID-ness vive solo nel commit message → A-BG43/A-SK49
   (label con attempt= nel filename, ledger campagna append-only,
   pre-flight fail-closed su same-label non committati), KG-88-1
   (sovrascrittura same-label ⇒ ENTRAMBE le versioni VOID), KG-88-2
   (ordine non ricostruibile dal contenuto versionato ⇒ claim VOID).
4. **Il fast-path stessa-rev BYPASSA i denti v4 (Klabnik)**: la campagna
   S-86.0 ha consumato la battery senza che NESSUNA macchina
   ricomputasse sha256(OUT) né verificasse lo stamp ledgerato (il path
   equivalence li ha, il path diretto no) → A-SK46, KS-SK-88-1.
5. **Il flag probe è un bool GLOBALE sotto teardown concorrenti
   (Matsakis)**: il garbling W=2 della sessione stessa PROVA l'overlap
   delle finestre — attribuzione strumento/misurando mente sotto W≥2 →
   A-MS36 (thread_local), KS-MS-88-1.
6. **phys_window_dump scrive ancora per frammento (Leijen)**: il fix
   atomicità copre census_line ma NON le righe mi_proc/ctx/mi_bin →
   A-DL37, KL-88-2; grammatica riga (`^pid=\d+ …\n` match singolo) come
   giudice, mai PIPE_BUF (non si applica ai file).
7. **§3.3-ter: ricetta d'innesco REFUTATA sull'oracle (Stogov)**:
   l'inversione degli observable appare solo col branch PERSIST
   (file_cache/FPM warm), NON con enable_cli SHM run1; plain e
   parent-earlier hoistano in ENTRAMBI i bracci; trovati 4° (costante di
   classe pre-decl) e 5° (get_declared_classes) observable → A-DS40 +
   fixture committate, KS-DS-88-2 (entry senza ancora = UNANCHORED).
8. **VW123 poggiava su un mapping NON verificato (Pedersen)**: l'arm
   union non emette la riga dispatch — "one hello per worker" era
   un'assunzione di protocollo → A-PP39 (dispatch row anche su union),
   KS-PP-88-2 (la NAMED-DEVIATION VW123 porta il tag envelope); la
   metrica giusta era comunque un'altra (Bak/Leijen convergenti:
   **A-DL38≡A-BB52, slope di committed steady-state**, non peak d'avvio).
9. **La promozione della scomposizione va SCOPED (Bak)**: il confound
   996.838 B non ha morso per ragione STRUTTURALE (coppia ANNIDATA: pad
   377 fns ⊇ hello 317 — Δfloor_inc = 996.838 ESATTO); su coppia a
   overlap parziale morderebbe → promozione limitata alle coppie
   annidate; A-BB53 (coppia DISGIUNTA) prima di ogni generalizzazione.
10. **Sigilli: superfici residue nominate** (Hoare): finestra dei
    PARAMETRI di put (droppano dopo i locali — A-TH40), accoppiamento
    marker↔grep per convenzione (A-TH41, KH88-1), carrier-a-VALORE via
    Clone+re-put (KH88-2 — chiusura vera = A-MS27), UFCS/alias/multiline
    fuori sweep (A-TH42); doctest VmGate vivi nella battery MA non
    contati (Matsakis A-MS37, KS-MS-88-2).

### Ordine vincolante di apertura WP-87(sessione) (non rinegoziare)

1. **Ri-giudizi DAI RAW ESISTENTI (nessun run nuovo)**: A-BB49
   (qualificatore numerico + verdetto overlap dai 10 raw: atteso
   REFUTED per la finestra process-counters) · A-BB51 (VABBA su peak
   footprint dai vmmap V1/V2 archiviati) · correzione MEASURE86 (VOVL,
   VABBA, VW123 tag KS-PP-88-2) · A-BG44-forma su verdict86 (entrambi i
   driver_sha) · A-DS40 (catalogo §3.3-ter emendato + fixture
   committate).
2. **Fix strumenti**: A-DL37 (dump atomico) · A-MS36 (flag per-thread)
   · A-PP39 (dispatch row su union) · A-DL36 (clamped flag).
3. **Catena evidenza**: A-SK46 (fast-path coi denti v4) · A-BG43/A-SK49
   (attempt= nel filename, ledger append-only, pre-flight same-label) ·
   A-AH43 (path via --show-prefix) · A-AH44 (comparatore toolchain) ·
   A-AH45 (delezioni archive) · A-BG45 (label solo se armati).
4. **Sigilli v5**: A-TH40/41/42/43 · A-MS37/38/39 · A-SK47/48 ·
   A-PP37/38/40 · A-DS38/39.
5. **Misura nuova (una campagna)**: A-DL38≡A-BB52 metà fisica come
   slope di committed steady-state (W∈{1..4}, R≥5, quiesce+collect,
   chiusura ±5%) · A-BB50 design net window per-thread (precondizione
   di ogni canary concorrente futuro) · A-BB53 coppia disgiunta.
6. **Delibere**: promozione scomposizione SCOPED alle coppie annidate
   (con A-BB53 come precondizione della generalizzazione) · A-DL39
   design split per-worker heap (solo design, tocca l'allocatore:
   delibera di Concilio).
7. **ROADMAP**: A-DS35 fase 1 secondo la spec Stogov Q3 (contratto in
   [[php-rust-todo-master]] via A-DS41; KS-DS-88-3: merge solo con
   battery fixture-oracle persist-branch + gate ORM/hk verdi).
8. Deferred invariati: A-MS27, A-PP18/27, A-AH38+dry-run; KS-DS-80-3
   invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH88-1 | payload marker/token grep cambiati senza positive-control stesso-commit | noprobe ADVISORY, campagne VOID |
| KH88-2 | re-put di CachedUnit clonato (carrier a valore) pre-A-MS27 | unicità produttore declassata, attribuzioni main-lane VOID |
| KH88-3 | panic in finestra pre-closure di put con campagna proseguita | run VOID (estende KH87-3) |
| KH88-4 | putord citato cross-thread o joinato con main_put senza mapping | claim d'ordine ADVISORY |
| KS-MS-88-1 | attribuzione probe/worker da run W≥2 con flag bool globale | attribuzione VOID |
| KS-MS-88-2 | parity-full PASS con doctest VmGate ≠2/assenti | sigillo !Send/!Sync ADVISORY |
| KS-MS-88-3 | nuovo caller di key_present in emissione senza delibera | classe A-TH35 riaperta |
| KS-SK-88-1 | battery consumata via path che salta ledger/sha | precondizione VOID |
| KS-SK-88-2 | PASS a valle flushata con blocco calibrazioni in fail | verdetto del blocco VOID |
| KS-SK-88-3 | riga giudicata da raw con NUL strippati | riga VOID |
| KS-SK-88-4 | raw same-label multi-generazione senza rev nella citazione | raw/citazione VOID |
| KS-AH-88-1 | stamp accettato da sorgente git show non canonica | equivalenza VOID |
| KS-AH-88-2 | .done che nomina matrix non prodotta dal SUO matrix-gate | battery VOID |
| KB-88-1 | net citato da run con span intersecanti sotto process-counters | cifra VOID |
| KB-88-2 | peak/spread/pin senza NOME della metrica | VOID (footprint vincolante) |
| KB-88-3 | max−min con R<16 come giudice di attribuzione | VOID (varianza/Levene) |
| KS-PP-88-1 | tier A soddisfatto da occorrenza non-invocativa | sweep VACUO |
| KS-PP-88-2 | attribuzione per-worker da arm senza dispatch row in-band | claim = envelope |
| KS-PP-88-3 | "vera req1" estesa al pickup-path senza A-PP38 | declassata |
| KL-88-1 | cifra per-thread da heap-visit sul default heap condiviso | VOID |
| KL-88-2 | righe phys_window_dump multi-thread pre-A-DL37 senza grammatica | escluse dal corpus |
| KL-88-3 | net=0 citato come costo-zero senza flag clamped | UNKNOWN |
| KS-DS-88-1 | log armato con main_evicted senza pair-checker su TUTTE le coppie | ordine ADVISORY |
| KS-DS-88-2 | entry catalogo senza fixture committata o con innesco enable_cli | entry UNANCHORED |
| KS-DS-88-3 | merge A-DS35 senza battery persist-branch + ORM/hk verdi | REJECT |
| KG-88-1 | raw sovrascritto same-label | ENTRAMBE le versioni VOID |
| KG-88-2 | claim d'ordine non ricostruibile dal contenuto versionato | claim VOID |
| KG-88-3 | .done scritto con FAILS>0 o non consumato dal verdict | marker VOID |
