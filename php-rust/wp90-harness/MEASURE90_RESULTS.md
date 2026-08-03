# MEASURE90_RESULTS.md — misure S-90.0 nelle FORME ordinate dal Concilio WP-91 — EMENDATO in S-91.0 (gradi deliberati dal Concilio WP-92, sanatoria p1)

Campagna measure90 (§Sintesi WP-91 p5): ATTRIBUZIONE di b, ITERAZIONE 2,
su metrica RIQUALIFICATA **PEAK** (A-DL48/KL-91-1) — scala Δcommitted
per FASE (contatore monotono ⇒ Δ esatti) + census al picco con coverage
dichiarata (KL-91-3) + esperimento clamped unificato (A-BB63+A-DL50).
Verdetto macchina: `wp90-harness/verdict90.a1.g2.out` (VERDICT90 PASS,
attempt=1, **generazione MASSIMA g2** — g1 SUPERSEDED in-band con
judge_sha risolvibile a blob committato per ENTRAMBE le generazioni,
A-BG53/KS-AH-91-2). Soglie dichiarate nel header del giudice E della
campagna (KB-91-2): fascia δ=0,15 · coverage 0,9 · robustezza 0,8 ·
pd discriminante 1000.

**EMENDAZIONE S-91.0 (sanatoria p1, Concilio WP-92 vincolante)**: i
GRADI delle cifre sono stati riqualificati dalla delibera del
team-misura e gli stimatori RIPARATI a macchina sui 30 raw già
committati (20 slope + 10 sweep, zero run nuove) — fonte:
`wp91-harness/repair90-estimators.out` (tether: l'estrattore nuovo
riproduce al byte le cifre del g2 PRIMA di riparare; estrattore
commit_at A-BG59 fixed, campi per NOME esatto). Questo documento
riporta le cifre nei gradi deliberati; il g2 resta il verdetto
STORICO della campagna, con le label ritirate qui nominate.

## GRADI DELLE CIFRE (delibera team-misura WP-92, composta a strati)

- **b_boot = 2.252.800 B = 2,15 MiB [derivata: companion /1048576] per
  worker (se 6.345 B): VERDICT-GRADE come MAGNITUDINE** — modi dominanti
  reali (non min-statistic), quiescenza triviale al pboot, semantica del
  contatore convalidata. NON è verdict-grade come COMPOSIZIONE:
  «bootstrap theap + stato phpr» resta ipotesi finché non gira il
  braccio bare-thread (A-DL-56). Da citare come *quanto*, mai come *di
  che cosa*.
- **b_work = 17.276.928 B = 16,48 MiB [derivata: companion /1048576]
  per worker (se 501.348 B): ADVISORY** — due ragioni indipendenti:
  (a) KS-PP-92-1: la quiescenza al census NON è provata (HTTP-complete
  ≠ request_end-complete; manca il testimone outstanding=0, A-PP-63);
  (b) i marginali «9/9 IN fascia» sono estimator-dipendenti: con la
  MEDIANA, WORK W8→W12 = 22.183.936 B (OUT alto) e W12→W16 =
  13.271.040 B (OUT basso). Citabile SOLO con etichetta ADVISORY e
  banda; mai come cifra di verdetto.
- **somma b_boot+b_work = 19.529.728 B = 18,63 MiB [derivata: companion
  /1048576] per worker: ADVISORY** (eredita il grado della lane più
  debole) — è la FORMA di testata onesta (lane pre-census-finale). La
  continuità col b m89 (19.575.603 B, delta 45.875 B) è SUGGESTIVA,
  non verdict-grade: due cifre ADVISORY che si somigliano non fanno
  una conferma.
- **b_peak: DA RIFARE come testata (KS-BB-92-1)** — tutti e 4 i punti
  PEAK hanno modes==R: il «modo dominante» non esiste e tiebreak=min
  è una statistica d'ESTREMO (a W=12 il min non rompe un tie:
  SELEZIONA la run outlier w12.r3). Il valore min-based 19.723.059 B
  resta nel g2 come storia; ripubblicato a MEDIANA (A-BB64):
  **b_peak(mediana) = 20.289.946 B = 19,35 MiB [derivata: companion
  /1048576] per worker** (se 1.084.655 B, banda 2se dichiarata
  [18.120.635, 22.459.256] B). Lo stimatore pooled per-run (n=20,
  df=18, CI onesto) è pubblicato in repair90-estimators.out.
- **ADVISORY per NOME = 7, non 4 (A-BG60)**: PEAK-W4/W8/W12/W16 E
  WORK-W8/W12/W16 (tutti modes==R nel g2). La dicitura «lane forti»
  è SCOPATA alla sola scala Δcommit di BOOT/WORK — il censimento
  per-modo della lane WORK è degenerato su 3/4 punti.
