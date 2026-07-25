# Rotta WORDPRESS-FIRST — WP-track (dopo WP-55: PhpStr growable + append-in-place `.=` = full −2,56% → 2,11×; checkpoint Fase 3: pilota = HASHED-ARRAY; WP-56 = Fase 3 pilota array)

> ⚡ **WP-55 (2026-07-25, `a8bc9aa`+`50e2fb2`)** — **PhpStr growable +
> append-in-place a refcount unico** (mandato WP-54): `PhpStr{hash:
> Cell<u64>, bytes: Vec<u8>}` (24→32B; RcBox 40→48B = +8B/stringa,
> quotato PRIMA: 713k vive al picco = +5,7MB = +0,36%) + op fuso
> `ConcatAssignSlot` (mirror ASSIGN_OP) per `$s .= rhs` su locale nudo:
> slot-Str-UNICO + rhs Str ⇒ `append` via `Rc::get_mut` (hash azzerato
> a ogni mutazione); alias/const-pool/Ref/non-Str ⇒ fallback
> byte-identico (binary_value_ab condiviso) = COW per costruzione.
> **Mechanism-check: probe 5k×1KB = 499ms → 2ms (250×) = ORACLE**.
> Giudici: full run44 **716,9s vs 735,7s old stessa-sera = −2,56%
> (−18,8s, bordo basso banda 15-90s) → 2,11× nuovo minimo**; ab55 media
> **FLAT** (leva full-only, atteso); footprint **+1,27%** (+8B +
> slack append-grown; guardia ≤2% ok, tenuto). **Checkpoint Fase 3
> (census fresco)**: arr vive ~421MB (39% proxy) vs str ~45MB (4%) ⇒
> **pilota heap-a-handle = HASHED-ARRAY** (10×; stringhe: falsificate
> come pilota). ⭐⭐ il full si giudica SOLO in coppia stessa-sera
> (deriva pomeriggio→sera ~2,6% = taglia delle leve); ⭐⭐ la banda si
> realizza al bordo che il SITO copre: restano i `.=` su prop/element
> (quotare per sito PRIMA di estendere il fuso). Parità: corpus 1421 +
> refl 290 IDENTICI, cargo 1639/0, ORM 3E/13F (16 nomi), hk 0E/0F,
> fail-set full BYTE-ID a run33 su run44 E run44-old, probe55-sem/dtor
> BYTE-ID. 2 divergenze PRE-esistenti a verbale (`.=` su undef senza
> warning; operandi oggetto in `.=` senza `__toString`). Stash:
> `phpr-wp55` (sha256 ecc04817…); old = `phpr-wp54` (e4341e77…).
> **Storia: `sessions/WP_SESSION_55.md`. WP-54:
> `sessions/WP_SESSION_54.md` (+ prereg).**

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

## Stato gate per nome (gate55 su `50e2fb2`, 2026-07-25, `wp55-harness/gate-out/`; classe layout/emit → corpus + refl + cargo + ORM + hk COMPLETI; ultima verifica gate22 integrale: `60c7e04`, archivio `gate-out-wp50-archived/`)

- Gate55 verde (2026-07-25, tree `50e2fb2`): corpus **1421 IDENTICO** per
  nome (baseline = `wp54-harness/gate-out/corpus.fails`) · **refl 290
  IDENTICO** · cargo **1639/0** · ORM 3484 **3E/13F fail-set IDENTICO**
  (16 nomi) · hk 1665 **0E/0F** · probe55-sem/dtor BYTE-ID new vs old
  (sentinelle append: alias-COW, const-pool, dtor del displaced,
  `__toString`-throw ⇒ slot intatto). Baseline corpus resta **1421**.
- Gate22 integrale (storico, su `60c7e04`): sess 28 · date 351 · probe
  gd/mysqli/media byte-id · http DIFF-set atteso (WP-14, 2 item) ·
  option 413/1061 · restapi 3508/15947 IDENTICI per nome COL conteggio.
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run44** (~/Claude/wpdev, WP-55, binario `50e2fb2`):
  30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi;
  idem run44-old su phpr-wp54); baseline resta
  `wp16-harness/full-out/run33-fails.txt` (⚠️ righe con prefisso
  numerico: normalizzare `s/^\d+\) //` prima del diff). Master-CPU
  **716,9s ≈11:57 = 2,11×** (−2,56% vs run44-old 735,7 STESSA SERA;
  ⚠️ lo stesso binario old aveva dato 717,3 il pomeriggio: deriva
  d'ambiente ~2,6%, fa fede SOLO la coppia). WP-40 11:39 = 2,06× =
  riferimento residuo ≈14s. Wall ~16 min = 1,4×. Archivi:
  `full-out/run44/`, `wp55-harness/full-old-out/`; storici run42/run43
  in `full-out/`, `wp54-harness/full-old-out/`.
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
```

## 🎯 PROSSIMO LAVORO — WP-56: Fase 3, pilota HASHED-ARRAY

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle. Il checkpoint d'ingresso Fase 3 è FATTO
(WP-55): il pilota è deciso dai pesi reali.

1. **Fase 3 — pilota HASHED-ARRAY** (checkpoint WP-55: arr vive ~421MB
   = 39% proxy vs str 4% — 10× di margine): redesign a tabella singola
   Zend-style (entries + indice SENZA Key duplicata, `array.rs:96-103`),
   arena-compatibile, VERTICALE end-to-end su questo solo tipo (mai
   dual-mode su tutto il value graph). Parità enorme da presidiare:
   ordine di iterazione, tombstone, chiavi numeriche-stringa,
   dtor-order. Multi-sessione per design (ROADMAP §Fase 3); checkpoint
   pilota = se >+2% CPU dopo due sessioni di ottimizzazione → verbale
   all'utente, l'arco continua.
2. **Residuo append non-locale** (lezione WP-55): i `.=` su
   proprietà/elementi (`$this->buf .= …`, `$a[k] .= …`) restano O(n²) —
   QUOTARE PER SITO (op-census: AssignOpPath/PropOpSet/FieldAssignOp con
   op=Concat, in ns/evento×frequenza) PRIMA di estendere il fuso.
3. **Residuo full vs WP-40 ≈14s**: gc-walk fasi mock del full (classify
   — il media non lo vede: classify_ms 351ms vs 62,7s sul full),
   drop/rc churn 4,8%, corpi handler diffusi.
4. **Divergenze `.=` PRE-esistenti (WP-55, a verbale)**: undef-lhs senza
   warning "Undefined variable"; operandi oggetto senza `__toString`
   (un `__toString` che throwa NON throwa in phpr) — catalogare in
   PHPR_DIVERGENCES o chiudere in una sessione funzionale.
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
   giudizi full da momenti diversi della giornata (WP-55: deriva ~2,6%).

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
Ultimo stato (WP-55): **media CPU 55,06/21,07 = 2,61× (A/B FLAT — leva
full-only) · footprint peak fisico 1,566/0,377G = 4,15× (A/B +1,27% =
+8B/stringa + slack append, guardia ≤2% ok) · full: run44 716,9s =
2,11× NUOVO MINIMO (−2,56% vs old STESSA-SERA 735,7s; residuo vs WP-40
2,06× ≈14s) · fail-set byte-id a run33 (88 nomi) su run44 e run44-old ·
probe `.=` 499ms→2ms = oracle (250×) · checkpoint Fase 3: pilota =
hashed-array (arr 421MB vs str 45MB) · dettaglio:
`gaps/REPORT_GAP_55.md`, trend: `gaps/GAP_TREND.md`**.
