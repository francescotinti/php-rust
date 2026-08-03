# MEASURE90_RESULTS.md — misure S-90.0 nelle FORME ordinate dal Concilio WP-91

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

## Verdetti (da verdict90.a1.g2.out)

- **VLADDER — la PRIMA attribuzione ESATTA di b (braccio 2 Leijen,
  scala Δcommitted su contatore monotono)**: scala monotone 20/20;
  additività IN fascia:
  **b_peak = 19.723.059 B = 18,81 MiB [derivata: companion /1048576] per worker**
  (metric=committed_postcollect_win0, PEAK; se(b) = 447.220 B,
  2σ = [18.828.619, 20.617.500] B — sovrapposto al b m89: stessa
  grandezza su protocollo iterato) =
  **b_boot = 2.252.800 B = 2,15 MiB [derivata: companion /1048576] per worker**
  (se = 6.345 B, 2σ = [2.240.109, 2.265.491] B — quasi deterministico) +
  **b_work = 17.276.928 B = 16,48 MiB [derivata: companion /1048576] per worker**
  (se = 501.348 B, 2σ = [16.274.233, 18.279.623] B) +
  residuo post-work 193.331 B = 0,18 MiB [derivata: companion /1048576]
  (segmento self+census finale, IN fascia 0,15·b).
  **Il driver di b NON è il boot dei worker (b_boot sopra): è la
  crescita di commit della FASE DI LAVORO (b_work sopra)** — prima
  scomposizione per fase mai prodotta su questo protocollo. Marginali
  9/9 IN fascia, OUT count=0 per NOME su tutte e tre le lane (A-BG56);
  robustezza in-band (A-BB62): b_minR ratio 1,000. ⚠️ A-BB58 sul
  braccio PEAK: tutti e 4 i punti hanno modes==R (tie=4 risolti
  tiebreak=min, DICHIARATI in-band A-BB61) ⇒ punti census-ADVISORY —
  il residuo post-work variabile (census/self) è entrato nella metrica
  exit; le lane BOOT/WORK (win=9, pre-census-finale) restano le lane
  forti dell'attribuzione.
- **VSELF — split per-worker REFUSED 20/20 (A-DL31)**: mi_heap_of su
  mimalloc v3 restituisce lo STESSO heap per tutti i worker (collisione
  di puntatore rilevata dal dente onestà heap=<ptr>); il census SELF
  per-worker legge UN heap W volte ⇒ la decomposizione per-worker resta
  APERTA per il concilio (limite STRUTTURALE del canale, dichiarato
  ex-ante nel header — non un FAIL di campagna).
- **VCOV (KL-91-3)**: il census al picco copre in media
  158.462.223 B = 151,12 MiB [derivata: companion /1048576] su
  275.198.771 B = 262,45 MiB [derivata: companion /1048576] di commit —
  ratio 0,576 < 0,9 ⇒ **la metà census dell'attribuzione è ADVISORY**
  (dichiarato; mai più un census muto benedetto — la metà ESATTA è la
  scala VLADDER). Nota: al picco coi worker vivi il census vede ratio
  0,576 del commit, contro il census post-teardown m89 che ne vedeva
  meno di un centesimo (Leijen WP-91 Q1).
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
  A-DL50 (scala Δpurged per-request): DECLARED-unavailable su questo
  canale — request_collect_mi vive sul path CLI/metro, non sul worker
  axum (wiring = candidato WP-92); purged cumulativo a exit riportato
  per record su ogni raw clamped.
- **VDISP/VARMS90/VORD90/VCKPT**: thr-set esatto con conteggio
  nreq/W + 1 self sulle slope; read-back purge_delay (env+ord15) col
  valore del braccio su OGNI raw; clamped==0 fuori sweep; checkpoint
  contati per NOME (pboot/pwork/peak ==1/1/1 win=9; exit_mi/
  exit_collect_mi ==1/1 win=0 — forma reale del canale, g2).

## Aperture dichiarate (per NOME)

1. **Decomposizione per-worker al picco**: canale strutturalmente muto
   (VSELF collision + subproc-visit cieco sui theap vivi) — servono API
   diverse (candidati WP-92: visita heaps registrati, censimento at-free
   per thread, o wiring request_collect_mi sul worker path).
2. **Driver di b_work (la componente di fase-lavoro sopra)**: attribuito
   alla FASE, non ancora al DETENTORE (size-class/heap) — la metà
   census resta ADVISORY (coverage 0,576); prossimo braccio: census
   at-peak con canale per-worker riparato.
3. **Clamp dt intermedi**: refutata la purge-al-free (P-CLAMP-PD) — il
   sospetto passa al bracket/counter del net (A-BB50 net per-thread,
   design87, torna prioritario).
4. **Residuo post-work 193.331 B/worker**: allocazioni census/self nel
   segmento [pwork, exit] — piccolo e IN fascia, dichiarato.
5. **A-BB60/A-PP49/A-BB50/A-DL39**: design invariati.
