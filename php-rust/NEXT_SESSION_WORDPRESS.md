# Rotta WORDPRESS-FIRST — WP-track (dopo WP-67: ⚡ **P-2 DE-LEAK SPEDITA** — Box::leak MORTO sull'include path: RetainSet per-richiesta (elsa =1.11.2, sign-off utente H-67.2) + cache Rc-owned, park = costruttore UNICO di &'m Module (M-67.1), zero unsafe aggiunti e UNO RIMOSSO · GATE67 PASS fails=0 a verdetto macchina (cargo 1647/0, reverse PER NOME) · coppia full PROPRIA + self-pair A-A′ stessa sera (PRIMO protocollo): −1,30% dentro spread A-A′ 1,26% ⇒ costo CPU ≈ ZERO; fail-set 88 BYTE-ID ×3 = run33 · leak-shape: pre-P2 1,62 MiB/richiesta RITENUTI (22 mod: 15 impure+7 eval) → post-P2 pendenza Σcommitted +2,55 KiB/req (soglia 50), dead 1998/2003, Δstubs=0 (KS-P67.3) · quota B-67.1: CPU residua impure ≈21 ms/richiesta (script-loader 8,4+comment 6,1+ProviderRegistry 1,5 = 76%) = la leva WP-68 · rigiochi KS-P1/S-66.4 PASS con verdict-file (SAPI -S sequenziale, worker unico) · 🔴 SCOPERTA S-67.2: divergenza ordering autoload-in-lowering PRE-ESISTENTE sul COLD (oracle: U-top·[autoload]; phpr: [autoload]·U-top) — catalogata PHPR_DIVERGENCES §3 + pinnata nel gate; defer-always chiuderebbe ANCHE questa (prova eseguibile per il concilio WP-68) · reqmark spedito: il marker ha reso VISIBILE il self-traffic WP (cron+loopback) che l'offset assorbiva in silenzio · stash phpr-wp67 c0f5cfff…, tree 2b92c78)

> ⚡ **WP-67 (2026-07-28 notte, `4fe85a1`→`2b92c78` release + `046c961`/
> `f0f0e72` census/doc)** — sintesi a 9 recepita INTEGRALE in
> `wp67-harness/design67.md` PRIMA del codice; predizioni committate
> PRIMA dei letti (34aed31). **Debiti d'apertura a delta zero**:
> H-67.1 breach leggibile; H-67.3/M-67.6 tripwire tail∩seed VIVO
> promosso a FATAL in ogni build + test negativo (cargo 1646→1647);
> E-67.1 finestra `l` depurata + split dup_fp/dup_cold (E6 sbloccata);
> G-67.1/K-67.5 evento `reqmark` (vocabolario 10→11) — offset+sleep
> DEPRECATO; B-67.2 contatore diretto parked_modules/bytes; K-67.1/2/3
> gate hardening (verdict-file, trap-FAIL, reverse PER NOME);
> L-67.3/G-67.3 analyze-l67.pl (per-causa esatta; sotto swap (a)/(b)
> opposte ⇒ run senza pressione ancora dovuta, phys standing SOSPESI).
> **P-2 SPEDITA** (design66-p2 emendato integrale): RetainSet
> (elsa =1.11.2, sign-off utente) + cache Rc-owned; park_module =
> costruttore UNICO (M-67.1); hit clone+park (K-M67.2/H-67.4); eval
> muoiono a fine richiesta; main = borrow del chiamante (M-67.2);
> census ptr→Weak = UN unsafe RIMOSSO, zero aggiunti (verifica a
> macchina). GATE67 PASS fails=0; probe67-nk N=1000: pendenza
> +2,55 KiB/req, dead 1998/2003, Δstubs=0; A/B pre-P2: 1,62 MiB/req
> ritenuti su wpdev che ora MUOIONO; rigiochi KS-P1/S-66.4 PASS;
> run55 TRIPLE: CPU ≈ 0 (−1,30% dentro A-A′ 1,26%), 88 BYTE-ID ×3.
> **🔴 S-67.2**: la fixture ordering-echo ha falsificato il COLD:
> divergenza autoload-in-lowering PRE-ESISTENTE vs oracle (catalogata
> + pinnata); la leva impure NON eseguita (scope onesto) — la forma
> vincolata publish+dep-replay va ripesata contro defer-always
> (Zend-fedele E purificante) al concilio WP-68, con la quota
> 21 ms/richiesta sul tavolo.
> **Storia: `sessions/WP_SESSION_67.md`. WP-66:
> `sessions/WP_SESSION_66.md`.**

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

