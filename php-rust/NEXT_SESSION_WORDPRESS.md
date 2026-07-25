# Rotta WORDPRESS-FIRST — WP-track (dopo WP-54: attribuzione CPU-secondi del full + reflect re-key = −7,4% media / −2,7% full; WP-55 = PhpStr growable + append-in-place + checkpoint Fase 3)

> ⚡ **WP-54 (2026-07-25, `5915b7b`+`f927bf7`+`14141ab`)** — **Metodo
> WP-45 applicato alla CPU: tabella CPU-secondi del full-master**
> (run42 campionata a 6 finestre `sample`, stratificata col cpu-rate del
> `.rss`, Σ=776,7s; tabella decisionale PRE-REGISTRATA prima dei numeri;
> strumenti in `wp54-harness/*.py`). Verdetti: corpi+dispatch 41,9% (in
> banda, arco chiuso) · crypt 11,7% ONESTO (probe bcrypt: phpr −14% vs
> oracle) · gc-walk 10,0% (unico canale prereg sopra soglia) · str-copy
> 6,5% + madvise 5,4% = **`.=` O(n²) CONFERMATO (probe 244× vs oracle:
> concat2 = alloc esatta + doppia copia, Box<[u8]> senza capacity)** ·
> malloc TOTALE 4,4%, di cui stringhe 0,3% (~10-20ns/pair) ⇒ fusione
> single-alloc FALSIFICATA come leva CPU; args-pool morta (0,14%);
> leaf-bit walk falsificata (foglie = 37,5% nodi ma 6,6% SLOT). **Leva
> aperta (census reflect: inherited 96,3%): re-key
> reflect_method_info_cache su (declaring class, mname)** dopo la
> resolve — hit-rate 11,7%→96,7%, ~450k descriptor-build/run evitati.
> Giudici: ab54 **−7,42% CPU new 6/6 netti → 2,61× MINIMO ASSOLUTO**;
> peak fisico **−5,79% → 4,16×**; full run43 **717,3s = 2,12×** (−2,71%
> vs old stesso-giorno, old replica run41: numero pulito; residuo vs
> WP-40 ≈18s). Parità: corpus 1421 + refl 290 IDENTICI, cargo 1639/0,
> ORM 3E/13F per nome, hk 0E/0F, fail-set full BYTE-ID a run33 su
> run43 E run43-old, probe54 NEW==OLD. Stash: `phpr-wp54` (sha256
> e4341e77…); old = `phpr-wp53` (e75c5abb…).
> **Storia: `sessions/WP_SESSION_54.md` (+ prereg in
> `WP_SESSION_54_PREREG.md`). WP-53: `sessions/WP_SESSION_53.md`.**

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

## Stato gate per nome (gate54 su `14141ab`, 2026-07-25, `wp54-harness/gate-out/`; superficie reflection → corpus + refl + cargo + ORM + hk COMPLETI; ultima verifica gate22 integrale: `60c7e04`, archivio `gate-out-wp50-archived/`)

- Gate54 verde (2026-07-25, tree `14141ab`): corpus **1421 IDENTICO** per
  nome (baseline = `wp53-harness/corpus.fails`) · **refl 290 IDENTICO** ·
  cargo **1639/0** · ORM 3484 **3E/13F fail-set IDENTICO** · hk 1665
  **0E/0F** · smoke probe54 (12 scenari ReflectionMethod) NEW==OLD
  BYTE-ID. Baseline corpus resta **1421**.
