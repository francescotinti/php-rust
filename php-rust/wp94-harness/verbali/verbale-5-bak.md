# Verbale SEDIA 5 (Bak) — Concilio WP-94 — A-DL-59 (page slack vs pendenza invisibile)

## VERDETTO
**CONFERMATO con emendamenti.** Ricomputo indipendente (python, senza dl59-join.sh) dai 20 raw `wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus` (NUL rimossi): **match al byte con dl59-join.out su ogni colonna e ogni run**. La refutazione dell'ipotesi page-slack di Leijen regge. La forma FORTE della conclusione («ad arena/chunk») è un overreach: va declassata a «fuori dai bin del heap visitato».

## Ricomputo (per NOME)
- **SLACK_naive, 20/20 al byte** (es. w4.r1=7.970.672; w16.r2=111.306.784; w12.r3=60.351.568).
- **SLACK_global e glob_d2, 20/20** (es. w12.r1 glob=4.956.061,3→4.956.061; d2=6.007.492,7→6.007.493: arrotondamento, non errore).
- **heap_commit, pwork_commit, freePgRows (16 solo su W=16, 4.194.304B=16×262.144), visheap_free_c, thrSumMatch W/W, d1mispl=d2mispl=0: 20/20 al byte.**
- Medie per-W identiche (8.098.364,8 / 28.261.120,0 / 59.613.318,4 / 110.385.798,4; global 2.024.591,2 / 3.532.640,0 / 4.967.776,5 / 6.899.112,4).
- OLS: naive slope=8.455.362,5, int=−32.963.974,4 (ratio **1,887**, int-ratio −0,458); global slope=401.467,5, int=341.355,0 (ratio **0,090**, int-ratio 0,005). 4-means≡20-pt (design bilanciato) ✓.
- Shared-heap: in ogni dump#1 **1 solo heap ptr, 1 solo committed** per tutti i thr; Σ_thr committed/pwork = 1,66× (W4) → 10,4× (W16); w16.r1 Σ=4.049.975.808 ✓ al byte. La Σ naive è genuinamente invalida.
- Autorità verificate su `wp91-harness/repair90-estimators.out`: slope_inv=4.480.174, int_inv=71.934.811, slope_vis=15.777.004, a_vis=692.184 ✓ (righe 101-102).

## Q1 — Le cifre reggono? **SÌ**, zero mismatch su ~260 valori confrontati.

## Q2 — Scelta dump#1: **sana, con una crepa dichiarabile.** La tripla ridondanza morde davvero: split alla prima size-class ripetuta ⊕ uguaglianza committed con mi_bin_thr_sum (20/20, W/W) ⊕ posizione vs marker `mi_proc ckpt=peak_inreq` (0 misplaced su 20). Un bin apparso per la prima volta in dump#2 romperebbe lo split, ma farebbe fallire thrSumMatch: il guard esiste. Fragilità residua: dump#1 NON è esattamente pwork — in **w16.r2 commit(peak_inreq)−commit(pwork)=+1.048.576B, w16.r3 +4.194.304B** (≤1,1%, 2/20 run; immateriale vs invisibile ~137MB, ma il .out dice «near-identical» senza quantificare).

## Q3 — Identità vis_model≈bin-committed: **fatto su TUTTE le 20 run**, non solo sulle medie: deviazione max **+0,172%** (w12.r4/r5), min −0,030%; pattern lieve concavo (W8/12 positivi) = residui del fit, non struttura. NON è coincidenza: il potere discriminante è reale — se il census contasse used_b invece di committed la deviazione sarebbe ~2,7-3,2% (slack/heap); osservato <0,18%. Caveat: è semi-tautologica (vis stimato dalla stessa famiglia census), quindi è un fatto di **coerenza interna** che colloca lo slack DENTRO il visibile — sufficiente per refutare Leijen, non un oracolo indipendente.

## Q4 — «L'invisibile vive ad arena/chunk» è l'UNICA lettura? **NO.** I dati provano solo: invisibile = pwork_commit − bin-committed, cioè **fuori dai bin del heap visitato**. Alternative non escluse, per NOME: (1) **segmenti/pagine abbandonati** di thread morti (di nessun heap; heaps_total=1 non li copre); (2) **MI_BIN_HUGE / allocazioni OS-direct** oltre l'ultimo bin (tls arriva a size=1.048.576 su W16: la soglia di copertura non è dimostrata); (3) **metadata di segment/pagina** mimalloc (scala con W). Esclusa invece la defer-heap: `src=defer visit=NULLHEAP` in tutti i dump delle 20 run ✓. La frazione invisibile cala 58,3%→35,1% (W4→W16), coerente con massa intercept-dominata ✓.

## Emendamenti
- **A-BB73-1**: declassare la chiusa a «l'invisibile vive FUORI dai bin del heap visitato»; arena/chunk-slack, abandoned segments, huge/OS-direct e metadata restano indistinti — discriminarli spetta al canale barrier A-DL-57/58 (dump arena stats + abandoned count).
- **A-BB73-2**: dichiarare il drift dump#1↔pwork con cifre (w16.r2 +1.048.576B, w16.r3 +4.194.304B; 18/20 identici).
- **A-BB73-3**: riformulare l'identità come test committed-vs-used (margine 3% contro <0,18% osservato), non come oracolo esterno.

## KS-BB-94-n
Nessun kill-shot. (Il min..max per-W della sezione identity nasconde le deviazioni per-run, ma verificate tutte ≤0,172%: nessuna si avvicina alla soglia.)

## Refutazioni capitali: **NO.**
