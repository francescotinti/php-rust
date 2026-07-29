# Rotta WORDPRESS-FIRST — WP-track (dopo WP-71: ⚡ **DEBITI D'APERTURA CHIUSI + ATTRIBUZIONE DEL RESIDUO CHIUSA A MACCHINA** — 🔴 M-71.1≡H-71.1 CHIUSO PER COSTRUZIONE: field_write/field_unset/unset_into a disciplina di confine (WriteWalk/UnsetWalk/UnsetIntoWalk), 6/6 repro BYTE-ID + 7° panic t6 chiuso, batteria h71walk 22/22, grep-gate walkborrow ZERO borrow nudi (K-M71.2) ⇒ **KH70-1 SCIOLTO come FAMIGLIA**; tripwire drain H-71.3 + tag=cellpark in ogni build · 🔴 S-71.1 readonly-extends BIDIREZIONALE (eager+defer, FatalAt Zend-shaped dal re-lower) + S-71.2 spl_autoload_call a lista VIVA (cursore element-stable, sticky-end oracle-pinned) — batteria s71 15/15 · **corpus RI-PIN 1421→1420 (−1 fix REALE ns_064 = famiglia M-71.1, PROVENANCE)** · **GATE71 PASS fails=0 AL 1° RUN — ATTESTATO (K-71.2 attempts=1)**: cargo 1651/0, fixture WP-67..71 TUTTE, refl 290, ORM 3E/13F, hk 1665, reverse 2F per nome · 🔴 **ATTRIBUZIONE (lock 05faefcc→27f512d3, addenda ADDITIVI pre-letto): scope-heap L-71.1 ⇒ H1-C1 MORTA (defer ritiene 0,000 con controllo POSITIVO 16,00 calls/req) = falsificati IN BLOCCO tutti i candidati in-scope; KL71-2 block-content dump ⇒ Σdelta 10.000/10.000 ESATTO, foglie "filter"/"callback"/hook = grafo WP_Metadata_Lazyloader ⇒ CICLI Rc NON RACCOLTI AL TEARDOWN; amplificazione K=10 → +80,000 obj/req ESATTO = MECCANISMO-CONFERMATO; ladder LINEARE (ratio 1,078, ~2,0 KiB/req phys)** · B-70.1/B-70.4 CONSEGNATI per-evento (8 evict osservati: 4 cron + 4 front su version.php — KB71-4 soddisfatto) · K-71.3 timbri retroattivi + P-71.1 verdict SOLO da script · **⇒ WP-72 = LEVA cycle-collect-al-teardown (ora LEGITTIMA) + validazione P-71.3 (two-boot post-fix < 2 MiB) = SBLOCCO AXUM** · stash phpr-wp71 38af5eaa…, tree 076ed4b)

> ⚡ **WP-71 (2026-07-29, `e0094c6`→`076ed4b`)** — sintesi a 9 recepita
> INTEGRALE in `wp71-harness/design71.md` PRIMA del codice; pre-registro
> G-71.1 lockato PRIMA dei letti (catena di lock additivi
> 05faefcc→e61f07df→744e4b00→27f512d3). Walker chiusi PER COSTRUZIONE
> (grep-gate), S-71.1/S-71.2 chiusi oracle-pinned, GATE71 PASS al 1°
> run ATTESTATO; attribuzione del residuo CHIUSA a verdetto macchina
> (cicli Rc al teardown, amplificazione lineare intera-esatta).
> **Storia: `sessions/WP_SESSION_71.md`. WP-70:
> `sessions/WP_SESSION_70.md`.**

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

