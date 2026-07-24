# Rotta WORDPRESS-FIRST — WP-track (dopo WP-47: attribuzione owner-level RIUSCITA, footprint 11,9×→4,3×; WP-48 = shrink unit + recupero CPU full)

> ⚡ **WP-47 (2026-07-24, `583dd30`→`f3bea49`+)** — **RI-ATTRIBUZIONE
> owner-level RIUSCITA al primo colpo e leva landed: il root-walk mem-census
> esteso a TUTTI i campi Zval-bearing del Vm (walk_frame completo con
> dyn_vars/this/ret_cell/iters/ext; fibers, handlers, ob, enum-cache,
> tabelle lazy/reflect) + contatori reached-vs-live DENTRO deep_size
> riconcilia il canale arr al 100,0% (3.114.573/3.114.573) e str al 96,7%
> — e l'owner del "3G irraggiungibile" di WP-45/46 è UNO:
> `reflect_method_info_cache`, 455.978 entry / 2,48G (ogni mock PHPUnit =
> ClassId fresco; memo mai evitta; ~5,4KB/descrittore). Erano anche i
> root "vivi" che il collector WP-46 camminava a vuoto (726k→276k).**
> **Leve: epoch eviction cap 8192 (`fa100ad`, PHP-invisibile) +
> gc_classify a 2 passate senza children-map globale né cloni scalari +
> isteresi soglia step-down solo se ≥1% dei root muore (`d684cd7`, base
> 50k intoccata). ESITO: peak media 4,88→1,77G = −63,8%, gap footprint
> 11,9×→4,3× (⭐⭐⭐ record); full master-CPU 21:00→≈19:20 (3,71×→3,42×);
> media CPU +1,3% (6/6, rebuild descrittori) TENUTO per direttiva.
> PARITÀ TOTALE: corpus 1421 IDENTICO ×2, gate22 completo verde col
> conteggio, full-suite BYTE-ID a run33 (88 nomi), cargo 1639/0.**
> **Storia: `sessions/WP_SESSION_47.md`. WP-46: `sessions/WP_SESSION_46.md`.**

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

## Stato gate per nome (gate22 completo su `d684cd7`, 2026-07-24, archivio `gate-out-wp47-archived/`)

- Gate22 verde: corpus **1421** IDENTICO alla baseline WP-46 (0 nuovi-fail)
  · sess 28 · date 351 · refl 290 IDENTICI · ORM 3E/13F · hk 0E/0F ·
  cargo **1639** (2 sentinelle GC incluse, anche rilanciate singole) ·
  probe gd/mysqli/media byte-id · http DIFF-set 16 byte-id al WP-44 ·
  option 413 / restapi 3508 identici per nome COL conteggio. Baseline
  corpus resta **1421** (`gate-out-wp46-archived/corpus.fails`).
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run34** (~/Claude/wpdev, WP-47): 30.472 test, 0E/2F/86W/73S,
  **fail-set BYTE-IDENTICO a run33**; baseline resta
  `wp16-harness/full-out/run33-fails.txt` (88 righe, = run34-fails.txt).
  Master-CPU **≈19:20 = 3,42×** (da 21:00/3,71× WP-46; riferimento WP-40
  11:39/2,06× non ancora recuperato — residuo = collect sul grafo vivo
  della full). Wall ~23 min = 2,0×. Multisite: 1 diff = minimo teorico.
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

## 🎯 PROSSIMO LAVORO — WP-48: shrink unit (Fase 1.1) + recupero CPU full-suite

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

**WP-47 ha chiuso l'attribuzione** (arr 100%, str 96,7%; owner unico
`reflect_method_info_cache` → eviction landed, −63,8% peak). Il residuo
(census post-fix, dettaglio in `sessions/WP_SESSION_47.md`):

1. **Fase 1.1 — shrink unit ~0,30G** (2046 moduli `Box::leak`ati; ora il
   canale singolo più grosso): `shrink_to_fit`/`Box<[T]>` sulle Vec
   ritenute di Func/Module + drop dei seed HIR post-link. Guardia CPU
   ≤+0,5%, A/B interleaved.
2. **Recupero CPU full-suite** (3,42× vs riferimento 2,06× WP-40):
   PRIMA attribuzione (gc/op census su un gruppo grande o sulla full) —
   il sospetto è il collect-walk sul grafo vivo della full; leve
   candidate: rooting più selettivo, collect ancorato a un segnale di
   confine se esiste, cap dell'escalation. Il +1,3% media (rebuild
   descrittori evitti) si può limare alzando il cap 8192 SE il census
   mostra thrash (misurare, non indovinare).
3. Coda invariata: cold-box Object (~96B×istanze), Fase 2 CPU
   (RET_DEREF+ret_shape, Sweep elision), interning const-array
   (divergenza conteggio documentata), str residuo walk 3% (metadati
   moduli), disciplina di confine per-test (Fase 1.4).
Residui famiglia gc pescabili: gc_047 (nota release iteratore al break),
gc_030 (trace shape dtor-da-collect), gc_022 (temp container mid-statement);
bug isset via `__get` con indici annidati (WP-42).

## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT DISCO**: `df -h /System/Volumes/Data`, non partire sotto
   ~15-20G liberi (WP-44 è partita a 16G; il cargo test DEBUG rigenera
   ~3,8G di `php-rust-output/debug/` — in sessione usare SEMPRE
   `cargo test --release`, e pulire `debug/` se ricompare).
   ⚠️ WP-47 è chiusa con Data a **11Gi** (deriva non di sessione: i target
   census/old sono sull'esterno; `php-rust-output` è 1,3G release-only) —
   prima di WP-48 liberare spazio di sistema (cache utente/log) o
   verificare che basti.
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

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_47.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-47): **media CPU 3,00× di giornata (+1,3% tenuto; riferimento
comparabile ~2,85× WP-46) · footprint peak fisico 1,77/0,41G = 4,3× ⭐⭐⭐
(da 11,9×: −63,8%) · full run34 ≈19:20 = 3,42× (da 3,71×), fail-set
byte-id a run33 · tabella viva: `gaps/REPORT_GAP_47.md`**.
