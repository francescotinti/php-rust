# Concilio — review delle tre nuove sedie sui dati WP-58 (2026-07-26)

> Sedie aggiunte per decisione utente (2026-07-26): **Leijen** (mimalloc),
> **Stogov** (Zend internals), **Gregg** (attribuzione). Ognuna ha
> analizzato l'intero prodotto WP-58 (sessione, gap report, design58,
> census 57/58b, codice arena). Le RICHIESTE sono integrate nel §WP-59 di
> `NEXT_SESSION_WORDPRESS.md`; questo file è l'archivio integrale.
>
> **Sintesi di convergenza (3/3)**: prima di ogni nuova leva, quota
> falsificata del ~1,1GB fuori-canale — con l'ipotesi dominante condivisa:
> ritenzione/frammentazione per-pagina mimalloc accoppiata al churn
> (~6GB/run) + strutture runtime non censite; il primo esperimento è
> zero-codice (`MIMALLOC_SHOW_STATS=1`) e può ridimensionare il mistero in
> 20 minuti. Divergenza utile: Stogov quota la **unit diet (−80..−150MB)**
> come la più grande leva GIÀ attribuita; Leijen indizia il **pool (leva B)**
> come colpevole primario della regressione full-only e ne prescrive
> l'attribuzione a binario `pool-off`; Gregg smonta l'anomalia dei 46k
> oggetti unreached in tre ipotesi ordinate con probe da 30 righe
> (sospetto n.1: sovra-conteggio contabile al choke `next_id`, condiviso
> con closure/generatori — firma: `created`==reached alla cifra).

---

## Sedia LEIJEN (allocatore mimalloc)

# Report — sedia Leijen (allocatore mimalloc), concilio WP-58→59

## 1. ANALISI — dove vive l'1,1-1,2GB non attribuito

Premessa strutturale: su arm64 mimalloc v2 organizza segmenti da 32MiB in slice/pagine da **64KiB**, ogni pagina serve UNA size-class, e una pagina torna all'OS solo quando è **completamente vuota**. Con `MIMALLOC_PURGE_DELAY=0` + `MIMALLOC_PURGE_DECOMMITS=1` (default) la ritenzione di pagine *interamente libere* è ≈0: quel canale è già chiuso dal vostro setup giudici. I metadati (descrittori pagina/segmento, embedded nei segmenti) pesano ~1-2%: ≤30MB su 1,6GB. Non è la storia.

Restano DUE soli candidati, e vanno separati perché prescrivono cure opposte:

**(a) Byte richiesti ma non censiti (lato codice, non allocatore).** Il census copre i canali valore (184MB) + unit (222,6MB). Tutto il resto che passa da `malloc` — celle `Rc<RefCell<Zval>>` sciolte, frame, `created` registry, gc-buffer, payload `Rc` degli Op, temporanei di compile, buffer I/O — è footprint *legittimamente in uso* che il proxy non vede. L'allocatore non "trattiene" nulla qui: gli è stato chiesto.

**(b) Frammentazione per-pagina da churn (lato allocatore).** Questo è il mio sospetto quantitativo principale per la quota residua. I numeri: 96B/blocco header ⇒ **682 blocchi per pagina 64KiB**; 5,46M array + 29M stringhe transitano per run negli stessi bin fini (≈5,4GB cumulati) in cui vivono i sopravvissuti. Un solo blocco vivo pinna 64KiB fisici. Se i sopravvissuti di coorti diverse sono sparsi (e lo sono: le **unit immortali da 222,6MB via `Box::leak` vengono allocate per-include LUNGO tutto il run**, interleavate col churn effimero negli stessi bin — generatore da manuale di page-pinning), il fisico per quelle classi = live/occupancy. Con 184MB live, un'occupancy media al picco del 15-18% spiega da sola ~1GB. Dopo 34M+ eventi alloc/free per processo è un'occupancy del tutto plausibile — ma è una *stima*, e l'allocatore sa dirvi il numero esatto: chiedeteglielo (sotto) invece di stimarlo per sottrazione, coerente col mandato Gregg di Fase 0 (`FOOTPRINT_CPU_ROADMAP.md` §Fase 0).

## 2. RICHIESTE per WP-59 (ordinate; ognuna falsificabile)

