# Rotta WORDPRESS-FIRST — WP-track (dopo WP-69: ⚡ **DEBITI DEL CONCILIO CHIUSI + FEDELTÀ DECLARE** — 🔴 S-69.1 demozione-a-cascata FIXATA defer-side (`demote_unbindable_chains` a fixpoint: Zend early-binda solo catene INTERAMENTE bindabili; s69-deep BYTE-ID con shadowing) + famiglia REDECLARE oracle-pinned (riserva first-wins PER-STATEMENT via span; `Cannot redeclare {kind} {name} (previously declared in {file}:{line})` col nome canonico del VECCHIO; FatalAt con Stack trace 8.5 dietro `fatal_error_backtraces` ini nuova, runner default Off come run-tests; escaping html fedele) — **corpus RI-PIN 1421→1422 con PROVENANCE (−2 fail REALI chiusi, +3 ex-skip runnable su gap eval-naming pre-esistente)** · KH69-2 SCATTATO: ciclo `&$a`/`$a[0]` panicava (BorrowMutError) ⇒ H-69.2 try_borrow_mut a semantica dichiarata, h69-cycle byte-id · M-69.1 pending_unit_call ELIMINATO (parametro), M-69.2, H-69.3 assert fold==mint al link, K-69.4 test cargo (1649/0), K-69.3 contatore corpi caldi baseline 178 · **L-68.1 forma DOPPIA eseguita: nk OFF 0,0000 (engine ≈0 sul fixture); counter censusown attribuisce 90,7% del 62,6 ON; 🔴 KL69-1 SCATTA sulla run PULITA (HITS-off+cron-free): residuo VERO ≈2,0-2,2 KiB/req su wpdev ⇒ FRONTE AXUM FERMO finché attribuito** · **B-69.5 cron ISOLATO** (vivi steady 528; Δconfine +498 una-tantum; 4 miss-fp version.php una-tantum = firma per B-67.4; B-69.2: fp context-sensitive PER COSTRUZIONE — salt = entry script) · spike: predictions69 LOCKATE prima (b91b4799), profilo a WP-70 su outlined da costruire (B-69.4/KG69-2 conformi) · GATE69 PASS fails=0 run2 (run1 FAIL(1) onesto ⇒ ri-pin sentinelle65/redecl) · stash phpr-wp69 c5b1e80b…, tree 76adc8c)

> ⚡ **WP-69 (2026-07-28 sera, `f663765`→`76adc8c`)** — sintesi a 9
> recepita INTEGRALE in `wp69-harness/design69.md` PRIMA del codice;
> predictions69 lockate PRIMA di profilo e letti (b91b4799, 07ef494).
> S-69.1 chiusa DEFER-SIDE con cascata a fixpoint + famiglia redeclare
> (4 forme pre-esistenti) oracle-pinned; KH69-2 scattato e chiuso
> (H-69.2); L-68.1 doppia: KL69-1 SCATTA su run pulita (residuo vero
> ~2 KiB/req, axum FERMO); cron isolato (P-68.4 confermato+isolato);
> spike ipotecato (KG69-2). GATE69 PASS fails=0 (cargo 1649/0, corpus
> 1422 ri-pin, refl 290, ORM 3E/13F, hk 1665, reverse 2F per nome,
> batteria s69 9+6 e h69-cycle nel gate ricorrente).
> **Storia: `sessions/WP_SESSION_69.md`. WP-68:
> `sessions/WP_SESSION_68.md`.**

## 📁 Convenzioni (decisione utente 2026-07-23)

- Qui SOLO: sintesi ultima sessione · decisioni in vigore · stato gate ·
  prossimo lavoro · backlog. Storia: `sessions/WP_SESSION_<n>.md` (un file
  per sessione, con lezioni e verdetti; ≤WP-27: memoria + git history).
  Gap perf: `gaps/REPORT_GAP_<n>.md` = SOLO le misure della sessione n
  (correzione utente 2026-07-25: la vecchia forma cumulativa era una
  trascrizione sbagliata dell'intento); il trend cumulativo vive in
  `gaps/GAP_TREND.md` (file unico vivo, una riga per sessione).
- **Chiusura della sessione N = skill `chiudi-sessione`** (decisione
  utente 2026-07-26: la procedura NON si ripete nei prompt; vive in
  `~/.claude/skills/chiudi-sessione/SKILL.md`): bonifica processi →
  verbale/ri-stash binario di parità → rotazione (WP_SESSION_N ·
  REPORT_GAP_N solo se misurato, MAI cumulativo · riga GAP_TREND ·
  questa testa + stato gate + §WP-N+1) → memoria → CONCILIO 9 sedie
  (verbali vincolanti in wp<N+1>-harness/) → gh-status-sync se file
  pubblicati cambiati → report utente + prompt N+1 CORTO. Commit+push
  a ogni passo.

## 🧭 Decisioni in vigore (fonte citabile: migration/RULEBOOK.md)

- **Zero `unsafe` nel value core** (RULEBOOK §0; NaN-boxing WP-32,
  SSO-union WP-38 e stadio-2 registri WP-44 bocciati — non riproporre
  senza rotta esplicita utente).
- **ARCO REGISTRI CHIUSO allo stadio 1 (WP-44, verbale a TRE forme)**:
  l'infra dual-mode resta dormiente a delta zero (`PHPR_REG_LOWER` = pass
  vuoto); gli stadi 3-4 NON si aprono. Falsificati sia l'ibrido
  enum-operand (v1/v2) sia i raw-registers monomorfi (v3): il costo è il
  numero di corpi handler caldi nel run_loop. Il census WP-42 resta mappa
  valida (Ret→DerefTop 40,5M = call ABI); ogni riapertura deve RIDURRE o
  mantenere i corpi caldi (dispatch-table/token-threading = ipotesi DA
  MISURARE, spesso perde in Rust; ristrutturazione del loop), mai
  aggiungerne. La macchina di riscrittura (pass a finestre + remap
  totale, gate/corpus-proven) vive in `35ff89f`/`1e365db`/`f4c80cf`.
  🆕 ROTTA UTENTE 2026-07-28: autorizzato per WP-69 (post WP-68) uno
  SPIKE DISPATCH bounded a una sessione DENTRO la condizione WP-44
  (ridurre/mantenere i corpi caldi) — dettagli e kill-switch in
  §WP-68 punto 6; i registri-come-rappresentazione restano falsificati.
