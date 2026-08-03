# Verbale sedia 9 — Brendan Gregg (metodologia di misura, attribuzione, leggibilità dai ledger)

VERDETTO: **CONCORDO CON EMENDAMENTI** (nessuna refutazione capitale: ogni cifra del doc ricomputata a macchina dai raw e dal g2 — additività b_peak=b_boot+b_work+residuo esatta al byte 19.723.059=2.252.800+17.276.928+193.331; medie VCOV 158.462.223/275.198.771 riprodotte al byte dai 20 raw; 34 raw=20+4+10 contati nel ledger; marginali 9/9 IN per NOME; battery 4 tentativi tutti ledgerati).

## Q1 — Il doc riporta esattamente il g2? QUASI: un conteggio ADVISORY è sotto-riportato
Il g2 marca `[A-BB58: modes==R, point ADVISORY]` su **7 punti**: PEAK W4/8/12/16 **e WORK W8/12/16** (tie=4, tiebreak=min). Il doc nomina solo i 4 del braccio PEAK e chiama BOOT/WORK "le lane forti". La scala Δcommit di VSLOPE-WORK resta esatta, ma il censimento per-modo della lane WORK è degenerato su 3/4 punti: il conteggio per NOME nel doc deve dire 7, non 4, e la dicitura "lane forti" va scopata alla sola scala Δcommit.

## Q2 — VCOV media-su-20 nasconde variazione per W? SÌ, misurata
Il giudice fa ratio delle medie pooled (vis/n)/(tot/n). Per-W (ricomputato dai raw): **0,415–0,421 (W4) → 0,545–0,555 (W8) → 0,567–0,626 (W12) → 0,634–0,650 (W16)**. Lo 0,576 non esiste in nessun raw. Peggio: la massa invisibile è dominata dall'INTERCETTA (~74 MB fissi + ~3,9 MB/worker); la **coverage MARGINALE** (pendenza di vis su W) è 15.781.205 B/worker ≈ **0,80 di b_peak** — per l'attribuzione di b il census è molto migliore di quanto lo 0,576 racconti. Un solo numero pooled sbaglia in entrambe le direzioni.

## Q3 — Storia g1→g2 leggibile dal solo ledger? NO
Il campaign ledger registra judge_sha per generazione e supersede_of=g1 (bene), ma `reason=see-verdict-file` (g1) e `reason=all-blocks-clean` (g2) sono puntatori, non ragioni: per sapere COSA è stato riqualificato (VCKPT sweep want 1/3→1/1) bisogna aprire i verdict file e diffare i blob dei giudici. "attempt=1 PULITO" regge SOLO perché entrambi i ledger portano le righe sporche (ABORT operatore a 8da340c nel battery-ledger; esito=FAIL g1 nel campaign ledger): la dicitura è legale ma va sempre citata in coppia con quelle righe, scope = fasi di misura.

## Q4 — commit_at ultima-riga: dove può mentire?
Il conteggio ==1 di VCKPT copre i nomi giudicati (ultima=unica). Ma nell'awk `v` NON è resettato per riga: una riga mi_proc che matcha win/ckpt ma è PRIVA di commit= eredita silenziosamente il commit della riga precedente. Verificato: 0/20 raw m90 colpiti (misscommit=0) — latente, non attuale. Inoltre VIS in VCOV usa regex-substring (/win=9/) non match di campo esatto. Sul clamp: nei 10 sweep **clamped ⇔ spans=OVERLAP 10/10** — legge osservata che il doc non dichiara; il "3/4 con pd=1000" è confuso: l'unico run senza clamp è anche l'unico NO-OVERLAP (negativo non controllato). La refutazione di P-CLAMP-PD regge, e il 10/10 la RAFFORZA verso bracket/counter.

## Emendamenti
- **A-BG57 "VCOV per-W + coverage marginale"**: il giudice riporti la tabella per-W e la pendenza di vis (coverage del margine), mai il solo pooled.
- **A-BG58 "reason= autosufficiente"**: righe verdict/supersede con reason=requalify:<blocco>:<old→new>; il ledger da solo deve rigenerare la storia g(n)→g(n+1).
- **A-BG59 "commit_at reset per riga"**: azzerare v a ogni riga mi_proc; riga matching senza commit= = FAIL, mai ereditare.
- **A-BG60 "ADVISORY per NOME=7"**: doc emendato: 4 PEAK + 3 WORK; "lane forti" scopata alla scala Δcommit.
- **A-BG61 "legge clamp⇔overlap"**: dichiararla 10/10 e progettare il negativo controllato (pd=1000 CON overlap forzato).

## Kill-switch
- **KS-BG-92-1**: VCOV citata senza tabella per-W ⇒ la metà census non è citabile nemmeno come ADVISORY.
- **KS-BG-92-2**: riga verdict il cui reason non ricostruisce la riqualifica senza aprire file esterni ⇒ supersede invalido alla campagna successiva.
