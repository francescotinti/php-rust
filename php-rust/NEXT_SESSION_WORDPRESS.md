# Rotta WORDPRESS-FIRST — WP-track (dopo WP-30: frame arena + MethodIc + PropIncDec IC)

> ⚡ **WP-30 (2026-07-21 notte, gated `fb5e9c2`→`06a3c5b`, 3 commit)**: frame
> arena + IC sul dispatch metodi + IC su `$o->n++`. **(1) Frame arena**:
> `FramePool` bounded (64 coppie, cap 512 slot) di buffer `(slots, stack)`;
> `Frame::with_buffers` riusa la capacity (l'unica alloc obbligatoria per
> chiamata era `vec![Undef; n_slots]`); pull nei ~17 siti caldi, riciclo nei 6
> siti di DROP (Ret/unwind/drive_to_return/coroutines×3) sempre DOPO
> gc_note_frame — ⭐ l'ordine di clear (slots→stack→resto) riproduce ESATTO il
> Drop derivato: id-reuse LIFO invariato (probe #201 su ricorsione 200 =
> oracle); i siti che MUOVONO il frame (park generatori/fiber, retired_main)
> NON riciclano. **(2) MethodIc** su MethodCall+StaticCall (gemello di PropIc:
> `Rc<Cell<(epoch, cid+1, defc, midx)>>`): hit dentro dispatch_instance_call a
> valle degli shunt Generator/Fiber/Closure; ⭐⭐ fill SOLO scope-indipendente =
> vincitore public **E nessun antenato proprio con un metodo `private`
> omonimo** (`private_shadow_in_chain`, primo-vince come parent_private_rebind
> — con Closure::bind qualsiasi scope passa dal sito; test runBare-pattern);
> static: keyed su `start`, hit DOPO il blocco builtin enum, fill = solo
> public (niente rebind su quel path). **(3) PropIc su PropIncDec**: gemello
> RMW del hit PropSet (fill solo classi plain_set_props ⇒ readonly/asym/typed/
> hook vacui sul hit; slot Undef/assente → fall-through); ⭐ audit:
> `Op::PropOpSet` è CODICE MORTO (i compound prop assign abbassano a
> PropGet+PropSet già IC-ati — annotato). **Esito misurato**: microbench
> call-heavy −4,4% user; full-suite master-CPU 15:27→**15:12** (−1,6%,
> run21 = run20 per nome); media group phpr 82,4→80,7s (−2%; il rapporto
> 3,8× sale solo perché l'oracle OGGI gira −9%: 21,0 vs 23,0 — rumore
> giornaliero, confrontare i rapporti con cautela). Riprofilo
> (`wp30-harness/ab-out/new-wp30.sample`): **resolve_method_runtime SPARITO
> dalla top-30** (era 4° con 117), resolve_prop_access 9%→3% del run_loop,
> from_elem/finish_grow spariti; recycle_frame costa ~2%. gate22 TUTTO verde
> (⚠️ /private/tmp/wp11-gates PERSO vendor/ per pulizia di /tmp: ri-estratti
> i tarball wp9-harness/gates/ e ri-runnati ORM 3E/13F + hk 0E/0F identici).
> cargo **1592** (+19 test: frame_pool_×5, method_ic_×6, static_ic_×4,
> prop_incdec_ic_×4).

# (storia WP-29)

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
- Gate22 WP-30 verde (wp22-harness/gate-out): corpus **1455** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F identico per nome · hk 1665 0E/0F ·
  cargo **1592**/0 · probe gd/mysqli/media byte-id · http battery DIFF-set = 16
  (WP-14) · option 413 e restapi 3514 identici per nome. ⚠️ i work-tree
  ORM/hk in /private/tmp/wp11-gates possono sparire (pulizia /tmp): se
  "Could not open input file: vendor/bin/phpunit", ri-estrarre i tarball da
  wp9-harness/gates/ e ri-runnare.
- **Full-suite single-site run21: IDENTICA a run20/run19/run17/run16 per nome
  (30.481 test, 0E/2F/86W/73S) = minimo teorico**, master-CPU 15:12.
  Archiviata in `wp16-harness/full-out/run21/`.
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
2. **Perf, prossima leva (dal profilo POST-WP-30,
   `wp30-harness/ab-out/new-wp30.sample`)**: frame arena + MethodIc +
   PropIncDec IC hanno chiuso il filone dispatch/call-alloc (WP-30: micro
   −4,4%, full −1,6%; resolve_method_runtime sparito dal profilo). I colli
   residui phpr-only, normalizzati sul run_loop: **run_loop stesso** (il
   collo dominante — opcode specialization/quickening, op-clone per
   iterazione), **memmove + drop/clone Zval** (value churn → la mossa grossa
   resta la value-representation), gc_note 231 + gc_sweep 156 (bookkeeping),
   memcmp 245 (confronti chiavi/stringhe), slot_of 166 RESIDUO (i
   field-walker di vm/arrays.rs — il punto A2.5 del piano WP-29 coperto solo
   in parte: ResolvedProp con slot nei walker), hashbrown get 241,
   PhpArray::insert 105, deref_object 108, is_instance_of+iface ~90 (cache
   per (class,target)), enter_callee/bind_params ~316 (lavoro VERO di
   binding/coercion — riducibile solo con call-site specialization),
   mi_malloc/free ~180 (STRETCH 1d: pooling del Vec args di pop_keys,
   valutato e rinviato — invasivo per ~1%).