- Micro-bench solo advisory: verdetti SOLO su A/B interleaved stesso-giorno
  sul workload reale. Gate per NOME a ogni commit; refactor layout/GC =
  sentinelle drop-order pinnate PRIMA; oracle-probe con `-d log_errors=0`.
- Commit AND push a ogni step; deviazioni deliberate = marker
  `BUG(port):` / `PERF(port):` / `TODO(port):`.
- **⚖️ CONCILIO DOPO OGNI SESSIONE (regola utente 2026-07-26)**: a
  chiusura della sessione N si riunisce il concilio di TUTTI gli
  sviluppatori — **9 sedie**: le 6 fondative del piano Concilio
  (`FOOTPRINT_CPU_ROADMAP.md`) Hoare (design Rust safe) · Matsakis
  (ownership/aliasing) · Klabnik (spec/testabilità) · Hejlsberg
  (compilatori/interning) · Bak (V8/HotSpot: alloc-rate, code-cache) ·
  Pedersen (disciplina di confine) + le 3 aggiunte 2026-07-26 Leijen
  (allocatore) · Stogov (Zend/opcache) · Gregg (metodologia), più
  co-optati di dominio quando il tema lo richiede (es. axum/HTTP ⇒
  web-runtime). Mandato: REFUTARE il
  report N + il programma N+1, leggendo i file di rotazione e dove
  serve il codice. Ogni membro delibera VERDETTO + emendamenti R1..Rn +
  kill-switch; verbali INTEGRALI in
  `wp<N+1>-harness/COUNCIL_WP<N+1>_REVIEWS.md`, VINCOLANTI per
  design<N+1> (recepiti PRIMA di toccare codice). Se una sessione
  chiude senza concilio, il PRIMO atto della successiva è convocarlo.
  Modello: `wp62-harness/COUNCIL_WP62_REVIEWS.md`.

## Stato gate per nome (WP-69 `76adc8c`: **GATE69 a VERDETTO MACCHINA — PASS fails=0** su `wp69-harness/gate-out/` al 2° run — matrice INTEGRALE nello stesso run (K-68.2); run1 FAIL(1) archiviato .superseded (ri-pin sentinelle65/redecl `base.php:174` con PROVENANCE, effetto diretto del fix redeclare) — cargo **1649/0** (tot≥1649) · sentinelle 5 assi BYTE-ID · fixture: gate-ordering + s68 (5+3, 2 lati) + **gate-ordering-s69 (9 unit-fixture 2 LATI + 6 redecl, pins con PROVENANCE)** + **gate-h69cycle** + droporder + editmiss · sentinels65 (ri-pin redecl 2 righe) + KS-S6 + seed_prefix_short=0 · KE-e · P1 · sem doppio pin · **corpus 1422 IDENTICO (baseline wp69-harness/gate-baseline/corpus69.fails, PROVENANCE)** · refl **290** · ORM **3E/13F** · hk **1665 OK** · reverse **2F PER NOME**; release = **phpr-wp69 (c5b1e80b…, tree `76adc8c`; +0f7c814 census-only cfg-gated, parità invariata)** stash additivo accanto a wp68 (8eea807e…); census **phpr-memgc69 (f034e62a)** con PHPR_CENSUS_HITS off-switch + tag=censusown; probe verdict-file: gate-ordering-s69 PASS · probe69-l681 FAIL(3) INFORMATIVO (disposizioni per banda in design69 §9.5) · probe69-cron FAIL(1) informativo (§9.7, isolamento OK) · KL69-1 SCATTATO (residuo ~2 KiB/req = fronte axum fermo); ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-68: vedi `sessions/WP_SESSION_68.md`