## Stato gate per nome (WP-67 `2b92c78`: **GATE67 a VERDETTO MACCHINA — PASS fails=0** su `wp67-harness/gate-out/` (K-67.1/2/3: trap-FAIL su uscite premature, cargo tot≥1646, purge log freschi, **reverse PER NOME** con baseline+PROVENANCE in `wp67-harness/gate-baseline/`) — cargo **1647/0** (+1 = test tripwire M-67.6) · sentinelle 5 assi BYTE-ID · sentinels65 + KS-S6 + **seed_prefix_short=0** · KE-e · P1 hit_cross=2 · S-65.3 + diff-oracle pinnato · **fixture gate-ordering (S-67.2) agganciata** · corpus **1421 IDENTICO** · refl **290** · ORM **3E/13F IDENTICO** · **hk 1665 OK** · **reverse 2F PER NOME**; release = **phpr-wp67 (c0f5cfff…, tree `2b92c78`)** stash additivo accanto a wp66 (5aa60d56…); catena serale run55 TRIPLE: fail-set **88 BYTE-ID = run33 ×3** (new/old/new′), CPU coppia −1,30% DENTRO spread A-A′ 1,26% ⇒ ≈0; probe verdict-file: probe67-census PASS · probe67-nk post-p2 PASS · probe67-ksp1 PASS · S-66.4 PASS; census: phpr-memgc66 (cea61455, PRE-P2) + phpr-memgc67 (95d07638, POST-P2); ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-66 (`fe33706`: GATE66 PASS fails=0 — cargo 1646/0 · corpus 1421 · refl 290 · ORM 3E/13F · hk 1665 OK asserito · reverse 2F a conteggio; stash phpr-wp66 5aa60d56…; dettaglio: `sessions/WP_SESSION_66.md`)

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

## 🎯 PROSSIMO LAVORO — WP-68: cacheabilità unit impure (forma DA RIDECIDERE al concilio con la prova S-67.2) + fronte axum multi-worker + code L-67

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi, no-revert. Laravel POSTICIPATA a valle.
Stato WP-67: P-2 SPEDITA e verificata (dettaglio:
`sessions/WP_SESSION_67.md` + `wp67-harness/design67.md` §8).

⚖️ **CONCILIO DI CHIUSURA WP-67: verbali in
`wp68-harness/COUNCIL_WP68_REVIEWS.md` — VINCOLANTI per design68,
recepire INTEGRALI prima di toccare codice.**

1. **Cacheabilità unit impure — LA DECISIONE DI FORMA VA RIAPERTA AL
   CONCILIO** con la prova eseguibile S-67.2 sul tavolo
   (`wp67-harness/fixtures/ordering/`): (a) forma WP-67 vincolata
   publish+dep-list+dep-replay (E-67.2/S-67.1) = hit==cold coerente ma
   ENTRAMBI ≠ Zend nell'ordering (divergenza pre-esistente resta); (b)
   forma defer-always (Stogov: "in Zend è ESATTAMENTE il caso
   cachato") = late binding fedele ⇒ unit PURE per costruzione, chiude
   ANCHE la divergenza cold — da valutare costo (defer di ogni extends
   non-seed) e rischi (ordine di dichiarazione). Quota sul tavolo:
   **21 ms/richiesta CPU + niente più leak** (il leak è già morto con
   P-2). Criterio di successo INVARIATO: le 15 diventano hit_cross
   (KB67-2), mai "publish avvenuta"; fixture: ordering-echo (pinnata),
   P-67.4 figlio-editato⇒MISS (si arma con la leva), catena impura ⇒
   never-published (KS67-1/KS-S67.1).
2. **Fronte axum vero (multi-worker) — ora SBLOCCATO da P-2**: KS-P2
   breakdown per-worker (cache PER-THREAD ×N — M-66.3), L-66.3 metro
   {1,100,1000} per-worker, batteria S-67.3 confini server
   (pre-registrata in design67 §5: session, POST/FILES/COOKIE,
   header/setcookie reset, ob, **static function-vars su unit cacheata
   KS-S67.2**, define/ini, shutdown, cross-worker), S-67.4 g2′
   include-vuoto, P-67.2 seed-materializzazione, S-67.5 set route
   minimo {/, permalink, ?s=, 404, /wp-json/wp/v2/posts, wp-login GET,
   POST commento}, B-67.4 replay multi-URL con plateau ways (KB67-3),
   B-66.3 sample sotto carico, G-66.5 coppia log-off/on server.
3. **Code L-67**: la colonna per-causa PULITA vuole una census run
   SENZA pressione di swap (G-67.4: swap loggato pre-run; su l661 le
   cause (a)/(b) si gonfiano opposte) — claim phys standing SOSPESI
   finché non c'è; KL67-1 terza run del metro Σcommitted (attesa:
   post-P-2 il counted unit cala — dichiarare l'atteso dal Δcounted
   PRIMA della run).