1. **`MIMALLOC_SHOW_STATS=1` (+`MIMALLOC_VERBOSE=1`) su una run media e una full** — zero codice, ~30 min. Output a fine processo: per-bin peak/current, commit/purge counters, malloc count. Falsifica subito: se il totale committed mimalloc ≪ picco fisico, il residuo NON vive nell'allocatore (mmap/stack/binario altrove) e (b) muore come ipotesi.
2. **Snapshot AL PICCO nel build census**: nel callback watermark già esistente (footprint > max+64MB) chiamare `mi_stats_print_out(out, arg)` + `mi_process_info(&elapsed,&user,&sys,&current_rss,&peak_rss,&current_commit,&peak_commit,&faults)`. ~1 giorno con FFI via libmimalloc-sys. Confronto: `peak_commit` vs picco fisico vs Σ census — inchioda quanto del picco è dentro mimalloc.
3. **Occupancy per size-class al picco**: sempre nel callback, `mi_heap_visit_blocks(mi_heap_get_default(), /*visit_blocks=*/false, visitor, …)` — per ogni area: used vs committed ⇒ occupancy per bin. Identità di riconciliazione pre-registrata (metodo Fase 0, ±10-15%): `Σ used_bins = census + non-censiti (a)`; `Σ (committed−used) = frammentazione (b)`; `+ metadata ≈ fisico`. Un giorno, stesso build della 2. Questa tabella È il deliverable: chiude la sottrazione-mistero.
4. **Attribuzione della regressione full-only +1..2,5%**: binario `pool-off` (feature gate o `class()`→`None` costante, NON solo `DEPTH=0` che pagherebbe comunque TLS) vs new, **full stessa-sera**. Se il delta sparisce ⇒ colpevole il pool; se resta ⇒ secondo binario con `SCAN_MAX=4`. Un asse per binario, mai i due insieme. Costo: due serate.
5. **Solo SE la 3 mostra occupancy bassa nei bin fini**: segregazione degli immortali — unit/`Box::leak` allocati su `mi_heap_new()` dedicato (in Rust: `Allocator` dedicato sui Vec di Module/Func, MAI `mi_heap_destroy` wholesale) così le pagine effimere si svuotano davvero e il purge-delay-0 le rende. Ponte a costo basso: `mi_collect(true)` al boundary test nei build census per quotare il beneficio prima di investire. Costo: multi-giorno, condizionato ai dati.

## 3. GIUDIZIO sulla block arena (leva B, `array.rs` mod pool)

**In larga parte ridondante col fast-path mimalloc.** L'alloc small di mimalloc è un pop dalla free-list *per-pagina* (~pochi ns, niente lock); il pool aggiunge su OGNI take/put: accesso `thread_local!` di std (check lazy-init a ogni accesso su macOS), borrow-flag `RefCell`, branch `class()`, check DEPTH — e su shelf saturo (DEPTH=16, inevitabile sul full con 30k test in un processo) paga tutto questo *e poi* libera comunque. Elisione di ~5-10ns di malloc pagando ~5-15ns di macchinario: saldo ≈0 o negativo — esattamente ciò che i giudici mostrano (media −0,13% flat, full +1..2,5%). Il precedente è WP-41: +0,62% di solo I-cache per uno shim nel path caldo; qui il codice extra vive nel **Drop di ogni array**, il path più caldo che esista sul full. **Candidato colpevole primario della regressione full-only: sì** (co-indiziato: scan 5-8 slot). Nota di coscienza da allocatore: ogni freelist user-level nasconde blocchi al purge e degrada la locality per-pagina di mimalloc; qui la ritenzione è bounded (<700KB) quindi il danno footprint è nullo, ma il beneficio dichiarato (−17MB) viene dalle leve A+C, non da B (il design58 stesso quota B "footprint ≈ ±0"). L'esperimento di attribuzione è la richiesta 4; se conferma, la via prevista dalla roadmap Fase 3 ("la singola versione regressiva si reverta, l'arco continua") si applica a B da sola — A e C restano.

## 4. VETI / AVVERTENZE

