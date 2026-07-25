# Rotta WORDPRESS-FIRST — WP-track (dopo WP-51: classify fuso −4,1% + Fase 1.4 LANDED gratis → full 2,31×, media 2,84×/4,34×; WP-52 = residuo classify (in-node marks da quotare) + Fase 1.3)

> ⚡ **WP-51 (2026-07-25, `ffb8e3b`+`84a22f6`)** — **Ob.1a: probe full-scan a
> scala FULL POSITIVO: `created-registry-only` 353,74MB→0, roots_total
> −354,2M (=100% del canale), freed 1.481.070 zval, live drop ≈−545MB;
> costo UN full-scan a full-graph = 2.489ms — kill-switch NON scattato.**
> Ob.2 leva A (`ffb8e3b`): classify a MAPPA FUSA (3 tabelle→1 `GcInfo`,
> 3→2 hash op/arco, pre-reserve della taglia dell'ultimo walk sui call
> ≥1024 root, zero footprint standing) → **run38 793,7s vs old 827,9s
> stessa notte = −4,1% (~24% del classify rimosso)**. Ob.1b leva B
> (`84a22f6`, **Fase 1.4 LANDED**): full-scan seed growth-gated
> (`GC_FULLSCAN_GROWTH=50k` su `created.len()`, check O(1) nel wrapper
> COLD `collect_cycles` — mai per-test, zero per i worker) → **run39
> 782,7s = 13:03 = 2,31×: B è GRATIS sulla full (il −11s vs run38 NON è
> mechanism-backed: rumore), totale notte −5,5% (2,44×→2,31×)**. Media
> (ab51): CPU **FLAT +0,01%** (guardia Fase 1 ≤+0,5% RISPETTATA) →
> **2,84×**; peak fisico −0,32% → **4,34×** (minimi di sempre entrambi).
> ⭐⭐ **mechanism-check census-B: la predizione footprint di B è
> FALSIFICATA nei conteggi — freed +0,1% (non +1,45M), canale
> created-registry-only INVARIATO a 353,7MB: è garbage di TEARDOWN, vivo
> fino allo smontaggio PHPUnit ⇒ invisibile a OGNI cadenza mid-run per
> costruzione del workload. B tenuta (gratis + ceiling Zend per processi
> long-running); esito footprint Fase 1.4 su PHPUnit = NEUTRO.** Leva A
> riconciliata alla cifra: classify_ms census 142,4→109,4s ≈ −34,2s full.
> Parità: 3 full stessa notte tutte BYTE-ID a run33; corpus 1421 ×2;
> cargo 1639/0. ⭐⭐ lezione launcher: daemonizer con `chdir "/"` ⇒ FINTI
> fail phpt (bug60771 scrive ./test.php nella CWD; la CWD è parte del
> contratto della suite — `cd` root php-8.5.7 DENTRO lo script di gate).
> Convenzione gap corretta (utente): REPORT_GAP_N per-sessione +
> `gaps/GAP_TREND.md` cumulativo.
> **Storia: `sessions/WP_SESSION_51.md`. WP-50: `sessions/WP_SESSION_50.md`.**

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

## Stato gate per nome (gate22 completo su `60c7e04`, 2026-07-25, archivio `gate-out-wp50-archived/`; fix GC-only `f034c6c` gated con corpus+cargo+full BYTE-ID)

- Gate22 verde: corpus **1421** IDENTICO alla baseline WP-46 (0 nuovi-fail;
  ri-verificato ×2 anche su `f034c6c`) · sess 28 · date 351 · refl 290
  IDENTICI · ORM 3484 3E/13F per nome · hk 1665 0E/0F · cargo **1639/0 ×2**
  · probe gd/mysqli/media byte-id · http DIFF-set atteso (WP-14, 2 item) ·
  **option 413/1061 · restapi 3508/15947 (1E = oracle) IDENTICI per nome
  COL conteggio**. Baseline corpus resta **1421**
  (`gate-out-wp46-archived/corpus.fails`).
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run39** (~/Claude/wpdev, WP-51, binario `84a22f6`+docs):
  30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi);
  baseline resta `wp16-harness/full-out/run33-fails.txt`. Master-CPU
  **782,7s ≈13:03 = 2,31×** (catena stessa notte: old wp50 827,9 = 2,44× →
  run38 leva-A 793,7 = 2,34× → run39 A+B 782,7; WP-40 11:39 = 2,06× =
  riferimento residuo). Wall ~17 min = 1,5×.
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
# ⚠️ launcher: MAI chdir "/" nel daemonizer per i gate phpt — cd root
#  php-8.5.7 DENTRO lo script (bug60771 scrive ./test.php nella CWD).
```

## 🎯 PROSSIMO LAVORO — WP-52: residuo classify + Fase 1.3

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

1. **Residuo CPU full 2,31× vs 2,06× WP-40 (~85s)**: il classify resta il
   candidato (era 142s; la fusione ne ha tolti ~34). Prossima leva da
   QUOTARE prima di scrivere: **in-node marks** (visited/in_edges/live nei
   valori invece della HashMap — elimina l'hashing residuo ~2 op/arco; ma
   costa +8B/nodo su PhpArray/Closure/Ref che oggi non hanno header GC:
   quotare i byte col mem-census e la CPU con un micro-conteggio archi
   PRIMA di toccare i layout; sentinelle drop-order pinnate PRIMA, WP-28).
   Alternativa più piccola: profilo del walk per separare hash-cost da
   borrow/iter-cost (i conteggi per-round sono in
   `wp51-harness/fullcensus-out/collect-full.log`).
2. **Fase 1.3 cold-box Object** (~96B×istanze, pattern WP-32) — prossima
   leva footprint; attribuzione transiente col criterio di lettura .rss
   corretto (blocco notte + finestra).
3. **reflect-cache**: la leva vera è l'OWNER della cardinalità (memo keyed
   su ClassId freschi dei mock, leak WP-47) — il cap è un cerotto
   (16384 falsificata: hit 16,3→16,5%; ritorno a 8192 = decisione utente,
   risparmierebbe ~42MB standing sul media).
4. **NON riproporre**: Fase 1.2 created→Weak/eviction rc==1 (falsificata);
   rooting selettivo; cap rerun stile Zend; **bound/guardie calcolate
   negli arm caldi del run_loop** (WP-50: +1 load+max = +17..35s — campo
   cache ai siti di mutazione); soglia adattiva/isteresi su buffer RAW;
   collect per-test (il gating a crescita è la forma giusta, WP-51).

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
Ultimo stato (WP-51): **media CPU 59,17/20,82 = 2,84× (A/B +0,01% flat) ·
footprint peak fisico 1,721/0,396G = 4,34× (A/B −0,32%) · full run39
782,7s ≈13:03 = 2,31× (old stessa notte 827,9 = 2,44×; riferimento WP-40
2,06×), fail-set byte-id a run33 su 3 run · Fase 1.4 LANDED gratis ·
dettaglio: `gaps/REPORT_GAP_51.md`, trend: `gaps/GAP_TREND.md`**.
