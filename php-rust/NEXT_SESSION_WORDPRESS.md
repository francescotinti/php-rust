# Rotta WORDPRESS-FIRST — WP-track (dopo WP-61: 🏁 php-server STANDALONE consegnato — WordPress admin inclusa su wpdev/src, accettazione 5/6 BYTE-ID + admin NORM-ID vs phpr -S; census v2 deep ESEGUITO: il ~600MB non-seed è ATTRIBUITO — net unit 648,8MB su --list-tests: CompiledClass-priv 58%, module-owned 31%, payload-Rc solo 14%; mappa FULL v2 detached da leggere come PRIMO ATTO di WP-62)

> ⚡ **WP-61 (2026-07-26, `c0eb97a`+`242a7ee`+`9ada23d`)** — **P1 (direttiva
> utente): php-server AUTONOMO consegnato.** php-cli esposto anche come lib
> (lib.rs: server+mime, main.rs INTATTO); php-server riscritto da wrapper
> axum GET-only a launcher sottile su `php_cli::server::serve`
> (--host/--port/--docroot/router, mimalloc, de-gitignorato). wpdev/src
> servito per la PRIMA volta: wp-config NUOVO → DB `wp` (era vuoto; grant
> aggiunto; wp_o/wp_p intatti), install via oracle su porta 8080, pretty
> permalinks; 🔵 wordpress-develop src/ ha un guard anti-unbuilt ⇒
> `npm install` + `grunt build --dev` obbligatori (senza, 500 + admin
> senza JS). **Accettazione meccanica VERDE**: front/asset/permalink/
> wp-login GET/POST **BYTE-ID** php-server vs phpr -S, dashboard admin
> **NORM-ID** (solo _wpnonce di sessione). Gate solo-server: cargo
> **1645/0**; ⚠️ hash phpr cambiato dai metadati del lib target ⇒
> ri-stash **phpr-wp61 (c7e93597…) = baseline di parità** (wp60 accanto).
> Quick-start php-server PUBBLICATO nel README. **P2: census v2 deep
> ESEGUITO** (metro nuovo: net-compile delta per unit ai 3 siti compile
> = deep VERO payload-Rc inclusi; walk address-dedup con split
> priv/shared per strong_count; binario phpr-memgc61 9600a662…):
> su `--list-tests` **net_tot 648,8MB su 1950 unit** (counted-v1 211MB)
> ⇒ il ~600MB di WP-60 è CHIUSO: **cls_priv 325,6MB (58%) + mod_owned
> 202,9MB + fns_priv 30,7MB; rc_residue (payload op) SOLO 89,6MB (14%)**
> — l'ipotesi payload-op ridimensionata; opkind owned=0 (payload tutti
> Rc); literal dup cross-unit 11,7MB (interning = banda modesta);
> net/re-include ~40× il counted (il template paga il prefisso stub del
> seed accumulato). **Mappa FULL v2 detached**:
> `wp61-harness/census-out/memcensus61-full.txt` (flag `.done`) — da
> leggere a inizio WP-62. **Storia: `sessions/WP_SESSION_61.md`. WP-60:
> `sessions/WP_SESSION_60.md`.**

## 📁 Convenzioni (decisione utente 2026-07-23)

