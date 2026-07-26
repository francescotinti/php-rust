# Concilio — review delle tre sedie sui dati WP-59 (2026-07-26, sera)

> Secondo giro del concilio esteso (Leijen/Stogov/Gregg), convocato sul
> prodotto WP-59 (la mappa del fuori-canale) per il programma WP-60.
> Le RICHIESTE sono integrate nel §WP-60 di `NEXT_SESSION_WORDPRESS.md`;
> questo file è l'archivio integrale. Contesto: ⚖️ revert leva B già
> FIRMATO dall'utente sul verbale Ob.3.
>
> **Sintesi di convergenza (3/3)**:
> 1. **Revert B per PRIMO, da solo, gate pieno + coppia full stessa-sera
>    → il binario revertato è la NUOVA baseline** (ogni delta successivo
>    sarebbe altrimenti inattribuibile). Il verdetto del revert è parità
>    per nome + footprint ±0, NON il CPU (il +0,56% atteso è dentro lo
>    spread ±0,6% — veto Gregg V3; criteri quantitativi Leijen §1b-iv).
> 2. **Contatori PRIMA delle leve** (nessuna banda si firma senza):
>    deep-size DIRETTO dei seed con split {firma, corpi, doc/attributes}
>    (sostituisce l'estrapolazione 2/3-1/3 del probe C), istogramma
>    unit-per-path con hash del contenuto (quota leak template su media
>    E full), census v2 DEEP con dedup per indirizzo (Stogov: il census
>    attuale sotto-conta 5× — rc>1 contati ZERO volte, Const piatti),
>    + istogramma per-bin delle allocazioni HIR (Leijen: il ritorno
>    fisico dipende dall'occupancy delle pagine liberate).
> 3. **Ordine leve: compile-cache (Fase 0.5, è un leak) → seed HIR
>    signature-only; A/B SEPARATI, mai cumulativi** (colpiscono lo
>    stesso canale: un cumulativo che regredisce non si spacca più).
> 4. Divergenza utile (complementare): Leijen fissa il gate FISICO
>    (phys_drop ≥ 80% dei byte droppati dal contatore; frag picco ≤ ~2×
>    29,8MB; MAI clone-poi-strip: il picco transitorio è il metro);
>    Stogov fissa il PERIMETRO semantico (seed_traits INTATTO — i corpi
>    dei trait servono al flattening; PropDecl/consts/enum_cases/
>    attributes/abstract_sigs interi; si strippa SOLO MethodDecl.body
>    +slots; chiave cache = realpath+mtime-ns+size, hash in fallback;
>    il cache-hit RI-ESEGUE il top-level e preserva i fatal redeclare);
>    Gregg ripara il METODO (finestra→test in-process, mai da stdout;
>    mappa census del FULL prima di firmare bande sul full;
>    positive-control della visita abandoned; riconciliare str al walk;
>    igiene DB negli harness; V8: niente righe "per differenza" nelle
>    tabelle — o strumento o etichetta "non attribuito").
> 5. Ammissioni a verbale: Leijen ritira la quota frammentazione
>    (stimò 15-18% di occupancy, reale ~98%); Stogov ritira l'interning
>    runtime come leva footprint e riquota la unit diet IN SU (unico
>    caso di banda allargata dopo misura); Gregg nota che il −0,56% del
>    pool è un punto dentro la banda di rumore (il revert si giustifica
>    dal rapporto costo/beneficio, non dalla cifra).

---

## Sedia LEIJEN (allocatore mimalloc)

# Report — sedia Leijen, concilio WP-59→60

## 1. ANALISI — che cosa dicono i dati WP-59 dal punto di vista dell'allocatore

### 1a. La mia ipotesi quantitativa principale è FALSIFICATA, e devo dire perché ho sbagliato

Nel giro WP-58→59 ho scritto: *"con 184MB live, un'occupancy media al picco del 15-18% spiega da sola ~1GB"*. La riconciliazione per-bin di Ob.1 dice: **frag al picco 29,8MB = 2%**; l'occupancy reale è Σused 1298,0 su Σcomm ~1327,8 = **~97,8%**. Il mio meccanismo (una viva per pagina pinna 64KiB) era corretto in astratto ma alimentato da un input falso: il "live" nei bin non era 184MB, era ~1,3GB — il non-censito VIVO (936,6MB) riempiva le pagine che io immaginavo semi-vuote. Quando gli immortali *dominano* un bin invece di esserne l'1%, l'interleaving con il churn non produce pinning: produce pagine piene. La lezione è la stessa che il concilio ha verbalizzato: la tabella per-bin uccide in 20 minuti una stima per sottrazione — e la mia stima era una stima per sottrazione con il segno giusto e la grandezza sbagliata di un ordine.

Dove invece i dati mi hanno dato ragione, alla cifra:
- **Pool (leva B)**: avevo scritto "saldo ≈0 o negativo… candidato colpevole primario: sì" e prescritto il binario `pool-off` con `class()`→None (non DEPTH=0). Esito: pool-off **−0,56%** sul full, footprint ±0, revert firmato dall'utente. Anche il co-indiziato scan 5-8 è stato correttamente processato come asse separato ed è uscito ASSOLTO (+0,66% riducendolo): un asse per binario, mai due insieme — il protocollo ha funzionato.
- **Metodo**: committed peak ≈ 93-102% del phys su entrambi i workload conferma che il mistero viveva dentro l'allocatore (richiesta 1), e la richiesta 2+3 (mi_process_info + `mi_heap_visit_blocks` nel watermark) è diventata esattamente la tabella che ha chiuso la partita. Il metro `/usr/bin/time -l` per la CPU esatta chiude il ±1,3% del campionatore.

### 1b. Che cosa resta di allocatore-rilevante ora che la frag è al 2%

**(i) Il compressor sul full (2,3G swapped al picco, maxrss 1,51 vs phys 3,89GB).** Questo è il dato allocatore-rilevante più importante rimasto. Significa: sul full lo standing compile-side non è solo *grande*, è *freddo* — macOS lo comprime perché nessuno lo tocca. Due conseguenze quantificabili: (a) il metro giudice phys_footprint è onesto (include le compresse) mentre maxrss mente di ~2,4GB — il mio veto WP-58 sul maxrss esce *rafforzato* dai dati; (b) la dieta compile-side ha un dividendo CPU nascosto non ancora quotato: ogni pagina HIR che il compressor comprime/decomprime è CPU di sistema che sparisce con la dieta. Firma attesa: dopo la dieta, il rapporto (compressed+swapped)/dirty in `footprint(1)` al picco full deve crollare, e una parte del sys-time con lui.

**(ii) Exit-frag 171MB (da 29,8 al picco).** Questa È la firma per-pagina che avevo predetto — ma appare solo *dopo i free di massa del teardown*, quando i sopravvissuti sparsi restano a pinnare pagine, e spiega il committed CURRENT 1,4GiB a processo finito (con 9,1GiB purgati cumulativi). **Non tocca il metro giudice** (peak): nessuna cura ora. Ma è un avviso preciso per la leva WP-60: *droppare i corpi HIR a metà vita del processo è un free di massa con sopravvissuti (le firme) potenzialmente interleavati* — vedi §3. E resta il promemoria: se phpr diventa mai long-running/server, questi 171MB diventano metro, e la segregazione heap degli immortali (mia richiesta 5 WP-59, rimasta condizionale e non eseguita) si riapre — anche se la dieta compile-side, rimuovendo gli immortali, ne riduce il bisogno alla fonte.

**(iii) Zona FFI: ~90-120MB dirty, DefaultMallocZone 58% frag (37,5M usati su 89M).** L'unica frammentazione *vera* a due cifre rimasta nel processo — ma è nel *system allocator* (zone-based: sqlite/zlib/gd/ICU via FFI), non in mimalloc. Ordine: 50-80MB ≈ 3-5% del fisico media. Qui, e solo qui, `malloc_history`/MallocStackLogging *funzionano* (R5 di Gregg). Priorità bassa: si quota solo se, a dieta fatta, il residuo la giustifica.

**(iv) Verifica del revert del pool — protocollo.** Attenzione a un punto che è facile sbagliare: **il binario del revert NON è phpr-pooloff59**. Quello era la release con feature `pool-off` (`class()`→None, TLS eliso); il revert rimuove `mod pool` + feature + ogni branch residuo nel Drop degli array. Atteso ≈ equivalente, ma il −0,56% non si eredita: si rimisura. Protocollo: (a) coppia full stessa-sera, CPU esatta `time -l`, banda attesa −0,3..−0,8% vs phpr-wp58 (lo spread serale è ±0,6%: se il delta esce dentro il rumore, il revert resta giustificato dal rapporto costo/beneficio — codice nel path più caldo a beneficio footprint nullo — non dalla significatività del singolo numero); (b) footprint ±0,5% (pool-off misurò 3,90 vs 3,89); (c) fail-set 88 per NOME byte-id a run33 — il pool è semanticamente invisibile, ogni diff è allarme; (d) DB reset anche nei probe (il flake wp_install di Ob.3 nasce da lì); (e) verificare nel disassembly/ispezione che il Drop di `PhpArray` torni al free mimalloc diretto senza check residui.

## 2. RICHIESTE per WP-60 (ordinate; ognuna falsificabile)

**R1 — Al contatore ex-ante compile-side (§WP-60.1) aggiungere l'istogramma size-class delle allocazioni HIR/payload-op.** Il piano prevede già bytes firma-vs-corpo e unit-per-path; io chiedo che il contatore registri anche in *quali bin* vivono quei byte (riusando `mi_heap_visit_blocks` già scritto in Ob.1c: diff per-bin con/senza il carico). Perché: il ritorno *fisico* di una dieta dipende dall'occupancy delle pagine liberate — byte droppati sparsi in pagine condivise rendono < 1:1; se l'HIR domina bin larghi/huge (probabile: `Vec<Stmt>` grandi), il ritorno è ≈ 1:1 e la banda è onesta. Falsificabile: Σ contatori riconcilia col non-censito 936,6MB (media) / 818MB (`--list-tests`) entro ±10-15%. Costo: ore sopra il lavoro già pianificato, harness Ob.1 riusato.

**R2 — Revert B col protocollo §1b-iv.** Falsificabile: CPU −0,3..−0,8%, footprint ±0,5%, 88 nomi byte-id. Costo: una serata (già deciso dall'utente; qui fisso solo i criteri di verifica).

**R3 — Gate per-bin post-dieta, pre-registrato.** Dopo ciascuna leva (0.5 e signature-only), ripetere la finestra-picco di Ob.1 con riconciliazione alla cifra. Gate: **phys_drop ≥ 80% dei byte droppati dal contatore R1**, e frag al picco che non superi ~2× i 29,8MB attuali. Se il drop fisico è < 80%, la differenza è frammentazione indotta dal free di massa (firme sopravvissute che pinnano le pagine dei corpi) e va vista nella tabella per-bin PRIMA di dichiarare la leva riuscita. Costo: 1 run census per leva, harness esistente.

**R4 — Quota del dividendo-compressor sul full (economica).** Un `footprint(1)` al picco del full prima/dopo la dieta: rapporto compressed/dirty e sys-time della coppia `time -l`. Falsificabile: la dieta deve ridurre le compresse più che proporzionalmente al dirty. Costo: zero codice, già negli harness Ob.0.

**R5 — (condizionale, dopo la dieta) MallocStackLogging=lite + `malloc_history` sulle sole zone MALLOC_*** per attribuire i 90-120MB FFI. Solo build census, mai run giudice. Costo: 1 run. Non aprirla prima: oggi è il 3-5% e la leva grande è altrove.

**R6 — Exit-frag: nessuna azione.** Registrare solo, in R3, se l'exit-frag scende da 171MB dopo la dieta (atteso: sì, perché gli immortali interleavati spariscono). È un dato gratis che conferma o smentisce il mio modello di pinning senza spendere una leva.

## 3. GIUDIZIO sulla leva grande — dieta compile-side (compile-cache keyed sul path + seed HIR signature-only)

**Direzione giusta, e lo dico da allocatore: è l'unico modo vero di ridurre il committed.** Il purge funziona già (PURGE_DELAY=0, frag 2%): non c'è nulla da "restituire all'OS" che l'OS non abbia già ripreso. Quando l'occupancy è al 98%, l'unica cura è *avere meno vivo* — e il vivo è per ~65% compile-side. Coerente anche col mio punto (a) di WP-58: "byte richiesti ma non censiti — l'allocatore non trattiene nulla, gli è stato chiesto". La mappa dice che quel ramo era quello vero.

**Compile-cache keyed sul path (Fase 0.5).** Lato allocatore è tutta in positivo: (a) chiude un leak lineare (unit.cum_n=200 su 200 re-include) — su un full da 30k test con template WP re-inclusi la banda può essere molto sopra il media, ma va quotata col contatore unit-per-path PRIMA (quante delle 2046 unit media sono re-compile? ignoto oggi); (b) una copia di Module per path = locality *migliore* (una copia calda in I/D-cache invece di N fredde) e rimozione dell'interleaving immortale/effimero che indiziavo — di riflesso, meno exit-frag (R6 lo verifica gratis); (c) niente refcount contention (processo single-thread sul path caldo). I rischi sono di parità semantica (binding di variabili/side-effect dell'include ripetuto), non di allocatore: gate pieno vincolante, non mio dominio.

**Seed HIR signature-only.** Qui i rischi allocatore/locality sono tre, tutti gestibili ex-ante:
1. **Il picco transitorio è il metro.** Se l'implementazione fa clone-completo-poi-strip dei corpi, durante il push in `seed_classes` esistono *due* copie dei corpi: lo standing scende ma il PICCO no — e il giudice misura il picco. Costruire il seed snello *direttamente* (mai materializzare il clone a corpi interi), oppure strip in-place prima del push.
2. **Free di massa con sopravvissuti interleavati.** Droppare i corpi lasciando vive le firme allocate nello stesso momento (e quindi plausibilmente nelle stesse pagine/bin) è il generatore di pinning da manuale — la versione a metà-vita dell'exit-frag 171MB. Mitigazione strutturale: il seed snello sia un *tipo nuovo* costruito ex-novo (allocazioni fresche, pagine proprie), non l'HIR originale con i `Vec<Stmt>` svuotati; così i corpi muoiono in blocco e le loro pagine si svuotano davvero. Il gate R3 (phys_drop ≥ 80% dei byte contati) falsifica proprio questo.
3. **Quota ex-ante obbligatoria** (regola "byte misurati", e lezione WP-56: l'estimatore sbagliò 5,7×): la predizione della leva si scrive in *phys per-bin* (contatore R1: bytes corpo per bin × occupancy attesa), non in byte contati. Solo dopo il contatore si firma la banda "centinaia di MB".

Il rischio di parità (eval/include che ri-lowera e ha bisogno dei corpi) è fuori dal mio dominio: gate pieno + corpus per nome, come da piano.

**Ordine che sottoscrivo**: contatore (R1, ore) → cache 0.5 (leak: si chiude prima di attribuire il resto, come da roadmap) → seed signature-only con gate R3 pre-registrato. Ogni leva col proprio A/B stessa-sera.

## 4. VETI / AVVERTENZE (vincolanti)

1. **Confermo integralmente i miei veti WP-58**, due dei quali escono rafforzati dai dati: mai toccare `MIMALLOC_PURGE_DELAY` nei giudici; **mai giudicare ritenzione/footprint col maxrss** — sul full mente di 2,4GB (1,51 vs 3,89GB, compressor); nessun nuovo pool/freelist sopra mimalloc — ora non più opinione ma verdetto sperimentale (−0,56% a beneficio footprint nullo).
2. **Il revert B si rimisura, non si eredita**: phpr-pooloff59 è un binario feature-gated, il revert è un tree diverso. Criteri in §1b-iv; footprint e fail-set nel gate, non solo la CPU.
3. **Mai clone-poi-strip a corpi interi nel seed** (il picco transitorio è il metro giudice): seed snello costruito ex-novo, allocazioni fresche separate dai corpi.
4. **La dieta si dichiara riuscita solo sul phys per-bin** (gate R3: phys_drop ≥ 80% dei byte droppati; frag picco ≤ ~2× 29,8MB). Un delta contatore senza riconciliazione fisica non è un risultato — è la mia stessa trappola di WP-58 a parti invertite.
5. **Non riaprire la frammentazione mimalloc come leva footprint** — è la mia ipotesi, falsificata dalla tabella; si riapre SOLO con una riconciliazione per-bin che dica altro. E **non "curare" l'exit-frag 171MB**: non tocca il metro peak; si osserva gratis in R6.
6. Restano operativi: mai `MIMALLOC_SHOW_STATS`/`VERBOSE` su run di cui serve il fail-set (propaga ai figli — lezione Ob.0); `mi_collect(true)` solo nei build census etichettati; mai `mi_heap_destroy` su heap con Drop Rust; malloc_history/Instruments ciechi su mimalloc (heap = regioni IOAccelerator su macOS 26, non VM_ALLOCATE) — validi solo sulle zone FFI.

*Chiudo con la nota di metodo che il concilio esige: su WP-59 la mia sedia ha azzeccato il colpevole CPU (pool) e il protocollo di misura, e ha sbagliato la quota footprint (frag 15-18% di occupancy stimata contro 98% reale) perché ha stimato per sottrazione ciò che andava misurato per-bin. La tabella di Ob.1 è ora il precedente vincolante: nessuna banda della dieta compile-side si firma prima del contatore R1.*

---

## Sedia STOGOV (Zend internals)

# Report — sedia Stogov, concilio WP-59→60

## 1. ANALISI — che cosa farebbe Zend qui, e che cosa phpr ritiene che Zend non ritiene

**Prima l'ammissione contabile.** La mia scommessa ex-ante è caduta sul secondo ramo, come pre-registrato: frammentazione 29,8MB = 2% (l'ipotesi condivisa 3/3 è morta in una finestra di misura — la riconciliazione alla cifra di Ob.1 è il lavoro metodologicamente più pulito visto in questa roadmap). Ne discendono tre revisioni esplicite delle mie posizioni WP-58:

- **Interning runtime come leva footprint: la ritiro.** Il suo tetto diretto era ≤62MB e il suo valore indiretto era condizionato a "≥400MB di frammentazione churn-correlata" — che è 30MB. Resta una leva CPU/churn di Fase 4, non di footprint.
- **La "unit diet −80..−150MB" era quotata sul metro sbagliato.** Il canale unit counted (222MB) è 1/5 del canale vero (~1,0GB): il mio riferimento Zend (40-70MB per lo stesso corpus di op_array) resta valido, ma il numeratore è quintuplicato. La leva si ridimensiona *in su* — ed è l'unico caso in questa roadmap in cui una banda si allarga dopo la misura.
- **Le mie richieste 2-3 di WP-58 (duplicati str, literal-array): declassate.** Concordo con la bozza §WP-60. Il giacimento non è lì.

**Che cosa fa Zend, strutturalmente.** In Zend l'AST ha la vita di una compilazione: `zend_compile` lo produce, `zend_ast_destroy` lo libera prima ancora che l'op_array esegua. Ciò che persiste per classe è la `zend_class_entry`: prop_info, firme, tabella metodi con op_array compatti (opline 32B, literal internati condivisi). Quando una classe tardiva fa `extends` di una precedente, il linking usa la **CE** — metadati runtime — mai il sorgente. Con opcache si aggiunge il livello: compile una volta per realpath (validato mtime/size), op_array immutabile condiviso. Senza opcache, un `include` non-`_once` ripetuto ricompila sì ogni volta, ma l'op_array del top-level viene **distrutto dopo l'esecuzione** — persistono solo funzioni/classi dichiarate (e ri-dichiararle è fatal, quindi non si accumulano).

**phpr ha invertito entrambe le frecce.** Dal codice letto:

1. `Vm.seed_classes: Vec<Rc<ClassDecl>>` + `seed_traits` + `main_hir: Option<&'m Program>` (vm/mod.rs ~1669-1691): l'immagine HIR accumulante ritenuta per seedare il lowering di eval/include. E `ClassDecl.methods: Vec<MethodDecl>` con `MethodDecl.decl: FnDecl` e `FnDecl.body: Vec<Stmt>` (hir.rs 401-432) = **l'AST completo di ogni corpo di ogni metodo, per sempre**. È l'analogo di un Zend che non chiamasse mai `zend_ast_destroy`. Il probe differenziale 3,8× classi vs 1,5× funzioni è esattamente la firma di questo: le funzioni libere pagano solo il Module compilato, le classi pagano Module + immagine HIR.
2. I Module sono `Box::leak` con lifetime `'m` (`modules: Vec<&'m Module>`): ogni re-include non-`_once` conia una unit **immortale** (probe B: cum_n=200). Zend qui alloca-e-libera; phpr alloca-e-ritiene. Non è frammentazione, è design: e infatti Ob.1 lo trova come standing vivo.

**Che cosa serve DAVVERO al seeding.** Il punto architetturale decisivo: il seeding serve al *lowering* di una unit successiva (una sottoclasse che estende, un `use T`, un riferimento a const/prop del parent). Il lowering di una sottoclasse consuma **forme**: nome, parent/interfaces, `props` (con i default const-expr — che vivono in `PropDecl`, non nei corpi), `static_props`, `consts`, firme dei metodi (visibilità/static/final/params/by_ref/hint), `abstract_methods`/`abstract_sigs`. Il **dispatch** dei metodi ereditati avviene a runtime sul `CompiledClass` via `class_module` — mai sull'HIR. I corpi (`MethodDecl.decl.body` + `slots`) nel seed di classe sono zavorra, con **una eccezione strutturale**: `seed_traits`. `LoweredTrait.methods` e `.closures` (hir.rs 118-142) DEVONO tenere i corpi, perché il flattening copia il corpo del trait dentro ogni consumer e lo ri-lowera (con `closure_shift`). Il seed signature-only è quindi una dieta **per-classe**, non per-trait — questa distinzione va nel design ex-ante o la parità salta al primo `use T` cross-unit.

**Perché il census mentiva di 5×.** `module_census_bytes` (vm/mod.rs 13711-13745): salta funzioni/classi con `Rc::strong_count > 1` (condivise = contate *zero* volte, non una), conta i `Const` come `size_of` piatto (i payload dietro `Rc` — stringhe/array literal — invisibili), e `size_of_val(&**c)` sul `CompiledClass` non scende nelle strutture interne. Il fix è un deep-census con dedup per indirizzo (seen-set di puntatori, ogni identità contata una volta) — prerequisito di qualunque A/B sulle leve compile-side, altrimenti misureremo la dieta con lo stesso metro che ha nascosto il giacimento per 14 sessioni.

**Equità del confronto con i 394MB dell'oracolo.** Va detto onestamente in verbale: il confronto è *già* equo, e per phpr è la notizia peggiore possibile. I 394MB dell'oracolo sono PHP CLI che (con o senza opcache) **non ritiene l'AST** e paga la discovery una volta come phpr; se l'oracle girasse opcache-ready il suo compile-side sarebbe perfino condiviso/immutabile. Gli ~800MB compile-side di phpr non hanno *nessuna* controparte strutturale in Zend: non è "Zend è più denso", è "Zend quella memoria non ce l'ha proprio". Manca però il numero di riferimento fine: quanto costa a Zend la costruzione della suite? Lo chiedo sotto (richiesta 4) — `--list-tests` sull'oracolo è l'oracle da 1 comando che WP-59 ha inventato per phpr; va puntato anche su Zend.

## 2. RICHIESTE per WP-60 (ordinate; contatori PRIMA delle leve)

**R1 — Contatore firma-vs-corpo nei seed (ex-ante, vincolante).** Deep-size di `seed_classes` + `seed_traits` + `main_hir` con split a tre: (a) corpi (`MethodDecl.decl.body` + `.slots` + closures dei trait), (b) firme/forme (tutto il resto di ClassDecl/FnDecl), (c) doc/attributes/file (i campi reflection-bound). Dump alla finestra di picco e a `--list-tests`. *Falsifica*: la banda del seed signature-only. Se i corpi sono ≥60% degli ~530MB di seed HIR la leva è da centinaia di MB; se sono <30%, la leva si sgonfia e il giacimento va cercato in (b)/(c). Costo: mezza giornata census-only. **Nessuna riga della leva 3 si scrive prima di questo numero.**

**R2 — Unit per path risolto + bytes duplicati.** Istogramma path→(n unit ritenute, bytes deep-census v2) sulle 2046 unit del media e sul full: quota del leak template = Σ bytes delle unit oltre la prima per path *a contenuto identico* (fingerprint già disponibile: la catena `unit_chain_fp` usa path+mtime+size). *Falsifica*: la banda della compile-cache — sul media potrebbe essere modesta (l'esecuzione aggiunge ~100MB permanenti, e la discovery è quasi tutta `require_once`), sul full con i template WP re-inclusi per 30k test potrebbe essere la voce dominante. Non firmo la banda prima di questo istogramma. Costo: ore (il fingerprint c'è già).

**R3 — Breakdown del terzo op (~270MB) con census v2.** Deep-census dedup-per-indirizzo dei Module: ops/payload Rc per kind di op, literal (per contenuto: quota duplicata TRA unit), slack dei Vec, exc_table, `CompiledClass` interno. Riattivare il gauge G_UNITS (oggi morto) sul metro nuovo. *Falsifica*: se i literal duplicati cross-unit dominano, si apre una dieta da compile-time interning (che è cosa diversa dall'interning runtime che ho ritirato: qui è dedup di dati immutabili a compile, zero trappole refcount); se domina lo slack, è `shrink_to_fit` al link (leva WP-48-style, predizione-misurata). Costo: mezza-una giornata.

**R4 — Target oracle del compile-side.** Sull'oracolo: `phpunit --list-tests` con `memory_get_peak_usage(true)` + `/usr/bin/time -l` (maxrss), stesso corpus, DB reset (regola nuova WP-59 anche nei probe). Dà il denominatore onesto per la voce "costruzione della suite": phpr 1,35GB vs Zend X. *Falsifica*: la tesi "il footprint è ~90% costruzione della suite" deve valere anche come rapporto: se Zend fa la stessa discovery in 150MB, il target della dieta compile-side è quantificato dall'alto. Costo: minuti.

**R5 — Sentinelle di parità pinnate PRIMA delle leve** (stile WP-56 order/tomb): (i) closure con `static $x` in file re-incluso non-`_once` — Zend condivide la cella per op_array: verificare cosa fa phpr OGGI vs oracolo, perché la compile-cache cambierebbe il comportamento (verso Zend o via da Zend — va saputo prima); (ii) classe anonima in file re-incluso (Zend fa binding once); (iii) fatal `Cannot redeclare` per funzione/classe in file re-incluso (la cache NON deve mangiarselo); (iv) `getDocComment`/`getFileName`/`getAttributes` dopo seed-thin (verificare che leggano dal CompiledClass, non dal seed HIR); (v) `eval` che estende classe con default di proprietà const-expr complessi (self::/parent::) e con attributes. Costo: mezza giornata; sono i probe-da-30-righe che questa roadmap ha imparato a pagare volentieri.

## 3. GIUDIZIO sull'ordine delle leve

Ordine: **R1-R5 (contatori+sentinelle) → Fase 0.5 compile-cache → seed signature-only → dieta payload op (esito R3)**. Motivazione:

1. **Compile-cache keyed sul path PRIMA del seed-thin**, per tre ragioni: è un *leak* (bug di ritenzione, non trade-off — la roadmap stessa dice "si chiude prima di attribuire il resto"); il suo meccanismo è l'unico possibile in questa architettura — i Module sono `&'m` leaked, quindi *liberare è strutturalmente precluso in safe Rust*: l'unica cura è **non allocare** (riuso del Module già linkato), che è esattamente il modello opcache; e riduce il rumore su cui si misurerà il seed-thin. Banda onesta: sul probe B è (N−1)/N del costo del file per N re-include — sul media/full NON firmo cifre prima di R2.
2. **Seed signature-only** è la leva con il giacimento attribuito più grande (2/3 di ~800MB, MATCH probe C 600-800 vs 818 osservati) ma con lo split firma/corpo NON ancora misurato: banda firmabile solo dopo R1. Il mio prior da Zend — in un AST i corpi pesano tipicamente parecchie volte i metadati di firma — dice che la leva è grande, ma la lezione WP-56 (estimatore 5,7×) mi vieta di firmare il numero: R1 costa mezza giornata, il prior non vale quel rischio. Nota di design: `main_hir` va trattato con lo stesso criterio (le funzioni libere del Program: al seeding servono le firme; i corpi servono solo se un percorso di ri-lowering li consuma — da verificare su `DeferredDecl`, che ri-lowera da *snippet sorgente* e quindi dovrebbe essere già indipendente dai corpi ritenuti).
3. **Payload op**: condizionato a R3. Se literal cross-unit duplicati ⇒ dedup a compile-time (immutabile, senza le trappole dell'interning runtime); se slack ⇒ shrink al link.
4. Il **revert della leva B** (già firmato dall'utente) e le mie quote declassate (duplicati str, literal-array) restano rispettivamente: primo atto amministrativo della sessione con gate pieno, e coda-se-avanza-tempo. Sul residuo full-only confermo la chiusura: ±0,6% di spread a metro esatto è la definizione operativa di rumore.

Aritmetica di prospettiva (non è una banda firmata, è il tetto del giacimento): media picco 1436MB = canali valore ~150 + unit counted ~212 + compile-side ~800 + runtime ~120 + frag 30 + non-mimalloc ~110. Se cache+seed-thin+payload realizzassero anche solo metà del compile-side, il footprint ratio scenderebbe da 4,07× verso ~3× — è l'unica famiglia di leve sul tavolo con questo ordine di grandezza, e per la prima volta nella roadmap la struttura-bersaglio ha un precedente Zend *esatto* (AST liberato, CE-only) invece di un'analogia.

## 4. VETI / AVVERTENZE (vincolanti)

**Sulla compile-cache (riuso di Module già linkati):**
- **VETO: chiave = realpath da solo.** La suite WP *riscrive* file temporanei sullo stesso path in-run; mtime+size può collidere nello stesso secondo a size uguale. Chiave minima: realpath + mtime a granularità ns + size (APFS la dà); in fallback stesso-secondo, hash del contenuto. Un cache-hit stantio è una divergenza dall'oracolo *senza opcache*, che ricompila sempre e vede sempre il contenuto nuovo — la cache deve essere invisibile per costruzione, ogni diff di fail-set è allarme (il riuso è semanticamente invisibile come lo era il pool: stesso standard di gate).
- **Il cache-hit deve RI-ESEGUIRE il top-level** (side effects, valore di `return` del template ricalcolato) e deve **preservare i fatal di redeclare**: un file che dichiara funzioni/classi re-incluso non-`_once` in PHP è fatal — il riuso non può trasformarlo in un no-op silenzioso. Via pulita: cache-hit pieno solo per unit *senza dichiarazioni top-level nuove*; per le altre, replicare il fatal.
- **Static-cell ids**: il riuso condivide le celle `static` delle closure tra tutte le esecuzioni del file (semantica per-op_array di Zend); la ricompila odierna probabilmente NO. Sentinella R5-i con verdetto oracolo PRIMA di scegliere: se phpr oggi diverge da Zend, la cache è l'occasione di chiudere la divergenza — ma va gated per nome, non scoperto dal full.
- **Class ids / classi anonime**: il binding once del Module riusato deve dare lo stesso osservabile di Zend (R5-ii); e `get_included_files()` deve continuare a registrare il path una volta sola, mentre il conteggio unit del census deve distinguere hit da miss (o l'A/B della leva si auto-inganna).
- **Closure/Generator che catturano `module_id`**: il riuso mantiene l'id del modulo — corretto per costruzione; ma vietato qualunque tentativo di *liberare* Module "ormai inutili": sono `&'m`, e closures/generators sopravvissuti all'include li referenziano. La cura è non-allocare, mai deallocare.

**Sul seed signature-only:**
- **VETO: toccare `seed_traits`.** I corpi dei trait sono il meccanismo del flattening cross-unit (`LoweredTrait.closures`, `closure_shift`): la dieta è solo per `seed_classes`/`main_hir`.
- **PropDecl/consts/enum_cases/attributes/abstract_sigs restano interi**: i default const-expr e le attributes-as-new-expr sono consumati dal lowering delle sottoclassi e da Reflection (il mock generator PHPUnit vive su `abstract_sigs`). Si strippa SOLO `MethodDecl.decl.body` (+`slots` se il contatore R1 li mostra rilevanti).
- **Reflection**: `getDocComment`/`getFileName`/`getStartLine` devono provenire dal CompiledClass — sentinella R5-iv prima del drop, e gate refl 290 + ORM 3E/13F + hk 0E/0F obbligatori (la ricetta ORM copre esattamente reflection/mock).
- **eval che ri-lowera con l'immagine**: il gate pieno deve includere probe eval-extends con parent multi-livello cross-unit e con trait nel mezzo — il percorso `DeferredDecl` (ri-lowering da snippet) va verificato indipendente dai corpi ritenuti, non assunto.

**Di metro e metodo:**
- Le leve compile-side si giudicano su **phys_footprint watermark + census v2 (deep, dedup per indirizzo)** — mai sul census attuale che sotto-conta 5×, mai su maxrss (sul full 2,3G stavano nel compressor). CPU full SOLO `/usr/bin/time -l`; mai fail-set da run con env mimalloc; DB reset anche nei probe che bootstrappano phpunit.
- **NON riproporre**: frammentazione mimalloc come leva (2%, riconciliata alla cifra — riaprire solo con una tabella per-bin che dica altro); interning runtime come leva footprint (questa sedia lo aveva promosso: lo ritira); reset per-request al boundary PHPUnit (il mio veto WP-58 resta: static-props = semantica).
- Restano vincolanti i miei veti WP-58 non consumati (dtor mai differiti; confronti `memory_get_peak_usage` solo come target per-canale).

La firma della sedia, per il verbale: **il ramo su cui è caduta la scommessa dice che phpr non deve imparare da Zend a essere denso — deve imparare da Zend a *dimenticare*. L'AST si libera, la firma si ritiene, il path si compila una volta. WP-60 misura (R1-R4), pinna (R5), e solo poi mette a dieta.**

---

## Sedia GREGG (attribuzione per accumulo)

# Report — sedia Gregg, concilio WP-59→60

Ho letto sessione, design59, gap report, la mia review precedente, la bozza §WP-60 e il grezzo `memcensus59-master.txt` (11 finestre + exit, per-bin completo). Il protocollo che avevo prescritto (watermark su phys_footprint, riconciliazione ±10%, supervisore a finestre) è stato eseguito meglio di quanto avevo chiesto: il gate è chiuso **alla cifra**, non a ±10%. Riconosco anche la conferma dell'ipotesi 1 sui 46k (recon obj 22.141==22.141) e l'adozione del metro CPU esatto. Detto questo, il mio mestiere è dire che cosa della mappa è inchiodato e che cosa è ancora stima — e ce n'è.

## 1. ANALISI — qualità della mappa WP-59

**Inchiodato (misura diretta, riconciliata):**
- L'identità di win10: phys 1436,2 = Σused 1298,0 + frag 29,8 + (commit−Σcomm) 104,3 + 4,1. Copertura ~100%, pre-registrata. Questo è il deliverable che avevo chiesto e c'è.
- Frag mimalloc 2% al picco (171MB solo all'exit). L'ipotesi 3/3 del concilio è morta di misura: bene così, è il sistema che funziona.
- Non-censito VIVO 936,6MB al picco, 918,6 ancora all'exit ⇒ standing, non churn. Solido.
- obj: recon esatta, peak 48,7MB. Chiuso.
- `--list-tests`: 818MB non censiti a bootstrap+discovery, zero test eseguiti. Misura diretta, un comando. È l'oracle più forte della sessione.
- Dal grezzo per-bin, un fatto che la sessione non ha ancora sfruttato: **il diff per-bin picco→exit separa già standing da churn per size-class**. Bin standing-dominati (used exit ≈ used win10): huge/1048576 **281,4MB**, 640B **88,1MB**, 6144 **69,5MB**, 1280 **62,9MB**, 3072 **44,6MB**, 1024 **44,0MB** ⇒ ~590MB di impronta compile-side con indirizzo di size-class. Bin churn-dominati (crollano all'exit): 112B (75,9→9,4MB, 711k→84k blocchi), 896 (33,4→3,9), 48, 160. Questa è la **firma per-bin** su cui pretendo vengano verificate le leve (sotto).

**Stimato / estrapolato (numeri che considero ancora deboli, in ordine di debolezza):**
1. **La spaccatura HIR ~2/3 vs payload op ~1/3**: estrapolazione dal probe C (3,8× vs 1,5× su 40+40 file), non una misura sui seed veri. Il "match 600-800 vs 818" ha ±25% di banda: conferma la *categoria* (compile-side), non la *ripartizione*. Su questa spaccatura si vogliono dimensionare due leve: non basta.
2. **"Runtime engine ~100-140MB"**: per differenza, mai censito. È la riga della tabella REPORT_GAP che non ha nessuno strumento dietro.
3. **La riga "~110MB metadata + non-mimalloc (FFI ~90-120MB)" è un errore di categoria**: a win10 il non-mimalloc misurato è **4,1MB** (phys 1436,2 − commit 1432,1); i 104,3MB sono *dentro* mimalloc ma non visitati. I "90-120MB FFI dirty" vengono da Ob.0 (run diversa, istante diverso) e non riconciliano con i 4,1MB del census run: le due osservazioni non possono stare nella stessa riga. E c'è di peggio: **se la visita abandoned è silenziosamente fallita** (esito non verificato, come ammesso), i blocchi vivi di eventuali heap abbandonati stanno esattamente lì, nei 104,3MB. Le due questioni aperte sono la stessa questione.
4. **str NON riconcilia al walk**: reached 211.727/16,9MB vs live 574.173/39,8MB — il 63% delle stringhe vive per conteggio è fuori dal walk, attribuito a "metadati moduli" per assunzione. arr e obj sono esatti; str no. Da chiudere o da dichiarare.
5. **"~90% costruzione della suite"**: 818/937 ≈ 87%, ma viene dal probe `--list-tests` (processo diverso), non dalle finestre del run — la mappa finestra→test è fallita (stdout bufferizzato, offset=0 ovunque). La traiettoria per-finestra resta anonima.
6. **Il full non ha una mappa propria**: di full sappiamo solo commit≈phys (Ob.0). La composizione 3,89GB è oggi un'estrapolazione ×2,7 dal media. WP-57 insegna che frequenza×taglia cambia per workload: non firmo bande sul full senza la sua tabella.
7. **Ob.3: n=1 per binario, spread serale ±0,6%**: il −0,56% del pool è un punto dentro la banda di rumore. La direzione è sostenuta dal meccanismo (Leijen) e il revert è comunque giusto (beneficio nullo, codice nel Drop caldo), ma il *numero* 0,6% non è un fatto — è una stima con barra d'errore pari a sé stessa.

## 2. RICHIESTE per WP-60 (ordinate)

**R1 — Riparare la mappa finestra→test IN-PROCESS (prerequisito, ~20 righe).** Mai più offset di stdout: phpunit bufferizza, e qualunque figlio può bufferizzare. Il callback watermark gira *dentro* la VM: logghi il top dello stack PHP (file:line + Class::method + profondità include) su ogni riga `mi_proc`, più wall-clock monotono sia nelle righe census sia nel log del supervisore (join post-hoc). Falsifica: 100% delle finestre con un nome. Costo: mezz'ora, census-only.

**R2 — Quota ex-ante leva compile-cache (Fase 0.5), col contatore che diventa il metro post.** Contatore census per path risolto: n_compile, bytes unit, e **hash del contenuto** ad ogni compile. Quota = Σ_path (n_unit − n_contenuti_distinti) × bytes — su media **e** full (i template wpdev sono il bordo che la leva copre; la lezione WP-55 dice che la banda si realizza al bordo del sito). Predizione registrata in design60 PRIMA dell'A/B: Δpeak phys media, Δpeak full, unit.cum_n atteso. Post: (a) mechanism-check alla cifra — unit.cum_n == n_path_distinti(+contenuti cambiati); (b) predicted-vs-actual phys ±15%; (c) drenaggio nei bin standing (640/huge) coerente. L'hash del contenuto serve anche alla parità: vedi veto V1.

**R3 — Quota ex-ante leva seed signature-only con deep-size DIRETTO, non estrapolato.** Walker census sui seed veri (`seed_classes`/`seed_traits`/`main_hir`) che spacca ogni classe in {firma, `MethodDecl.body`, altro} — istogramma per classe, su media e su `--list-tests`. Questo sostituisce la spaccatura 2/3-1/3 con una misura e dà la predizione in byte. Post: contatore body-bytes-nei-seed ≈ 0 alla cifra; diff per-bin (mi aspetto drenaggio dominante nelle righe huge/1048576 e 640 — se drena altrove, il modello mentale era sbagliato e lo vogliamo sapere); phys ±15%. Costo: mezza giornata.

**R4 — Mappa del FULL: sì, serve, ed è economica.** Una (1) run census full con la strumentazione WP-59 così com'è (watermark phys +128MB ⇒ ~30 finestre, supervisore invariato) + il probe bootstrap-only equivalente a `--list-tests` sul full. Motivo: le predizioni R2/R3 vanno registrate anche sul metro full (3,89GB), e il media non è un modello del full per fede. Costo: 1 run detached + 1 probe. Senza questa, ogni "banda sul full" in design60 è un numero inventato.

**R5 — Chiudere le due code del protocollo WP-59:** (a) positive-control sulla visita abandoned: thread che alloca e muore, il visitor DEVE vederlo; se no, la colonna aband si dichiara morta e i 104,3MB restano "mimalloc non-visitato" dichiarato; (b) riconciliazione str al walk (o censo dei 362k/23MB fuori-walk, o riga esplicita "metadati moduli: misurato X"). Costo: ore.

**R6 — Igiene ambientale codificata nell'harness, non nella memoria:** pre-flight DB (conteggio, regola MySQL esistente) *dentro* gli script di probe che bootstrappano phpunit, non come disciplina; e pensionare `MIMALLOC_SHOW_STATS` via env a favore delle chiamate FFI programmatiche già in uso (`mi_process_info`/visit) nei soli build census — l'env propaga ai figli, il codice no.

## 3. GIUDIZIO di metodo sulla prossima sessione

**Leva-con-contatore-integrato, che è la forma matura di misura-poi-leva.** I contatori di R2/R3 non sono probe usa-e-getta: sono il metro che viaggia con la leva e che dopo l'A/B deve leggere ≈0 (il pattern accounted+sync di WP-57/58 applicato al compile-side). La sequenza che pretendo:

1. **Prima il revert della leva B, da solo, con gate pieno + coppia full stessa-sera** — e quel binario diventa la NUOVA baseline. Se il revert entra nello stesso A/B delle leve nuove, qualunque delta CPU sarà inattribuibile per costruzione.
2. **Poi le due leve compile-side in A/B SEPARATI**, cache prima (è un leak, superficie di parità minore), signature-only dopo. Mai cumulativo: colpiscono lo stesso canale (unit/compile-side) — un A/B cumulativo che regredisce non si spacca più.
3. **Barre d'errore dichiarate**: sul CPU full lo spread misurato a metro esatto è ±0,6% ⇒ ogni claim CPU <1,5% richiede o coppie ripetute o la dicitura "dentro il rumore". Sul footprint, il verdetto è predicted-vs-actual ±15% + riconciliazione per-bin, non "è sceso".

Che cosa impedirebbe di attribuire una regressione, oggi: (i) baseline contaminata dal pool non ancora revertato — risolto dal punto 1; (ii) finestre anonime — risolto da R1; (iii) assenza di mappa full — risolto da R4; (iv) stato DB sporco dai probe — risolto da R6. Se questi quattro entrano, ogni regressione di WP-60 avrà un indirizzo (per-bin, per-finestra, per-leva) entro una sessione.

## 4. VETI / AVVERTENZE (vincolanti)

- **V1 — La compile-cache NON può essere keyed sul solo path.** Le suite scrivono file temporanei, li includono, li riscrivono e li re-includono: una cache path-only serve HIR stantio = bug di parità silenzioso. Chiave minima path+(mtime,size,inode) o hash contenuto (che R2 già raccoglie); probe pre-registrato write→include→rewrite→include nel gate. È la stessa disciplina di opcache: non è opzionale.
- **V2 — Signature-only: dimagrire SOLO le copie seed, mai l'HIR dell'unit proprietaria.** Il backlog ha già `ast_printing.phpt` che vuole l'AST dei corpi; e ogni percorso eval/re-lower che tocchi un corpo va coperto dal gate pieno (corpus + refl 290 + ORM + hk). Se il deep-size R3 trova classi i cui corpi servono al re-lowering, si tengono e si dichiara la quota trattenuta — niente "banda piena" per fede.
- **V3 — Il revert B non si giudica dal CPU.** Il guadagno atteso (+0,56%) è dentro lo spread ±0,6%: il gate del revert è parità per nome + footprint ±0; se la coppia full mostra −0,5% bene, ma non è quello il verdetto.
- **V4 — Mai più mappe di progresso da stdout di processi figli** (phpunit bufferizza; qualunque figlio può farlo). Il nome del test si prende in-process (R1) o da un canale con flush esplicito.
- **V5 — Igiene DB nei probe**: ogni probe che bootstrappa phpunit/wptests resetta il DB o usa un DB dedicato; il full successivo esegue il pre-flight a conteggio. Il flake di Ob.3 è costato un'ambiguità sul fail-set di un A/B da una serata.
- **V6 — Effetto osservatore**: env mimalloc mai su run di cui serve il fail-set (propaga ai figli); vmmap/footprint sospendono il target, ≤1 snapshot/s, solo census; `mi_collect(true)` solo nei build census etichettati.
- **V7 — Trappole Darwin confermate**: maxrss mente sul full (1,51 vs 3,89GB — compressor); phys_footprint e RSS mai sommati né diffati tra loro; su macOS 26 lo heap mimalloc appare come regioni IOAccelerator (os_tag 100) — malloc_history/leaks/Instruments-Allocations restano ciechi; CPU del full SEMPRE da `/usr/bin/time -l`.
- **V8 — Nessuna riga "per differenza" nelle tabelle di WP-60.** La riga "runtime engine ~100-140MB" e la riga "FFI ~90-120MB" di REPORT_GAP_59 non si citano più come attribuzioni: o hanno uno strumento (R5, census dedicato) o si etichettano "non attribuito". La sottrazione-mistero è esattamente ciò che questa sessione ha appena finito di uccidere: non reintrodurla dalla porta di servizio.

La mappa WP-59 è il miglior prodotto di misura del progetto finora: un'identità chiusa alla cifra, un'ipotesi di concilio falsificata in una finestra, un artefatto contabile confermato e fixato. I punti deboli elencati sopra non la invalidano — dicono dove la prossima sessione deve piantare i paletti prima di spendere le leve.