2-bis. **Valutazione suggerimenti Gemini (`20260721-gemini.md`, verificati
   sul codice il 2026-07-21)**:
   - **✅ Punto 1 (op-clone nel run_loop) — VALIDO, è la prossima leva
     consigliata.** `run.rs:92` clona l'op a ogni istruzione. Correzione al
     meccanismo proposto: NON serve alcun `Rc::clone` del func —
     `Frame.func` è GIÀ `&'m Func` (Copy): `let func = self.frames[top].func;
     let op = &func.ops[ip];` dà un `&'m Op` che NON borrowa `self` (il
     lifetime 'm del modulo sopravvive a tutto), quindi il match può girare
     per reference mentre i handler mutano i frame. Costo del refactor:
     ~tutte le arm del match legano i payload BY VALUE (muovono dal clone) →
     vanno riscritte a reference con clone SOLO nei punti d'uso che
     possiedono davvero (name.clone() dove serve un owned). Grande ma
     meccanico; il payload Rc (WP-22) e le IC Rc-condivise (WP-29/30)
     restano corretti (una cella raggiunta per reference è la stessa cella).
     Nota: le lezioni "il dispatch CLONA l'op" diventano storiche dopo
     questo cambio — aggiornare i commenti di PropIc/MethodIc.
   - **⚠️ Punto 2 (value churn / size Zval) — GIÀ NOTO, in parte superato.**
     `size_of::<Zval>()` è GIÀ 16 B (static assert in array.rs, WP-27);
     "misurarlo" non serve. Il NaN-boxing a 8 B = la leva
     value-representation già in roadmap (grossa: tagga puntatori Rc,
     tocca tutto). Il "passare &Zval nelle utility" è generico: i clone
     restanti sono in gran parte semantica PHP (deref_clone, read_slot).
   - **🟡 Punto 3 (string interning / Symbol u32) — CANDIDATO valido ma
     medio-termine.** Da valutare DOPO op-clone e quickening: i memcmp 245
     includono confronti PHP-level legittimi (chiavi array runtime) che
     l'interning delle stringhe STATICHE non elimina; i nomi
     metodo/proprietà sono già hash-cached (zhash WP-29) e i siti caldi
     sono già IC-ati (WP-29/30). ROI ridotto rispetto a quando fu scritto.
   - **❌ Punto 4 (fast-path scalari in gc_note) — GIÀ IMPLEMENTATO.**
     gc_note è un match sull'enum: scalari cadono in `_ => {}` (no-op
     immediato, mod.rs:1705); Array scalar-only già saltati via
     `may_hold_containers`. I 231 campioni sono lavoro VERO (hash-entry su
     gc_roots per gli Object). Niente da fare.
   - **🟡 Punto 5 (call-site specialization di bind_params su firma cachata
     nella IC) — plausibile ma speculativo/rischioso**: enter_callee+
     bind_params (~316) contengono lavoro non skippabile (i move degli
     argomenti); la coercion loop gira già solo se
     `param_hints.iter().any(is_some)` — micro-idea concreta e SICURA:
     precomputare `has_hints: bool` su Func a compile-time invece dello
     scan per-call. La specializzazione completa richiede guardie di firma
     con parità delicata (coercion/TypeError order) — solo dopo le leve
     sopra.
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
| WP-30 | 80,7/21,0 = **3,8×** ⚠️ | 4,80/0,40GB = **12,1×** | 15:12/5:39 = **2,7×** | ~20/11,5 min = **1,7×** |

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).

## Lezioni operative (nuove WP-30)
- ⭐⭐ **Una method-IC keyed solo sulla classe receiver è UNSOUND anche per
  vincitori public** se un antenato dichiara un `private` omonimo: il
  parent_private_rebind dipende dallo scope chiamante e `Closure::bind`
  porta qualsiasi scope su qualsiasi sito. Fill = public + scan
  `private_shadow_in_chain` (freddo, a fill-time).
- ⭐⭐ **Riciclare buffer di frame è sicuro solo se l'ordine dei decrementi Rc
  resta bit-identico al Drop derivato** (slots.clear() → stack.clear() →
  drop(resto), stesso ordine di dichiarazione dei campi) e solo nei siti di
  DROP post-gc_note_frame — mai nei siti che MUOVONO il frame (park
  generatori/fiber, retired_main).
- ⭐ I by-ref args aliasano via `Rc<RefCell<Zval>>`, MAI dentro il buffer
  slots ⇒ il riciclo del backing store non può creare dangling.
- ⭐ Prima di aggiungere una IC a un op, verificare che l'op sia EMESSO:
  `Op::PropOpSet` era codice morto (compound → PropGet+PropSet).
- ⭐ /private/tmp può perdere i work-tree dei gate (vendor/ sparito a metà
  gate22): l'errore "Could not open input file" NON è una regressione —
  ri-estrarre i tarball wp9-harness/gates/ e ri-runnare solo ORM/hk.
- Il rapporto oracle↔phpr del media group balla col giorno (oracle ±9%):
  per le OTTIMIZZAZIONI fidarsi di A/B interleaved new/old e full-suite;
  la tabella gap va letta coi rapporti MA annotando gli assoluti.

## Lezioni operative (WP-29)
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
