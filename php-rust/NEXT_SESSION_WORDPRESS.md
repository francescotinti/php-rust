# Rotta WORDPRESS-FIRST — WP-track (dopo WP-52: in-node marks −42,7% classify + cold-box Object → full 2,14×, media 2,83×/4,36×; WP-53 = residuo ~30s vs 2,06× + reflect-cache owner / Fase 2)

> ⚡ **WP-52 (2026-07-25, `1a52712`+`ebd44f3`)** — **Ob.1 in-node marks:
> `WalkMark{epoch,slot}` 8B NEI nodi (Object dentro GcMark, campo nuovo su
> PhpArray/Closure; Ref su mini-mappa — non ha struct), stato del walk in
> `Vec<NodeRec>` contigua che È la worklist FIFO (stessa sequenza ⇒ stessi
> conteggi): classify_ms census 109.383→62.678 = −46,7s (−42,7%), OLTRE
> la quota −25..−34s (surplus = get-per-pop pass-2 + iterazione contigua
> vs mappa). Conservazione alla cifra: collects 303=303, calls 152=152,
> freed Δ−16 su 14,2M.** Ob.2 Fase 1.3 cold-box (`rare:
> Option<Box<ObjRare>>` = −64B/istanza; **dyn_entries ESCLUSO e
> verbalizzato: è lo storage di stdClass = fast-path WP, rischio classe
> WP-44**): −56,0B/oggetto alla cifra (87,5%, obj.live_n conservato
> 525.258=525.258), churn obj −2,02GB. **Giudici: full stesso-giorno
> run40 725,1s = 2,14× vs old phpr-wp51b 787,4s = 2,32× (−7,9% raw,
> ~−6,7% corretto — mechanism-backed); media ab52 CPU −0,57% new 6/6 →
> 2,83× (minimo di sempre); peak fisico +0,61% (+10,4MB, guardia ≤+2% ok
> ma predizione −2MB MANCATA: size-class mimalloc 80→96B sugli array —
> regressione piccola TENUTA e verbalizzata).** Parità: corpus 1421
> IDENTICO ×2, cargo 1639/0 ×2, fail-set BYTE-ID a run33 su census52 +
> run40 + run40-old. ⭐⭐ RSS ps dei full (new 4024 vs old 2897 MB) =
> accounting MADV/pagine tenute residenti dai mark, NON footprint — il
> peak fisico media si muove di +0,61%. Residuo vs WP-40 2,06×: **~30s**
> (era ~85s). Binario stashed: `phpr-wp52` (sha256 57607da3…).
> **Storia: `sessions/WP_SESSION_52.md`. WP-51: `sessions/WP_SESSION_51.md`.**

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

## Stato gate per nome (gate22 completo su `60c7e04`, 2026-07-25, archivio `gate-out-wp50-archived/`; leve GC/layout WP-52 `1a52712`+`ebd44f3` gated con corpus ×2 + cargo ×2 + 3 full BYTE-ID)

- Gate22 verde: corpus **1421** IDENTICO alla baseline WP-46 (0 nuovi-fail;
  ri-verificato ×2 in WP-52 su entrambi gli alberi — set =
  `wp52-harness/corpus.fails`) · sess 28 · date 351 · refl 290
  IDENTICI · ORM 3484 3E/13F per nome · hk 1665 0E/0F · cargo **1639/0 ×2**
  · probe gd/mysqli/media byte-id · http DIFF-set atteso (WP-14, 2 item) ·
  **option 413/1061 · restapi 3508/15947 (1E = oracle) IDENTICI per nome
  COL conteggio**. Baseline corpus resta **1421**.
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run40** (~/Claude/wpdev, WP-52, binario `ebd44f3`+docs):
  30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi;
  idem run40-old e census52); baseline resta
  `wp16-harness/full-out/run33-fails.txt` (run39 archiviato in
  `full-out/run39/`). Master-CPU **725,1s ≈12:05 = 2,14×** (coppia
  stesso-giorno: old phpr-wp51b 787,4 = 2,32×; WP-40 11:39 = 2,06× =
  riferimento residuo ~30s). Wall ~16,3 min = 1,4×.
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
```

## 🎯 PROSSIMO LAVORO — WP-53: residuo ~30s + reflect-cache owner / Fase 2

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

1. **Residuo CPU full 2,14× vs 2,06× WP-40 (~30s)**: il classify census
   residuo è 62,7s e ora è dominato da borrow/iter/strong_count (l'hashing
   è stato eliminato in WP-52) — se si insiste sul classify, la lente è il
   "profilo del walk" (separare borrow-cost da traversal); rendimenti
   decrescenti attesi. In alternativa aprire la **Fase 2 CPU della roadmap
   (mai toccata)**: RET_DEREF+ret_shape COME UNA SOLA MODIFICA (~40,5M
   DerefTop sprecati) + Sweep emit-time elision (~47M noop) — op-census
   come mechanism-check, guardia footprint ≤+2%.
2. **reflect-cache**: la leva vera è l'OWNER della cardinalità (memo keyed
   su ClassId freschi dei mock, leak WP-47) — il cap è un cerotto
   (16384 falsificata: hit 16,3→16,5%; ritorno a 8192 = decisione utente,
   risparmierebbe ~42MB standing sul media).
3. **Footprint**: canali grossi rimasti = hashed-array (Fase 3, redesign) e
   interning stringhe (Fase 1.5, se il censimento duplicati lo quota);
   il +10,4MB del +8B/nodo WP-52 è pagato e verbalizzato.
4. **NON riproporre**: Fase 1.2 created→Weak/eviction rc==1 (falsificata);
   rooting selettivo; cap rerun stile Zend; **bound/guardie calcolate
   negli arm caldi del run_loop** (WP-50: +1 load+max = +17..35s — campo
   cache ai siti di mutazione); soglia adattiva/isteresi su buffer RAW;
   collect per-test (il gating a crescita è la forma giusta, WP-51);
   **cold-box di dyn_entries** (storage di stdClass = fast-path, WP-52);
   giudizi footprint dall'RSS telemetrico dei full (accounting MADV —
   solo peak fisico /usr/bin/time -l).

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
Ultimo stato (WP-52): **media CPU 58,84/20,82 = 2,83× (A/B −0,57%, new
6/6) · footprint peak fisico 1,720/0,395G = 4,36× (A/B +0,61% = +10,4MB,
tenuta e verbalizzata) · full run40 725,1s ≈12:05 = 2,14× (old
stesso-giorno 787,4 = 2,32×; riferimento WP-40 2,06×, residuo ~30s),
fail-set byte-id a run33 su 3 run · classify −42,7% (in-node marks) +
cold-box Object −56B/ist · dettaglio: `gaps/REPORT_GAP_52.md`, trend:
`gaps/GAP_TREND.md`**.