## (storico) Stato gate WP-67 (`2b92c78`: GATE67 PASS fails=0 — cargo 1647/0 · corpus 1421 · refl 290 · ORM 3E/13F · hk 1665 OK · reverse 2F PER NOME; run55 triple 88 BYTE-ID ×3; stash phpr-wp67 c0f5cfff…; dettaglio: `sessions/WP_SESSION_67.md`)

- Gate56 verde (2026-07-26, tree `0943f58`): corpus **1421 IDENTICO** per
  nome (baseline = `wp55-harness/gate-out/corpus.fails`) · **refl 290
  IDENTICO** · cargo **1640/0** (nuovo test model-based
  `keyless_index_churn_matches_model`) · ORM 3484 **3E/13F fail-set
  IDENTICO** (16 nomi) · hk 1665 **0E/0F** · probe56
  order/tomb/numkey/dtor BYTE-ID new vs old (pinnate PRIMA del layout;
  order/tomb/numkey anche = oracle). Baseline corpus resta **1421**.
- Gate22 integrale (storico, su `60c7e04`): sess 28 · date 351 · probe
  gd/mysqli/media byte-id · http DIFF-set atteso (WP-14, 2 item) ·
  option 413/1061 · restapi 3508/15947 IDENTICI per nome COL conteggio.
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run45** (~/Claude/wpdev, WP-56, binario `0943f58`):
  30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi;
  idem run45-old su phpr-wp55); baseline resta
  `wp16-harness/full-out/run33-fails.txt` (⚠️ righe con prefisso
  numerico: normalizzare `s/^\d+\) //` prima del diff). Master-CPU
  **697,8s ≈11:38 = 2,06× NUOVO MINIMO** (−2,66% vs run45-old 716,9
  STESSA SERA — l'old replica run44 alla cifra: ambiente stabile; il
  riferimento residuo WP-40 699s è CHIUSO). Wall ~16 min = 1,4×.
  Archivi: `full-out/run45/`, `wp56-harness/full-old-out/`; storici
  run44 in `full-out/`, `wp55-harness/full-old-out/`.
  ⚠️ RSS: telemetria `.rss` in APPEND — max PER SEGMENTO, e il filtro
  orario da solo NON basta (le ore dei giorni prima passano il confronto
  stringa): prima delimitare il blocco della notte (ultimo time-reset),
  poi la finestra oraria (run38 1696 / run39 1945 / old 1700 = rumore).
- Suite phpt (misura): xsl 63/64 (da CWD root php-8.5.7) · tidy 44/45 ·
  asym 38/39. Suite phpt SEMPRE con path ASSOLUTO.

## Harness full-suite

