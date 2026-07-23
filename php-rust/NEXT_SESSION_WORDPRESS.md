# Rotta WORDPRESS-FIRST — WP-track (dopo WP-43: stadio 1 registri ACCETTATO a delta zero → WP-44 = stadio 2 Binary/CmpJmp a operandi diretti)

> ⚡ **WP-43 (2026-07-23, `9cc141b`)** — **STADIO 1 Leva B ESEGUITO E
> ACCETTATO: infrastruttura registri a delta zero.** `Func.max_temps` (=0,
> PAST n_slots — distinto dai temp compiler già fusi in n_slots) ·
> `Frame::with_buffers` dimensiona `n_slots+max_temps` (unico sito, zero
> campi nuovi nel Frame) · `bytecode::Operand{Stack|Slot|Temp|Const}` ·
> `compile/reg_lower.rs` pass VUOTO dietro `PHPR_REG_LOWER` agganciato in
> `compile_body` (funnel di tutti i corpi caldi) · `UnitKey.reg_mode` ·
> dump `PHPR_DUMP_OPS` (canale-diff per gli stadi futuri) · test identità
> (cargo **1637**). Tre criteri passati: dump flag-on/off **byte-id su
> 162k righe**; gate22 TUTTO verde per nome; **A/B 6 round RUMORE ZERO**
> (new 55,70 vs old 56,38, segno alternato). ⚠️ **Incidente MySQL**: il
> datadir vero è `mysql-wp8/data` sul drive ESTERNO (socket
> `/private/tmp/mysql-wp8.sock`) — il server è morto e il restart naive
> apre il datadir brew VERGINE ⇒ option/restapi/http **FALSI VERDI a 0
> nomi** (validare sempre col conteggio: option 413, restapi 3508);
> recuperato con mysqld_safe daemonizzato sul datadir esterno. **Storia:
> `sessions/WP_SESSION_43.md`.**

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

- **Zero `unsafe` nel value core** (RULEBOOK §0; NaN-boxing WP-32 e
  SSO-union WP-38 bocciati — non riproporre senza rotta esplicita utente).
- **Bytecode a registri = unica "leva lunga" approvata**; l'arco è APERTO:
  piano e census in `REGISTER_BYTECODE_PLAN.md` (WP-42). JIT fuori
  orizzonte; arena per-request collide con byte-parity dtor.
- Micro-bench solo advisory: verdetti SOLO su A/B interleaved stesso-giorno
  sul workload reale. Gate per NOME a ogni commit; refactor layout/GC =
  sentinelle drop-order pinnate PRIMA; oracle-probe con `-d log_errors=0`.
- Commit AND push a ogni step; deviazioni deliberate = marker
  `BUG(port):` / `PERF(port):` / `TODO(port):`.

## Stato gate per nome (gate22 completo su `9cc141b`, 2026-07-23)

- Gate22 verde (wp22-harness/gate-out): corpus **1447** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F · hk 0E/0F · cargo **1637** ·
  probe gd/mysqli/media byte-id · http DIFF-set 16 · option 413 / restapi
  3508 identici per nome. ⚠️ Se un gate attraversa una finestra disco-pieno:
  RILANCIARE le suite che scrivono (corpus/sess) — i "nuovi fail" ENOSPC
  mentono. (Se ORM/hk in /private/tmp spariscono: ri-estrarre i tarball da
  wp9-harness/gates/.)
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine in
  /opt/homebrew/var/mysql: utente 'wp' assente, wp_o mancante ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio corretto: `mysqld_safe
  --datadir=".../mysql-wp8/data" --socket=/private/tmp/mysql-wp8.sock`
  daemonizzato (double-fork+setsid). Un gate DB-dipendente "IDENTICO" va
  SEMPRE validato col conteggio nomi (option 413, restapi 3508).
