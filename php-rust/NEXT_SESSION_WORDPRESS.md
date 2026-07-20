# Rotta WORDPRESS-FIRST — WP-track (dopo WP-29: slot-index/IC proprietà + dispatch ci)

> ⚡ **WP-29 (2026-07-20 sera, gated `4297fe5`→`f375bc9`, 6 commit)**: punti 1+2
> del piano perf. **(A) Proprietà**: `PropInfo.slot` precalcolato (allineato a
> `PropsLayout`; virtual-hooked = None) + `Props::get_slot/replace_slot` +
> `PropAccess::Slot { key, slot }` + gemelli slot-aware `read/write_property_at`
> (write-through-Ref identico) + de-dup PropOpSet/PropIncDec (era 2 resolve +
> 2 slot_of) + Cow::Borrowed nei FieldScope (via i to_vec per Prop-step) +
> **PropIc**: inline-cache monomorfica per-op-site su PropGet/PropSet/PropIsset
> (`Rc<Cell<(epoch, class_id+1, slot)>>` — il dispatch CLONA l'op ⇒ cella
> condivisa; PartialEq sempre-true per la unit-cache; epoch per-run perché gli
> id classe cambiano tra run sui moduli riusati). Fill SOLO scope-indipendente
> (public hook-free; SET solo plain_set_props — le closure sono ri-bindabili)
> e ⭐ ANCHE dai fast-path WP-25 (senza, la cache resta fredda per sempre sulle
> classi all-public). **(B) Dispatch**: `methods_ci` per classe (ci-hash
> ordinata, binary search in resolve_method_runtime, ⚠️ soglia ≥12 metodi —
> sotto, lo scan early-exit vince; ⭐ PRIMO-vince dentro la stessa classe:
> alias di trait duplicano i nomi, bug61998) + `Module.fn_ci` (via lo scan
> O(n) di invoke_named col prelude) + registry builtin SipHash→FxHash + LcKey
> stack-buffer (via il to_ascii_lowercase allocante da class_index/linked) +
> Hash di PhpStr = zhash cached (zend_string->h). **Esito misurato**: media
> group **−0,4% (rumore)** — è dominato da gd/webp/mysql; **full-suite
> master-CPU 16:43→15:27 = −7,6%** (run20; il carico dispatch-heavy
> beneficia). run20 = run19 per nome; gate22 TUTTO verde.

# (storia WP-28)

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
- Gate22 WP-29 verde (wp22-harness/gate-out): corpus **1455** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F identico per nome · hk 1665 0E/0F ·
  cargo **1573**/0 · probe gd/mysqli/media byte-id · http battery DIFF-set = 16
  (WP-14) · option 413 e restapi 3514 identici per nome.
- **Full-suite single-site run20: IDENTICA a run19/run17/run16 per nome
  (30.481 test, 0E/2F/86W/73S) = minimo teorico**, master-CPU 15:27.
  Archiviata in `wp16-harness/full-out/run20/`.
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
2. **Perf, prossima leva (dal profilo POST-WP-29,
   `wp29-harness/ab-out/new-t45.sample`)**: il tetto misurato di
   slot-index+interning sul MEDIA group è ~1% (dominato da gd/webp/mysql);
   sulla FULL-suite vale −7,6%. I colli residui phpr-only, in ordine:
   run_loop/dispatch (1188 campioni — opcode specialization/quickening),
   memmove 328 + drop/clone Zval ~250 (value churn → la mossa grossa è la
   value-representation), call machinery bind_params+enter_callee+Frame::drop
   ~165 (**frame arena**: il candidato col miglior ROI/rischio), GC
   note/sweep ~180, PhpArray::insert+hashbrown ~130, is_instance_of+iface
   ~75 (cache per (class,target)). Estensioni facili del lavoro WP-29:
   PropIc anche su PropOpSet/PropIncDec e IC (class→(defc,midx)) sui siti
   MethodCall.
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
| WP-29 | 82,4/23,0 = **3,6×** | 4,84/0,40GB = **12,1×** | 15:27/5:39 = **2,7×** | ~22/11,5 min = **1,9×** |

## Lezioni operative (nuove WP-29)
- ⭐⭐ **Le inline-cache vanno riempite da OGNI percorso che risolve** — se un
  fast-path esistente intercetta il traffico prima del percorso generale (il
  solo che riempiva), la cache resta fredda per sempre e paghi solo il guard.
- ⭐⭐ **Il media group NON misura il dispatch**: è dominato da gd/webp/mysql
  (le stesse C lib dell'oracle). Le ottimizzazioni VM si vedono sulla
  FULL-suite (−7,6% qui) — scegliere il benchmark in base a cosa si ottimizza.
- ⭐ **hash-then-bsearch perde contro lo scan early-exit sotto ~12 voci**
  (stessa soglia di HASH_SCAN_MIN); e FxHasher `write_u8` per byte = un round
  per byte, SEMPRE lowercase su stack buffer + un `write(slice)`.
- ⭐ **Op payload con stato runtime**: il dispatch CLONA l'op → la cella va
  `Rc`-condivisa; `PartialEq` sempre-true per non rompere la unit-cache;
  epoch per-run perché gli id classe non sono stabili tra run sui moduli
  riusati.
- ⭐ Le closure sono ri-bindabili (`Closure::bind`): MAI cachare per-sito un
  esito di visibilità non-public.
- Il worktree per il binario old: se l'HD interno è pieno, `rm -rf` del
  profilo debug di php-rust-output (2,2GB ricreabili) prima di cambiare
  target dir.

## Lezioni operative (WP-28)
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
  http-kernel 1665 0E/0F · cargo (**1573**) · probe: gd 11/11, mysqli 11/11,
  media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP suite per-classe =
  oracle (option 413 · media 762 · post 906 · user 1341 · query 1889 ·
  restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 · sitemaps 132 ·
  classi WP-17/18). Script: `wp22-harness/gate22.sh` (lanciarlo col
  daemonizer; ~22 min).
- Full-suite single-site: solo miglioramenti per nome vs **run20 (= run19 =
  run17 = run16; 1 diff: wp_is_stream #2)**. Multisite: vs **ms-out WP-28
  (1 diff idem)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI, sotto watchdog o
  daemonizer, marker .done su disco; Serena per Rust (in timeout: verificare
  lo stato del file prima di riprovare); Vexp/Read per il C; Read/Write tool
  per i .php; log `tr -d '\0'`; uploads azzerati prima di ogni full run.