```bash
"/Volumes/Extreme Pro/Claude/wp16-harness/run-full-detached.sh" phpr
# col daemonizer perl (double-fork+setsid) — il task-kill a 10' non deve
# raggiungere la run. MAI due gate22 insieme; uploads azzerati PRIMA di
# ogni full run; non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr>
# WP-49: per-collect log dietro gc-census: PHPR_GC_COLLECT_LOG=<path>
# WP-50: probe full-scan fine-run (census-only): PHPR_GC_EOR_FULL_COLLECT=1
# (harness: wp50-harness/{pipeline50,ab50a,ab50b,census50-media,
#  run-full-census50,run37,run37-old}.sh + ab50-report.pl; binario census
#  phpr-memgc50 in phpr-mem-target/release/)
# WP-51: wp51-harness/{run-full-census51,run-full-census51b,corpus51,
#  corpus51-diffs,run38,run38-old,ab51}.sh; binario census A+B
#  phpr-memgc51; stash A/B: phpr-wp51a (leva A) / phpr-wp51b (A+B).
# WP-52: wp52-harness/{daemonize.pl (SENZA chdir "/"),corpus52,
#  build-memgc52,run-full-census52,run40,run40-old,ab52,orchestrate52}.sh;
#  binario census phpr-memgc52; stash: phpr-wp52 (sha256 57607da3…).
#  orchestrate52.sh = catena sequenziale census→full-new→full-old→ab.
# ⚠️ launcher: MAI chdir "/" nel daemonizer per i gate phpt — cd root
#  php-8.5.7 DENTRO lo script (bug60771 scrive ./test.php nella CWD).
# WP-53: wp53-harness/{corpus53,build-opcensus53,census53,run41,
#  run41-old,ab53,ab53-report,orchestrate53}.sh; binari OP-census
#  phpr-op53 (phpr-op-target/) e phpr-op52 (phpr-op-target-old/, build da
#  worktree della ROOT repo + php-server/+Cargo.lock copiati a mano);
#  stash: phpr-wp53 (sha256 e75c5abb…). PHPR_OP_CENSUS=1 → dump STDERR.
# WP-54: wp54-harness/{run42,sample42,sample-media,census54,census54b,
#  gate54,ab54,run43,run43-old,orchestrate54}.sh + {attr,aggregate,
#  subtree}.py (parser alberi `sample` + stratificazione) + probe54.php;
#  binario gc-census phpr-gc54{,b} in phpr-op-target/; sample-out/ =
#  finestre win1-6 (full) + mwin1-3 (media) + attribution-table.txt;
#  stash: phpr-wp54 (sha256 e4341e77…). PHPR_GC_CENSUS=<path> → append.
# WP-55: wp55-harness/{build-memgc55,census55-media,gate55,run44,
#  run44-old,ab55,orchestrate55}.sh + probe55-{sem,dtor,concat}.php
#  (sentinelle append + probe O(n), baseline old in probe55-*-old.txt);
#  binario census phpr-memgc55 in phpr-mem-target/ (tree wp54);
#  census-media-out/memcensus.txt = tabella checkpoint Fase 3;
#  stash: phpr-wp55 (sha256 ecc04817…).
# WP-56: wp56-harness/{design56 (quota pin a),gate56,census56-media,
#  build-memgc56,run45,run45-old,ab56,orchestrate56}.sh + probe56-
#  {order,tomb,numkey,dtor}.php (sentinelle 4 assi, baseline old+oracle)
#  + probe56-concat-sites.php (quota `.=` per sito); binario census
#  phpr-memgc56 (tree NUOVO 0943f58) in phpr-mem-target/;
#  stash: phpr-wp56 (sha256 65466c64…).
# WP-57: wp57-harness/{design57.md (note+verdetti),run57-census-full,
#  run57-census-debug,census57-media,aggregate-concat57.pl,daemonize.pl}
#  + probe57-arrlive.php (validazione live-accounting); binari census:
#  phpr-op57 (dcb589ea…, phpr-op-target/) e phpr-memgc57 (41a21c65…,
#  phpr-mem-target/); dati: full-out/opcensus57-debug.txt (tabella `.=`
#  full), census-media-out/memcensus57.txt (arr esatto + arr_shape).
#  Coda di sessione (indagine panic): gate57.sh (baseline = gate-out
#  wp56), probe57-yieldfrom-stale.php (regressione fix e9a1679),
#  run57-census-debug.sh; trappola diagnostica ip>=len nei build census.
#  Stash NUOVO: phpr-wp57 (sha256 a5ae7d27…) — release col fix engine.
# WP-58: wp58-harness/{design58.md (quota su size misurate),gate58.sh,
#  build-memgc58.sh,census58-media.sh,ab58.sh,run46.sh,run46-old.sh,
#  orchestrate58.sh,daemonize.pl} + probe58-*-{old,new}.txt (sentinelle
#  5 assi: order/tomb/numkey/dtor/yieldfrom) + probe58-objlive.php
#  (validazione Ob.2); census-media-out/memcensus58{,b}.txt; binari
#  census phpr-memgc58 (4c9b8da3…) e phpr-memgc58b (bc47f237…, con Ob.2)
#  in phpr-mem-target/; full: full-out/run46/ + wp58-harness/full-old-out/.
#  Stash: phpr-wp58 (sha256 2d7efdf8…).
# WP-60: wp60-harness/{design60.md (criteri P1 pre-registrati + verdetti
#  P2-P4 + scoperta unit-cache),gate60.sh,run48-new.sh,run48-old.sh,
#  orchestrate60-evening.sh,build-memgc60.sh,probe60-listtests.sh,
#  census60-media.sh,census60-full.sh,gap60-media.sh,probe60-p4*.php
#  (+output -{oracle,new,wp58}.txt = sentinelle Stogov pinnate)};
#  out: gate-out/, full-{new,old}-out/ (run48, time -l), census-out/
#  (memcensus60-{listtests,media,full}.txt = seed split + unitpath +
#  ctx), gap-out/ (coppia media). Binario census phpr-memgc60
#  (d1c8dc81…, phpr-mem-target/). ⚠️ telemetria .rss di run48-new
#  agganciata al wrapper time (profilo assente, metro time -l intatto).
# WP-61: wp61-harness/{design61.md (census v2 deep pre-registrato),
#  build-memgc61.sh,probe61-listtests.sh,census61-full.sh}; binario census
#  phpr-memgc61 (9600a662…, phpr-mem-target/); out: census-out/
#  (memcensus61-listtests.txt = v2 attribuito; memcensus61-full.txt =
#  mappa FULL v2, run detached — flag census61-full.done). Batteria HTTP
#  php-server: scratchpad battery61.sh (pattern run-http.sh WP-9, porta
#  8080, docroot ~/Claude/wpdev/src, DB wp).
# WP-62: wp62-harness/{design62.md (recepimento sintesi a 9 + §5 verdetti
#  P0/M2 + §6 decision point),uploads-guard.sh (Gregg R7: backup-wipe|
#  restore, UPLOADS_GUARD_DIR per self-test),build-memgc62.sh,calib62.sh
#  (taratura ±10%),probe62-listtests.sh,census62-full.sh,gate62.sh,
#  probe62-relbase.php,ab62-media.sh,orchestrate62.sh (catena serale:
#  gap media→census full→run49 new/old, guard attorno)}; out: gate-out/,
#  census-out/ (memcensus62-{listtests,full}.txt con colonne prefix +
#  tag=unitcache/cachehit/netguard/reloc), calib-out/, ab-media-out/,
#  eve-out/ (run49). Binari: census phpr-memgc62 (105df139…,
#  phpr-mem-target/); diagnostico phpr-relb62 (97ed6469…,
#  phpr-relbase-target/, feature relbase-probe MAI parità).
#  uploads-backups/ = tar+manifest su drive esterno (guard).
# WP-63: wp63-harness/{design63.md (spec contratto v2 + raffinazione
#  link-time baking + §10 verdetti),probe63-fresh.sh (G1 n=6),
#  sentinels63.sh (matrice E2: KK1'+KS-S2 via PHPR_UNIT_CACHE_LOG),
#  probe63-p1.sh (P1-a..d via -S, docroot p1-doc/),gate63.sh (off|on|
#  flip → gate-out-<modo>/),census63-listtests.sh (KE2/B7/L3),
#  calib63-manysmall.sh (L1),build-memgc63.sh,orchestrate63.sh};
#  out: fresh-out/, sent-out/, p1-out/, census-out/ (memcensus63-lt-
#  {off,on}, memcensus63-full = mappa FULL a flag ON), eve-out/
#  (run50 + media + progress). Binari: census phpr-memgc63 (515b9760…,
#  phpr-mem-target/; tag=reloc + tag=compilens debuttano qui).
#  ⚠️ colonne prefixsum a flag ON dominate dalla mappa eager
#  TRANSIENTE (free fuori-seg): il giudice è net/net_tot, mai le
#  colonne. Env: PHPR_STUB_ELISION=0 torna al contratto v1;
#  PHPR_UNIT_CACHE=0 = cache-off diagnostico.
# WP-64: wp64-harness/{design64.md (recepimento sintesi a 9 + §4-A
#  predizioni K4'' + §4-B spec-sketch leva coi rischi WP-44 + §4-C esiti
#  catena + §9 verdetti),COUNCIL_WP64_REVIEWS.md,gate64-debts.sh (gate63
#  flip + assert KS-S6 alignhit),probe64-s4.php(+inc),build-memgc64.sh,
#  census64-listtests.sh,orchestrate64.sh (media→census full→run51
#  new/old, guard attorno),watch-sample64.sh (B4)}; out: gate-out-debts/,
#  census-out/ (memcensus64-{lt,full} CON colonne pfx_slotnames/pfx_fnvec/
#  sc + compilens map_ns/remap_ns), eve-out/ (run51), sample-out/.
#  Binario census phpr-memgc64 (6d6b518b…, phpr-mem-target/).
#  ⚠️ probe63-p1.sh EMENDATO (detector cache-off cerca `hit (intra|cross)`,
#  mai la sottostringa "hit" — collide con l'evento alignhit).
# WP-65: wp65-harness/{design65.md (recepimento sintesi 9 sedie + §5
#  predizioni K4'' bilaterali + §8 verdetti), COUNCIL_WP65_REVIEWS.md,
#  build-memgc65{,b}.sh, census65-listtests.sh (coppia log-off/on G-65.2),
#  census65-full.sh, sentinels65.sh (+sent-baseline/ = elide per-unit
#  PINNATI), probe65-kee.sh (KE-e stale-id via -S porta 8497),
#  probe65-p1.sh (detector ANCORATI, out-dir propria, porta 8495),
#  probe65-sem.sh (+sem-units/ = S-65.3 a 3 lati), gate65.sh (matrice
#  INTEGRALE K-65.1), orchestrate65.sh, run53-pair.sh (ordine invertito),
#  run54-pair.sh (coppia build-ADIACENTE hyg-vs-leva = il giudice del
#  costo leva), watch-sample65.sh}; out: census-out/ (memcensus65-{lt-
#  logoff,lt-logon,full} pre-leva + memcensus65b-full post-leva +
#  design65.sha256 = snapshot K4''), gate-out/, eve-out/ (run52/53/54),
#  sample-out/, kee-out/. Binari: census phpr-memgc65 (e2a2016a) e
#  phpr-memgc65b (02b7d9d2) in phpr-mem-target/; igiene-only phpr-hyg65
#  (b93a6e91) in phpr-wp65hyg-target/ (worktree: Cargo.lock copiato dal
#  vivo + --locked). Env nuovi: PHPR_MI_COLLECT_EXIT=1 (checkpoint
#  standing exit_collect_mi, census-only).
# WP-66: wp66-harness/{design66.md (recepimento sintesi 9 + §8 verdetti),
#  design66-p2.md (mini-design P-2 de-leak M-66.4), predictions66.md
#  (+.locked + copia repo sessions/WP66_PREDICTIONS.md = snapshot K-66.1),
#  gate66.sh (VERDETTO MACCHINA: FAIL counter, gate66.verdict, done solo
#  a zero; hk/reverse asseriti), sem-baseline/ (sem-oracle.diff.expected
#  + PROVENANCE), probe66-ksp1.sh (KS-P1 replay wpdev N=10, porta 8080,
#  segmentazione uc.log per-richiesta via offset), probe66-s4.sh
#  (batteria S-66.4, docroot s4-doc/, porta 8499), census66-l661.sh
#  (census full su memgc65b + supervisor vmmap del FIGLIO del wrapper
#  time), analyze-l661.pl}; out: gate-out/ (gate66.verdict), p1-out/
#  replay/ (uc.log, seg*.log, fpseq*), s4-out/, census-out/
#  (memcensus66-l661.txt, l661-vmmap-*.txt, l661-attribution.txt;
#  run1 = wrapper, tenuta come lezione). Stash phpr-wp66 (5aa60d56…).
# WP-67: wp67-harness/{design67.md (recepimento sintesi 9 + §8 verdetti),
#  predictions67.md (+.locked + copia repo sessions/WP67_PREDICTIONS.md,
#  commit 34aed31), gate67.sh (K-67.2 trap-FAIL + tot>=1646 + purge;
#  K-67.3 reverse PER NOME; aggancia fixtures/gate-*.sh),
#  gate-baseline/{hk-reverse-fails.txt,PROVENANCE}, analyze-l67.pl
#  (per-causa esatta, unità MiB, dirty/swap separati),
#  build-memgc66.sh, probe67-census.sh (B-67.1: census server
#  PER-RICHIESTA via dump per run top-level; segmentazione reqmark),
#  probe67-nk.sh (P2-a leak-shape N=1000, fixture nk-doc/),
#  probe67-ksp1.sh (rigioco KS-P1 post-P2, segmenti CLASSIFICATI per
#  entry-script del reqmark — il self-traffic WP è visibile),
#  run55-triple.sh (coppia propria + A-A′ nella stessa catena),
#  droporder/ (sentinelle P-2 pinnate, baseline == oracle),
#  fixtures/ordering/ (S-67.2: pin divergenza + gate-ordering.sh)};
#  out: gate-out/ (gate67.verdict PASS), census-out/server/, nk-out/,
#  ksp1-out/, eve-out/ (run55 new/old/new2). Binari census:
#  phpr-memgc66 (cea61455, PRE-P2) e phpr-memgc67 (95d07638, POST-P2)
#  in phpr-mem-target/. Env nuovi: PHPR_MI_COLLECT_REQ=1 (checkpoint
#  standing per-richiesta, census-only).
# WP-68: wp68-harness/{design68.md (recepimento sintesi 9 + §10 verdetti
#  con disposizione K-68.3 di OGNI predizione), predictions68.md (+.locked
#  + copia repo sessions/WP68_PREDICTIONS.md, commit 9bc603f),
#  probe68-pre.sh (precondizioni KS-P68.1 a verdict), probe68-ksp1.sh
#  (KB67-2: attese cold=0/hit_cross=512), probe68-census.sh (quota
#  S-68.4, G-68.4 ×3 run, KS68-2 nel verdetto), probe68-sem.sh (S-65.3
#  forma WP-68: DOPPIO diff pinnato, sem-baseline/+PROVENANCE),
#  probe68-b683.sh (B-68.3 slope wpdev N=1000, slope-only L-68.3),
#  build-memgc68.sh, census68-metro.sh (KL67-1 con protocollo no-swap
#  L-68.4), gap68-media.sh, run56-triple.sh, gate68.sh (K-68.1 verdict
#  .superseded / K-68.4 trap self-test / M-68.1 assert park),
#  fixtures/{gate-ordering-s68.sh (5+3 casi 2 LATI, pins/+PROVENANCE),
#  gate-droporder.sh (P-68.1), gate-editmiss.sh (P-67.4),
#  ordering-h68/ (f-intra g-declared h-livecond)}}; out: gate-out/
#  (gate68.verdict PASS + .superseded-*), pre-out/, ksp1-out/,
#  census-out/ (quota-table, memcensus68-metro), b683-out/, eve-out/
#  (run56 new/old/new2), stubs-out/, gap-out/. Binario census
#  phpr-memgc68 (5027e4e8) in phpr-mem-target/. ⚠️ ri-pin WP-68 con
#  PROVENANCE: gate-ordering (atteso=oracle), sem-baseline (doppio pin),
#  sentinels65 main.elide 177→176 (copia .pre-wp68 accanto).
# WP-69: wp69-harness/{design69.md (recepimento sintesi 9 + §9 verdetti
#  con disposizione PER BANDA), predictions69.md (+.locked b91b4799 +
#  copia repo sessions/WP69_PREDICTIONS.md, commit 07ef494),
#  gate69.sh (gate68 + fixtures s69 + baseline corpus69), gate-baseline/
#  {corpus69.fails (1422), PROVENANCE-corpus69.txt},
#  fixtures/{gate-ordering-s69.sh (9 unit 2 LATI + 6 redecl),
#  gate-h69cycle.sh, gen-pins69.sh, pins/ (+PROVENANCE-s69), s69-*/ (9),
#  h69-cycle/, redecl/}, probe69-l681.sh (L-68.1 3 leg),
#  probe69-cron.sh (B-69.5, inject/restore DISABLE_WP_CRON),
#  count-handler-bodies.sh (K-69.3, baseline 178),
#  build-memgc69.sh}; out: gate-out/ (gate69.verdict PASS run2),
#  l681-out/ (l681-table), cron-out/ (cron-table). Binario census
#  phpr-memgc69 (f034e62a) con PHPR_CENSUS_HITS=0 off-switch +
#  tag=censusown capacity-aware. ⚠️ ri-pin WP-69 con PROVENANCE:
#  corpus69 1422, sentinels65 redecl.elide 2 righe (.pre-wp69).
# WP-59: wp59-harness/{design59.md (verdetti Ob.0-3 + gate pre-registrati),
#  ob0-media.sh,ob0-full.sh (diagnostiche MIMALLOC_SHOW_STATS+vmmap),
#  build-memgc59.sh,ob1-census59-media.sh,ob1-supervisor.sh (finestre
#  phys: vmmap+footprint per flag-file),ob1-report.pl (tabella
#  riconciliazione+top bin),build-pooloff59.sh,run47-pooloff.sh,
#  run47-new.sh,run47-scan4.sh,orchestrate59-evening.sh}; out: ob0-out/,
#  census-media-out/ (memcensus59-master.txt = solo master pid),
#  full-{pooloff,new,scan4}-out/ (CPU esatta in *.time). Binari: census
#  phpr-memgc59 (00ca4d3d…, phpr-mem-target/); diagnostici phpr-pooloff59
#  (93aea5a3…, phpr-pooloff-target/), phpr-scan4-59 (b955afb1…,
#  phpr-scan4-target/). Probe scratchpad: probe59-{inc,boot,classes,
#  funcs,null} (leak template + moltiplicatore compile-side).
```