## Stato gate per nome (WP-71 `076ed4b`: **GATE71 PASS fails=0 AL 1° RUN, ATTESTATO (gate71.attempts=1, K-71.2)** su `wp71-harness/gate-out/` — trap self-test esercitata — cargo **1651/0** (tot≥1651) · sentinelle 5 assi BYTE-ID · fixture: TUTTE le WP-67..70 + **gate-h71walk (22 casi M-71.1/H-71.2, 19 pin oracle + t4/t6/t7 pin phpr PROVENANCE-h71)** + **gate-s71 (15 casi S-71.1/S-71.2, vi_trait pin phpr = backlog S-71.3)** + **gate-walkborrow (K-M71.2: zero borrow nudi nei 23 corpi di walk-path, 6 BORROW-OK motivati)** · sentinels65 + KS-S6 + seed_prefix_short=0 · KE-e · P1 · sem doppio pin · **corpus 1420 IDENTICO (baseline wp71-harness/gate-baseline/corpus71.fails, PROVENANCE: −1 fix reale ns_064)** · refl **290** · ORM **3E/13F** · hk **1665 OK** · reverse **2F PER NOME**; release = **phpr-wp71 (38af5eaa…, tree `076ed4b`)** stash additivo accanto a wp70 (a666382e…); census **phpr-memgc71 (2c4691d7, scope-heap src=defer)** + **71b/71c (blockdump)** con tag=cellpark; probe verdict-file (P-71.1, tutti da script): probe71-tripla **PASS** (KL71-1 totale 20,000 spread 0,000; H1-C1 MORTA) · probe71-amp **K=10 MECCANISMO-CONFERMATO (+80,000 esatto)** · probe71-ladder **PASS LINEARE (ratio 1,078)** · probe71-cron **PASS (8 evict per-evento)**; KL69-1: attribuzione CHIUSA ⇒ axum si sblocca con P-71.3 post-fix; ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-70: vedi `sessions/WP_SESSION_70.md`

