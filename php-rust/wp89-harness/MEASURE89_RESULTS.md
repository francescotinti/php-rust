# MEASURE89_RESULTS.md — misure S-89.0 nelle FORME ordinate dal Concilio WP-90

Campagna measure89 (§Sintesi WP-90 p6): ATTRIBUZIONE di b — slope BASE +
slope RET0 (braccio MIMALLOC_PAGE_FULL_RETAIN=0 con read-back ord 36) +
positivo EAGER (A-DL44, ord 4) + census per-theap (A-DL46-census) +
stagger-sweep invertito con swap-fixture (A-BB59). Verdetto macchina:
`wp89-harness/verdict89.a1.g3.out` (VERDICT89 PASS, attempt=1,
**generazione MASSIMA g3** — g1/g2 SUPERSEDute in-band, KS-SK-90-3; ogni
generazione porta judge_sha, A-AH51). Cifre BYTES-FIRST con companion
VERIFICATO; banda KL-85-2 RITIRATA (KB-90-2): nessun confronto di banda.

## Identità

- git campagna: ed427f4 · **attempt=1 PULITO** (nessun VOID in-run; le
  due generazioni di giudice rifiutate — g1 FAIL su regime clamped
  NUOVO, g2 FAIL su pin di forma-canale — sono FAIL del GIUDICE,
  ledgerate per generazione con judge_sha, non VOID della campagna)
- battery-89pre: **PASS (16/16 CONTATO) a 30f5a87** (attempt 4 — i
  quattro tentativi TUTTI ledgerati per esito nel battery-attempts
  ledger A-AH50: FAIL 1/16 a 692c697 su morso A-TH51, REFUSE porcelain
  a 65086e6, PASS intermedio 05726aa consumato solo dallo stamp, PASS
  finale 30f5a87); consumo **--same-rev v6+A-AH50** (finestra con
  attempts-ledger allowlistato)
- binario mem-census e2f27c9c671b737a ENFORCED contro matrix committed
  (tether ANCHE lato giudice, A-SK59); identity v2 + srv_boot_epoch
  GIUDICATO (A-BG51/KG-90-3) + pid-echo x-phpr-pid alla prima request
  di ogni fase; MIMALLOC_PURGE_DELAY=0 con read-back; read-back
  PER-BRACCIO su OGNI raw (VARMS: ord36 = 0 su ret0 / 2 altrove; ord4
  = 1 su eagerpos — A-DL41/A-DL44); census per-theap in-band su ogni
  raw slope (VTHEAP, dichiarazioni KL-90-3). DECLARED DEVIATION
  invariata: collect all'atexit su heap CONDIVISO v3 (A-DL39 design).

## Verdetti (da verdict89.a1.g3.out)

- **VSLOPE-BASE — b con incertezza in-band (A-BB57)**: modi dominanti
  su metric=committed_postcollect_win0 (⚠️ metrica RIQUALIFICATA
  **PEAK-metric** — A-DL48/KL-91-1, Concilio WP-91: commit_current ≡
  commit_peak su macOS; ri-verificato S-90.0 a macchina: commit ==
  peak_commit su 80/80 righe win=0 dei 40 raw slope BASE+RET0; **b è la
  pendenza del PICCO di commit**, non del residente, finché la
  separazione non è provata):
  W=4 156.762.112 B = 149,50 MiB [derivata: companion /1048576] (×2 su R=5)
  W=8 228.065.280 B = 217,50 MiB [derivata: companion /1048576] (×2)
  W=12 316.276.736 B = 301,62 MiB [derivata: companion /1048576] (modes==R: punto ADVISORY, A-BB58)
  W=16 388.366.336 B = 370,38 MiB [derivata: companion /1048576] (modes==R: punto ADVISORY, A-BB58).
  Marginali per-ΔW TUTTI dentro la fascia ex-ante δ=0,15:
  17.825.792 B = 17,00 MiB [derivata: companion /1048576]
  22.052.864 B = 21,03 MiB [derivata: companion /1048576]
  18.022.400 B = 17,19 MiB [derivata: companion /1048576].
  **b (LSQ modi dominanti) = 19.575.603 B = 18,67 MiB [derivata: companion /1048576] per worker**,
  se(b) = 584.723 B, 2σ = [18.406.157, 20.745.049] B,
  grade=verdict-grade-candidate.
