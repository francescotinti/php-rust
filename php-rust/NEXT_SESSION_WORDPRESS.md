# Rotta WORDPRESS-FIRST — WP-track (dopo WP-59: la MAPPA del fuori-canale è FATTA — frag mimalloc 2% (ipotesi concilio FALSIFICATA), ~65% del fisico = compile-side VIVO non censito (HIR seeds + payload op); leak template-include CONFERMATO; obj de-fantasmato 56,1→48,7MB; ⚖️ verbale revert leva B aperto)

> ⚡ **WP-59 (2026-07-26, `c52c394`+`1e06464`+`e22f373`+docs)** — **Sessione di MISURA del
> concilio esteso** (binari di parità INTATTI = phpr-wp58 2d7efdf8…;
> codice solo census-only + 2 feature diagnostiche). **Ob.0**: mimalloc
> v3.3.2, committed peak ≈ 93-102% del phys peak su media E full ⇒ il
> mistero vive DENTRO l'allocatore; FFI ~90-120MB; maxrss onesto (sul
> full 2,3G in compressor). **Ob.1 (finestre phys nel census,
> `c52c394`)**: gate pre-registrato riconciliato ALLA CIFRA (win10: phys
> 1436,2 = Σused 1298,0 + frag 29,8 + resto 108,4) ⇒ **frammentazione
> 2% = ipotesi dominante del concilio FALSIFICATA; non-censito VIVO
> 936,6MB = 65%**, e `--list-tests` (0 test) ne mostra 818MB già a fine
> bootstrap+discovery ⇒ il footprint è ~90% COSTRUZIONE DELLA SUITE.
> Probe differenziale classi-vs-funzioni: compile-side = 3,8× il counted
> sulle classi (1,5× funcs) ⇒ **canale unit VERO ≈ 1,0GB** (channel 222MB
> = 1/5); colpevoli con nome: `seed_classes`/`main_hir` (HIR con
> `MethodDecl.body` interi, ≈2/3) + payload Rc op (≈1/3). **Probe B:
> LEAK template-include CONFERMATO** (200 re-include ⇒ unit.cum_n=200):
> Fase 0.5 SI APRE. **Ob.2**: 46k unreached = FANTASMI del choke next_id
> (closure/generator; ipotesi 1 Gregg) — fix `next_obj_id`, recon obj
> 22.141==22.141 ESATTO, **obj peak 56,1→48,7MB**. **Ob.3 (3 full
> stessa-sera, CPU esatta time -l)**: new 790,83s (fail-set 88 BYTE-ID a
> run33 ✓), pool-off 786,44s = **−0,56%** (unico costo identificato,
> footprint ±0), scan4 796,05s = **+0,66%** (scan-mode ASSOLTO; spread
> serata ±0,6% ⇒ banda WP-58 in buona parte rumore del campionatore) —
> ⚖️ verbale revert leva B all'utente (racc.: revert). Rapporti
> INVARIATI: media 2,61× · footprint 4,07× · full rif. 2,11×.
> Census: **phpr-memgc59 (00ca4d3d…)**; diagnostici phpr-pooloff59
> (93aea5a3…), phpr-scan4-59 (b955afb1…). **Storia: `sessions/
> WP_SESSION_59.md`. WP-58: `sessions/WP_SESSION_58.md`.**

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

## Stato gate per nome (gate58 su tree WP-58, 2026-07-26, `wp58-harness/gate-out/`: corpus **1421 IDENTICO** · refl **290 IDENTICO** · ORM **3E/13F IDENTICO** · hk **0E/0F** · cargo **1645/0**; release = **phpr-wp58 2d7efdf8…**; fail-set FULL ri-validato: run46 + run46-old BYTE-ID a run33 (fix yield_from incluso); ultima verifica gate22 integrale: `60c7e04`)

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

## 🎯 PROSSIMO LAVORO — WP-60: dieta del compile-side (la mappa WP-59 comanda; programma post-concilio)

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle. **La mappa WP-59 ha ribaltato le priorità**:
frammentazione mimalloc 2% (ipotesi concilio falsificata); il giacimento
è il **compile-side vivo non censito ≈800MB** (HIR seeds ≈2/3 + payload
Rc op ≈1/3) + **leak template-include** (Fase 0.5). Dettaglio:
`sessions/WP_SESSION_59.md` + `wp59-harness/design59.md`.

1. **Quota ex-ante delle due leve compile-side** (census-only, PRIMA di
   scrivere le leve — regola "byte misurati"):
   (a) contatore bytes HIR per classe, firma-vs-corpo (`MethodDecl.body`)
   nei seed (`seed_classes`/`seed_traits`/`main_hir`);
   (b) conteggio unit per path risolto (quota del leak template:
   quante delle 2046 unit media sono re-compile dello stesso file);
   (c) gauge G_UNITS (oggi morto) + breakdown payload Rc op.
2. **Fase 0.5 — compile-cache keyed sul path risolto** (roadmap: "è un
   LEAK, si chiude prima di attribuire il resto"): riuso del Module già
   linkato per include non-`_once` ripetuti. Parità: gate PIENO (classe
   layout/engine) + full A/B stessa-sera.
3. **Seed HIR signature-only**: il lowering di una sottoclasse usa
   forme/firme, non i corpi ⇒ clone snello al push in `seed_classes`
   (corpi droppati), stesso trattamento per `main_hir` se separabile.
   Banda potenziale: centinaia di MB — da confermare col contatore 1a.
   Rischio parità: eval che ri-lowera con l'immagine (gate pieno).
4. **⚖️ Revert leva B (pool): DECISO dall'utente (2026-07-26 sera, sul
   verbale Ob.3)** — pool-off −0,56% full, footprint ±0, beneficio B
   nullo (il −17MB di WP-58 era A+C); concorde la sedia Leijen (unica
   pronunciata). Esecuzione in WP-60: revert della SOLA B (A e C
   restano; rimuovere anche mod pool + feature pool-off ormai inutile),
   gate PIENO (corpus 1421 per nome + refl 290 + cargo + ORM 3E/13F +
   hk 0E/0F + sentinelle 5 assi) + coppia full stessa-sera.
5. **Quote Stogov declassate dalla mappa** (solo se margine): duplicati
   str (tetto ≤62MB), literal-array, peak oracle per-test.
   (Il "residuo full-only" WP-58 è CHIUSO da Ob.3: pool ~0,6% + rumore
   del campionatore; nei prossimi harness full usare SEMPRE
   /usr/bin/time -l per la CPU.)

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
4. **NON riproporre**: **ipotesi frammentazione/ritenzione mimalloc come
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
Ultimo stato (WP-59, sessione di misura — parità intatta):
**media CPU 2,61× · footprint 4,07× (INVARIATI, nessun A/B) · full:
riferimento run46 2,11×; Ob.3 stessa-sera (metro nuovo time -l, tree
user): new 790,83s = fail-set 88 BYTE-ID a run33 ✓, pool-off −0,56%,
scan4 +0,66% (assolto) · obj peak de-fantasmato 48,7MB · LA MAPPA: frag 2%,
compile-side vivo ≈800MB = la leva di WP-60 · dettaglio:
`gaps/REPORT_GAP_59.md`, trend: `gaps/GAP_TREND.md`**.