## (storico) Stato gate per nome (WP-70 `feca5b2`: **GATE70 a VERDETTO MACCHINA — PASS fails=0 AL 1° RUN** su `wp70-harness/gate-out/` — matrice INTEGRALE in un solo run, trap self-test esercitata — cargo **1650/0** (tot≥1650) · sentinelle 5 assi BYTE-ID · fixture: ordering + s68 + gate-ordering-s69 + gate-h69cycle + droporder + editmiss + **gate-h70cycle (8 casi ciclici K-M70.1, 7 pin oracle + wopset PROVENANCE)** + **gate-s70neg (12 casi KS-S70.1/2 con pin `load:M`)** + **gate-h70trait (5 casi H-70.4)** · sentinels65 + KS-S6 + seed_prefix_short=0 · KE-e · P1 · sem doppio pin · **corpus 1421 IDENTICO (baseline wp70-harness/gate-baseline/corpus70.fails, PROVENANCE: −1 fix reale)** · refl **290** · ORM **3E/13F** · hk **1665 OK** · reverse **2F PER NOME**; release = **phpr-wp70 (a666382e…, tree `feca5b2`; +1f73b2b census-only cfg-gated, parità invariata)** stash additivo accanto a wp69 (c5b1e80b…); census **phpr-memgc70 (9201ed94)** con tag=cellskip + tag=defermini; probe verdict-file: probe70-step0 **PASS ESISTE-SU-RELEASE (Δ5,8 MiB)** · probe70-tripla **PASS con slope in colonna (run1 FAIL parser superseded)**: mediana 2,1121 KiB/req · 20,000 obj/req, KL70-1 leak=SI, K-M70.2 cellskip=0 · K-69.3 = 178 invariato; KL69-1 resta SCATTATO (attribuzione aperta ⇒ axum fermo); ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-69: vedi `sessions/WP_SESSION_69.md`

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
# WP-70: wp70-harness/{design70.md (recepimento sintesi 9 + §7 verdetti),
#  COUNCIL_WP70_REVIEWS.md, predictions70.md (+.locked cbf251fe con
#  ADDENDUM pre-letto + copia repo sessions/WP70_PREDICTIONS.md),
#  gate70.sh (etichette K-70.4, corpus70, cargo>=1650),
#  gate-baseline/{corpus70.fails (1421), PROVENANCE-corpus70.txt},
#  fixtures/{gate-h70cycle.sh (8), gate-s70neg.sh (12),
#  gate-h70trait.sh (5), h70-*/, s70-neg/, h70-trait/, pins/ con
#  PROVENANCE-h70}, probe70-step0.sh (P70-0-bis vmmap two-boot),
#  probe70-tripla.sh + analyze70-tripla.pl (segmentazione: snapshot =
#  blocco mi_bin chiuso da tag=modrecon — win= NON è il contatore),
#  build-memgc70.sh}; out: gate-out/ (gate70.verdict PASS 1° run),
#  step0-out/ (step0-table), tripla-out/ (tripla-table2, census1-3).
#  Binario census phpr-memgc70 (9201ed94) con tag=cellskip (M-70.2)
#  + tag=defermini (P70-D: calls+net_b run_deferred). ⚠️ ri-pin WP-70
#  con PROVENANCE: corpus70 1421 (−1 class_order_autoload1).
# WP-71: wp71-harness/{design71.md (recepimento sintesi 9 + §7 verdetti
#  con attribuzione chiusa), COUNCIL_WP71_REVIEWS.md, predictions71.md
#  (+.locked, catena lock ADDITIVI 05faefcc→27f512d3 + copia repo
#  sessions/WP71_PREDICTIONS.md), gate71.sh (fixture WP-67..71 +
#  attempt-counter K-71.2 + baseline corpus71), gate-baseline/
#  {corpus71.fails (1420), PROVENANCE-corpus71.txt},
#  fixtures/{gate-h71walk.sh (22), gate-s71.sh (15), gate-walkborrow.sh
#  (K-M71.2 grep-gate), h71-walk/, s71/, pins/ con PROVENANCE-h71},
#  hoare71/ + stogov71/ (fixture dei verbali, salvate dallo scratch),
#  analyze71.pl (parser G-71.3/G-71.4 + straddle + src=defer),
#  probe71-tripla.sh, probe71-reanalyze.sh (causa obbligatoria K-71.4),
#  probe71-ladder.sh (L-71.4), probe71-cron.sh (B-70.1/B-70.4),
#  probe71-amp-analyze.sh (verdict amplificazione), analyze71-blockdump
#  .pl + analyze71-bddelta.pl (KL71-2), build-memgc71.sh}; out:
#  gate-out/ (gate71.verdict PASS + .attempts), tripla-out/,
#  ladder-out/, cron-out/ (cron-table71), amp-out/, blockdump-out{,2}/.
#  Binari census phpr-memgc71 (2c4691d7) + 71b (91038d46) + 71c
#  (blockdump ≤1024) in phpr-mem-target/. Env nuovi (census-only):
#  PHPR_MI_BLOCKDUMP=<base> + PHPR_MI_BLOCKDUMP_AT="500,1000".
#  ⚠️ ri-pin WP-71 con PROVENANCE: corpus71 1420 (−1 ns_064).
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

## 🎯 PROSSIMO LAVORO — WP-72: LEVA cycle-collect-al-teardown (SBLOCCA axum via P-71.3) + debiti Hejlsberg + spike outlined (sessione dedicata sbloccata)

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi, no-revert. Stato WP-71: debiti d'apertura
chiusi PER COSTRUZIONE, gate71 PASS al 1° run attestato, attribuzione
del residuo CHIUSA a macchina (dettaglio: `sessions/WP_SESSION_71.md` +
`wp71-harness/design71.md` §7).