- Gate22 integrale (storico, su `60c7e04`): sess 28 · date 351 · probe
  gd/mysqli/media byte-id · http DIFF-set atteso (WP-14, 2 item) ·
  option 413/1061 · restapi 3508/15947 IDENTICI per nome COL conteggio.
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run43** (~/Claude/wpdev, WP-54, binario `14141ab`):
  30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi;
  idem run43-old su phpr-wp53); baseline resta
  `wp16-harness/full-out/run33-fails.txt` (⚠️ righe con prefisso
  numerico: normalizzare `s/^\d+\) //` prima del diff). Master-CPU
  **717,3s ≈11:57 = 2,12×** (−2,71% vs run43-old 737,3 stesso-giorno;
  l'old replica il suo run41 738,6 ⇒ ambiente stabile). WP-40 11:39 =
  2,06× = riferimento residuo ≈18s. Wall ~17 min = 1,4×. Archivi:
  `full-out/run42/` (run di misura campionata, non riferimento),
  `full-out/run43/`, `wp54-harness/full-old-out/`.
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
```

## 🎯 PROSSIMO LAVORO — WP-55: PhpStr growable + append-in-place + checkpoint Fase 3

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle. La tabella CPU-secondi WP-54
(`wp54-harness/sample-out/attribution-table.txt`) è la mappa: le leve si
aprono da lì, quotate in secondi.

1. **PhpStr growable + append-in-place a refcount unico** (mandato dai
   dati WP-54): `hash: Cell<u64> + bytes: Vec<u8>` (+8B/stringa, quotare
   in SIZE-CLASS — lezione WP-52) e `$s .= x` che estende in place via
   `Rc::get_mut` quando il ZStr è unico (mirror di zend_string_extend).
   Chiude il canale O(n²) del `.=` (probe: 244× vs oracle) quotato
   15-90s sul full (str-copy 8-50s + gran parte di madvise 41,8s).
   ⚠️ il vecchio mandato "fusione single-alloc via Rc<[u8]>" è
   INCOMPATIBILE con l'append-in-place (immutabile, niente capacity) e
   da solo NON è leva CPU (0,3% del full): i due design si decidono
   INSIEME, con l'append-in-place prioritario. Classe layout/emit ⇒
   gate pieno: corpus+cargo+full BYTE-ID + sentinelle drop-order.
2. **Checkpoint d'ingresso Fase 3** (ri-attribuzione BYTE post Fase 1-2,
   metodo WP-45): scegliere il tipo pilota heap-a-handle {stringhe vs
   hashed-array} coi pesi REALI post-WP-54 — naturale farlo insieme al
   design PhpStr (se vince "stringhe", i due lavori si fondono).
3. **Residuo full vs WP-40 ≈18s**: canali rimasti dalla tabella: gc-walk
   fasi mock del full (classify — il media non lo vede: classify_ms 351ms
   vs 62,7s sul full), drop/rc churn 4,8%, corpi handler diffusi.
4. **Ritorno cap reflect 16384→8192**: dopo il re-key le evictions sono
   1/run (era 28) — quasi irrilevante; decisione utente se farlo comunque.
5. **NON riproporre**: fusione single-alloc COME LEVA CPU (0,3% del full,
   ~10-20ns/pair — WP-54); Fase 2.3 args-Vec pool (0,14% — morta);
   leaf-bit sul walk (foglie 6,6% degli SLOT — WP-54); leve sul canale
   crypt (probe: phpr −14% vs oracle = onesto); Fase 1.2 created→Weak;
   rooting selettivo; cap rerun stile Zend; bound/guardie negli arm caldi
   (WP-50); soglia adattiva su buffer RAW; collect per-test; cold-box
   dyn_entries (WP-52); giudizi footprint da RSS telemetrico; Sweep-
   elision oltre whitelist (WP-53); micro-op quotati solo in conteggi.

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
Ultimo stato (WP-54): **media CPU 54,51/20,90 = 2,61× MINIMO ASSOLUTO
(A/B −7,42%, new 6/6 netti — reflect re-key) · footprint peak fisico
1,564/0,376G = 4,16× (A/B −5,79%) · full: run43 717,3s = 2,12× (−2,71%
vs old stesso-giorno, ambiente stabile; residuo vs WP-40 2,06× ≈18s) ·
fail-set byte-id a run33 (88 nomi) su run43 e run43-old · tabella
CPU-secondi del full in `wp54-harness/sample-out/attribution-table.txt`
(crypt ONESTO, `.=` O(n²) → mandato WP-55) · dettaglio:
`gaps/REPORT_GAP_54.md`, trend: `gaps/GAP_TREND.md`**.