## 🎯 PROSSIMO LAVORO — WP-70: attribuzione residuo ~2 KiB/req (SBLOCCA axum) + spike dispatch (outlined+profilo) + ways/fp al concilio

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi, no-revert. Stato WP-69: debiti concilio chiusi,
fedeltà declare consegnata (dettaglio: `sessions/WP_SESSION_69.md` +
`wp69-harness/design69.md` §9).

⚖️ **CONCILIO DI CHIUSURA WP-69 ESEGUITO: verbali INTEGRALI + sintesi
in `wp70-harness/COUNCIL_WP70_REVIEWS.md` — VINCOLANTI per design70,
recepire INTEGRALI prima di toccare codice. Verdetti: 0 concordi
secchi · 6 con emendamenti (Klabnik, Hejlsberg, Bak, Pedersen,
Leijen, Gregg) · 3 OPPOSIZIONI con refutazioni EMPIRICHE (Hoare,
Matsakis, Stogov).**

0. **DEBITI D'APERTURA (delta-zero, dai verbali)**: (i) 🔴 **H-69.2
   CHIUSA A METÀ (Hoare+Matsakis, panic ESEGUITO)**: `&$a[0][0]` /
   `&$o->self->x` panicano ancora (`RefCell already borrowed` nei
   walker `elem_cell`/`field_cell` — borrow_mut nudi residui;
   H-70.1≡M-70.1, KH70-1 BLOCCA run_loop/spike); (ii) 🔴 **Stogov:
   declare deferito fallito = SKIP SILENZIOSO** (classe fantasma,
   autoload bypassato — S-70.1..3, KS-S70.1/2); (iii) gli altri
   emendamenti dei verbali (K-70.x pre-registri caccia residuo,
   E-70.1/2 passo-0 release strumento-free + firma per-bin
   ~16-20 obj/req 112-128 B ⇒ ipotesi defer-mini, G-70.x forma dei
   pre-registri, L-70.1, P-70.x).
