# MEASURE84_RESULTS.md — misure S-84.0 nelle FORME ordinate dal Concilio WP-85

Campagna decisiva per la delibera peak (§Sintesi WP-85 punto 5). Cifre di
memoria BYTES-FIRST (A-DL26/KL-85-2). Verdetto macchina:
`wp84-harness/verdict84.out` (VERDICT84 PASS, fail-closed).

## Identità

- git campagna: 937e79d (nessun commit mid-campaign, KG-84-2 per-run)
- battery-84pre: PASS 15/15 a 937e79d per NOME (`wp84-battery-out/`,
  fuori repo) — corpus Zend 1418 IDENTICO + refl 290 IDENTICO per NOME
  sul binario di parità nuovo
- binari: phpr parità c4448075401dee5f (stash additivo `phpr-wp84`);
  union axum d440c3411c12401a; census d70b86d0502ea7e7; mem-census
  85fd009f66e7d3e4 — ognuno ENFORCE contro la riga matrix per-run
- matrix: rigenerata in battery (prima voce), ULTIMA pre-campagna,
  git=937e79d
- NESSUNA fase slope (VC resta CHIUSA verdict-grade ~55–75×, A-BB37);
  vincoli della prossima slope: A-PP28 `w=` in-band (cablato nel binario),
  A-BG33 (base per-request via driver), A-BG35 (regime nel verdict)

## Verdetti (da verdict84.out)

- **VDL24 — PER-THREAD (KL-85-1 soddisfatta, la misura che DECIDE)**:
  hello-only W=2, una richiesta per worker distinto, righe
  `unitcache_main_entry` PER THREAD al teardown (thr= e arm=axum-worker
  in-band, alloc_id su ogni riga): thr0 net(ord1) = 7.349.977 B = 7,01 MiB
  E thr1 net(ord1) = 7.349.977 B = 7,01 MiB — il secondo worker RIPAGA
  integralmente il primo lower (identico al byte, e identico all'ord1
  di campagna-83: il residuo one-time è di THREAD — prelude/interner
  thread-local — non di processo). **La formula del budget ×W REGGE:
  rw_budget × W** (20.648.477 B = 19,69 MiB per worker, dal record
  WP-83). Il gap col marginale fisico — 3.605.572 B = 3,44 MiB/worker [derivata: (243.204.096+243.367.936+243.236.864 − 198.868.992−211.861.504−210.911.232) B / 30; record S-82.0 VP, /usr/bin/time -l peak RSS W=10 R=3] —
  resta materia A-DL27 (addendi opachi) — sorgenti DIVERSE, mai «⇒»
  (A-DL25). Sanatoria KL-85-2 (Concilio WP-86): la v1 diceva «3,44
  MB/worker, vmmap» — sorgente errata (è il VP time -l) e senza byte.
- **VP — A-BB34 rerun: pin MOSSO, esito NOMINATO (mai silente)**: peak
  W=10 R=3 post-partizione:
  228.278.272 B = 217,7 MiB · 239.878.144 B = 228,8 MiB ·
  240.287.744 B = 229,2 MiB — FUORI dal pin 232±1 MiB di S-82.0
  (che era 232/232/232, spread 0%). KB-85-2 è soddisfatta DALLA
  riesecuzione; la delibera peak usa QUESTI valori. Lettura candidata
  (non deliberata): la partizione + i denti S-84.0 hanno mosso il
  layout; lo spread è tornato (r1 sotto degli altri due di
  12.009.472 B = 11,5 MiB [derivata: 240.287.744−228.278.272]) — il pin
  identità 0-spread di S-82.0 non è più riproducibile così com'era;
  giudizio al Concilio WP-86.
- **VA2 — PASS, il VOID di WP-83 è SANATO (KS-DS-85-1 sollevata)**:
  ogni fixture contata porta full-body vs oracle su arm UNION alla
  STESSA rev, LEDGERATO PRIMA della finestra contata
  (`wp84-harness/evidence/fixture-oracle.ledger`, 5 righe PASS a
  937e79d). Steady a3==0 su tutti e tre i bracci; derivate ora
  verdict-grade: [derivata] register share = +4,0 call/req; include-HIT
  share ≤ +52,0 call/req (upper bound vs opcache inheritance-cache,
  A-DS25/A-BB33) — le stesse cifre della campagna-83, stavolta con
  l'oracolo che le sorregge.
- **VW2 — PASS: modello piecewise CONFERMATO alla bisezione, sito
  NOMINATO (A-BB38 pin ARMATO, KB-85-3 soddisfatta)**: il sito è il
  fallback heap dello stack-buffer di std (`run_path_with_cstr`,
  MAX_STACK_ALLOCATION=384 — costante di libreria già in allowlist),
  raggiunto dai DUE syscall path di `main_unit_key`
  (fs::canonicalize + fs::metadata): ogni conversione path→CString
  alloca len+1 byte su heap quando len ≥ 384. Bisezione ESATTA:
  len=383 → a_calls=2,0 / a_bytes=766 == 2×len; len=384 → a_calls=4,0 /
  a_bytes=1538 == 2×len + 2×(len+1); controllo hello len=98 →
  2,0 / 196. Modello pinnato: a_calls = 2 + 2·[len≥384],
  a_bytes = 2·len + 2·(len+1)·[len≥384]. Il vecchio 2×len è RITIRATO
  (sostituito dal piecewise; A-SK34: nessun gate lo codifica più).
- **Guardia KS-PP-85-1**: 0 righe reqns senza `w=1` nei raw di campagna
  (nessuna fase slope eseguita).

## Aperture dichiarate (per NOME — mai chiusure in silenzio)

1. **Delibera peak ×W**: le DUE precondizioni del Concilio WP-85 sono
   soddisfatte (A-DL24 esito PER-THREAD; rerun A-BB34 eseguito) MA il
   pin identità è MOSSO (da 228.278.272 B = 217,7 MiB a
   240.287.744 B = 229,2 MiB, contro il pin 232±1 MiB) — la delibera è
   del Concilio WP-86 sui valori nuovi.
2. **A-DL27**: gli addendi di rw_bytes−bytes_counted restano opachi; il
   gap 7.349.977 B = 7,01 MiB (net per-thread) ↔ 3.605.572 B = 3,44 MiB/worker [derivata: v. VDL24 sopra, sanatoria KL-85-2] (marginale fisico, S-82.0 VP time -l)
   è la stessa domanda vista da due sorgenti.
3. **A-PP18**: APERTA, NON ingaggiata in questa campagna (nessuna
   riconciliazione Δglobal a W>1: la VDL24 legge net per-entry
   depositati al publish per-thread, richieste sequenziali; A-PP24
   onorata).
4. **Slope futura**: A-BG33/A-BG35 vivono nel prossimo verdict slope;
   A-PP28 è già nel binario (riga reqns porta `w=`).
5. **Registry condivisa**: SOLO con A-BB35 + riapertura ESPLICITA
   KH81-3 (KH84-4) — ora che A-DL24 dice PER-THREAD, la frazione
   condivisibile è materia concreta per il Concilio WP-86 (walk 69,1% /
   footprint 66,1% [derivata: net(ord1)/rw_budget e 1−1/2,95, A-DL25]).