- **VSLOPE-RET0 (braccio retain=0)**:
  b_ret0 = 19.329.843 B = 18,43 MiB [derivata: companion /1048576] per worker, se(b) = 1.319.393 B,
  2σ = [16.691.057, 21.968.630] B — grade ADVISORY (**DUE marginali
  fuori fascia, per nome e byte — sanatoria A-BB64≡A-BG56, Concilio
  WP-91**: 24.395.776 B = 23,27 MiB [derivata: companion /1048576] su
  W4→8 E 16.400.384 B = 15,64 MiB [derivata: companion /1048576] su
  W12→16).
- **VATTR — l'ATTRIBUZIONE, esito NEGATIVO nominato (KL-90-4)**:
  b_ret0 ≈ b_base (i due 2σ largamente sovrapposti) ⇒ **P-RET0
  REFUTATA: b SOPRAVVIVE a MIMALLOC_PAGE_FULL_RETAIN=0** — la
  ritenzione di pagine piene per size-class (candidato n.1, Leijen) NON
  è il driver del costo marginale per-worker. **Citazione corretta
  (A-BB64 + KB-91-1/2, Concilio WP-91): NOT-attributed, verdict-grade
  PREVIA ROBUSTEZZA — ora mostrata: 2σ-floor b_ret0 =
  16.691.057 B = 15,92 MiB [derivata: companion /1048576] ≥ soglia
  DICHIARATA 0,8·b_base; stimatore anti-moda W8 insensibile (ratio
  1,019); tie-alt immateriale. Il label del g3 «verdict-grade … grade
  inherits the slope grades above» va letto ADVISORY-inherited (se
  eredita, eredita il min dei bracci: b_ret0 è ADVISORY).** KL-90-4
  (census per-theap + braccio retain in-band) resta condizione
  NECESSARIA, non sufficiente. Il driver
  residuo resta APERTO, da nominare al Concilio WP-91 (candidati
  residui Leijen: pagine per-theap PARZIALMENTE usate — W×bins×pagina —
  pagine abandoned con blocchi vivi, minimal_purge_size ord 44).
- **VSWEEP — A-BB59, spans GIUDICATO (A-SK58)**: cal byte-riprodotte
  per la QUARTA campagna consecutiva (7.801.102 B entrambi i lati,
  floor_inc 1.161.206 B). dt=0 OVERLAP obbligatorio ✓, dt=20
  NO-OVERLAP ✓ e **P-DT20 CONFERMATA: net==cal AL BYTE 4/4 lati**
  (zero-swallow riprodotto). 🔵 **REGIME NUOVO ai dt intermedi**: a
  dt∈{1, 2} ms (entrambi gli ordini) e dt=5 ms (solo afirst) il net del
  lato SECONDO arriva **clamped=1** (deflazionato sotto zero dal
  teardown/purge del primo dentro la finestra del secondo) — fisica mai
  entrata in m88; la transizione sta fra 5 e 10 ms. Run clamped
  DICHIARATI ed esclusi dal tally (nets sweep comunque VOID per-thread,
  KB-88-1). **Discriminatore ordine-vs-fixture: UNSTABLE (order-match
  2/3, padA-side 2/3, regime=OVERLAP-only in-band A-BG52)** — il
  surplus non segue né l'ORDINE puro né il FIXTURE puro:
  timing-attached, classe m87/m88 confermata anche a ordine invertito.
- **VUCLOG**: DECLARED-ABSENT (A-DS45 consumata in m88; positivo
  ≥1-coppia in F16b battery).

## Aperture dichiarate (per NOME)

1. **Attribuzione di b, iterazione 2**: retain full-page ESCLUSO a
   macchina; b resta reale su questo protocollo —
   19.575.603 B = 18,67 MiB [derivata: companion /1048576] per worker,
   coerente con l'ordine di grandezza m87/m88. Prossimi bracci
   discriminanti: census per-theap a worker VIVI (collect in-request,
   non post-teardown), pagine parzialmente usate per bin, ord 44.
2. **Regime clamped dt 1-5 ms**: il canale process-counters sotto
   stagger parziale produce net negativi clampati — serve il net
   per-thread (A-BB50, design87) per ogni cifra in quel regime.
3. **Discriminatore surplus**: UNSTABLE anche con ordine invertito —
   l'attaccamento è di timing fine (sub-ms), non di ordine di sparo.
4. **A-BB50/A-DL39**: invariati (design87); A-MS27/A-PP18/A-PP27/
   A-AH38 invariati (backlog).