- **Non toccare `MIMALLOC_PURGE_DELAY` nei giudici** (né "per vincere CPU"): l'ambiente è parte del metro; ogni cambio invalida i confronti old/new.
- **Non giudicare la ritenzione col maxrss** di `/usr/bin/time -l`: è un watermark e col purge-decommit mente (lezione WP-39); per gli esperimenti allocatore usare `mi_process_info`/vmmap phys_footprint.
- **Non aggiungere altri pool/freelist sopra mimalloc** (es. Args-Vec pool, Fase 2.3) prima del verdetto della richiesta 4: si duplicherebbe il fast-path un'altra volta al buio.
- **Non curare la frammentazione con tuning di opzioni**: (committed−used) è guidato dal *pattern* (immortali interleavati con effimeri), si cura segregando gli heap — non esiste un env var che deframmenta.
- **Mai `mi_heap_destroy` su heap con oggetti Rust con `Drop`** (i distruttori non girano: unsound semanticamente); e mai `mi_collect(true)` nei binari giudice CPU — solo nei build census, etichettato.

---

## Sedia STOGOV (Zend internals)

# Report della sedia Stogov — perché 394MB ↔ 1.602MB (WP-58)

## 1. ANALISI STRUTTURALE per canale

**Prima la verità contabile, che i vostri metri esatti hanno finalmente reso possibile**: canali valore 184MB (12%) + unit 222,6MB (14%) + **~1.195MB non attribuito (74%)**. Zend sta a 394MB TOTALI. Quindi: anche portando i canali valore a densità-Zend perfetta il rapporto resta ~3,8×. Il confronto per-struttura spiega il *come* Zend è denso, ma NON spiega il 4× — il 4× vive nel non-attribuito e nella unit.

