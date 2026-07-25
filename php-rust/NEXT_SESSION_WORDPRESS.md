# Rotta WORDPRESS-FIRST — WP-track (dopo WP-56: Fase 3 tranche 1, indice keyless PhpArray = footprint −1,79%, full −2,66% GRATIS → 2,06× NUOVO MINIMO; WP-57 = quota frequenza `.=` per sito + ri-quota arco Fase 3 su metro non-biased)

> ⚡ **WP-56 (2026-07-26, `0943f58`)** — **Fase 3 pilota HASHED-ARRAY,
> tranche 1: indice keyless** (tabella singola Zend-style). `Repr::Hashed`
> perde la `FxHashMap<Key,u32>` (Key DUPLICATA, ~25B+ctrl/bucket) per
> `KeyIndex`: open-addressing `Box<[u32]>` di sole posizioni, chiave
> letta da `entries[pos]`; `entries` INTATTE ⇒ i 4 assi di parità
> (ordine, tombstone, chiavi num-str, dtor-order) invariati per
> costruzione; hash = fmix64(int | zhash cached); α≤1/2; rebuild dai
> VECCHI slot; arena-compatibile (solo u32). **Giudici: ab56 footprint
> peak −1,79% (−28,1MB, 6/6 pulito) → 4,08×; CPU media −0,33% (2,61×) =
> pilota GRATIS, checkpoint ≤+2% superato subito; full run45 697,8s vs
> old 716,9s stessa-sera = −2,66% → 2,06× NUOVO MINIMO (= WP-40:
> residuo ≈14s CHIUSO; churn indice −296MB/run)**. ⭐⭐ mechanism-check
> onesto: banda canale −25..−45% NON verificata — l'estimatore
> live×death-avg sovrastima arr 5,7× (standing reale ~77→71,6MB reached,
> −7,4%; per-shape confermato: −62,3B/arr sui 4,75M morti alla cifra);
> **correzione al checkpoint Fase 3: margine arr/str reale ~2,7×, le
> prossime tranche si quotano su metro NON-biased**. Ob.2 metà probe:
> `.=` O(n²) VIVO su element 517ms / prop 541 / static 592 / nested 576
> vs oracle ~1ms (local fuso ok); frequenza per sito → WP-57. Parità:
> corpus 1421 + refl 290 IDENTICI, cargo 1640/0 (nuovo test churn
> model-based), ORM 3E/13F, hk 0E/0F, fail-set full BYTE-ID a run33 (88)
> su run45 E run45-old, probe56 ×4 BYTE-ID (order/tomb/numkey anche =
> oracle). Divergenza PRE-esistente NUOVA a verbale: dtor dei valori
> RIMOSSI (unset/mass-unset) differiti a fine script (overwrite
> puntuale). Stash: `phpr-wp56` (sha256 65466c64…); old = `phpr-wp55`
> (ecc04817…). **Storia: `sessions/WP_SESSION_56.md`. WP-55:
> `sessions/WP_SESSION_55.md`.**

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

## Stato gate per nome (gate56 su `0943f58`, 2026-07-26, `wp56-harness/gate-out/`; classe layout → corpus + refl + cargo + ORM + hk COMPLETI; ultima verifica gate22 integrale: `60c7e04`, archivio `gate-out-wp50-archived/`)

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
```

## 🎯 PROSSIMO LAVORO — WP-57: quota frequenza `.=` per sito + ri-quota arco Fase 3

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle. Fase 3 tranche 1 (indice keyless) CHIUSA
verde in WP-56 (footprint −1,79%, full −2,66%, CPU gratis).

1. **Ob.2 metà frequenza (mandato WP-55/56)**: op-census esteso con
   conteggio+byte dei Concat su AssignOpPath/PropOpSet/StaticPropOpSet →
   ns/evento×frequenza sul FULL; il probe (wp56-harness/
   probe56-concat-sites.php) ha già dato il per-evento: element 517ms /
   prop 541 / static 592 / nested 576 vs oracle ~1ms su 5k×1KB. SE i
   secondi lo giustificano, estensione del fuso append-in-place ai siti
   prop/element (mirror ConcatAssignSlot; sentinelle probe55/56 pronte).
2. **Fase 3, tranche 2 — PRIMA la ri-quota su metro NON-biased**
   (lezione WP-56: l'estimatore live×death-avg sovrastima arr 5,7×):
   live-accounting esatto del canale arr (o walk al watermark) per
   misurare lo standing REALE al picco (EOR reached: ~72MB) e ordinare
   le tranche successive (arena handle-based entries, shrink entries,
   cold-path) coi numeri veri. L'arco continua per direttiva; le
   tranche si ordinano coi dati.
3. **Residuo full vs oracle (2,06×)**: attribuzione WP-54 ancora valida:
   corpi+dispatch 41,9% · gc-walk 10% · str-copy residuo (siti `.=`
   non-locali, item 1) · crypt onesto.
4. **Divergenze a catalogo (PHPR_DIVERGENCES_FROM_PHP.md)**: dtor dei
   valori RIMOSSI da array (unset/mass-unset) differiti a fine script
   (WP-56, pre-esistente); famiglia `.=` (WP-55): undef-lhs senza
   warning; operandi oggetto senza `__toString`.
5. **NON riproporre**: **cap reflect 16384→8192 (DECISIONE UTENTE
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
Ultimo stato (WP-56): **media CPU 54,665/20,945 = 2,61× (A/B −0,33% —
pilota keyless GRATIS in CPU) · footprint peak fisico 1,536/0,376G =
4,08× (A/B −1,79% = −28,1MB, 6/6 pulito) · full: run45 697,8s = 2,06×
NUOVO MINIMO (−2,66% vs old STESSA-SERA 716,9s; residuo vs WP-40
CHIUSO) · fail-set byte-id a run33 (88 nomi) su run45 e run45-old ·
census: −62,3B/arr sui morti alla cifra, churn −296MB/run; banda canale
NON verificata (estimatore 5,7× — correzione checkpoint Fase 3: arr/str
reale ~2,7×) · probe `.=` non-locali ~500× vivi · dettaglio:
`gaps/REPORT_GAP_56.md`, trend: `gaps/GAP_TREND.md`**.