- **CI onesti (A-BB68)**: con 4 punti (df=2) «2σ⇒95%» è falso
  (t(0,975;2) = 4,303 ⇒ la banda 2se copre ~82%). Tutte le bande di
  questo doc sono dichiarate «2se»; le bande t sono in
  repair90-estimators.out (pooled df=18: t = 2,101).
- **Robustezza non tautologica (A-BB66)**: il vecchio ratio min-of-R
  ==1,000 era VOID per costruzione (min≡dominante con tie totale —
  KS-BB-92-2). Robustezza REALE misurata: min/mediana = 0,972 ·
  media/mediana = 0,999 (soglia 0,8: entrambe IN).
- **Run-outlier per NOME (A-BB67)**: w12.r3 E w4.r1 — TUTTI i
  contatori oltre 2·MAD dalle sorelle; nessuna delle due può diventare
  il punto della sua W via min (il residuo W4 nasceva proprio da
  PEAK-min che pescava w4.r1).

## Identità

- git campagna: bb4b388 · **attempt=1 PULITO** (nessun VOID in-run; g1
  FAIL del GIUDICE — pin VCKPT sweep calibrato sulla lettera di design
  anziché sul canale reale, requalificato in g2 con giudice COMMITTATO
  PRE-giudizio: il self-tether A-BG53 rifiuta giudici non committati)
- battery-90pre: **PASS (16/16 CONTATO) a 4c99520** (4 tentativi TUTTI
  ledgerati: ABORT a1 a 8da340c per errore operatore head-moved — commit
  non-evidence con battery in volo, lezione a verbale; FAIL a2 1/16 sul
  dente budget A-SK61 morso dal proprio harness; REFUSE a3 porcelain su
  matrix orfano; PASS a4); consumo **--same-rev v6 + A-AH50/A-AH54** e
  consumazione LEDGERATA (A-AH57: riga phase=consume con checker_sha)
- binario mem-census 73e6d13f048d1610 ENFORCED contro matrix committed
  (tether ANCHE lato giudice, A-SK59); identity v2 con **srv_boot_epoch
  dal PROCESSO (ps lstart — A-BG55)** + preflight=ok in-band; pid-echo
  prima request ASSERITA post-wait_up (A-PP53); ckpt NOMINATI su ogni
  riga census (A-BG54/KG-91-2: estrattori per nome, mai posizionali)
- **VPEAK (A-DL48/KL-91-1): commit==peak_commit su OGNI riga mi_proc di
  OGNI raw** — la premessa PEAK-metric regge sull'intera campagna

## Verdetti (da verdict90.a1.g2.out, nei GRADI emendati; stimatori riparati in repair90-estimators.out)

- **VLADDER — attribuzione di b per FASE (braccio 2 Leijen, scala
  Δcommitted su contatore monotono)**: scala monotone 20/20;
  additività WITHIN-RUN esatta 20/20 (identità aritmetica
  pboot+Δwork+Δpost==exit — A-BB65). Decomposizione nei gradi sopra:
  **b_boot** (VERDICT-GRADE come magnitudine) + **b_work** (ADVISORY) =
  somma ADVISORY 19.529.728 B (vedi §GRADI); b_peak DA RIFARE
  (ripubblicata a mediana). **Il driver di b resta la FASE DI LAVORO**
  (b_work sopra) — l'ordinamento boot≪work è robusto a TUTTI gli
  stimatori (mode/mediana/media/pooled), anche se la cifra di b_work è
  ADVISORY.
- **Residuo cross-lane: SELECTION MISMATCH, non fisica (A-BB65)** — la
  label del g2 «the residue is the post-work segment: self+final census
  allocs» è RITIRATA: refutata dai raw (Δcommit post-work = 0 su 17/20;
  non-zero solo w4.r5 +5.046.272 B, w16.r2 +1.048.576 B, w16.r3
  +5.177.344 B — il segmento self+census a commit costa ZERO nel caso
  tipico). Il residuo 193.331 B misura il mismatch fra punti scelti da
  RUN DIVERSE per lane; per-W (est=mode_min): W4 −2.097.152 B · W8 0 B
  · W12 262.144 B · W16 393.216 B — dominato dal W4 dove PEAK-min
  pesca la run outlier w4.r1.
- **VSELF — split per-worker REFUSED 20/20 (A-DL31)**: mi_heap_of su
  mimalloc v3 restituisce lo STESSO heap per tutti i worker (collisione
  di puntatore rilevata dal dente onestà heap=<ptr>); il census SELF
  per-worker legge UN heap W volte ⇒ la decomposizione per-worker resta
  APERTA (limite STRUTTURALE del canale, dichiarato ex-ante nel header
  — non un FAIL di campagna). Meccanismo spiegato dal Concilio WP-92
  (Leijen): TLS del CHIAMANTE su heap statico ⇒ la via è il census
  ON-THREAD (A-DL-52).