⚖️ **CONCILIO DI CHIUSURA WP-71 ESEGUITO: verbali INTEGRALI + sintesi
in `wp72-harness/COUNCIL_WP72_REVIEWS.md` — VINCOLANTI per design72,
recepire INTEGRALI prima di toccare codice. Verdetti: 0 concordi
secchi · 8 con emendamenti/riserve · 1 OPPOSIZIONE (Matsakis: 2
wrong-result SILENZIOSI provati sul verbo UNSET). Deliberazioni
chiave: il claim "KH70-1 sciolto come FAMIGLIA" si EMENDA in
"famiglia PANIC chiusa nei corpi gateati"; S-71.2 è ROTTO su 3 forme
nuove; la leva è RI-SAGOMATA a mass-teardown Zend-fedele (dtor-walk
store-order, non trace-GC); lo sblocco axum è RIORDINATO (P-71.3 AND
E-71.H1/H2 AND P-71.4).**

0. **🔴 DEBITO D'APERTURA (delta-zero, opposizione Matsakis +
   Stogov)**: M-72.1 UnsetWalk guadagna i token AA (Descend + leaf
   offsetUnset) + guard readonly/leaf-op-parity in
   field_unset_prop_step (fixture f1/f6 pin oracle: oggi f1 resetta
   senza Fatal, f6 no-op silenzioso su AA) · S-72.1 dedup del
   duplicato al register + fix `callable_eq` per `[$obj,'m']` ·
   S-72.2 cursore SOSPESO muore con l'elemento (lookup annidata b5b)
   · S-72.3 prepend-durante-lookup (Zend = livelock posizionale:
   replicare o BUG(port)+PROVENANCE) · H-72.1/M-72.2 grep-gate esteso
   (drain + run.rs + conteggio BORROW-OK pinnato) · H-72.3 drain non
   fabbrica valori · H-72.5 invariante chiavi-mai-Ref · K-72.1 stamps
   k72 per predictions71 (C2/C3/C4/C5/C6/P-71.3/KS-P71.2).

1. **🔴 LEVA al teardown — FORMA RI-SAGOMATA DAL CONCILIO (S-72.4):
   MASS-TEARDOWN Zend-fedele, non trace-GC** — dal sorgente 8.5.7:
   Zend in release NON cicla il GC a fine richiesta; fa dtor-walk
   sull'object-store in ORDINE DI CREAZIONE (ciclici COMPRESI) poi
   free wholesale; phpr diverge GIÀ oggi sull'ordine dtor (c1
   reverse ⇒ sentinella store-order da pinnare PRIMA, KS-S72.2).
   design72 dichiara PRIMA del codice: fonte dei root + prova di
   completezza (census vivi post-collect==0, KH72-1); ordine di
   smantellamento pinnato (drain RefLeaf → semina root WP-49 →
   dtor-walk/collect → assert parked==0 ∧ cellpark/drainfails==0 →
   Drop VM, P-72.3) con sentinelle drop-order; politica __destruct
   dei cicli con fixture byte-parity PRIMA (P-72.4, KH72-2);
   **cap CPU NUMERICO nel lock** (B-72.2/L-72.3: ≤0,5 ms/req media E
   ≤1% full, coppia stessa-sera, fast-path buffer-vuoto); B-72.1
   istogramma possible-roots PRIMA del design; bande post-fix
   LOCKATE: tripla used_n ∈ [−0,009,+0,5] PASS, (0,5,1,5] PARZIALE
   con ri-attribuzione, used_b < 0,15 KiB/req, per-bin → 0 con
   160|192 che DECIDE C4 (KG72-2); controllo negativo G-72.3 (amp71
   K=10 post-fix ∈ [−0,5,+5]); uc-steady nel verdict (E-72.H2);
   P-71.3 in forma ladder three-boot con rumore dichiarato (L-72.1).
   **SBLOCCO AXUM = ULTIMO ATTO, riordinato (E-72.H1 vincolante):
   P-71.3 AND E-71.H1/E-71.H2 timbrati AND gate-axum P-71.4/P-72.6**
   (KS72-H1: sblocco senza audit = revocato).
