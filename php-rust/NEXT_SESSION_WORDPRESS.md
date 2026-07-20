# Rotta WORDPRESS-FIRST — WP-track (dopo WP-28: gap estensioni chiusi + GC free-order Zend)

> 🏁 **WP-28 (2026-07-20, gated `b72d14f` + `29bbb4e`)**: chiusura dei gap
> estensioni del handoff WP-27. **(1) asymmetric visibility 29→38/39**:
> `prop_indirect_guard` (container-fetch W/RW/UNSET di prop readonly/set-denied
> — Zend get_property_ptr_ptr+read_property: valore oggetto passa, RW su
> typed-uninit = uninit fatal, unset su uninit no-op, altrimenti "Cannot
> indirectly modify") cablato in field_write/field_unset/field_cell/
> asym_set_ref_copy; assign-on-null Warning→**Error** con verbo assign/modify;
> promotion porta set_visibility (cpp_*); ridichiarazione PLAIN di prop hooked
> **EREDITA gli hook** (GH-19044); msg readonly esplicito protected(set).
> **(2) ext/xsl 57→63/64**: trace-shaping (frame prelude→prelude SPARISCONO dal
> backtrace, call-site prelude = "[internal function]" — bug49634 + 3 corpus);
> registerPHPFunctionNS; sezione xsl in phpinfo; **input-callback libxml FFI per
> compress.zlib://** (xslt008/-mb/009 — ⚠️ xslt009 passa con CWD = root di
> php-8.5.7, convenzione make test: misurare la suite xsl dalla root).
> **(3) GC free-order Zend-fedele**: gc_queue max-heap→**FIFO** (ordine di nota
> = ordine di release = ordine free/destructor Zend) + **gc_birth** (le entry di
> gc_track/re-seed sono seed interni phpr: la cascata del padre le CONSUMA) +
> **gc_release_cascade** (untrack dei discendenti esclusivi senza distruttore ⇒
> Object::drop postorder replica la cascata, id del PADRE in cima al free-list)
> + purge var_dump_debug/stringify_args al release. Probe id unset/temp/
> multi-unset ESATTI vs oracle; tidy resta 44/45 (010: solo il caso
> var_dump-albero, inquinato dalle over-note del dump — residuo).
> **Gate22 tutto verde** (nessuna regressione da FIFO/trace su ORM/hk/option/
> restapi) · corpus 1476→**1455** (21 rimossi, 0 nuovi) · **run19 = run17 per
> nome** · **multisite riconfermata: 1 diff (wp_is_stream #2) = minimo teorico**.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- Gate22 WP-28 verde (wp22-harness/gate-out): corpus **1455** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F identico per nome · hk 1665 0E/0F ·
  cargo **1567**/0 · probe gd/mysqli/media byte-id · http battery DIFF-set = 16
  (WP-14) · option 413 e restapi 3514 identici per nome.
- **Full-suite single-site run19: IDENTICA a run17/run16 per nome (30.481
  test, 0E/2F/86W/73S) = minimo teorico**. Archiviata in
  `wp16-harness/full-out/run19/`.
- **Full-suite multisite RICONFERMATA (WP-28): 1 diff per nome — minimo
  teorico** (31.278 test, 0E/2F; solo `wp_is_stream #2`;
  `wp19-harness/ms-out/diff-names-wp28.txt`).
- Suite phpt estensioni (misura): **xsl 63/64** (⚠️ da CWD = root php-8.5.7) ·
  tidy 44/45 · asymmetric_visibility **38/39**. Suite phpt SEMPRE con path
  ASSOLUTO.

## Harness full-suite (WP-16 — invariato)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
"$H/run-full-detached.sh" phpr   # lanciarlo con un daemonizer perl (double-fork
                                 # + setpgrp) da un task BACKGROUND: il task-kill
                                 # a 10' non deve raggiungere la run
# ⚠️ MAI due gate22 insieme; MAI probe su wptests durante una run;
#   azzerare wpdev/src/wp-content/uploads prima di ogni full run;
#   non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/;
#   marker ms-phpr.done)
```

## 🎯 PROSSIMO LAVORO
1. **Validazione Laravel** ([[php-rust-roadmap-wp-first]]): WP-track satura
   (full-suite e multisite al minimo teorico; estensioni citate chiuse salvo
   3 residui strutturali sotto).
2. **Slot-index fast path** (CPU, facoltativo, da WP-27): `resolve_prop_access`
   conosce la classe → restituire l'INDICE di slot precalcolato in `PropInfo`
   e saltare `PropsLayout::slot_of`; aggancio ai flag per-classe esistenti.
3. **Residui strutturali** (se si vuole il 100% delle suite estensioni):
   - `ast_printing.phpt`: serve un vero zend_ast_export sull'HIR (il dedent
     del sorgente non basta: manca la riga vuota dopo le class decl).
   - xsl `bug69168`: i nodi passati a php:function devono ALIASARE il doc
     live (phpr DOM ≠ libxml: servirebbe sync-back o DOM libxml-backed).
   - tidy `010`: ordine free nel caso var_dump-di-albero (le over-note
     sintetiche del dump inquinano il FIFO; i casi unset/temp sono esatti).

## Candidati successivi
1. **CPU residua strutturale** (profilo wp22-harness/prof-out/): method
   dispatch fast-path; interning nomi; memmove da concat. ⚠️ A/B SOLO coppie
   interleaved.
2. **Memoria packed residua**: mimalloc in-place realloc o reserve esplicita
   nei costruttori bulk.
3. Ordine destructor per oggetti CON `__destruct` nel subtree (Ret-hook usa
   ancora gc_cascade, non gc_release_cascade) — nessun test lo copre oggi.
4. Verbo "increment/decrement" per `$null->p++` (oggi "assign") — serve
   threading del verbo nel funnel FieldIncDec.
5. Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).

## 📊 REPORT GAP PERF ORACLE↔PHPR — ATTIVITÀ RICORRENTE DI FINE SESSIONE
A OGNI chiusura di sessione, prima del commit finale di memoria/handoff,
misurare e riportare all'utente il gap aggiornato e aggiornare la tabella
(⚠️ confrontare RAPPORTI, mai i tempi assoluti di giornate diverse):
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

| sessione | media CPU (phpr/oracle) | media footprint | full-suite master-CPU | full-suite wall |
|---|---|---|---|---|
| WP-26 (baseline) | 85,8/21,0 = **4,1×** | 5,0/0,4GB = **12,7×** | (wall, non comparabile) | ~1,9× |
| WP-27 | 82,7/21,1 = **3,9×** | 4,78/0,40GB = **12,0×** | 16:11/5:39 = **2,9×** | ~22/11,5 min = **1,9×** |
| WP-28 | 87,6/23,0 = **3,8×** | 4,83/0,40GB = **12,2×** | 16:43/5:39 = **3,0×** | ~22/11,5 min = **1,9×** |

## Lezioni operative (nuove WP-28)
- ⭐⭐ **Ordine free/destructor Zend = ordine delle RELEASE**: la coda dei
  candidati GC deve essere FIFO in ordine di nota; le entry di gc_track alla
  nascita (e i re-seed light-demoted) NON sono release — vanno marcate
  (gc_birth) e consumate dalla cascata del padre, altrimenti bloccano il
  riuso id children-first di Zend.
- ⭐ **Cache per-id (var_dump_debug/stringify_args) vanno purgate al FREE**,
  non solo al riuso in next_id: un debugInfo memoizzato tiene vivi i
  contenitori dell'oggetto e falsa i conteggi di esclusività della cascata.
- ⭐ **zend_std_read_property W/RW/UNSET** su prop readonly/set-denied:
  oggetto→copia (l'indirezione via handle non scrive lo slot), UNDEF+UNSET→
  no-op, altrimenti "Cannot indirectly modify"; ptr_ptr RW+UNDEF+typed →
  uninit fatal PRIMA di readonly/aviz (vale anche per prop pubbliche!).
- ⭐ **I prelude sono gli internals C di Zend anche nei BACKTRACE**: frame
  prelude→prelude si elidono; un frame user chiamato dal prelude rende
  "[internal function]" senza chiavi file/line nell'array.
- ⭐ **Suite phpt e CWD**: run-tests gira dalla ROOT di php-src — i test che
  usano path relativi (xslt009: document('compress.zlib://ext/...')) passano
  solo da lì. Il runner eredita la cwd: misurare le suite ext dalla root.
- ⚠️ Il timeout dei task background è 10 min: run >10' vanno lanciate con un
  daemonizer perl (double-fork + setpgrp + exec-array per i path con spazi) e
  monitorate sul marker .done.

## Invarianti (aggiornati WP-28)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline:
  **corpus 1455** · sess 28 · date 351 · refl 290 (SOLO rimozioni ammesse;
  fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F per nome ·
  http-kernel 1665 0E/0F · cargo (**1567**) · probe: gd 11/11, mysqli 11/11,
  media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP suite per-classe =
  oracle (option 413 · media 762 · post 906 · user 1341 · query 1889 ·
  restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 · sitemaps 132 ·
  classi WP-17/18). Script: `wp22-harness/gate22.sh` (lanciarlo col
  daemonizer; ~22 min).
- Full-suite single-site: solo miglioramenti per nome vs **run19 (= run17 =
  run16; 1 diff: wp_is_stream #2)**. Multisite: vs **ms-out WP-28 (1 diff
  idem)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI, sotto watchdog o
  daemonizer, marker .done su disco; Serena per Rust (in timeout: verificare
  lo stato del file prima di riprovare); Vexp/Read per il C; Read/Write tool
  per i .php; log `tr -d '\0'`; uploads azzerati prima di ogni full run.
