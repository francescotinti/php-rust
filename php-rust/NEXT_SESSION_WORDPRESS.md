# Rotta WORDPRESS-FIRST — WP-track (dopo WP-46: GC container-roots LANDED a parità record, ma il bersaglio footprint ~3G NON è caduto → WP-47 = ri-attribuzione owner-level)

> ⚡ **WP-46 (2026-07-24, `d4a02fa`→`e6af390`)** — **Cycle collector esteso
> ai root NON-oggetto (array condivisi, inner delle Ref, closure), fedele a
> zend_gc.c letto via Vexp: buffer container a `Weak` (CtrWeak, dedup
> `weak_count==0`, zero perturbazione dei conteggi), cicli puro-container
> spezzati svuotando le Ref-cell white (unico punto di taglio in safe Rust:
> array COW, capture fisse), conteggio `gc_collect_cycles()` Zend-esatto
> (esclusione dtor-subtree, peel refcount-dead, eccezione DELREF-muto) —
> 18/18 probe oracle byte-id. Under-note storici chiusi: PropUnset, BindRef
> (`$a =& $b`), typed-unset; `gc_enable/disable/enabled/status` host-side
> reali (INI zend.enable_gc, latch gc_active per gc_016/gc_049);
> `*RECURSION*` senza `&`. PARITÀ RECORD: corpus 1447→1421 (0 nuovi, 26
> fixati), famiglia gc 36→14, gate22 COMPLETO verde (option 413 / restapi
> 3508 identici per nome col conteggio), cargo 1639/0.**
> **MA il mechanism-check ha FALSIFICATO il bersaglio: collector operativo
> (11 collect, 726k root processati) e freed=543 — arr 3,113M/1,9G e str
> 23,8M/1,29G INVARIATI alla cifra; i root notati sono dati VIVI. ⭐⭐ La
> diagnosi WP-45 "3,08G = cicli irraggiungibili" è da RI-ATTRIBUIRE:
> irraggiungibile-dal-root-walk ≠ ciclo morto (il walk non cammina
> FramePool/operand-stack/tabelle VM). A/B: +7,0% CPU e +2,5% footprint
> TENUTI per direttiva no-revert (2,85× media, 12,3× peak).**
> **Storia: `sessions/WP_SESSION_46.md`. WP-44: `sessions/WP_SESSION_44.md`.**

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

## Stato gate per nome (gate22 completo su `e6af390`, 2026-07-24, archivio `gate-out-wp46-archived/`)

- Gate22 verde: corpus **1421** (da 1447: 0 nuovi-fail, 26 rimozioni — 22
  famiglia gc + bug35163×2/bug35239/foreach_002 dai fix &*RECURSION*/
  BindRef/unset-note) · sess 28 · date 351 · refl 290 IDENTICI · ORM
  3E/13F · hk 0E/0F · cargo **1639** (1637 + 2 sentinelle WP-46) · probe
  gd/mysqli/media byte-id · http DIFF-set 16 byte-id al WP-44 · option 413
  / restapi 3508 identici per nome. ⚠️ nuova baseline corpus = **1421**
  (aggiornare i confronti; fail-set in `gate-out-wp46-archived/corpus.fails`).
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

## 🎯 PROSSIMO LAVORO — WP-47: RI-ATTRIBUZIONE owner-level del footprint

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi comunque, **niente revert su insuccesso**.
Laravel POSTICIPATA a valle.

**WP-46 ha eseguito la leva dominante e il mechanism-check l'ha
falsificata come leva footprint** (dettaglio in testa e in
`sessions/WP_SESSION_46.md`): il collector esteso Zend-model è landed,
corretto e gate-proven, ma i possible-root che transitano dai note-site
nel run media sono VIVI (726k root → freed 543) e i canali non si sono
mossi di una cifra. Le due letture da discriminare:
(A) i cicli morti muoiono per vie non notate — ma ogni probe costruito per
trovare il drop silente è finito su un sito notato o su un write-through;
(B) **il ~3G "irraggiungibile" del WP-45 è in realtà TENUTO da holder vivi
fuori dalle 11 categorie del root-walk** (FramePool con slot sporchi?
operand stack di frame? ob/resources/iteratori/tabelle VM non censite?).

**WP-47 = attribuzione di SECONDA generazione, PRIMA di ogni altra leva:**
1. **Owner-tracer**: strumento mem-census che campiona CHI tiene i 3,1M
   array vivi (1-su-N: al drop dell'ultimo handle noto risali... o più
   semplice: walk COMPLETO del Vm — ogni campo che può tenere Zval/Rc —
   e bilancio per-categoria fino a riconciliare il 1,9G arr + 1,29G str).
2. Root-walk esteso: FramePool (i frame ritirati azzerano gli slot?),
   frames stack completo, ob/output buffers, resources, iteratori,
   session, typed_refs, IC/tabelle — qualsiasi campo Vm Zval-bearing.
3. Se emerge il vero holder → la leva giusta (drain pool al retire?
   disciplina di confine per-test? created→Weak Fase 1.2?) si sceglie coi
   dati, non si indovina.
4. Recupero CPU possibile solo DOPO l'attribuzione (rooting selettivo /
   collect al confine test) — il +7% resta finché non si sa se serve.
In coda (invariati): shrink unit ~0,3G (Fase 1.1), interning const-array
(divergenza conteggio documentata), cold-box Object, Fase 2 CPU
(RET_DEREF+ret_shape, Sweep elision).
Residui famiglia gc pescabili: gc_047 (nota release iteratore al break),
gc_030 (trace shape dtor-da-collect), gc_022 (temp container mid-statement).

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