2. **Debiti Hejlsberg (ORA GATE dello sblocco axum, E-72.H1)**: E-71.H1 audit delle
   letture di class_index su entry seed fuori da note_seed_super
   (stmt.rs ~617 riserva; class.rs 45/356) + E-71.H2 fixture
   caso-negativo hoist-timing nel gate (KS71-H1 armato). Cache
   deferred resta DESIGN-ONLY (E-71.H3, chiave read-set; beneficio
   pre-registrato 8,0 ms/req + 1526 KiB/req churn) — SHIP solo dopo
   la leva 1 (il canale ora è attribuito, KB71-3 decade a leva
   consegnata ma la firma va ri-pinnata post-fix).
3. **ways/fp: delibera digest RINVIATA (Bak B-72.5)**: il singolo
   flip prova il MECCANISMO, non il regime — servono multi-flip
   forzato (≥20), ms/evict del re-lower, rate reale (~8/1000 req =
   possibile verdetto "non vale una chiave nuova").
4. **SPIKE DISPATCH (sessione dedicata, SBLOCCATA)**: K-M71.1 è
   sciolto (batteria M-71.1 verde + grep-gate). Restano vincolanti:
   B-71.1/KB71-1 (cifre ri-pinnate a script: 95+90+312, 104/178
   puliti), forma B-71.2 DispatchOutcome, tier-1 hot×puliti, recinto
   P70-S (cap 2%/5%, coppia stessa-sera), KH70-3 grep-gate sul diff.
5. **Backlog**: S-71.3 trait-redecl cross-unit (pin oracle pronto in
   pins/s71-vi_trait.oracle-target.txt) · famiglia STRING-OFFSET-BIND
   (t6/probe-strbind: catalogare in PHPR_DIVERGENCES o chiudere) ·
   famiglia DIAG-LINE-attraverso-alias (t4+wopset) · famiglia
   quiet-fetch (t7, H-71.5) · famiglia EVAL-NAMING · Δbytes STUBS ·
   metro no-swap · quinta coppia G-66.4 · replica quota 7,84.

**VETI (restano vincolanti)**: invariati WP-64..70 + WP-72 (concilio):
**leg di misura da comandi inline** (P-72.1/KS-P72.1: solo runner
committati, hash pre-letto K-72.2); **conteggi per-pattern del
blockdump citati come misura** (KG72-4: solo foglie address-free);
**delibera digest ways/fp su singolo flip** (B-72.5); **claim di
famiglia oltre il recinto del gate** (M-72.1/H-72.2). E nuovi WP-71:
**letto zero senza CONTROLLO POSITIVO dello strumento = NON-CITABILE**
(lezione H1-C1: il canale deve dimostrare di firare nello scope);
**bande sbagliate si correggono con un NUOVO lock derivato dal letto,
mai con aggiustamento post-hoc** (amplificazione K=3→K=10, KG71-2);
**attribuzione di un ritenuto = contenuto dei blocchi o retained-form,
mai net/flow** (KK71-1/KK71-3 rinnovati); verdict SOLO da script
committato (KS-P71.1, ora prassi).


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
Ultimo stato (WP-70, sessione fedeltà+caccia — gap classico NON
rimisurato): **riferimenti invariati: media CPU 2,58× · footprint
media ~3,0-3,1 (banda 2,9-3,2) · full CPU 2,06-2,11× · peak
~1,98-2,03GB · server hit_cross 512/512 · corpus RI-PIN 1422→1421
(−1 fix reale) · 🔴 residuo per-request CONFERMATO E MISURATO:
esiste su RELEASE (Δ5,8 MiB/4000 req) + tripla 2,1121 KiB/req ·
20,000 obj/req (spread 0,024/0,003 = MDE fondato) · defer path
16,00 calls/req · attribuzione ≥80% APERTA ⇒ axum FERMO · baseline
parità phpr-wp70 (a666382e…, gate70 PASS fails=0 1° run) ·
dettaglio: `sessions/WP_SESSION_70.md`, trend: `gaps/GAP_TREND.md`**.
