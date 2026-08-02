# VERBALE — Daan Leijen, sedia 7, Concilio WP-91

VERDETTO: **CON EMENDAMENTI** (una refutazione capitale di premessa: la
metrica di b è un contatore CONGELATO AL PICCO su macOS).

## Q1 — census per-theap a win=0: STRUTTURALMENTE MUTO, e la premessa "il grosso vive negli abandoned" è FALSA

Dai raw (m89.slope.w12.r1): a win=0 il census vede theap post-collect
223.600 B (~250 KB, un heap), bins src=main 571.200 B, bins src=aband
200.512 B — **tutte** le righe aband hanno used_n≥1. **Abandoned free
(used==0): ZERO righe, 0 B, in TUTTI i raw m89** (verificato su ogni
finestra). Il commit di processo a win=0 è 327.417.856 B: il census copre
~0,8 MB, **il 99,76% non sta in NESSUNA pagina** (mi_arena pages
current=36, pages_abandoned current=10). La cifra chiesta non è
ricavabile dai tag: non esiste.

Perché: su macOS release `_mi_prim_decommit` (prim/unix/prim.c:493-508)
usa MADV_FREE_REUSABLE e pone `*needs_recommit=false`; in os.c:590-592
`mi_os_stat_decrease(committed,…)` gira SOLO se needs_recommit —
**quindi mai**. Il contatore committed è monotono: nei 20/20 raw slope
(e negli slope0) `commit == peak_commit` AL BYTE, mentre phys a win=0 è
3,5-4,8 MB piatto su tutti i W. **committed_postcollect_win0 ≡ commit
PEAK**: b = 19.575.603 B/worker è la pendenza del PICCO, non del
trattenuto. Controprova: phys_peak 52,0/131,4/217,8/299,5 MB (mediane
W4/8/12/16) ⇒ ~20,6 MB/worker ≈ b. KL-90-4 formalmente eseguita, ma la
gamba census era VACUA per l'attribuzione a quel timing; la refutazione
P-RET0 regge sul solo braccio (b_ret0≈b_base). **Sì al collect+census
IN-REQUEST — ma per attribuire il PICCO, non un residuo che fisicamente
non esiste (~4 MB).**

## Q2 — P-RET0: refutazione valida ma il candidato era mezzo-morto in partenza

page.c:749: `page_full_retain = (block_size > MI_SMALL_MAX_OBJ_SIZE ? 0
: theap->page_full_retain)` — **solo small**; per pagine large il retain
è 0 incondizionato in ENTRAMBI i bracci ⇒ l'arm non poteva discriminare
nulla sul lato large. **Nessun ordinale gemello per large esiste**
(options.c:166, ord36 default 2, clamp [-1,32] in init.c:310; retain=0
mantiene allow_page_abandon=true, solo <0 lo spegne, init.c:309).
Con retain=0 restano committed: pagine full con blocchi vivi
(mi_page_to_full, page.c:775), abandoned con blocchi vivi, pagina
candidate per coda — ma a win=0 il census le limita a <1 MB totali: il
vero detentore è il contatore mai decrementato (Q1).

## Q3 — iterazione 2, ordinati per potere discriminante

1. **Census+collect IN-REQUEST al picco (worker VIVI)** — read-back:
   heaps_total==W+1 pinnato, coppia phys/phys_peak nella finestra.
   Unico braccio che decompone ciò che b misura davvero.
2. **Δcommitted per finestra disgiunta** (contatore monotono ⇒ Δ esatto,
   lezione WP-88): attribuisce il picco per FASE senza census.
3. Pagine parzialmente usate per bin: dentro il braccio 1, non
   standalone (a win=0 già misurate <1 MB).
4. Abandoned con blocchi vivi: 10 pagine a win=0 — non spiega 19,5
   MB/worker; solo come voce del braccio 1.
5. ord 44 minimal_purge_size: agisce sul residuo FISICO, non sul commit
   ⇒ potere NULLO su b come definita: ritirarlo da questa lane.

## Q4 — clamped dt 1-5 ms: coerente

purge_delay=0 ⇒ mi_arena_schedule_purge purga SINCRONA nel teardown
(arena.c:2059-2067): la deflazione fisica del lato primo cade dentro la
finestra 1-5 ms del secondo; a dt≥10 il teardown è già concluso. Nota:
può deflazionare solo un canale FISICO (il commit è monotono). Riga
probante: **Δpurged/Δpurge_calls per finestra** (key=purged già
in-band); pin: clamp ⇔ Δpurged>0 nella finestra del lato clamped.

## Emendamenti

- **A-DL48**: dichiarare commit_current≡commit_peak su macOS (catena
  prim.c:504→os.c:591) e riqualificare committed_postcollect_win0 come
  **PEAK-metric**; il giudice verifica commit==peak_commit su ogni raw.
- **A-DL49**: braccio census in-request al picco (worker vivi), pin
  heaps_total==W+1; attribuzione di b per bin/heap al PICCO.
- **A-DL50**: emettere Δpurged/Δpurge_calls per finestra nello sweep col
  pin di Q4.
- **A-DL51**: ord44 ritirato dalla lane commit; eventuale lane phys.

## Kill-switch

- **KL-91-1**: se in un raw win=0 commit≠peak_commit, ogni cifra fondata
  su A-DL48 è VOID.
- **KL-91-2**: finché b è definita su committed_postcollect_win0, ogni
  VATTR basato su soli knob post-picco (retain/abandon/purge) è VOID.
- **KL-91-3**: census a worker vivi verdict-grade solo se copre ≥90% del
  commit al picco; sotto = ADVISORY (mai più un census muto benedetto).

— Daan Leijen, sedia 7
