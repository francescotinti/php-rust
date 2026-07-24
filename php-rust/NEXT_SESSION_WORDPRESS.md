# Rotta WORDPRESS-FIRST — WP-track (dopo WP-48: Fase 1.1 shrink unit landed al 100,0% del predetto + attribuzione CPU full CHIUSA; WP-49 = leve collector + Fase 1 residua)

> ⚡ **WP-48 (2026-07-24, `33e358a`+`ba16547`)** — **Fase 1.1 shrink unit
> LANDED col mechanism-check più pulito della roadmap: contatore
> `unit_slack` (census) misura PRIMA lo slack cap−len dei moduli leakati
> = 70.625.796 byte → `Module::shrink()` a fine compile (shrink_to_fit su
> ops/lines/consts/exc_table + tutti i Func annidati: metodi, thunk
> const/attr, hook, param-default; Rc condivisi skippati via get_mut) →
> il canale unit cala ESATTAMENTE di quel numero (299,9→229,3MB, 100,0%
> predicted-vs-actual, soglia era ≥70%; unit_slack post = 0). ⭐ lo slack
> era il 23,5% del canale: "0,30G" era la taglia del CANALE, non della
> leva. A/B 6 round: CPU +0,04% = RUMORE (guardia ≤+0,5% rispettata,
> prima leva memoria a costo zero); peak fisico −0,52% (il picco è
> mid-run, lo slack matura a fine run — proxy-peak census −67MB).**
> **Ob.2 ATTRIBUZIONE CPU FULL CHIUSA coi numeri (`ba16547`, contatori
> gc-census): `classify_ms` 494,9s ≈ 8:15 su 1005 collect / 9,93M root /
> 14,16M freed = l'INTERO residuo 3,4× vs 2,06× WP-40 (≈7:41); la
> reflect-cache cap 8192 è in thrash (890k miss / 16% hit / 108
> eviction) ma è SECONDARIA (~10× sotto). Le leve vanno al collector,
> non all'eviction.** Analisi ritenzione: seed HIR NON rilasciabili
> (servono alla lowering seeded; WP-20 già minimale). PARITÀ TOTALE:
> corpus 1421 IDENTICO, gate22 completo verde col conteggio, full run35
> BYTE-ID a run33 (88 nomi), cargo 1639/0 ×2.
> **Storia: `sessions/WP_SESSION_48.md`. WP-47: `sessions/WP_SESSION_47.md`.**

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

## Stato gate per nome (gate22 completo su `33e358a`+`ba16547`, 2026-07-24, archivio `gate-out-wp48-archived/`)

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
- **Full-suite run35** (~/Claude/wpdev, WP-48): 30.472 test, 0E/2F/86W/73S,
  **fail-set BYTE-IDENTICO a run33** (88 nomi); baseline resta
  `wp16-harness/full-out/run33-fails.txt`. Master-CPU **≈19:10 = 3,39×**
  (run34 ≈19:20 = 3,42×: rumore, nessuna leva CPU in WP-48; riferimento
  WP-40 11:39/2,06× — **residuo ATTRIBUITO: classify 494,9s su 1005
  collect, vedi WP_SESSION_48**). Wall ~24 min = 2,1×. RSS transiente
  mid-run 5431MB invariato da run34. Multisite: 1 diff = minimo teorico.
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

## 🎯 PROSSIMO LAVORO — WP-49: leve collector (bersaglio attribuito) + Fase 1 residua

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

**WP-48 ha chiuso Fase 1.1 (shrink al 100,0% del predetto) e l'attribuzione
CPU full** (classify 494,9s / 1005 collect / 9,93M root = l'intero residuo
3,4× vs 2,06×; reflect-thrash secondario). Dettaglio: `sessions/WP_SESSION_48.md`.

1. **Leve collector full-suite (ora coi numeri)**: cadenza/escalation
   della soglia sulla full (soglia arrivata a 1,05M ma 1005 collect:
   capire il pattern per-test prima di toccare), rooting più selettivo
   (quanti dei 9,9M root processati sono ricorrenti vivi?), e SOLO come
   terza leva il cap reflect-cache più alto (thrash provato 16% hit;
   costo ~5,4KB/entry: cap 16384 ≈ +45MB, misurare non indovinare).
   Nota: freed 14,16M — sulla full il collector libera davvero; una leva
   di cadenza deve reggere il footprint (guardia footprint sull'A/B).
2. **Fase 1 residua**: `created` registry → Weak (item 1.2, il buco più
   grosso rimasto), cold-box Object (1.3, ~96B×istanze), disciplina di
   confine per-test (1.4). Poi Fase 2 CPU (RET_DEREF+ret_shape, Sweep
   elision), interning const-array.
3. **Attribuzione transiente 5,4G mid-run full** (invariato run34→35;
   il binario census più lento mostra max 1,77G ⇒ è pacing-dipendente):
   serve uno snapshot al picco SULLA full.
Residui pescabili: str residuo walk 3% (~39MB metadati moduli), gc_047
(release iteratore al break), gc_030 (trace shape), gc_022 (temp
mid-statement); bug isset via `__get` con indici annidati (WP-42).

## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT DISCO**: `df -h /System/Volumes/Data`, non partire sotto
   ~15-20G liberi (WP-44 è partita a 16G; il cargo test DEBUG rigenera
   ~3,8G di `php-rust-output/debug/` — in sessione usare SEMPRE
   `cargo test --release`, e pulire `debug/` se ricompare).
   ⚠️ WP-48 è chiusa con Data a **~10Gi** (partita a 16 dopo pulizia
   cache Google/Arc/bun: quelle leve sono ESAURITE): `php-rust-output`
   ora 2,1G (artifact di `cargo test --release`: si può fare `cargo
   clean` selettivo dei test artifact), resto probabilmente snapshot
   APFS/purgeable — verificare con `tmutil listlocalsnapshots /` e al
   bisogno `tmutil deletelocalsnapshots`.
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

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_48.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-48): **media CPU 2,93× di giornata (+0,04% = rumore,
guardia rispettata) · footprint peak fisico 1,747/0,395G = 4,42× (canale
unit −70,6MB = 100,0% del predetto) · full run35 ≈19:10 = 3,39× (≈ run34),
fail-set byte-id a run33 · residuo CPU ATTRIBUITO al classify (494,9s /
1005 collect) · tabella viva: `gaps/REPORT_GAP_48.md`**.
