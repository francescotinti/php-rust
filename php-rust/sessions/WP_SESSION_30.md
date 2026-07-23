# WP_SESSION_30 — archivio storico della sessione WP-30

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

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

## Lezioni operative della sessione

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

## Appendice — handoff storico post-WP-30 (record verifica direttive Gemini 20260721; eseguito in WP-31/32)

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
   sul codice il 2026-07-21)**. ⚠️ il documento è stato POI aggiornato da
   Gemini recependo questa revisione (ora 3 sezioni allineate ai verdetti:
   op-clone = bersaglio primario col meccanismo &'m corretto, has_hints =
   micro-task immediato, NaN-boxing/interning = medio termine; il punto
   gc_note è stato rimosso perché già implementato). La numerazione
   Punto 1–5 qui sotto si riferisce alla versione ORIGINALE ed è tenuta
   come record della verifica. Unica precisazione residua sul doc
   aggiornato: l'op-clone NON alloca (payload Rc dal WP-22) — copia la
   struct Op + refcount bump per istruzione; il guadagno è togliere QUELLO,
   non "allocazioni".

   **→ ✅ ESEGUITO in WP-31** (−29,8% micro, −14,3% full) **e WP-32**
   (`43fc0c4`→`f020a33`: CmpJmp + path_apply + Frame slimming; −8,7% micro,
   −4,7% media, memmove −52%, NaN-boxing BOCCIATO con motivazione — vedi
   sezione in testa). **Prossime leve (WP-33+), dal profilo post-WP-32
   (`new-wp32.sample`)**: GC inlined è ora il blocco phpr-only più grosso
   (gc_note 206 + gc_sweep 156 + gc_note_frame — rivedere il costo dello
   sweep per-statement / gc_note write-barrier); slot_of 159 nei
   field-walker (A2.5 WP-29, mai completato); enter_callee 194 +
   bind_params 109 (call-site specialization, Gemini §5, con guardie di
   firma); hashbrown get 209 (chiavi stringa runtime); drop/clone Zval
   404+323 = churn semantico residuo; SSO su PhpStr (localizzato in
   zstr.rs, taglia mi_malloc). Oppure: validazione Laravel.
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
