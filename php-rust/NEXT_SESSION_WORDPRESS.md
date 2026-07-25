# Rotta WORDPRESS-FIRST — WP-track (dopo WP-50: banda light-sweep chiusa col bound-cache → full 2,41×; reflect-cap falsificata; Fase 1.4 quotata POSITIVA; WP-51 = probe full-scan sulla full + classify + Fase 1.3)

> ⚡ **WP-50 (2026-07-24/25, `3922e6b`+`60c7e04`+`f034c6c`)** — **Ob.1: il
> light-sweep 1,01G ingressi era la BANDA [soglia, purge_floor): il
> fast-path sweep-empty usava la sola soglia, il trigger usa max(soglia,
> floor) — ogni statement sweep nella banda entrava nel corpo a fare
> nulla. La forma-1 della leva (max() nell'arm caldo) REGREDIVA +17..35s
> nonostante −825,6M ingressi (legge WP-44 su una GUARDIA: +1 load+max
> nell'arm Op::Sweep = I-cache > risparmio, invisibile ai census);
> forward-fix `gc_sweep_bound` (campo cache, refresh ai 4 siti di
> mutazione): **run37c 816,8s = 13:37 = 2,41× — sotto ENTRAMBI gli old
> stesso-sera (828,9/832,7, spread 3,8s), −14s ≈ il risparmio teorico:
> bilancio RICONCILIATO**; 6 full stesso-sera tutte BYTE-ID a run33.
> Reflect-cap 16384 FALSIFICATA come leva CPU (miss −0,3%, hit 16,3→16,5%;
> costo footprint media +41,7MB standing nei buffer ctr = il peak +2,64%
> dell'A/B — TENUTA, eventuale ritorno a 8192 = decisione utente). ⭐⭐ il
> "RSS max 5431 invariato" di run34-37 era un artefatto: telemetria .rss
> in APPEND, max cross-segmento — i veri massimi di segmento sono
> 1,6-2,3G. **Ob.2 Fase 1.4 quotata POSITIVA: canale created media = 100%
> garbage ciclica collettabile (114,73MB→0 col probe full-scan
> `PHPR_GC_EOR_FULL_COLLECT`, 874ms; Δroots_total = 99,98% del canale) —
> ma collect_cycles si radica SOLO dai buffer: i cicli quiescenti sono
> invisibili anche ai collect espliciti, serve il seed da `created`.**
> Parità: corpus 1421 ×2, cargo 1639/0 ×2, gate22 verde col conteggio.**
> **Storia: `sessions/WP_SESSION_50.md`. WP-49: `sessions/WP_SESSION_49.md`.**

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
- **Full-suite run37c** (~/Claude/wpdev, WP-50, binario `f034c6c`): 30.472
  test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi);
  baseline resta `wp16-harness/full-out/run33-fails.txt`. Master-CPU
  **816,8s ≈13:37 = 2,41×** (old stesso-sera 828,9/832,7; run36 13:55 =
  2,46×; WP-40 11:39 = 2,06× = riferimento residuo). Wall ~18 min = 1,6×.
  ⚠️ RSS: telemetria `.rss` in APPEND — max PER SEGMENTO (veri massimi
  1,6-2,3G; il "5431 invariato" storico era cross-segmento).
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
```

## 🎯 PROSSIMO LAVORO — WP-51: Fase 1.4 sulla full + classify + Fase 1.3

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

1. **Census full col probe full-scan** (`PHPR_GC_EOR_FULL_COLLECT=1` in
   `wp50-harness/run-full-census50.sh`, binario `phpr-memgc50` GIÀ pronto):
   quota il canale created a scala full (353,7MB a fine run) + costo del
   full-scan collect a full-graph → disegno leva Fase 1.4 (cadenza per
   CRESCITA di created, es. ogni +50k — mai per-test: 30k×collect = il
   male WP-49; PEAK-value da misurare, il peak è mid-run).
2. **Residuo CPU full 2,41× vs 2,06× WP-40**: il classify (142s census,
   139,7s nei 149 round-1, ~938ms/call su ~60k root, ~94k white/call =
   lavoro utile) è l'INTERO residuo; il numero di round è già minimo ⇒
   la leva è il COSTO del walk. Solo coi numeri per-round in mano.
3. **Fase 1.3 cold-box Object** (~96B×istanze, pattern WP-32);
   attribuzione transiente da RIVALUTARE (il 5,4G era artefatto di lettura
   cross-segmento; transiente vero 1,6-2,3G per segmento).
4. **reflect-cache**: la leva vera è l'OWNER della cardinalità (memo keyed
   su ClassId freschi dei mock, leak WP-47) — il cap è un cerotto
   (16384 falsificata: hit 16,3→16,5%; ritorno a 8192 = decisione utente,
   risparmierebbe ~42MB standing sul media).
5. **NON riproporre**: Fase 1.2 created→Weak/eviction rc==1 (falsificata);
   rooting selettivo; cap rerun stile Zend; **bound/guardie calcolate
   negli arm caldi del run_loop** (WP-50: +1 load+max = +17..35s — campo
   cache ai siti di mutazione); soglia adattiva/isteresi su buffer RAW.

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

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_50.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-50): **media CPU 2,86× (A/B leve flat) · footprint peak
fisico 1,755/0,394G = 4,45× (leva 2: +2,64% attribuito ai descriptor
ritenuti) · full run37c 816,8s ≈13:37 = 2,41× (old stesso-sera
828,9/832,7; riferimento WP-40 2,06×), fail-set byte-id a run33 su 6 run ·
Fase 1.4 quotata POSITIVA (canale created media 100% collettabile col
full-scan) · tabella viva: `gaps/REPORT_GAP_50.md`**.