**STR — dove Zend "bara" legittimamente.** `zend_string` = 24B header (gc 8 + h 8 + len 8) + payload+NUL in **UNA** allocazione ZMM. PhpStr (zstr.rs, WP-55) = RcBox 48B (strong+weak 16 + hash 8 + Vec 24) + buffer byte **SEPARATO** = due allocazioni, ~2× overhead per istanza. Ma la vera arma è l'**interning**: ogni literal, nome di classe/metodo/proprietà/chiave è internato a compile-time, `GC_IMMUTABLE`, refcount **no-op**, una copia per contenuto per l'intero processo. Dei vostri 29M costruzioni/run: quelle da literal-load in phpr sono già Rc::clone (payload Op), quindi il churn è prodotto runtime (concat, itoa, substr, sprintf) che **esiste anche in Zend** — Zend non ha nemmeno una cache itoa oltre ai single-char `ZSTR_CHAR`. Il delta strutturale onesto su str è: (a) dedup dello standing (574k vive EOR, 62,3MB peak — quota duplicata IGNOTA, va censita), (b) 24 vs 48B di header, (c) 1 vs 2 allocazioni (= pressione doppia sull'allocatore, che paga nel non-attribuito). Il vostro `.=` in-place (WP-55) replica già `zend_string_extend`: quel canale è pari.

**ARR — il risultato che il concilio deve sentire.** Bucket Zend 32B (zval 16 + h 8 + key 8), packed 16B/slot, header 56B, `HT_MIN_SIZE=8` ⇒ un packed da 1 elemento **costa a Zend 56+128 = 184B**. phpr post-WP-58: 96B blocco + entries ⇒ dal census il packed b=1 sta a **128B/array**. **Per istanza minuscola phpr è oggi PIÙ denso di Zend.** Zend vince per POPOLAZIONE, non per layout: (a) `zend_empty_array` singleton immortale — ogni `$a=[]` alloca **zero**; voi avete 9.372 packed vuoti standing al mark e una quota ignota dei 5,46M cum; (b) **immutable arrays**: ogni literal tutto-costante è costruito UNA volta a compile-time, l'assegnazione non tocca refcount né copia — in un workload PHPUnit (config, dataProvider, default `[]`) una fetta consistente dei 5,46M/2,6GB di churn in Zend **non viene mai costruita**. L'istogramma (22k packed 1-2 el, 21k hashed 1-4 el) è esattamente la popolazione che l'immutable-literal falcia.

**OBJ.** `zend_object` = 56B + props dichiarate **inline** come slot zval (16B); la HT delle properties si materializza solo on-demand. phpr: header + `Props` in allocazione separata. 56,1MB / 67,8k vivi ≈ 262B/obj medio vs Zend ~56+16·n — fattore ~1,5-2× per istanza, canale piccolo (3,7%).

**UNIT — 222,6MB, il più grande blocco ATTRIBUITO.** Zend: opline 32B, literals = zval che puntano a stringhe **internate condivise tra tutti gli op_array**, class entry compatte. 222,6MB/2046 unit ≈ 109KB/unit: Op enum + payload Rc con literal per-unit duplicati. Riferimento Zend plausibile per lo stesso corpus: 40-70MB. Banda −80..−150MB.

**ZMM e l'equità del workload.** Attenzione al mito: in QUESTO confronto (30k test in un processo, niente teardown per-request) **Zend NON sta usando la sua arma migliore** — il reset wholesale ZMM non scatta mai. Il 394MB è Zend "in modalità sfavorita" e vince comunque 4×: quindi il gap NON è "manca l'arena per-request", è **densità per-istanza + churn evitato alla fonte + disciplina dei bin ZMM sotto churn** (chunk 2MB, size-class senza header per-alloc, riuso immediato delle free-list ⇒ frammentazione bassa con 2,8GB+2,6GB di churn). Il sospetto primario per il vostro 1,1GB: retention/frammentazione mimalloc accoppiata al churn (una viva per pagina blocca il purge anche con `PURGE_DELAY=0`) + strutture runtime non censite (frame, IC, cache, slack dei Vec).

## 2. RICHIESTE per WP-59 (ordinate)

1. **Attribuzione di 2ª generazione del ~1,2GB fuori-canale** (già Ob.1 candidato in sessione — la sedia lo rende prerequisito assoluto): vmmap per regione + `MIMALLOC_SHOW_STATS` per size-class (reserved vs committed vs live) + census delle strutture non tracciate (frames, IC, RefCell, slack capacità). Falsificabile: riconciliazione al fisico entro ±10%. Costo: 1 sessione census. **Nessuna leva è quotabile prima della mappa.**
2. **Censimento duplicati contenuto-stringa al mark**: hash del contenuto di tutte le PhpStr vive → byte e count della quota duplicata + top-N contenuti. Falsifica ex-ante la banda interning (tetto diretto ≤62,3MB canale; misura anche il n. allocazioni risparmiate = accoppiamento col punto 1). Costo: mezza giornata nel build census.
3. **Censimento literal-array**: taggare i siti array-new e contare, dei 5,46M cum, la quota (a) `[]` vuoti, (b) tutto-costanti da literal. Falsifica la banda empty-singleton + immutable-literal (churn −X GB, CPU su malloc 4,4% attribuito). Costo: mezza giornata (pattern op-census esistente).
4. **`memory_get_peak_usage(true/false)` per-test lato oracle** (listener PHPUnit): dà il peak ZMM = "solo dati" di Zend, cioè il **target per-canale onesto** (spacca il 394 in permanent+binario vs dati). Costo: ore. Falsifica la tesi "i canali valore phpr sono già densità-Zend".
5. **Quota unit**: breakdown per-unit (ops / payload / literal / meta) + duplicazione di contenuto literal TRA unit (quota internabile cross-unit). Falsifica la banda unit-diet −80..−150MB. Costo: ore-mezza giornata.

## 3. GIUDIZIO — prossima leva grande

Sequenza: **prima la mappa (richiesta 1), poi la unit diet, e l'interning solo col censimento in mano.** Motivazione quotata: (i) la tranche arena appena chiusa ha reso −17MB su un canale da 65MB — i canali valore sono un giacimento esaurito al 12%; (ii) la **unit (222,6MB) è l'unico blocco attribuito con riferimento Zend noto 3-5× più denso** ⇒ banda onesta −80..−150MB = 5-9% del fisico, la più grande OGGI quotabile sui byte veri; (iii) l'**interning+immutable-literals** è la leva con cui Zend vince davvero, ma sul vostro metro il suo effetto DIRETTO è piccolo (tetto 62MB str, dedup ignota) — il suo valore vero è uccidere il churn alla fonte (29M str + 5,46M arr) e quindi la quota di 1,1GB che l'attribuzione dovesse assegnare a frammentazione da churn, più CPU (malloc 4,4%, cache). Scommessa ex-ante della sedia: se la richiesta 1 attribuisce ≥400MB a retention mimalloc churn-correlata, l'interning/immutable diventa la leva grande della Fase 4; se il grosso è standing non censito, la leva è la dieta di quelle strutture. Non firmo bande sul non-attribuito: la lezione WP-56/57 (estimatore 5,7-6×) vale doppio qui.

## 4. VETI e AVVERTENZE

- **VETO: nessun reset per-test/per-request simulato al boundary PHPUnit.** I 27,4MB di static-props sono SEMANTICA (stato condiviso voluto tra test), non garbage; un'arena-per-request romperebbe l'osservabile. Legittimo solo un purge-hint allocatore tra test (costa CPU, va A/B).
- **Interning — tre trappole di parità**: (a) la tabella intern deve tenere **Rc forti** (o flag interned), altrimenti una internata con owner unico passa da `Rc::get_mut` nel path `.=` e viene mutata in-place con hash/contenuto stantii nella tabella; (b) `debug_zval_dump` stampa i refcount: l'oracle stampa già semantica-interned, quindi internare muove VERSO la parità, ma ogni output refcount oggi verde va ri-gated per nome; (c) mai internare nel path append.
- **Immutable-literal array**: solo tutto-scalari (niente dtor da differire); il **cursore interno è osservabile** (`reset/next/current/key`) — il singleton/immutabile condiviso deve separarsi anche su mutazione del cursore, come fa `SEPARATE_ARRAY` in Zend, o la parità si rompe in silenzio.
- **Dtor**: nessuna arena/differimento sul Drop degli oggetti (ordine dei distruttori = output osservabile, lezione frame-arena WP-29); la divergenza pre-esistente sui dtor da `unset` (WP-56) non deve allargarsi.
- **Confronti scorretti da non fare**: `memory_get_peak_usage` (solo ZMM, esclude interned/permanent/malloc) vs peak fisico — usarlo SOLO come target per-canale; RSS ps vs footprint (WP-39, MADV_FREE); e non attribuire il 4× al layout HashTable — i vostri stessi metri lo falsificano: per istanza minuscola phpr (128B) batte già Zend (184B); Zend vince **non istanziando affatto**.

---

## Sedia GREGG (attribuzione per accumulo)

# Report sedia Gregg — attribuzione del picco fisico non-attribuito (per ACCUMULO)

## 1. ANALISI — protocollo di attribuzione su macOS per questo processo

**Il difetto strutturale attuale: il watermark guarda il metro sbagliato.** Scatta ogni +128MB di *proxy interno*, che culmina a 404MB; il fisico sale a 1.602MB. Tre quarti della salita avviene in finestre che il watermark non delimita. Primo intervento: ri-chiavare il trigger sul **fisico vero** — `task_info(mach_task_self(), TASK_VM_INFO, …)` campo `phys_footprint` (costo ~1µs, si campiona da un thread a 100ms o ogni N op nel build census) — soglia +128MB di *fisico*. A ogni scatto: dump census esistente + nome del test corrente + snapshot esterno.

**Finestre → vmmap differenziale per regione.** A ogni scatto del watermark (via file-flag letto da un supervisore esterno, così il processo non si auto-sospende): `vmmap --summary $PID`, `vmmap $PID | grep -E 'VM_ALLOCATE|MALLOC|Stack|mapped file|__DATA'`, `footprint $PID` — e si diffano finestre consecutive per categoria. **Avvertenza capitale: phpr usa mimalloc come global allocator ⇒ lo heap NON appare come `MALLOC_TINY/SMALL/LARGE` ma come regioni `VM_ALLOCATE`** (mmap dirette). Le regioni `MALLOC_*` che compaiono sono le zone di sistema: cioè **solo la memoria delle lib C in FFI** (sqlite, zlib, gd, ICU). Il diff per categoria separa quindi gratis: heap Rust/mimalloc (VM_ALLOCATE) vs heap FFI (MALLOC_*) vs stack thread vs file mappati vs dyld/__DATA.

**Heap-vivo vs heap-ritenuto vs non-malloc — tre metri indipendenti:** heap-vivo = canali live-esatti (184MB) + unit (222,6MB) + strutture fuori census; heap-ritenuto = `mi_stats` (committed − used) per finestra, a costo zero; non-malloc = per differenza nel diff vmmap. **malloc_history/MallocStackLogging è CIECO su mimalloc** (zone-based): usabile così com'è solo per la quota FFI; per l'intero heap serve un build census con allocatore di sistema (lite ≈ 2-4× CPU, mai su run giudice). `footprint(1)` one-shot al picco = riconciliatore finale (dirty/compressed/swapped per categoria).

**Gate del protocollo** (stile Fase 0): per ogni finestra, Σ(Δ regioni vmmap) ≈ Δ phys_footprint entro ±10%; sul run intero, tabella {canali vivi, ritenzione mi_stats, zone FFI, non-malloc} che copre ≥90% del 1,1GB, con il *nome del test in corso* per finestra = lista di colpevoli per accumulo.

## 2. RICHIESTE per WP-59 (ordinate)

**R1 — Quota ritenzione allocatore, zero-codice (il colpevole più probabile prima di tutto).** A/B media ×6 con `MIMALLOC_SHOW_STATS=1` + one-shot `vmmap --summary $PID | grep 'Physical footprint'` all'istante del picco vs maxrss stesso run. Falsifica: se il peak scende di X, X è ritenzione; se maxrss ≫ vmmap-phys, la differenza è pagine MADV_FREE riusabili (residuo contabile, non memoria vera). Costo: 0 codice, ~20 min. **Può ridimensionare il mistero prima di spendere altro.**
**R2 — Watermark ri-chiavato sul fisico** (`task_info` phys_footprint, thread 100ms, census-only, soglia +128MB) che scrive `tag=phys_wm test=<nome> phys=<B>` e tocca un flag-file. Falsifica: le finestre coprono ≥90% della salita 0→1.602MB. Costo: mezza giornata + 1 run census.
**R3 — Supervisore vmmap+footprint differenziale sulle finestre di R2** (loop su flag-file, `vmmap --summary` + `footprint` per finestra, diff per categoria). Falsifica: Σ Δ-categorie ≈ Δ phys ±10%. Costo: script + 1 run.
**R4 — mi_stats per finestra** (`mi_stats_print_out` nel callback watermark, census-only): committed vs used sincrono coi test. Falsifica: ritenzione(R4) ≈ delta di R1 alla cifra. Costo: ~20 righe.
**R5 — condizionale**: se R3 mostra `MALLOC_*` (FFI) rilevante ⇒ MallocStackLogging=lite + `malloc_history -allBySize` per finestra; se domina VM_ALLOCATE non spiegato da R4 ⇒ build census con allocatore di sistema + stessa pipeline. Costo: 1 run 2-4 min.

## 3. ANOMALIA 46k oggetti unreached

Dati: exit live_n 67.779 vs created 22.140; walk 68.596 vs reached 22.141 (Δ=46.455). **Indizio forte: arr riconcilia ESATTO (63.436==63.436).** Se il walk avesse un buco di root, gli array posseduti dai 46k oggetti sarebbero anch'essi unreached — non lo sono. Ipotesi per probabilità:
1. **Doppio/sovra-conteggio del live nel canale obj** — la parte fissa è addebitata al choke `next_id`, e `next_id` è l'allocatore di ID condiviso tra PIÙ tabelle (closure, generatori, handle interni, lazy): se ogni ID incrementa live_n obj ma solo gli oggetti nel registry `created` sono raggiungibili dal walk, il Δ è un artefatto contabile. Coincidenza created==reached alla cifra = firma di questa ipotesi.
2. **Popolazione pinnata fuori registry** (clone, unserialize, costruzioni interne) — vivi davvero ma invisibili al walk e ai dtor di teardown (rischio parità `__destruct`).
3. **Garbage ciclico ritenuto** — sfavorita dall'arr-recon esatto.
**Probe più economico (~30 righe census-only):** a EOR, istogramma per-ClassId/kind della differenza registry−reached; discriminante 1-vs-3: `collect_cycles` forzato pre-walk (se live_n crolla di ~46k è ciclico).

## 4. VETI / AVVERTENZE (trappole Darwin)

- **maxrss conta le pagine MADV_FREE non ancora reclamate**: parte del "1.602MB" può essere pagine riusabili — verificare (R1) quanto sopravvive nel Physical footprint di vmmap allo stesso istante.
- **phys_footprint ≠ RSS** (compresse incluse vs escluse): mai sommare o diffare i due metri.
- **mimalloc è invisibile a malloc_history/leaks/Instruments-Allocations** (zone-based): su questo binario risultati vuoti o fuorvianti; lo heap vero sta in VM_ALLOCATE.
- **Effetto osservatore MallocStackLogging**: 2-4× CPU + memoria propria del log — mai su run giudice; escludere la regione "MALLOC (stack logging)" dal ledger.
- **vmmap/footprint sospendono brevemente il target**: max ~1 snapshot/s, solo build census; il campionatore ps a 20s tronca asimmetricamente i bordi (lezione WP-58).