1. **🔴 CACCIA AL RESIDUO ~2 KiB/req (KL69-1 SCATTATO — blocca axum)**:
   protocollo pulito PRONTO (memgc69, HITS-off + DISABLE_WP_CRON,
   probe69-l681/cron come template); leak-shape per-bin sulla finestra
   di regime; nk-doc = 0,0000 ⇒ il residuo è workload-specifico
   (candidati: canali per-request del server WP — accumuli error_log/
   header/transient-shape, slack mimalloc). Attese pre-registrate PRIMA.
2. **SPIKE DISPATCH (continua, KG69-2)**: (i) build outlined
   feature-gated (dispatch-profile) dei bracci caldi WP-42, validata in
   coppia vs release (outlined MAI giudice); (ii) profilo sample con
   predictions69 GIÀ lockate (b91b4799 — bande P69-S-a/b, tabella
   ex-ante, MDE 0,52% IR); (iii) decisione da tabella; K-69.3 contatore
   (baseline 178) pre/post nel worktree; G-69.1 co-giudice IR.
3. **ways/fp AL CONCILIO (B-67.4)**: input pronti — fp context-sensitive
   per costruzione (salt = entry, design69 §9.4b), firma version.php
   ×4 miss-fp al confine, 8 evizioni con 2 contesti, opzione
   digest-dello-stato-seed (trappola S-69.5 nella chiave). KB69-3
   detector armato prima del multi-worker.