- **Full-suite run32** (~/Claude/wpdev, trunk@5e3fced): 30.472 test,
  0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run31**; baseline =
  `wp16-harness/full-out/run32-fails.txt` (88 righe). Master-CPU ~12:50
  nominale su giornata rumorosa — riferimento resta ~11:39 (WP-40) con
  A/B same-day flat. Multisite (WP-28): 1 diff = minimo teorico.
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

## 🎯 PROSSIMO LAVORO (Leva B, stadio 2 — dal piano `REGISTER_BYTECODE_PLAN.md` §5)

0. **PRE-FLIGHT DISCO**: `df -h /System/Volumes/Data`, non partire sotto
   ~15-20G liberi (WP-43 è partita a 18G: ok; pulire eventuale
   `php-rust-output/debug/` rigenerato — a fine WP-43 già pulito).
   **PRE-FLIGHT MYSQL**: `mysql -h 127.0.0.1 -u root -e "SHOW DATABASES"`
   deve elencare wp_o/wp_p/probe — altrimenti vedi ⚠️ MySQL in "Stato
   gate" (datadir esterno, MAI mysql.server start naive).
   ⚠️ Taratura census: il 30,77% data-movement è quota del CONTEGGIO
   dispatch, non del tempo CPU — il tetto resta ~8-15% (piano §2).

1. **Stadio 2 — Binary/CmpJmp a operandi diretti** (assorbe binary_fast/
   CmpJmpConst WP-33/34, SOSTITUZIONE mai convivenza — I-cache è il
   rischio n.1): `Binary{l,r,dst}` con sorgenti Slot/Const/Temp; il pass
   `reg_lower::lower_func` (oggi vuoto, wiring già in `compile_body`)
   riscrive i trigrammi LoadSlot,LoadSlot,Binary. Bigrammi target dal
   census: ThisPropGet→CmpJmpConst 29,9M · CmpJmpConst→PushConst 16,3M ·
   Dup→StoreSlot+StoreSlot→Pop ~9M l'uno. Vincoli piano §3: mai riordinare
   oltre op osservabili (flush diagnostico WP-33), RHS-first, Ref-slot →
   forma generica. **Il diff di stadio si prova col dump `PHPR_DUMP_OPS`**
   (flag-on vs flag-off: devono differire SOLO le sequenze riscritte).
   A/B go/no-go ≥4 round; revert secco se flat/regressione.
2. **NON riproporre**: leve locali sul canale churn (esaurite: fusioni
   WP-33/34, shim gc_note WP-41, by-borrow WP-42); NaN-boxing; SSO union.
3. Fronte footprint (12×): NON aggredito; quando si apre → PRIMA una
   sessione di attribuzione memoria data-driven (metodo WP-26). I verdetti
   sul doc Gemini "vincoli safe-Rust" sono in WP_SESSION_42 (AST-leak
   falso; unit-cache=opcache deliberato; PhpArray già dual-repr).
4. **Validazione Laravel** ([[php-rust-roadmap-wp-first]]) alla chiusura
   dell'arco perf.

## Backlog aperto (non legato a una sessione)

- 🆕 **isset via prefisso `__get` con indici annidati** perde il walk sul
  risultato (`isset($mg->m['a']['b'])` → false, oracle true) — bug
  funzionale preesistente trovato dalla probe WP-42
  (wp42-harness/probe-isset-div.php §3). Candidato fix.
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
- Disco root: a inizio/fine WP-43 ~18G liberi (l'utente ha liberato spazio
  dopo WP-42) — pre-flight §0 resta obbligatorio (soglia ~15-20G); pulire
  `php-rust-output/debug/` quando cargo test lo rigenera (~3,2G).

## 📊 Report gap perf — ricorrente di fine sessione

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_43.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-43): **media 2,68× (A/B rumore zero, old 2,71× same-day) ·
full [run32, non rilanciata: delta zero] · footprint raw 8,5× su oracle
alto di giornata — riferimento strutturale resta ~12×**.
