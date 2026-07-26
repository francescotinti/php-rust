# Rotta WORDPRESS-FIRST — WP-track (dopo WP-58: Fase 3 tranche 2 "arena" ESEGUITA — peak fisico −1,05% (−17MB, dentro banda), media CPU FLAT, full +1..+2,5% solo-full TENUTO; obj live-ESATTO; WP-59 = attribuzione fuori-canale, decisione utente)

> ⚡ **WP-58 (2026-07-26, `8e54185`→`ff033ce`)** — **Fase 3 tranche 2
> eseguita** (decisione utente, banda onesta nota). Pin (a) su size
> MISURATE: blocco `Rc<RefCell<PhpArray>>` 104B→bin 112; buffer entries/
> index già su bin esatti ⇒ rounding entries ≈ 0 e l'arena condivisa a
> handle u32 è IRRAGGIUNGIBILE in safe Rust (RULEBOOK §0) ⇒ tranche
> materializzata in TRE leve (array.rs, sentinelle 5 assi byte-id):
> **A dieta header** (cursor+holds in una u32 ⇒ blocco 96B = bin esatto,
> −16B/array reali) · **C scan-mode ≤8 slot** (niente KeyIndex sui
> piccoli: 21,4k standing senza indice) · **B block-arena**
> ownership-transfer (shelf thread-local pow2 bounded <700KB su crescita/
> rebuild/Drop; zero indirection = zero parità). **GATE58 VERDE: corpus
> 1421 IDENTICO · refl 290 · ORM 3E/13F · hk 0E/0F · cargo 1645/0.**
> **Giudici**: ab58 → **peak fisico −1,05% = −17MB (1,619→1,602G, dentro
> banda −10..−20MB) · CPU media −0,13% FLAT** (guardia pilota ok);
> mechanism-check ALLA CIFRA (hashed b1/b2 = −32B×n esatti; recon arr
> 63.435==63.435 intatta). **FULL stessa-sera: run46 e run46-old
> ENTRAMBE 0E/2F/86W/73S, fail-set 88 BYTE-ID a run33 ×2 ⇒ fix
> yield_from VALIDATO sul full** (⚠️ WP-57 chiusa; baseline resta run33).
> ⚠️ CPU full new +1,0..+2,5% (raw 714,4 vs 696,6s; troncamento
> campionatore asimmetrico) — regressione SOLO-full (media flat), TENUTA
> per no-revert; attribuzione a WP-59 (candidati: TLS+RefCell del pool
> sul Drop path, scan 5-8 slot); cumulato Fase 3 full resta negativo.
> **Ob.2**: live-accounting ESATTO esteso a obj (Props accounted+sync+
> Drop + parte fissa al choke next_id; rare/proxy walk-only) — validato
> alla cifra (1501==1501, 352.208==352.208); **prima misura esatta:
> obj peak 56,1MB ≈ 3,7% del fisico; canali valore al picco ≈ 12%** —
> la leva GRANDE resta FUORI (unit 222,6MB + ~1,1GB non attribuito).
> Rapporti: media CPU 2,61× · footprint 4,07× · full 2,11× di giornata.
> Release: **phpr-wp58 (2d7efdf8…)**; census phpr-memgc58 (4c9b8da3…) e
> phpr-memgc58b con Ob.2 (bc47f237…). **Storia: `sessions/
> WP_SESSION_58.md`. WP-57: `sessions/WP_SESSION_57.md`.**

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
```

## 🎯 PROSSIMO LAVORO — WP-59: attribuzione fuori-canale (programma del CONCILIO ESTESO — review integrale delle 3 sedie in `doc/analysis/COUNCIL_WP58_REVIEWS.md`, ⚖️ mandato utente 2026-07-26: "integra i loro feedback e richieste nella prossima sessione")

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle. Convergenza 3/3 delle nuove sedie
(Leijen/Stogov/Gregg): PRIMA la quota falsificata del ~1,1GB
fuori-canale; ipotesi dominante = ritenzione/frammentazione per-pagina
mimalloc accoppiata al churn (~6GB/run; unit immortali `Box::leak`
interleavate con l'effimero negli stessi bin = page-pinning) + strutture
runtime non censite (frame, IC, Rc sciolti, slack Vec).

1. **Ob.0 (Gregg R1 / Leijen 1 — ZERO codice, ~20-30 min, FARE PER
   PRIMO)**: run media + full con `MIMALLOC_SHOW_STATS=1` (+VERBOSE) e
   one-shot `vmmap --summary` al picco vs maxrss dello stesso run.
   Falsifica subito: committed mimalloc ≪ fisico ⇒ il residuo NON è
   nell'allocatore; maxrss ≫ vmmap-phys ⇒ quota MADV_FREE contabile.
   Può ridimensionare il mistero prima di spendere altro.
2. **Ob.1 — strumentazione census "Fase 0-bis"** (Leijen 2+3, Gregg
   R2-R4; build census, mai giudici): (a) watermark RI-CHIAVATO sul
   fisico vero (`task_info` phys_footprint, +128MB, tag col nome del
   test) + flag-file; (b) `mi_stats_print_out` + `mi_process_info` nel
   callback; (c) occupancy per size-class al picco via
   `mi_heap_visit_blocks` (visit_blocks=false); (d) supervisore esterno
   vmmap+footprint per finestra (⚠️ heap mimalloc = regioni VM_ALLOCATE,
   NON MALLOC_*; le MALLOC_* = zone FFI). **Gate pre-registrato**:
   Σ used_bins = census+non-censiti; Σ(committed−used) = frammentazione;
   +metadata ≈ fisico ±10-15%; tabella copre ≥90% del 1,1GB con nome
   del test per finestra.
3. **Ob.2 — probe anomalia 46k obj unreached** (Gregg §3, ~30 righe
   census-only): istogramma per-ClassId/kind della differenza
   registry−reached + `collect_cycles` forzato pre-walk. Sospetto n.1:
   sovra-conteggio al choke `next_id` (condiviso con closure/generator/
   handle — firma: created==reached alla cifra). ⚠️ Se confermato, il
   fix contabile corregge ANCHE l'obj_fixed di WP-58-Ob.2 (alloca la
   parte fissa a ids non-Object mai liberati) e sgonfia il canale obj.
4. **Ob.3 — attribuzione regressione full-only +1..+2,5%** (Leijen 4):
   binario `pool-off` (feature/`class()`→None, NON DEPTH=0 che paga il
   TLS comunque) vs new, full STESSA-SERA; se il delta resta, secondo
   binario `SCAN_MAX=4`. UN asse per binario. Se il pool è colpevole:
   la roadmap Fase 3 consente il revert della SOLA versione regressiva
   (⚖️ decisione utente al verbale — A e C restano comunque).
5. **Quote Stogov (se margine, altrimenti WP-60; mezza giornata l'una)**:
   (a) censimento duplicati contenuto-stringa al mark (tetto interning);
   (b) censimento literal-array (quota `[]` vuoti + tutto-costanti dei
   5,46M cum ⇒ banda empty-singleton/immutable-literal); (c)
   `memory_get_peak_usage` per-test lato oracle (target per-canale
   onesto); (d) breakdown unit per-componente (ops/payload/literal/meta
   + duplicazione cross-unit) — **unit diet = la più grande leva GIÀ
   attribuita: banda Stogov −80..−150MB**. Giudizio Stogov sull'ordine:
   mappa → unit diet → interning (solo col censimento in mano; il suo
   valore vero è uccidere il churn alla fonte se Ob.1 attribuisce
   ≥400MB alla frammentazione churn-correlata).
6. **Residuo CPU full vs oracle (2,11× di giornata)**: attribuzione
   WP-54 vigente (corpi+dispatch 41,9% · gc-walk 10% · crypt onesto).

**VETI del concilio (vincolanti per WP-59)**: mai toccare
MIMALLOC_PURGE_DELAY nei giudici; mai giudicare ritenzione col maxrss;
nessun nuovo pool/freelist sopra mimalloc prima del verdetto Ob.3;
nessun reset per-test al boundary PHPUnit (static-props = SEMANTICA);
mai `mi_heap_destroy` su heap con Drop Rust; `mi_collect(true)` solo
nei build census etichettati; malloc_history/Instruments-Allocations
sono CIECHI su mimalloc (heap = VM_ALLOCATE); interning (futuro): Rc
forti in tabella + mai nel path append + re-gate output refcount;
immutable-literal (futuro): solo tutto-scalari + cursore separabile.
4. **NON riproporre**: **arena condivisa a handle u32 per le entries
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
Ultimo stato (WP-58, Fase 3 tranche 2):
**media CPU 2,61× (A/B −0,13% flat) · footprint 4,07× (A/B −1,05% =
−17MB, dentro banda) · full run46 2,11× di giornata (⚠️ +1,0..+2,5%
solo-full vs old stessa-sera, TENUTO no-revert; fail-set ×2 byte-id a
run33 = fix yield_from validato) · obj peak ESATTO 56,1MB = 3,7% del
fisico (ultimo estimatore morto; canali valore ≈12% al picco) ·
dettaglio: `gaps/REPORT_GAP_58.md`, trend: `gaps/GAP_TREND.md`**.