4. **Fronte axum (DOPO 1)**: batteria P-69.2 precondizioni estese +
   ordine S-67.3 {1: define/shutdown, 2: ob/header, 3: session/
   superglobali, 4: cross-worker}; P-69.3/KS-P69.1 editmiss-PARENT;
   P-69.4 retry-autoload; P-69.1 assign-poi-unset; M-69.3 censimento
   Ref(Undef)+autovivify; P-69.5 quota seed_globals mere-mention.
5. **Backlog nuovi WP-69**: famiglia EVAL-NAMING nei trace/fatal
   (3 phpt pinnati: frame "eval()", location "%s(%d) : eval()'d code",
   SensitiveParameter; presente anche su thrown) · Δbytes STUBS ·
   metro (KL68-2/L-69.4: solo da reboot no-swap) · quinta coppia
   G-66.4 no-swap · replica a distanza quota 7,84 (G-69.3).

**VETI (restano vincolanti)**: invariati WP-64..68 + nuovi WP-69:
**pendenze wpdev senza DISABLE_WP_CRON** (il confine cron invalida le
finestre — L-69.2); **leg L-68.1 senza UNIT_CACHE_LOG**; **cifre
census-own come "spiegazione totale"** (il counter attribuisce il
90,7%: sotto c'è il residuo vero); **profilo dispatch su binario
non-outlined** (B-69.4).


## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT = skill `apri-sessione`** (decisione utente 2026-07-26:
   disco×2 / MySQL con datadir giusto / hash binario di parità /
   residui detached / uploads / ordine di lettura — procedura in
   `~/.claude/skills/apri-sessione/SKILL.md`). ⚠️ path CANONICO dello
   stash = drive esterno `phpr-old-target/release/` (lo stash in
   ~/Claude ha prodotto un A/B con 12 run da 0,00s — WP-50).