- **VCOV per-W (A-BG57/KS-BG-92-1 — la tabella è la forma citabile; il
  pooled 0,576 non esiste in NESSUN raw)**:
  | W | ratio min | ratio max | ratio medio |
  |---|---|---|---|
  | 4 | 0,415 | 0,421 | 0,416 |
  | 8 | 0,545 | 0,555 | 0,551 |
  | 12 | 0,567 | 0,626 | 0,585 |
  | 16 | 0,634 | 0,650 | 0,646 |
  **Coverage MARGINALE (pendenza di vis su W): 15.777.004 B/worker =
  0,778 di b_peak(mediana)** — per l'ATTRIBUZIONE di b il census è
  molto migliore di quanto il pooled racconti: la massa invisibile è
  dominata dall'INTERCETTA (71.934.811 B fissi + 4.480.174 B/worker).
  La metà census resta ADVISORY (ratio < 0,9 dichiarata), ma con la
  tabella per-W e la coverage marginale a verbale.
- **VSWEEP90 — cal**: 7.801.102 B = 7,44 MiB [derivata: companion /1048576]
  su ENTRAMBI i pad, byte-riprodotte r1==r2 — **QUINTA campagna
  consecutiva AL BYTE, pur col binario census CAMBIATO**
  (A-MS50/ckpt/self: l'ancora è del protocollo, non del binario).
  **P-DT20 CONFERMATA: net==cal AL BYTE 4/4 lati** (zero-swallow).
- **VSWEEP90 — P-CLAMP-PD REFUTATA (esito ex-ante alternativo)**: con
  MIMALLOC_PURGE_DELAY=1000 ≥ finestra (read-back ord15 in-band su ogni
  raw) il clamp PERSISTE su 3/4 run dt∈{1,5} ⇒ la deflazione dei dt
  intermedi NON è il decommit da purge-al-free dentro la finestra
  dell'altro lato: **la colpa è del bracket/counter** (ramo alternativo
  di Bak Q3, ora selezionato a macchina). Il meccanismo puntato da
  Leijen Q4 (arena purge sincrona) è ESCLUSO come driver del clamp.
  **LEGGE clamp⇔overlap (A-BG61, dichiarata a macchina): 10/10 sweep —
  clamped_rows>0 SE E SOLO SE spans=OVERLAP** (l'unico run pd1000
  senza clamp è anche l'unico NO-OVERLAP del suo braccio: negativo NON
  controllato — il negativo controllato, pd=1000 CON overlap forzato,
  è A-DL-54). La legge RAFFORZA l'attribuzione al bracket/counter.
  A-DL50 (scala Δpurged per-request): DECLARED-unavailable su questo
  canale — request_collect_mi vive sul path CLI/metro, non sul worker
  axum (wiring = canale iterazione 3); purged cumulativo a exit
  riportato per record su ogni raw clamped.
- **VDISP/VARMS90/VORD90/VCKPT**: thr-set esatto con conteggio
  nreq/W + 1 self sulle slope; read-back purge_delay (env+ord15) col
  valore del braccio su OGNI raw; clamped==0 fuori sweep; checkpoint
  contati per NOME (pboot/pwork/peak ==1/1/1 win=9; exit_mi/
  exit_collect_mi ==1/1 win=0 — forma reale del canale, g2).

## Aperture dichiarate (per NOME)

1. **Decomposizione per-worker al picco**: canale strutturalmente muto
   (VSELF collision + subproc-visit cieco sui theap vivi) — la via
   deliberata dal Concilio WP-92 è il census ON-THREAD alla barriera
   (A-DL-52, con testimone outstanding=0 A-PP-63: la barriera È il
   testimone), fallback heap espliciti per worker.
2. **Promozione di b_work a verdict-grade**: tre condizioni CONGIUNTE
   (delibera team-misura): riga pwork con outstanding=0 (A-PP-63) E
   stimatore non-tautologico (A-BB64/66) E census per-worker on-thread
   senza collisione heap ptr (A-DL-52/KS-DL-92-1). Due su tre lasciano
   b_work ADVISORY.
3. **Composizione di b_boot**: braccio bare-thread (A-DL-56) per
   separare thread-runtime da stato phpr — senza, b_boot resta un
   *quanto* senza *di che cosa*.
4. **Clamp dt intermedi**: refutata la purge-al-free (P-CLAMP-PD); la
   legge clamp⇔overlap 10/10 punta al bracket/counter del net (A-BB50
   net per-thread, design87, prioritario) — negativo controllato =
   A-DL-54 (pd=1000 CON overlap forzato).
5. **A-BB60/A-PP49/A-BB50/A-DL39**: design invariati.