4. **E6 ri-quota**: colonne DEPURATE (E-67.1: dup_fp/dup_cold split)
   ARMATE in 2b92c78 — serve un census full; soglia in forma E-66.2;
   KS67-2: mai su dup_lc_ns non depurato.
5. **G3**: invariato — quarta coppia media o risoluzione criterio ≥3+3
   (G-66.4). **P2 seed signature-only**: invariato (Gregg R6).

**VETI del concilio (restano vincolanti)**: invariati da WP-64/65
(NEXT_SESSION storico + `sessions/WP_SESSION_64.md`: mai
MIMALLOC_PURGE_DELAY nei giudici; mai maxrss come giudice; nessun
pool/freelist; interning/immutable-literal con condizioni; ecc.).
- **NON riproporre (nuovi WP-67)**: **offset+sleep come delimitatore di
  richiesta** (reqmark spedito: segmentare SEMPRE per marker,
  classificando i segmenti per entry-script — il self-traffic WP
  esiste); **tripwire debug-only** (il gate gira `--release`: la forma
  è FATAL-in-ogni-build + test negativo); **re-lower aggiuntivo**
  (Hejlsberg c: l'ultima iterazione del retry È il lower a seed
  stabilizzato); **publish senza dep-replay** (respinta WP-67, salvo
  ridecisione esplicita del concilio con la prova S-67.2); **cifre
  <1% senza self-pair A-A′ stessa sera** (ora protocollo eseguito:
  run55 triple); **phys standing come giudice** (solo Σcommitted,
  KG67-2).
- **NON riproporre (ereditati WP-63/64/65)**: bande CPU di coppia più
  strette dello spread storico; giudizi peak da coppia singola vs
  stash; forma Rc-prefix per slot_names; leva CPU su map/remap;
  rounding al peak; eventi log con sottostringhe; baking emit-time;
  peak-differential; bande NET→PHYS; colonne prefixsum a flag ON come
  attribuzione; riaprire cache re-link senza numeri sopra soglia.

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
Ultimo stato (WP-67, P-2 de-leak spedita):
**media CPU riferimento 2,58× · footprint media ~3,0× (mediana n=3,
banda 2,9-3,2 aperta su G-66.4) · full: CPU 2,06-2,11× (run55 triple:
P-2 costo ≈ 0, −1,30% dentro spread A-A′ 1,26%), peak ~1,98-2,03GB
(riferimento WP-65) · leak server: pre-P2 1,62 MiB/richiesta ritenuti
→ post-P2 pendenza +2,55 KiB/req (bookkeeping census), dead
1998/2003 · CPU residua impure ≈21 ms/richiesta (leva WP-68) ·
baseline parità phpr-wp67 (c0f5cfff…, gate67 PASS fails=0) ·
dettaglio: `sessions/WP_SESSION_67.md` + `gaps/REPORT_GAP_67.md`,
trend: `gaps/GAP_TREND.md`**.