1. **Rotta ([[php-rust-roadmap-wp-first]])**: dopo la roadmap
   footprint+CPU si apre la VALIDAZIONE LARAVEL (metodo = ricetta gate
   ORM/hk: oracolo+composer build, phpr esegue).
2. **Backlog pescabile da [[php-rust-todo-master]]** — candidato pronto:
   🆕 bug isset via prefisso `__get` con indici annidati
   (probe wp42-harness/probe-isset-div.php §3).
3. **NON riproporre**: stadi 3-4 registri; leve locali canale churn
   (esaurite WP-33/34/41/42); NaN-boxing; SSO union.

## Backlog aperto (non legato a una sessione)

- 🆕 **Panic census-only** (WP-57): prima run full con binario op-census →
  panic master a ~6 min, `run.rs:478` `index out of bounds: len 78,
  index 78` (PC oltre fine func.ops); rerun identico completa la suite ⇒
  non-deterministico; MAI visto sui binari di parità (~45 full run).
  Evidenza: `wp57-harness/full-out/full-census.txt` + WP_SESSION_57.
- 🆕 **isset via prefisso `__get` con indici annidati** perde il walk sul
  risultato — bug funzionale preesistente (WP-42). Candidato fix.
- 🆕 Deprecation PHP 8.5 (chiave null/float-frac) non emesse dentro
  isset/empty (`coerce_key_silent` muto); `isset($nonAA['k'])` non lancia
  Error — famiglia quiet-fetch, catalogare in PHPR_DIVERGENCES se si
  decide di non chiudere.
- Residui strutturali: `ast_printing.phpt` (serve zend_ast_export
  sull'HIR) · xsl `bug69168` (nodi php:function devono aliasare il doc
  live) · tidy `010` (free-order var_dump-di-albero).
- Ret-hook usa ancora gc_cascade (non gc_release_cascade) per oggetti con
  `__destruct` nel subtree — nessun test lo copre oggi.
- Verbo "increment/decrement" per `$null->p++` (oggi "assign").
- Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).
- str residuo walk 3% (~39MB metadati moduli) · interning const-array ·
  gc_047 (release iteratore al break) · gc_030 (trace shape) · gc_022
  (temp mid-statement).

## 📊 Report gap perf — ricorrente di fine sessione

Trend cumulativo e metodo di misura: **`gaps/GAP_TREND.md`** (file unico
vivo); misure per-sessione in `gaps/REPORT_GAP_<N>.md`. A ogni chiusura:
misurare media (user CPU + footprint) e full-suite master-CPU, scrivere
REPORT_GAP_N (sola sessione N), aggiungere la riga a GAP_TREND, riportare
il gap all'utente.
Ultimo stato (WP-69, sessione fedeltà/debiti — perf NON rimisurata):
**riferimenti invariati: media CPU 2,58× · footprint media ~3,0-3,1
(banda 2,9-3,2, G-66.4 a n=3+1) · full CPU 2,06-2,11× · peak
~1,98-2,03GB · server hit_cross 512/512 · corpus RI-PIN 1421→1422
(−2 fix reali, +3 ex-skip runnable) · 🔴 KL69-1: residuo VERO
≈2,0-2,2 KiB/req su wpdev (run pulita HITS-off+cron-free) — fronte
axum FERMO finché attribuito · baseline parità phpr-wp69 (c5b1e80b…,
gate69 PASS fails=0) · dettaglio: `sessions/WP_SESSION_69.md`,
trend: `gaps/GAP_TREND.md`**.