- Qui SOLO: sintesi ultima sessione · decisioni in vigore · stato gate ·
  prossimo lavoro · backlog. Storia: `sessions/WP_SESSION_<n>.md` (un file
  per sessione, con lezioni e verdetti; ≤WP-27: memoria + git history).
  Gap perf: `gaps/REPORT_GAP_<n>.md` = SOLO le misure della sessione n
  (correzione utente 2026-07-25: la vecchia forma cumulativa era una
  trascrizione sbagliata dell'intento); il trend cumulativo vive in
  `gaps/GAP_TREND.md` (file unico vivo, una riga per sessione).
- Chiusura della sessione N: scrivere `sessions/WP_SESSION_N.md`; scrivere
  `gaps/REPORT_GAP_N.md` con le SOLE misure della sessione N e aggiungere
  la riga N a `gaps/GAP_TREND.md`; sostituire la sintesi qui in testa e
  aggiornare stato gate / prossimo lavoro; commit+push.

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

## Stato gate per nome (WP-61 `9ada23d`: cambio SOLO server-layer+census ⇒ gate ridotto da direttiva = cargo **1645/0** + batteria HTTP php-server vs phpr -S **5/6 BYTE-ID + admin NORM-ID**; release = **phpr-wp61 c7e93597…** — hash cambiato dai soli metadati del lib target, main.rs intatto, ri-stash a verbale; ultimo gate PIENO = gate60 su `915ea27`, `wp60-harness/gate-out/`: corpus **1421 IDENTICO** · refl **290 IDENTICO** · ORM **3E/13F IDENTICO** · hk **0E/0F** · sentinelle 5 assi **BYTE-ID**; fail-set FULL run48 BYTE-ID a run33; ultima verifica gate22 integrale: `60c7e04`)

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

## 🎯 PROSSIMO LAVORO — WP-62: mappa FULL v2 → Fase 0.5 compile-cache → seed signature-only (A/B SEPARATI; il metro ora è il NET del census v2)

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, no-revert. Laravel POSTICIPATA a valle.
Stato WP-61: php-server consegnato (verifica personale = Francesco);
census v2 attribuito su --list-tests. Dettaglio: `sessions/
WP_SESSION_61.md` + `wp61-harness/design61.md`.

0. **PRIMO ATTO: leggere la mappa FULL v2** (`wp61-harness/census-out/
   memcensus61-full.txt`, run detached di fine WP-61 — verificare flag
   `census61-full.done`): ri-quotare in NET la banda Fase 0.5 (era
   −240MB+ counted; lo smoke dice net/counted ~2-3× sul dup) e leggere
   il modrecon del full (su --list-tests: cls_priv 58% / mod_owned 31% /
   rc_residue 14% — se il full conferma, le leve future guardano ANCHE a
   CompiledClass/Module tables, non solo al payload op).
   ⚠️ **uploads**: da quando Francesco carica media nella verifica
   personale, i futuri harness full NON devono più azzerare
   `src/wp-content/uploads` senza backup (census61-full è stato l'ultimo
   wipe naive a dir vuota).
1. **Fase 0.5 compile-cache = RILASSARE la chiave della unit-cache
   ESISTENTE** (WP-20, `run_include`: unit_key path+mtime+size c'è già,
   il double-check strutturale c'è già; il miss è causato da
   `unit_chain_fp` foldato a ogni load). Vincoli pinnati: celle static
   FRESCHE a ogni esecuzione (P4-i, phpr==oracolo OGGI: non
   regredire); fatal redeclare preservato (P4-iii); hash contenuto in
   fallback stesso-secondo (V1); hit/miss distinti nel census; probe
   write→include→rewrite→include nel gate. **Predizione ex-ante (da
   registrare in design61 PRIMA dell'A/B): −240MB+ counted sul FULL**
   (bersagli nominati: version.php ×899, script-modules-packages ×704,
   script-loader-packages ×400 = 204MB), **media ≈ 0 (0,18MB: leva
   full-only)** ⇒ il metro è la coppia full + mappa census
   (memcensus60-full come baseline) + phys per-bin (drenaggio atteso
   nei bin standing). Gate PIENO (la cache è semanticamente invisibile
   come lo era il pool: ogni diff = allarme).
2. **DOPO, in A/B SEPARATO: seed signature-only con banda MISURATA
   ~130-195MB** (corpi 195,0MB × gate fisico Leijen ≥80%): si strippa
   SOLO `MethodDecl.body`+`slots` di seed_classes/main_hir;
   **seed_traits INTATTO** (i corpi servono al flattening); PropDecl/
   consts/enum_cases/attributes/abstract_sigs interi; seed snello
   EX-NOVO (mai clone-poi-strip: il picco transitorio è il metro);
   sentinella P4-iv (reflection dal CompiledClass) + P4-v (eval
   extends) nel gate; predicted-vs-actual ±15%; frag picco ≤ ~2×29,8MB.
3. Post-dieta (code): footprint(1) compressed/dirty sul full
   (dividendo-compressor, Leijen R4); exit-frag 171MB solo da osservare
   (R6); FFI malloc_history solo se il residuo lo giustifica (R5).
4. 🆕 **php-server: front-end axum (richiesta utente 2026-07-26,
   sessione dedicata)** — oggi php-server è il SAPI sequenziale
   byte-parity (scelta giusta per l'estrazione e i probe); l'evoluzione
   è axum davanti (HTTP production-grade: keep-alive, TLS, concorrenza)
   + pool di worker engine dietro (il VM è `Rc`/!Send ⇒ un engine per
   thread, dispatch blocking). La parità byte del SAPI resta il metro
   dei probe di accettazione.

**VETI del concilio (restano vincolanti)**: mai toccare
MIMALLOC_PURGE_DELAY nei giudici; mai giudicare ritenzione col maxrss
(WP-59: sul full 2,3G in compressor — maxrss 1,5G vs phys 3,9G);
nessun nuovo pool/freelist sopra mimalloc (verdetto Ob.3: il pool
esistente costa ~0,6% full a beneficio nullo); nessun reset per-test al
boundary PHPUnit (static-props = SEMANTICA); mai `mi_heap_destroy` su
heap con Drop Rust; `mi_collect(true)` solo nei build census
etichettati; malloc_history/Instruments-Allocations sono CIECHI su
mimalloc (⚠️ correzione WP-59: su macOS 26 il heap mimalloc appare come
regioni **IOAccelerator** — os_tag 100 — NON VM_ALLOCATE); mai env
MIMALLOC_SHOW_STATS/VERBOSE su run di cui serve il fail-set (propaga ai
figli ⇒ errori artefatti); DB reset anche nei PROBE che bootstrappano
phpunit (il flake wp_install di Ob.3 nasce da lì); interning (futuro):
Rc forti in tabella + mai nel path append + re-gate output refcount;
immutable-literal (futuro): solo tutto-scalari + cursore separabile.
4. **NON riproporre**: **la ripartizione "seeds ≈2/3 del compile-side
   ≈530MB" (WP-60: contatore diretto = 221MB, corpi 195 — le bande
   seed-thin si citano SOLO dalle righe tag=seedsum)**; **la colonna
   `aband` del visitor mimalloc come attribuzione (WP-60:
   positive-control DEAD — "mimalloc non-visitato" è l'unica etichetta
   onesta)**; **leva B/pool sopra mimalloc (revertata e ri-misurata:
   −0,93% CPU, footprint −0,19% — il verdetto sperimentale è
   definitivo)**; **ipotesi frammentazione/ritenzione mimalloc come
   leva footprint (WP-59: 2% al picco, riconciliazione alla cifra —
   riaprire SOLO con una tabella per-bin che dica altro)**; **interning
   come leva grande di footprint (WP-59: il giacimento è compile-side;
   tetto diretto str ≤62MB)**; **arena condivisa a handle u32 per le entries
   (WP-58 pin a: restituire `&/&mut` da uno slab condiviso richiede
   unsafe o riscrittura closure-based della VM — RULEBOOK §0; il
   rounding lato entries è ≈0, i buffer pow2 cadono su bin mimalloc
   esatti)**; **fuso `.=` sui siti non-locali (prop/element/
   static — WP-57: 48.934 ev / 0,70MB / 23µs sul FULL, falsificato dal
   workload; riaprire SOLO con un workload nuovo che copra il bordo)**; **cap reflect 16384→8192 (DECISIONE UTENTE
   2026-07-25: resta 16384** — working set misurato ~17,8k appena sopra
   il cap, evictions 1/run: dimezzare = thrash garantito per un
   risparmio non quotato); stringhe come pilota Fase 3 (checkpoint WP-55:
   4% del proxy — falsificate); fusione single-alloc (incompatibile con
   growable + 0,3% CPU — WP-54); Fase 2.3 args-Vec pool (0,14% — morta);
   leaf-bit sul walk (foglie 6,6% degli SLOT — WP-54); leve sul canale
   crypt (probe: phpr −14% vs oracle = onesto); Fase 1.2 created→Weak;
   rooting selettivo; cap rerun stile Zend; bound/guardie negli arm caldi
   (WP-50); soglia adattiva su buffer RAW; collect per-test; cold-box
   dyn_entries (WP-52); giudizi footprint da RSS telemetrico; Sweep-
   elision oltre whitelist (WP-53); micro-op quotati solo in conteggi;
   giudizi full da momenti diversi della giornata (WP-55: deriva ~2,6%);
   **quote di canale sull'estimatore live×death-avg** (WP-56: arr 5,7×,
   str 4,9× di over-count — solo reached-set o live-accounting esatto);
   bande di canale senza istogramma per-repr della popolazione (WP-56).

## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT DISCO**: `df -h /System/Volumes/Data`, non partire sotto
   ~15-20G liberi (WP-49 partita a 16-17G dopo rm del debug/ 834M
   ricomparso; in sessione usare SEMPRE `cargo test --release`, e pulire
   `debug/` se ricompare).
   **PRE-FLIGHT MYSQL**: `mysql -h 127.0.0.1 -u root -e "SHOW DATABASES"`
   deve elencare wp_o/wp_p/probe — altrimenti vedi ⚠️ MySQL sopra.
   **STASH OLD**: copiare subito il binario release corrente in
   `/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-wp50`
   (old dell'A/B di WP-51). ⚠️ path CANONICO = drive esterno: lo stash
   in `~/Claude/phpr-old-target` (WP-50) ha prodotto un A/B con 12 run
   da 0,00s.
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
Ultimo stato (WP-61, consegna php-server + census v2 — perf NON
rimisurata, engine intatto):
**media CPU 2,58× · footprint 4,08× · full 2,11× (tutti riferimento
WP-60) · baseline parità phpr-wp61 (c7e93597…, ri-stash per metadati
lib target — cargo 1645/0) · census v2: net unit 648,8MB su
--list-tests (cls_priv 58% / mod_owned 31% / rc_residue 14%), literal
dup 11,7MB, mappa FULL v2 in census-out (WP-62) · dettaglio:
`sessions/WP_SESSION_61.md`, trend: `gaps/GAP_TREND.md`**.
