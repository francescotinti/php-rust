# Rotta WORDPRESS-FIRST — WP-track (dopo WP-49: purge tombstone al trigger → full 2,46×; Fase 1.2 falsificata dal census; WP-50 = residuo CPU + Fase 1.4/1.3)

> ⚡ **WP-49 (2026-07-24, `08fe322`+`951e2fd`+`e2d7805`)** — **leva collector
> dall'attribuzione WP-48: PURGE dei tombstone al trigger (modello Zend
> `gc_remove_from_buffer`). Il log per-collect nuovo (PHPR_GC_COLLECT_LOG)
> ha scagionato rerun-loop e chiamate esplicite e trovato la causa: trigger
> a maggioranza tombstone in 530/530 call (soglia 350k-1M, sopravvissuti
> ~7-30k — i Weak morti di gc_ctr_roots non lasciavano MAI il buffer).
> Leva: retain dei vivi al trigger + collect solo su pressione viva +
> gc_purge_floor anti-degenerazione. RISULTATI: full run36 master-CPU
> 19:10→≈13:55 = −27% (3,39×→2,46×), fail-set BYTE-ID a run33, RSS max
> 5431MB INVARIATO; A/B media CPU −1,19% (5/6) E peak −2,22% (prima leva
> che migliora entrambe); mechanism-check a CONSERVAZIONE: round 1005→297,
> classify_ms −62%, freed +0,5% (stessa garbage in ⅓ dei round), soglia
> ferma a 50k base. Ob.2: Fase 1.2 created→Weak FALSIFICATA PRIMA di
> scriverla — census split `created-dead-rc1[n=14]=113KB` su 114,73MB:
> il canale è garbage CICLICA (un registry Weak non la libera); obiettivo
> de-pinning (385MB full) → Fase 1.4. Parità: cargo 1639/0 ×2, corpus 1421
> BYTE-ID, gate22 verde col conteggio.**
> **Storia: `sessions/WP_SESSION_49.md`. WP-48: `sessions/WP_SESSION_48.md`.**

## 📁 Convenzioni (decisione utente 2026-07-23)

- Qui SOLO: sintesi ultima sessione · decisioni in vigore · stato gate ·
  prossimo lavoro · backlog. Storia: `sessions/WP_SESSION_<n>.md` (un file
  per sessione, con lezioni e verdetti; ≤WP-27: memoria + git history).
  Gap perf: `gaps/REPORT_GAP_<n>.md` (l'ultimo = tabella viva).
- Chiusura della sessione N: scrivere `sessions/WP_SESSION_N.md`; copiare
  l'ultimo REPORT_GAP in `gaps/REPORT_GAP_N.md` aggiungendo la riga N;
  sostituire la sintesi qui in testa e aggiornare stato gate / prossimo
  lavoro; commit+push.

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

## Stato gate per nome (gate22 completo su `e2d7805`, 2026-07-24, archivio `gate-out-wp49-archived/`)

- Gate22 verde: corpus **1421** IDENTICO alla baseline WP-46 (0 nuovi-fail)
  · sess 28 · date 351 · refl 290 IDENTICI · ORM 3484 3E/13F per nome ·
  hk 1665 0E/0F · cargo **1639/0** · probe gd/mysqli/media byte-id ·
  http DIFF-set atteso (WP-14) · **option 413/1061 · restapi 3508/15947
  (1E = oracle) IDENTICI per nome COL conteggio**. Baseline corpus resta
  **1421** (`gate-out-wp46-archived/corpus.fails`).
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run36** (~/Claude/wpdev, WP-49): 30.472 test, 0E/2F/86W/73S,
  **fail-set BYTE-IDENTICO a run33** (88 nomi); baseline resta
  `wp16-harness/full-out/run33-fails.txt`. Master-CPU **≈13:55 = 2,46×**
  (run35 19:10 = 3,39×; WP-40 11:39 = 2,06× = riferimento residuo).
  Wall ~18 min = 1,6×. RSS transiente mid-run 5431MB INVARIATO.
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
# (harness census: wp49-harness/run-full-census49*.sh, ab49.sh, report)
```

## 🎯 PROSSIMO LAVORO — WP-50: residuo CPU full + Fase 1 residua (1.4/1.3)

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

1. **Residuo CPU full ≈1,20× vs WP-40** (13:55 vs 11:39): candidati coi
   numeri WP-49 — classify residuo (142s census sui 297 round veri),
   reflect-cache thrash (890k miss / 16% hit / cap 8192; cap 16384 ≈ +45MB
   teorici: misurare CPU E footprint), light-sweep 1,01G ingressi post-leva
   (i buffer restano pieni più a lungo ⇒ meno fast-path sweep-empty:
   corpi economici ma 5× più ingressi — micro-leva candidata).
2. **Fase 1.4 disciplina di confine per-test** — ora è ANCHE la leva
   footprint per il canale created (385,2MB full a fine run = garbage
   ciclica non collettata, verdetto census WP-49). ⚠️ un collect per test =
   il male appena curato: PRIMA quotare con log per-collect quanto costerebbe
   (roots vivi per confine), POI decidere cadenza (ogni N test?).
3. **Fase 1.3 cold-box Object** (~96B×istanze, pattern WP-32) e
   **attribuzione transiente 5,4G** (pacing-dipendente: serve snapshot al
   picco sulla full).
4. **NON riproporre**: Fase 1.2 created→Weak/eviction rc==1 (falsificata:
   rc1 = 0,1% del canale); rooting selettivo (reroot 8,7%, live 11% —
   i root sono garbage legittima); cap rerun stile Zend (round 2 = 0-11ms).

## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT DISCO**: `df -h /System/Volumes/Data`, non partire sotto
   ~15-20G liberi (WP-49 partita a 16-17G dopo rm del debug/ 834M
   ricomparso; in sessione usare SEMPRE `cargo test --release`, e pulire
   `debug/` se ricompare).
   **PRE-FLIGHT MYSQL**: `mysql -h 127.0.0.1 -u root -e "SHOW DATABASES"`
   deve elencare wp_o/wp_p/probe — altrimenti vedi ⚠️ MySQL sopra.
   **STASH OLD**: copiare subito il binario release corrente in
   `phpr-old-target/release/phpr-wp49` (old dell'A/B di WP-50).
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

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_49.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-49): **media CPU 3,01× di giornata rumorosa (interleaved
−1,19%, 5/6) · footprint peak fisico 1,715/0,394G = 4,35× (−2,22%) · full
run36 ≈13:55 = 2,46× (−27% da run35; riferimento WP-40 2,06×), fail-set
byte-id a run33 · Fase 1.2 falsificata (canale created = cicli) · tabella
viva: `gaps/REPORT_GAP_49.md`**.
