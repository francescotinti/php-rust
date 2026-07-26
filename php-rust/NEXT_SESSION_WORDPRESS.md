# Rotta WORDPRESS-FIRST — WP-track (dopo WP-57: sessione di QUOTA — canale `.=` non-locale MORTO sul full (0,70MB≈23µs, fuso esteso NON si apre); arr peak ESATTO 66,4MB = 4,3% del fisico ⇒ tranche 2 arena vale ~−1%; WP-58 = decisione di rotta sul fuori-canale)

> ⚡ **WP-57 (2026-07-26, `b4e85ca`+`b23de56`+`0b5790f`)** — **Sessione di
> QUOTA, binari di parità INTATTI** (release verificato = `phpr-wp56`
> 65466c64… ⇒ nessun gate; riferimenti perf WP-56 restano validi).
> **Ob.1**: op-census esteso a 6 famiglie di siti `.=` non-locali
> (eventi + Σ lhs byte = volume O(n²) + istogramma log2(lhs); scoperta:
> `$o->p .= x` NON emette PropOpSet — lower a Swap;Binary;PropSet ⇒
> macchinetta a stati nel census; validato ORACLE alla cifra sul probe56,
> banda rebuild ~30GB/s). **VERDETTO sul FULL (prima run census di sempre
> sul full): 48.934 eventi, Σ lhs 0,70MB ≈ 23µs — canale MORTO, Ob.1c
> (fuso esteso prop/element) NON si apre**: il ~500×/evento del probe è
> reale ma il workload non copre MAI il bordo (nessun lhs >1KB); media:
> 221 ev. Lo "str-copy 12%" WP-54 era il sito LOCALE (già chiuso WP-55).
> **Ob.2**: canale arr da estimatore a **live-accounting ESATTO**
> (accounted+census_sync+Drop, drift-proof; riconciliazione reached==live
> alla UNITÀ: 63.435==63.435) + istogramma per-repr nel walk EOR.
> **arr peak ESATTO 66,4MB = ~4,3% del peak fisico 1.536MB** (il "39% del
> proxy" WP-55 era l'artefatto, 6,0× alla cifra); str peak 62,3MB; unit
> standing 222,6MB; popolazione arr = MINUSCOLI (packed 1-2 el 22k;
> hashed 1-4 el 21k; overhead fisso 64B/arr ≈30% del canale).
> **Ri-quota tranche 2: arena handle ≈ −10..−20MB (~−1% fisico),
> exact-fit ≈ −1..−2MB — la leva GRANDE di footprint è FUORI dal canale
> arr** (~1,15GB fuori dai canali valore + unit 222,6MB). Divergenze
> WP-55/56 catalogate in PHPR_DIVERGENCES (`0b5790f`). ⚠️ NUOVO a
> backlog: panic census-only run.rs:478 (ip==len, non-deterministico,
> mai sui binari di parità — evidenza in WP_SESSION_57). Binari census:
> `phpr-op57` (dcb589ea…), `phpr-memgc57` (41a21c65…). **Storia:
> `sessions/WP_SESSION_57.md`. WP-56: `sessions/WP_SESSION_56.md`.**

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

## Stato gate per nome (gate56 su `0943f58` resta CORRENTE — WP-57 non ha toccato i binari di parità: i commit b4e85ca/b23de56 sono solo codice `#[cfg(op-census|mem-census)]` compilato via, release verificato = phpr-wp56 65466c64…; ultima verifica gate22 integrale: `60c7e04`)

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
#  NESSUNO stash nuovo: parità invariata, resta phpr-wp56.
```

## 🎯 PROSSIMO LAVORO — WP-58: tranche 2 Fase 3 coi numeri veri O attacco al fuori-canale (decisione di rotta)

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle. WP-57 (quota) ha chiuso ENTRAMBE le metà del
mandato: canale `.=` non-locale MORTO (0,70MB sul full) e arr peak esatto
66,4MB = 4,3% del fisico.

1. **DECISIONE DI ROTTA (utente)**: con l'arena handle-based quotata
   onestamente a ~−10..−20MB (~−1% fisico), l'ordine sensato dell'arco
   diventa: (a) eseguire comunque la tranche 2 arena (direttiva "tutte le
   fasi"; banda piccola ma CPU-side l'indice keyless ha già pagato −2,66%
   full — l'arena può ripetere il pattern su alloc/dealloc); oppure
   (b) prima il FUORI-CANALE: il fisico 1.536MB ha ~1,15GB oltre i canali
   valore (attribuzione Fase 0/WP-54: allocatore/frammentazione/COW
   PHPUnit) + unit 222,6MB standing. Il verbale WP-57 consegna i numeri;
   la scelta d'ordine è di rango utente.
2. **Metro**: estendere accounted+census_sync al canale obj (ultimo
   death-accounted; pattern validato alla unità su arr).
3. **Residuo full vs oracle (2,06×)**: attribuzione WP-54 aggiornata:
   corpi+dispatch 41,9% · gc-walk 10% · crypt onesto; str-copy residuo =
   copie GENERICHE (i siti compound non-locali valgono 23µs — chiuso).
4. **NON riproporre**: **fuso `.=` sui siti non-locali (prop/element/
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
Ultimo stato (WP-57, sessione di quota — riferimenti perf = WP-56):
**media CPU 2,61× · footprint 4,08× · full run45 2,06× (binari parità
INTATTI, phpr-wp56 65466c64…) · QUOTA: canale `.=` non-locale MORTO sul
full (48.934 ev / 0,70MB ≈ 23µs — fuso esteso non si apre) · arr peak
ESATTO 66,4MB = 4,3% del fisico (estimatore WP-55 era 6,0× over alla
cifra; riconciliazione reached==live 63.435==63.435) · tranche 2 arena
ri-quotata ~−10..−20MB (~−1%) · dettaglio: `gaps/REPORT_GAP_57.md`,
trend: `gaps/GAP_TREND.md`**.
