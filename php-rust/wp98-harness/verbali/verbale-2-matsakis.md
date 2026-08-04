# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — WP-98

## VERDETTO: ESITO CONFERMATO, MOTIVAZIONE REFUTATA

Non costruire `TakeSlot` in S-96.0 è giusto, e tutto ciò che trovo spinge nella
stessa direzione (meno guadagno, più rischio). Ma la riga `nota-guard-di-tipo`
del riconteggio è un errore di lettura, e il perimetro Str-first è stato
calcolato con un numeratore senza il suo denominatore.

## Refutazioni capitali

**RC-MS-98-1 — «il buco è minuscolo» è REFUTATO.** `would_take_safe_ref`
(zvalcensus.rs:109) è `matches!(cell, Zval::Ref(_))`: misura l'unico canale di
aliasing **visibile nel tag della cella**. Il buco vero è «lo slot è osservabile
altrove», e ne esiste almeno uno che quel test non può vedere per costruzione:
`current_frame_args` (mod.rs:10493) legge `frame.slots[i]` **VIVI di OGNI frame
sullo stack** al momento della chiamata, e alimenta gli `args` di
`debug_backtrace` (host.rs:4343) e dell'altro costruttore di trace
(mod.rs:13288). `renounce()` scorre **solo `func.ops`**: se l'osservatore sta
nel corpo di una *callee* (o nella macchina delle eccezioni), `observes_scope`
non scatta mai, la cella resta una `Str` semplice per tutto il tempo, e uno slot
di parametro svuotato da un take si legge `Undef`→NULL nel trace. In Zend i CV
non si consumano mai (Stogov, WP-97): è divergenza garantita, non probabile.
3307 è dunque un **limite inferiore di un canale enumerato**, non la misura del
buco. La lezione del passo 0 di S-96.0 — «una cura enumerabile contro un attacco
non enumerabile è vacua per costruzione» — si applica alla lettera alla
conclusione del passo 1.

**RC-MS-98-2 — la banda Str è un numeratore orfano.** `guadagno_..._str` =
(9.989.963 / 53.561.185) × moltiplicatore. Il moltiplicatore §P1 è la quota CPU
del canale su **tutte** le letture rc: moltiplicare una frazione di CONTEGGIO
per un tasso di COSTO **medio** assume che clone/drop costino uguale per
variante. `Str` è precisamente la variante col drop più economico (dec + branch;
nessun distruttore, nessuna liberazione ricorsiva, nessun teardown dual-repr):
la sua quota di costo sta **sotto** la sua quota di conteggio. 0,84–1,21% è
quindi un maggiorante, oltre che SCREEN.

## Emendamenti

**A-MS-98-1 (misurare il canale invisibile, a costo di un run).** Gli slot
`< n_params` sono osservabili cross-frame: escluderli da `movable_safe` e
RI-CONTARE. Il delta è la taglia del canale che `would_take_safe_ref` non vede.
Finché quel numero non esiste, nessuna banda safe/str è citabile.

**A-MS-98-2 (base di costo ≠ base di guadagno).** La guardia si paga dove il
take è **emesso** (safe: 25.826.594 esecuzioni, 51.691 siti), il guadagno si
incassa dove il tipo combacia (9.989.963 = 38,7%). Ogni netto futuro sia scritto
come `gain(str) − guard(safe)`; oggi è stato scritto solo il primo termine.

**A-MS-98-3 (flag vs opcode: equivalenza dichiarata).** §WP-97 punto 1 è
ownership-**equivalente** a `TakeSlot`: identico trasferimento, identico
perimetro, identici A-MS-97-2/3/4. Cambia SOLO il lato costo. Va scritto, perché
nessuno ri-apra la questione di correttezza sotto la forma nuova. Due corollari:
(a) «deciso a compilazione» **non** dà conoscenza di tipo — la guardia runtime
resta (e costa poco: `read_slot` discrimina già il tag); (b) un bit dentro
`LoadSlot` viaggia invisibile nella unit cache TL (WP-81) in ogni richiesta
successiva: l'analisi stia nel COMPILATORE, il bit sia funzione pura degli ops,
mai mutato.

## Kill-switch

**KS-MS-98-1**: nessun take si emette prima che il gate contenga la trappola
cross-frame (callee che chiama `debug_backtrace()` e legge un parametro del
chiamante, più una eccezione con trace). Se la trappola non morde sul binario
pre-leva, non è un dente.
**KS-MS-98-2**: se `would_take_safe_ref` viene ancora citato come «il buco», il
numero si ritira invece di difenderlo.
