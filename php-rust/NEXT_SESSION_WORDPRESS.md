# Rotta WORDPRESS-FIRST — WP-track (dopo WP-44: stadio 2 registri BOCCIATO su A/B in TRE forme e revertato → ARCO REGISTRI CHIUSO; si apre la validazione Laravel o il backlog)

> 🚫 **WP-44 (2026-07-23, `35ff89f`+`1e365db`+`f4c80cf` revertati — tree
> finale BYTE-IDENTICO a `e3c8e0b`/WP-43)** — **STADIO 2 Leva B eseguito,
> provato e BOCCIATO su A/B in TRE forme: +1,17% (v1 enum-operand),
> +1,28% (v2 enum + risoluzione singola), +1,01% (v3 "raw registers" dal
> rebuttal Gemini: 7 shape monomorfe u16, ZERO dispatch operandi runtime,
> mirror const-lhs a compile time) — 18/18 round new>old, oracle
> 20,74-20,80 stabilissimo su 6 serie. Revert secco; le fusioni
> WP-32/33/34 restano; l'infra stadio 1 resta dormiente.** Prima dei
> verdetti erano PASSATI tutti i criteri di parità: dump flag-off byte-id
> a WP-43, cataloghi diff puliti, **gate22 COMPLETO verde 2× (off e on)**
> per v1/v2 e **corpus intero flag-on 1447 IDENTICO** per v3 — bocciatura
> solo fisica CPU, non correttezza. ⭐⭐ Epitaffio (3 forme = verbale
> solido): il costo strutturale è il NUMERO DI CORPI HANDLER CALDI nel
> run_loop, NON lo stile di estrazione degli operandi — il rebuttal
> "colpa dell'enum" è falsificato (v3 la migliore ma sempre sopra old);
> l'elisione dei LoadVar non ripaga il working-set I-cache/BTB aggiunto.
> ⭐⭐ Un gate a flag ambientale vuole la PROVA POSITIVA nel log
> (`gate22-regon.sh` conta le forme nel dump e abortisce a 0; `ps eww`
> su macOS non mostra l'env nemmeno dei processi propri).
> **Storia: `sessions/WP_SESSION_44.md`.**

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

## Stato gate per nome (gate22 completo ×2 su `35ff89f`, 2026-07-23; tree finale = stesso bytecode a flag off)

- Gate22 verde due volte (flag OFF e flag ON, archivi
  `wp22-harness/gate-out-wp44-{off,on}-archived/`): corpus **1447** ·
  sess 28 · date 351 · refl 290 IDENTICI · ORM 3E/13F · hk 0E/0F ·
  cargo (1640 col pass; **1637** sul tree finale revertato) · probe
  gd/mysqli/media byte-id · http DIFF-set 16 · option 413 / restapi 3508
  identici per nome. Post-revert: tree byte-id a `e3c8e0b` (già gated),
  dump probe byte-id, out==oracle — full gate NON rilanciato.
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run32** (~/Claude/wpdev, trunk@5e3fced): 30.472 test,
  0E/2F/86W/73S, fail-set BYTE-IDENTICO a run31; baseline =
  `wp16-harness/full-out/run32-fails.txt` (88 righe). Riferimento
  master-CPU ~11:39 (WP-40) = 2,06×. Multisite: 1 diff = minimo teorico.
- Suite phpt (misura): xsl 63/64 (da CWD root php-8.5.7) · tidy 44/45 ·
  asym 38/39. Suite phpt SEMPRE con path ASSOLUTO.

## Harness full-suite

```bash
"/Volumes/Extreme Pro/Claude/wp16-harness/run-full-detached.sh" phpr
# col daemonizer perl (double-fork+setsid) — il task-kill a 10' non deve
# raggiungere la run. MAI due gate22 insieme; uploads azzerati PRIMA di
# ogni full run; non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr>
```

## 🎯 PROSSIMO LAVORO — ⚡ SUPERSEDED (2026-07-24): ROADMAP FOOTPRINT+CPU

**Decisione utente 2026-07-24: il fronte perf RIAPRE con
`FOOTPRINT_CPU_ROADMAP.md`** (piano "concilio", approvato): footprint-first
(12× mai aggredito), safe-only, TUTTE le fasi si eseguono comunque e
**niente revert in caso di insuccesso** (direttiva esplicita — supera la
legge revert-su-regressione per queste sessioni). WP-45 = Fase 0
(attribuzione byte-per-struttura + purge day-zero + domanda
template-include). La validazione Laravel è POSTICIPATA a valle della
roadmap. La sezione sotto resta per riferimento storico.

## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT DISCO**: `df -h /System/Volumes/Data`, non partire sotto
   ~15-20G liberi (WP-44 è partita a 16G; il cargo test DEBUG rigenera
   ~3,8G di `php-rust-output/debug/` — in sessione usare SEMPRE
   `cargo test --release`, e pulire `debug/` se ricompare).
   **PRE-FLIGHT MYSQL**: `mysql -h 127.0.0.1 -u root -e "SHOW DATABASES"`
   deve elencare wp_o/wp_p/probe — altrimenti vedi ⚠️ MySQL sopra.

1. **Rotta ([[php-rust-roadmap-wp-first]]): con l'arco perf chiuso si apre
   la VALIDAZIONE LARAVEL** (installazione + test suite di un'app Laravel
   reale, metodo = ricetta gate ORM/hk: oracolo+composer build, phpr
   esegue). È il passo finale della roadmap WP-first.
2. **In alternativa (o in coda), backlog pescabile da
   [[php-rust-todo-master]]** — candidato pronto: 🆕 **bug isset via
   prefisso `__get` con indici annidati** (`isset($mg->m['a']['b'])` →
   false, oracle true; probe wp42-harness/probe-isset-div.php §3).
3. Fronte footprint (~12×): NON aggredito; quando si apre → PRIMA una
   sessione di attribuzione memoria data-driven (metodo WP-26).
4. **NON riproporre**: stadi 3-4 registri (chiusi con l'arco); leve locali
   canale churn (esaurite WP-33/34/41/42); NaN-boxing; SSO union.

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

## 📊 Report gap perf — ricorrente di fine sessione

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_44.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-44): **media 2,66× (old di giornata pulitissima; main =
binario WP-43) · full [run32, non rilanciata: tree revertato byte-id] ·
footprint raw 10,0× di giornata — riferimento strutturale resta ~12×**.
