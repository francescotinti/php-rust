# Verbale sedia 5 — Bak (V8/HotSpot: alloc-rate, code-cache, path caldi) — Concilio WP-104

## VERDETTO

S-102 è una sessione di strumenti fatta bene: il census pila (attese PRIMA,
dinamico che arbitra, ledger push==pop, linearità 300:1) soddisfa
A-BA-103-1 **sul giudice prop**. Ma il programma S-103 contiene tre vizi da
refutare: un denominatore aggregato che non esiste, una banda H-C2 derivata
da un canale MAI contato, e una leva-nulla lasciata a «gate dichiarato»
quando è precondizione di OGNI Δ tra binari diversi.

## (a) Census pila — soddisfa, con una riserva capitale

Il livello SORGENTE è quello giusto: è l'unità che una leva manipola, e il
fan-out compilato (len+as_slice+ptr::read per un pop) è costante per
primitiva sul fast-path — si cancella in Δ/conteggio **per primitiva**. Non
serve il livello compilato. Ma NEXT (voce aperta) scrive «Δ_A/B ÷ 23»: è un
errore di categoria. Push, pop, peek, len, elem hanno costi compilati
DIVERSI (un push muove uno Zval intero; un len è un load); l'aggregato 23
produce una «tariffa media» priva di potere predittivo — la stessa
non-tariffa già refutata dal Concilio WP-98. Il census per sito×prim
fornisce già i denominatori giusti: si usino quelli. Riserve minori: il
push del fallback PropGet non è taggato (il sito `Other` esiste ed è
inutilizzato) e il census non conta le riallocazioni del Vec (a regime
stazionario sono zero per costruzione, ma non è CERTIFICATO).

## (b) Leva-nulla — va nell'ORDINE, non nella dichiarazione

ABAB interleaved controlla la deriva TEMPORALE; non controlla il bias di
LAYOUT tra due binari diversi. Esperienza V8/HotSpot: il solo alignment di
codice produce swing dell'1-5%, cioè l'ordine dei Δ che H-C2 pretende di
leggere ([8,22] ns/iter su ~160 ns/iter). H-C1a/b sono passate con margine,
ma H-C2 no-per-diritto: la leva-nulla (build con modifica inerte, ABAB R=5,
banda layout pubblicata PRIMA) costa mezz'ora ed è il pavimento di
credibilità di ogni A/B futura — slot-diretti inclusi. Prefisso di H-C2.

## (c) H-C2 — il canale ~11 drop/iter è un numero statico mai contato

Il census S-102 conta TRANSITI, non esecuzioni di drop-glue. Il 4
gc_note/iter di S-101 e il 9 pop/iter di S-102 corroborano la scala ma
nessuno dei due È il canale di H-C2 (drop scalari che attraversano la
guardia). Pre-registrare [8,22] su ~11 non contato ripete l'errore che il
census esisteva per estirpare. Il sito di conteggio c'è già: la guardia
scalare di H-C1a. Contare prima, ri-derivare la banda dal canale CONTATO.

## (d) H-D — attribuzione per sito: esatta, mai a campione

~2 alloc/chiamata × ~35 B è un canale piccolo e regolare: il sampling
(heap-profiler style) è lo strumento sbagliato. Forma concreta, due passi:
1) **istogramma per size-class nel CountingMi** (bucket 0-16/17-32/33-48/
49-64/65+, atomics, zero tocchi ai siti): con due sole alloc/chiamata, due
taglie distinte identificano quasi da sole i siti; 2) **tag TL RAII di
sito** (`SiteTag` enum, set/reset nei costruttori candidati, letto da
CountingMi::alloc). Indiziati per NOME: `frame.ret_cell =
Some(Rc::new(RefCell::new(Zval::Null)))` (run.rs ~3459: un Rc fresco per
chiamata, inner ≈ 32-40 B — compatibile col ~35 B medio), args Vec,
pooled_frame su pool-miss, diags. Niente backtrace: 20M eventi lo vietano.

## Emendamenti

- **A-BA-104-1**: il criterio pre-registrato di ogni leva-pila NOMINA quali
  transiti (sito×primitiva) rimuove; il denominatore del Δ_A/B è QUEL
  sotto-insieme, mai l'aggregato 23. La voce «Δ_A/B ÷ 23» in NEXT si emenda.
- **A-BA-104-2**: leva-nulla promossa nell'ordine S-103 come prefisso di
  H-C2; verdetti H-C2 dentro la banda layout = VOID (si ripete, non si
  interpreta).
- **A-BA-104-3**: drop-census sulla guardia H-C1a PRIMA del criterio H-C2;
  banda ri-derivata dal canale contato.
- **A-BA-104-4**: attribuzione H-D = istogramma size-class + tag TL RAII
  (indiziato primo: ret_cell); vietato il sampling.
- **A-BA-104-5** (igiene): taggare i push dei fallback su `Other`; contatore
  grow nel push per certificare zero-realloc a regime.

## KS (kill-switch)

- **KS-BA-104-1**: nessun costo/transito da aggregati che mischiano
  primitive diverse.
- **KS-BA-104-2**: nessun verdetto A/B tra binari diversi sotto la banda
  layout della leva-nulla.
- **KS-BA-104-3**: un canale mai contato non entra in un criterio
  pre-registrato (generalizza KS-GR-103-1 dai ns ai conteggi).

## Refutazioni capitali

**SÌ, tre**: (1) «Δ_A/B ÷ 23» è un denominatore inesistente — le primitive
non sono fungibili; (2) la banda H-C2 poggia su ~11 drop/iter mai contati;
(3) ABAB senza leva-nulla non distingue una leva dal code-layout — la
taratura è precondizione, non gate opzionale.
