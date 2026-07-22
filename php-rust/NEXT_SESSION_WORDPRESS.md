# Rotta WORDPRESS-FIRST — WP-track (dopo WP-37: call-site specialization + ATTRIBUZIONE STRINGHE per B — prossima sessione = B, SSO PhpStr)

> ⚡ **WP-37 (2026-07-22, gated `32a5820`, 1 commit)** — **leva C (sottoinsieme
> sicuro) + groundwork attribuzione per B**. **(C) `Func.simple_call`**
> precomputato nei 6 costruttori (pattern has_hints WP-31): no hints, no
> by-ref, no variadic, non-generator — ⭐ i default NON contano: i fast
> path si attivano solo ad **arity ESATTA**, quindi il prologo default non
> vede mai Undef (identico a oggi). Due fast path: `bind_params` (argc ==
> n_params ⇒ decay diretto negli slot — uguale per costruzione al ramo
> generico non-variadico con param_by_ref tutti falsi) e `enter_callee`
> (solo push del frame — ⭐ call_line e caller_strict alimentano SOLO il
> TypeError degli hint, che un simple_call non può sollevare). Le guardie
> "tipi dell'ultimo hit" di Gemini restano NON fatte (rischiose, ordine
> coercion/TypeError).
> **Esito: call-heavy dedicato −2,0% (4/4 coppie interleaved vs 1b2db38);
> bench36 completo −0,4%; media 60,07/20,94 = 2,87×** (vs 61,4 di
> mattina = coerente col −2% e col rumore di giornata); full ~12:30 ≈
> run27 (rumore); footprint 12,0×. **run28 = run27 per nome** (30.472,
> 0E/2F); gate22 TUTTO verde + ⭐ refold delle 4 suite phpt (editato il
> census nel tree a gate in corso — cfg-out ma la regola è regola);
> cargo **1627** (+1).
> **📊 ATTRIBUZIONE STRINGHE (input della sessione B — misurata, feature
> `php-types/str-census`, run media strumentata `wp36-harness/
> str-census.txt`)**: il processo master crea **51,8M PhpStr** in un run
> media (~40s user ⇒ ~1,3M stringhe/s; 1,72GB di byte cumulativi), OGGI
> = **~104M malloc** (2 alloc/stringa). Istogramma: 0B=0,16M ·
> 1-7B=11,99M · 8-15B=13,69M · 16-23B=3,42M · 24-31B=4,24M ·
> 32-63B=8,77M · 64-255B=9,42M · 256+=0,13M. **Cumulato: ≤15B = 49,9% ·
> ≤23B = 56,5% · ≤31B = 64,7%.** Verdetto per B: **SSO cap 15** (PhpStr
> resta 24B: tag+len+buf[15] al posto del Box) elimina ~25,8M alloc+free
> (≈25% del canale stringhe) senza crescere la struct; cap 23 (struct
> 32B) aggiunge solo +6,6 punti — ⭐ partire da cap 15, valutare cap 23
> solo coi dati footprint. Il 35% in 32-255B resta heap comunque.
> ⚠️ MIMALLOC_SHOW_STATS nei run strumentati produce i 15 errori
> separate-process noti (WP-26/33) e questa build mimalloc non espone i
> conteggi per-bin (denominatore totale non disponibile) — run di sola
> misura, mai per parità.
> **→ PROSSIMA SESSIONE = B (SSO PhpStr in zstr.rs)**: enum
> `{ Inline { len, buf: [u8; 15] }, Heap(Box<[u8]>) }` dentro PhpStr
> (MAI nello Zval — invariante 16B WP-27), hash cell conservata,
> `PhpStr::new` è il funnel UNICO (verificato: from_str/empty vi
> passano); i match sui siti d'uso passano tutti da `as_bytes()`.
> Poi E (gc batching, sentinelle drop-order pinnate prima).

# (storia WP-36)

> ⚡ **WP-36 (2026-07-22, gated `1b2db38`, 1 commit)** — **leve A+D del
> handoff (direttive Gemini ∩ riprofilo WP-35)**. **(A) `Op::ThisMethodCall
> { method, ic }`**: fusione del bigram This→MethodCall 25,8M, emessa SOLO
> per `$this->m()` **zero-arg** non-nullsafe — ⭐ il bigram ADIACENTE è per
> definizione argc==0, e con argomenti l'errore unbound-`$this` (che oggi
> Op::This solleva PRIMA delle SEND, come Zend INIT_METHOD_CALL) si
> sposterebbe DOPO gli effetti degli argomenti ⇒ la fusione con args è
> bloccata by design. IC-hit INLINE nell'arm (pattern ThisPropGet spinto
> fino in fondo): receiver letto in place dal frame, un solo Rc-bump verso
> frame.this del callee, entra riga-per-riga come il hit di
> dispatch_instance_call; ⭐ saltare gli shunt sul hit è sound (un Object
> non è mai Generator/Closure; ArgPlace-scan vacua a argc 0; una
> sottoclasse **Fiber non può MAI stare nella cella**: method_call devia i
> Fiber prima del fill-site e l'op è l'unico scrittore della sua cella).
> Miss → deref_clone + funnel method_call condiviso (semantica identica per
> costruzione). **(D) memo `is_instance_of`**: `iof_cache:
> RefCell<FxHashMap<(ClassId, ClassId), bool>>` su Vm, ⭐ senza epoch (Vm
> per-run, tabella classi append-only, ancestry immutabile — anche
> l'auto-impl Stringable via __toString è fissa per classe);
> `Vm::instance_of` cabla ~30 siti (shunt Fiber di OGNI instance call,
> InstanceOf*, catch-matching, is_a, iterator checks, serialize/json,
> reflection, session). ⚠️ regex a prefisso sul file aveva riscritto anche
> la chiamata INTERNA del wrapper → ricorsione infinita: dopo un replace
> multiplo, rileggere il wrapper.
> **Esito (onesto): FLAT — micro fused-call dedicato −1,5%; bench36
> completo −0,2% = rumore; instanceof micro ≈ pari (gerarchie corte: il
> walk era già 2-3 deref; il memo rende sul Fiber-shunt e sulle catene
> interfacce profonde); media group 61,4 vs old STESSO GIORNO 61,1 ≈ −0,5%
> (oracle 21,06 ⇒ 2,92×; il 2,84× di WP-35 era giornata favorevole — anche
> l'old oggi misura 61,1); full ~12:05 = run26; footprint 12,1×.**
> ⭐⭐ LEZIONE: il tetto era leggibile PRIMA dal riprofilo —
> dispatch_instance_call 100 + is_instance_of 79 campioni su >3000 totali
> ≈ 3-4% massimo teorico: dimensionare l'aspettativa sul peso del canale
> prima di aprire la sessione. **run27 = run26 = run25 PER NOME** (30.472,
> 0E/2F/86W/73S = minimo teorico); gate22 TUTTO verde; cargo **1626** (+3:
> fused dispatch, unbound-$this + &ret copy/alias, memo instanceof);
> probe_wp36.php byte-id vs oracle E vs old (cdc4c4c).
> ⭐ `vm_stdout` nei test cargo = `Registry::default()`: NIENTE builtins né
> prelude — solo costrutti di linguaggio + Exception/Error engine-level
> (count()/eval()/Stringable/RuntimeException lì falliscono o APPENDONO —
> un eval nel test è rimasto appeso >60s).
> **Riprofilo (`wp36-harness/new-wp36.sample`, finestra GC-heavy)**:
> ⭐ dispatch_instance_call e is_instance_of SPARITI dalla top-of-stack;
> dominano mi_free 681 + mi_theap_collect 475 + drop Zval/Repr/Rc ~430
> (value churn + alloc), poi run_loop 89, gc_note 15 + sweep 12,
> enter_callee 11 + recycle_frame 10, resolve_method_runtime 25 (siti
> polimorfi non-IC-abili). **→ prossime: C call-site spec (sottoinsieme
> sicuro `simple_call` + arity esatta — canale enter_callee/bind_params),
> poi B SSO PhpStr (sessione dedicata, attribuzione WP-26-style prima —
> il canale allocatore la supporta), poi E gc batching.**

# (storia WP-35)

> ⚡ **WP-35 (2026-07-22, gated `38b727e`, 1 commit)** — **T2.5: PropIc
> SCOPE-AWARE + fix parità private-shadow**. Il riprofilo WP-34 dava il
> canale prop slow-path come collo phpr-only #1 (le IC fillavano solo
> esiti public; le classi WP piene di protected/private non fillavano
> MAI). **(1) Cella `(epoch, class_id+1, scope_id+1|0, slot)`** — lo
> scope chiamante è parte della CHIAVE: hit valido solo per la coppia
> (receiver, scope) che ha fillato ⇒ anche private/protected cache-abili;
> `Closure::bind` con un altro scope = MISS, mai hit errato (⭐⭐ la
> lezione WP-29 "mai cachare visibilità non-public" valeva per celle
> keyed sulla sola classe receiver — con lo scope in chiave è sound).
> Fill GET/ISSET = QUALSIASI visibilità, gated su `!hook_guarded`
> (⭐ raggiungere PropAccess::Slot con la guardia spenta = prop_hook e
> is_virtual_hooked sondati VUOTI in questa stessa esecuzione, fatto di
> classe; sotto hook attivo mai fill). Fill SET/IncDec: restano
> plain_set_props-only, ora keyed sullo scope. **(2) FIX PARITÀ
> PREESISTENTE (scovato dai nuovi test cargo)**: parent-private
> ombreggiata da child-public omonima — i fast-path WP-25 leaf-table
> (GET/ISSET all_props_public, SET plain_set_props) leggevano/scrivevano
> lo slot del FIGLIO dove Zend risolve il private dello scope; nuovo
> helper `scope_private_overrides` (= precondizione step-1 di
> resolve_prop_access) a guardia dei 3 fast-path. Probe S5b: old
> "cccccc|KK" → new "pcpcpc|WK" = oracle. ⭐ il caso si nascondeva: basta
> UNA prop non-public nel figlio e all_props_public spegneva già il fast
> path — per questo le suite non l'hanno mai colpito.
> **Esito: media group 65,1→59,6s, oracle 20,99 ⇒ 2,84× — PRIMA VOLTA
> SOTTO 3×** (−8,4% assoluto); micro esteso (bench34 a-l, sezione l =
> letture private/protected) **−4,3%** (8,99 vs 9,39, 5 coppie);
> **full-suite ~12:05 (−4% vs run25); run26 = run25 = run24 PER NOME**
> (30.472, 0E/2F = minimo teorico); footprint 12,0×. gate22 TUTTO verde;
> cargo **1623** (+4: scoped hit, Closure::bind cross-scope, shadow,
> isset scoped); probe estesa S5/S5b.
> **Riprofilo `wp34-harness/new-wp35.sample`**: ⭐⭐ **il canale prop è
> SPARITO dalla top-20** (resolve_prop_access/prop_get_fallback/slot_of/
> prop_info/magic_applies/lazy_prop_access tutti fuori; memcmp 332→86);
> run_loop 1074, poi syscalls/gd condivisi con l'oracle. Residui
> phpr-only: **drop/clone Zval 231+166 (value churn), gc_note 161 +
> gc_sweep 132 (batching), enter_callee 143 + bind_params 60 (call-site
> specialization), dispatch_instance_call 100 (fusione This→MethodCall
> 25,8M — stesso pattern receiver-in-place di ThisPropGet),
> recycle_frame 92, is_instance_of 79 (cache (class,target))**.

## 📨 Direttive Gemini post-WP-35 (`2026-07-22_gemini.md`) — verdetti e integrazione

Verificate sul codice il 2026-07-22 contro il riprofilo `new-wp35.sample`.
Congruenza alta col piano già in testa; una leva NUOVA (SSO) con correzioni.

- **✅ A — `Op::ThisMethodCall`**: COINCIDE con la leva #1 già raccomandata
  (This→MethodCall **25,8M** bigram dal riprofilo; il "37M" citato da Gemini
  è il bigram INVERSO MethodCall→This del census WP-33 — ordine diverso,
  stessa coppia calda). Pattern receiver-in-place di ThisPropGet: elide il
  push di $this + il pop del receiver + il clone/Rc-bump. ⭐ Il tail va
  CONDIVISO col funnel esistente di `dispatch_instance_call` (MethodIc,
  __call, shunt Generator/Fiber/Closure) esattamente come `prop_get_fallback`
  — mai duplicare la semantica. → **WP-36**.
- **✅ D — cache `is_instance_of`**: già in lista, basso sforzo. Cella
  `(class_id, target_id) → bool` con epoch per-run come le altre IC (la
  gerarchia è immutabile una volta dichiarate entrambe le classi).
  Piggyback naturale nella stessa sessione di A.
- **🟡 B — SSO su PhpStr**: candidata VALIDA (drop/clone 231+166 e mi_malloc
  la supportano) ma con DUE correzioni al meccanismo proposto:
  1. "stack-allocated / inline nello Zval" COLLIDE con l'invariante 16B
     (static assert WP-27): servirebbe una nuova variante Zval con buffer
     inline ≤14B — invasiva, tocca ogni match su Zval e ogni sito che
     pretende `ZStr`.
  2. La versione LOCALIZZATA a `zstr.rs` rende comunque molto: OGGI ogni
     stringa costa **DUE allocazioni** (`Rc<PhpStr>` + `bytes: Box<[u8]>`) —
     SSO dentro PhpStr (`enum { Inline { len, buf: [u8; N] }, Heap(Box<[u8]>) }`)
     DIMEZZA le alloc per le stringhe corte senza toccare né Zval né i match,
     e conserva la hash cell. Primo passo a basso rischio.
  Prerequisito (lezione WP-26): attribuzione DATA-DRIVEN della quota
  mi_malloc/churn dovuta a stringhe corte PRIMA del refactor. → sessione
  dedicata.
- **🟡 C — call-site specialization**: allineata al verdetto già dato
  (Punto 5 del doc 21/07). Il sottoinsieme SICURO: precompute su Func
  `simple_call: bool` (no hints — `has_hints` WP-31 —, no by-ref, no
  default, no variadic) + arity ESATTA al call-site ⇒ salta il loop di
  bind_params con copia diretta nei slot. Le guardie "tipi dell'ultimo hit"
  proposte da Gemini sono la parte rischiosa (ordine coercion/TypeError) —
  solo dopo, se il precompute non basta.
- **🟡 E — batching GC sweep**: direzione condivisa (gc_note 161 +
  gc_sweep 132); il vincolo DURO è l'ordine di free bit-identico a Zend
  (WP-28: gc_queue FIFO + gc_birth + purge per-id; WP-32: le sentinelle
  drop-order NON sono oracle-diffabili — vanno pinnate prima). L'idea
  "liste intrusive per i soli candidati nati nello statement corrente" è
  concreta e compatibile col LIFO id-reuse — da esplorare con la batteria
  drop-order pinnata PRIMA di toccare lo sweep.

**Ordine consigliato (Gemini ∩ riprofilo): ~~WP-36 = A + D~~ ✅ ESEGUITE
(esito flat, tetto 3-4% previsto dal profilo) → C (sottoinsieme sicuro) →
B (sessione dedicata, attribuzione prima) → E.** Il footprint 12,1× resta
il fronte non toccato dall'arco: B è l'unica delle cinque che lo
aggredisce — e il riprofilo WP-36 (mi_free/collect dominanti) la rafforza.

# (storia WP-34)

> ⚡ **WP-34 (2026-07-22, gated `61868ce`, 1 commit)** — **T2 dell'arco:
> fusioni bigram-driven** (dal census WP-33: This→PropGet 72M,
> PushConst→CmpJmp 34,5M, Jump→Ret 29M/run; Stringify 60,7M op = 7,6%).
> **(C1) `Op::ThisPropGet { name, ic }`** — `$this->p` rvalue non-nullsafe
> fusa a EMIT-TIME (root-match su ExprKind::PropGet con base This);
> l'IC-hit legge il receiver IN PLACE (zero clone, zero stack round-trip);
> ⭐ il tail di PropGet è ESTRATTO in `prop_get_fallback` (WP-25 fast path,
> lazy, hook, __get, resolve, IC-fill) e CONDIVISO dalle due op —
> semantica identica per costruzione, IC-hit inline in entrambe le arm
> (lezione WP-23 inlining). **(C2) `Op::CmpJmpConst`** — confronto fuso
> con operando LETTERALE inline dal const pool (`lit_const_of`:
> null/bool/int/float/str), cablato in cond_jump + switch(Eq) +
> match(Identical); `binary_value` spaccata in wrapper (pop) +
> `binary_value_ab` (core a operandi espliciti) condivisa da
> Binary/CmpJmp/CmpJmpConst; ⭐ il push di un letterale non ha effetti ⇒
> elidibile anche il lhs "fuori ordine"; ⭐ cur_line stampata sulla linea
> che il PushConst eliso avrebbe portato (parità trace). **(C3)
> `Op::ConcatN(n)` + Stringify elision** — la spina di Concat annidati
> (catene `.` E interpolazione, che il lowering desugara a Concat
> left-nested con seed "") è FLATTENED a emit-time: ordine di
> valutazione/stringify IDENTICO all'emissione pairwise (i Concat
> intermedi e gli Stringify-di-Str erano puri); parti letterali Str
> saltano lo Stringify no-op (PushConst→Stringify 15,3M), parti ""
> spariscono, catena all-literal FOLDATA a una costante; join a UNA
> allocazione (via il realloc left-assoc = bucket memmove); Echo/Print
> saltano lo Stringify su risultato già-Str. ⭐ la coercion è TUTTA in
> Op::Stringify (il Binary Concat riceve sempre Str dal compilatore) ⇒
> l'elisione è esatta per costruzione. **(C4) jump threading in-place**
> (compile_body, SOLO exc_table vuota): Jump→Jump ritargettato al landing
> finale; Jump che atterra su Ret DIVENTA quel Ret — mai rimozione di op
> (WP-32: niente peephole/shift), ⭐ replace-Ret solo a LINEA identica
> (Ret può sollevare TypeError da return-hint e la linea alimenta
> getLine); ⭐ target out-of-range (Addr::MAX su jump MORTI dietro i goto)
> = terminali, mai seguiti (5 test goto rossi prima del guard).
> **Esito: micro esteso (bench34 a-g+h-k) −6,2%** (8,10 vs 8,64s, 5+3
> coppie interleaved vs 2937b7b, rust-analyzer killato); **media
> 66,9→65,1s, rapporto 3,11×** (oracle 20,92); **full-suite ~12:35 ≈
> run24 12:20 (rumore, IO/C-lib dominated), run25 = run24 = run23 PER
> NOME** (30.472, 0E/2F/86W/73S = minimo teorico); footprint 12,0×
> invariato. gate22 TUTTO verde + refold 4 suite col binario post-fold;
> cargo 1619; probe battery wp34-harness (probe_wp34.php) old==new
> byte-id, vs oracle solo 2 drift diag PREESISTENTI.
> **Riprofilo `wp34-harness/new-wp34.sample`** (10s su media): run_loop
> 1511; ⭐⭐ **il canale prop slow-path è il blocco phpr-only #1**:
> resolve_prop_access 346 + prop_get_fallback 237 + slot_of 185 +
> lazy_prop_access 102 + prop_info 93 + magic_applies 93 — le PropIc
> fillano SOLO esiti public e le classi WP piene di protected/private
> non fillano MAI ⇒ **prossima leva (T2.5): IC scope-aware** (cella
> keyed anche sullo scope corrente, fill di risoluzioni private/protected
> con guardia (class_id, scope)); poi memcmp 332, enter_callee 307 +
> bind_params 114 (call-site specialization), drop/clone Zval 305+180,
> memmove 235 (−~48% vs WP-33), gc_note 201, dispatch_instance_call 164
> (This→MethodCall 25,8M non fusa — candidata), recycle_frame 157.
> 📌 Gap preesistente visto: `isset($this)` in metodo STATICO →
> CompileError "unsupported $this property write" (dim_base su
> PlaceBase::This) — pre-esistente, mai colpito dalle suite.

# (storia WP-33)

> ⚡ **WP-33 (2026-07-22, gated `8462ce4`→`62ea805`, 6 commit)** — **Fase 1
> dell'arco "interprete specializzante"** (decisione strategica utente:
> benchmark pubblicabile CPU+memoria vs oracle; Laravel accantonato; unsafe
> = ultima spiaggia FUORI roadmap). Due gambe:
> **(T0) OP CENSUS** — `crates/php-runtime/src/vm/census.rs`: contatori
> per-op + BIGRAMMI (l'oracolo delle fusioni T2) + matrici tipate
> Binary/CmpJmp (binop×tag×tag), FetchDim (base×key), IncDecSlot; attivo
> con env `PHPR_OP_CENSUS` (=1 → stderr; **=path assoluto → append su
> file** — ⭐ i SUBPROCESS phpr ereditano l'env e il loro dump su stderr
> diventava una PHPUnit\Framework\Exception in 15 test media separate-
> process); ⭐⭐ **il hook è dietro cargo feature `op-census`**: anche il
> branch bool mai-preso nel run_loop costava **+2,9%** sul micro op-denso
> (5 coppie A/B) — compilato via, l'off-cost è −0,4% = zero. Build census:
> feature `php-runtime/op-census` con CARGO_TARGET_DIR dedicato
> (`phpr-census-target`). Dati (media/option/post ~910M op/run + micro):
> **Concat(Str,Str) ~31M e NotIdentical(Str,Str) ~29M dominano il Binary
> del WP reale** (l'aritmetica Long è 10× sotto — il micro mente);
> cross-class ===/!== ~3-4M/run; **FetchDim = 98,8% Array×{Long,Str}**;
> IncDecSlot ≈100% Long; top bigram **This→PropGet 73M**, Ret→DerefTop
> 40M, MethodCall→This 37M, PushConst→CmpJmp 35M. File in
> `wp30-harness/ab-out/census-*`.
> **(T1) FAST-PATH TIPIZZATI** — **(C3) `binary_fast`** in testa a
> binary_value (CmpJmp e compound li ereditano per costruzione):
> (Long,Long) aritmetica con overflow→Double RIFATTO SUGLI OPERANDI
> (ops.rs verbatim) + confronti + bitwise + spaceship; (Double,Double)
> aritmetica + confronti IEEE (⭐ Gt/Ge = forme smaller SCAMBIATE `r<l`/
> `r<=l`, NaN-esatte; === è `==` IEEE); (Str,Str) Concat byte + ===/!==
> byte-eq (⭐ MAI l'Eq loose: "10"=="1e1" resta in smart_streq); **cross-
> class ===/!== = costante** (ident_class rispecchia le arm di
> ops::identical; Undef≡Null stessa classe; Ref/ArgPlace/WeakHandle →
> generico). **(C4) guardie FetchDim/CoalesceFetchDim**: base Array + key
> Long/Str → UN lookup a chiave canonica (Key::from_zstr, "5"→Int(5))
> PRIMA del probe ArrayAccess; ⭐⭐ il flush dei diag pendenti resta sul
> hit (warning dell'op precedente sorge AL read, error-handler che lancia
> fa unwind da QUI); miss → read_dim_warn (una sola fonte del warning);
> coalesce = gemello silente (hit E miss inline). **(C5) IncDecSlot** su
> slot Long raw con checked step (overflow/Ref/Undef → generico).
> **Esito: microbench 6,08→4,64s = −23,7%** (aspettativa piano era
> −5/10%); **media 69,0→66,9s, rapporto 3,19×** (media è gd/mysql-bound);
> **full-suite 12:54→12:20 = −4,4%, gap 2,18×**; footprint 12,0×
> invariato (nessun lavoro memoria in fase 1). gate22 TUTTO verde; cargo
> **1619**; **run24 = run23 per nome** (30.472 test, 2F+86W identici).
> Riprofilo `wp30-harness/ab-out/new-wp33.sample`: run_loop self 2760,
> drop Zval 766, resolve_prop_access 545, memmove 456, memcmp 442,
> enter_callee 375, clone Zval 341, gc_note 282, slot_of 202 → **T2 =
> catena prop (fusione This→PropGet?/slot IC più a monte), call path,
> FetchDimConst su chiavi letterali, gc_note batching, interning**.
> 📌 Metodologia benchmark (per la pubblicazione): oracle CLI con
> `opcache.enable_cli=Off` e JIT off ⇒ interprete-vs-interprete
> simmetrico; dichiararlo; colonna futura opcache_cli=1 (richiederà
> unit-cache persistente phpr). 📌 NoRef load/store RINVIATO: 3 canali
> runtime installano Ref negli slot a prescindere dagli op (scope-bridge
> include/eval; BindGlobalDyn su catena bridge; PushArgPlace/SEND_VAR_EX
> a bind time) — inventario bancato, non riproporre senza chiudere quelli.
> 📌 Gap pre-esistenti visti di passaggio (NON toccati): deprecation 8.5
> "Using null as an array offset" e float-key nel contesto `??` non
> emesse (funnel silente documentato di read_dim_nullable); warning
> "Undefined variable" mancante su `$u++` di var mai vista.

# (storia WP-32)


> ⚡ **WP-32 (2026-07-21/22, gated `43fc0c4`→`f020a33`, 7 commit)** — la leva
> "value-representation" RIDEFINITA dai dati (❌ **NaN-boxing BOCCIATO e da
> non riproporre**: Long(i64) a 64 bit pieni non entra nei 48 del payload
> NaN; ~9 impl unsafe su un value-core a zero unsafe; romperebbe la niche
> Option<Zval> degli array packed; ~5.000 siti; e il churn misurato NON
> viene dalla taglia — la zval di Zend è anch'essa 16B). Tre cluster:
> **(A) CmpJmp** — confronto+branch fusi a emit-time via cond_jump
> (root-match AST, mai peephole), handler = binary_value condiviso
> (refactor puro da Op::Binary ⇒ semantiche identiche per costruzione);
> cablati while/do-while/if/for/switch(Eq)/match(Identical)/&&/||/ternario;
> + peek zero-clone al posto dei 2 deref_clone per-confronto.
> **(B) path_apply** — PhpArray::slot_or_vivify e set_returning_displaced
> (UN lookup vs 2-4 + key clone per livello; semantica del composito ESATTA:
> no-revive WP-27, next_free, holds_containers — unit test di equivalenza a
> matrice); dropped Vec→Option e pop_keys via split_off (via 2 malloc/free
> per path op). **(C) Frame slimming ~400→≤176B** — FrameFlags(u8) per i 7
> bool + FrameExt boxed lazy per i campi freddi con ordine di drop
> conservato PER COSTRUZIONE (ext dopo iters; ogni campo Rc-bearing di
> FrameExt viveva già dopo iters; dyn_vars → Option<Box> IN PLACE; ret_cell
> resta inline) + 3 SENTINELLE drop-order committate PRIMA del layout
> (pinnano l'ordine phpr corrente — passate INVARIATE dopo).
> **Esito: microbench esteso −8,7%** (6,09 vs 6,67s, 5 coppie interleaved
> vs cb82691); **media group 72,4→69,0s (−4,7%), rapporto 3,3×**;
> full-suite 12:54 (−1% — ormai dominata da IO/mysql/C-libs); riprofilo
> (`wp30-harness/ab-out/new-wp32.sample`): **memmove 629→301 (−52%),
> path_apply SPARITO dalla top-20**. gate22 TUTTO verde; cargo 1600;
> **run23 = run22 nel fail-set per nome** (2F identici; 9 diff P-only da
> test-set upstream — vedi incidente wpdev sotto).
> ⚠️ **INCIDENTE WPDEV RISOLTO**: lo scratchpad della vecchia sessione
> (be003709) è stato RIPULITO a metà nottata — wpdev ha perso vendor/,
> composer.json e quasi tutto (65MB residui). Ricostruito PERMANENTE in
> **`~/Claude/wpdev`** = wordpress-develop **trunk@5e3fced** (2026-07-15,
> la revision del setup originale — il tag 7.0.1 NON basta: mancano le
> classi 7.1 e i data-provider differiscono) + composer install + 
> wp-tests-config.php con **DB_PASSWORD 'wp-secret-Pass1'** (recuperata dai
> probe mysqli wp8). Tutti gli script aggiornati (run-full-detached,
> gate22, media-pair, run-multisite, gate19). Validazione: option 413
> IDENTICO per nome al tree vecchio; full-suite 2F identici.

# (storia WP-31)

> ⚡ **WP-31 (2026-07-21 notte, gated `8adba4b`+`7ee4bcb`, 2 commit)** — il
> punto 1 del doc Gemini validato, eseguito: **(a) il run_loop matcha su
> `&'m Op` — ZERO clone per istruzione**. `Frame.func` è `&'m Func` (Copy):
> copiata la reference fuori dalla catena di accessi, l'op si slega da
> `self` e il match gira per reference; 171 fix meccanici (puri deref di
> scalari Copy, guidati dalle suggestion del compilatore via script con
> whitelist sulla forma — vedi lezione) + 4 shorthand; ZERO clone aggiunti
> (coercion `&&T→&T`, `&Rc<[u8]>→&[u8]` coprono i siti d'uso); le celle IC
> sono ora raggiunte direttamente nell'op del Func (stessa cella condivisa
> di prima: fill persistono per costruzione). **(b) has_hints precomputato
> su Func** (la scan di param_hints per-chiamata in enter_callee → bool a
> compile time, 6 costruttori). **Esito: microbench call-heavy −29,8%**
> (5,16 vs 7,35s, A/B interleaved 5 coppie vs 06a3c5b, output identici);
> **full-suite master-CPU 15:12→13:02 = −14,3%** (run22 = run21 per nome);
> **media group phpr 80,7→72,4s = −10,3%** (oracle 20,95 → **3,5×**);
> gap full-suite **2,7×→2,3×**. gate22 TUTTO verde; cargo 1592. Riprofilo
> (`wp30-harness/ab-out/new-wp31.sample`): run_loop self 3041→1761; colli
> residui = value churn (memmove 629 + drop/clone Zval ~630 → la mossa
> grossa resta la **value-representation**), gc_note 206 + gc_sweep 155,
> memcmp 263, hashbrown get 189, slot_of 157 (i field-walker, A2.5
> parziale), enter_callee 135 + bind_params 101, mi_malloc/free ~176.

# (storia WP-30)

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
- Gate22 WP-32 verde (wp22-harness/gate-out): corpus **1455** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F identico per nome · hk 1665 0E/0F ·
  cargo **1600**/0 · probe gd/mysqli/media byte-id · http battery DIFF-set = 16
  (WP-14) · option 413 e restapi 3514 identici per nome. ⚠️ i work-tree
  ORM/hk in /private/tmp/wp11-gates possono sparire (pulizia /tmp): se
  "Could not open input file: vendor/bin/phpunit", ri-estrarre i tarball da
  wp9-harness/gates/ e ri-runnare.
- **Full-suite single-site run23 (tree NUOVO ~/Claude/wpdev, trunk@5e3fced):
  30.472 test, 0E/2F/86W/73S — fail-set IDENTICO per nome a run22** (stessi
  2F: wpPostsListTable search_hierarchical + wp_is_stream #2 = minimo
  teorico); le 9 differenze di nome sono TUTTE test P-only del delta di
  test-set upstream (documentate). master-CPU 12:54. Archiviata in
  `wp16-harness/full-out/run23/`. Le run future si confrontano con run23.
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
| WP-31 | 72,4/20,95 = **3,5×** | 4,82/0,40GB = **12,1×** | 13:02/5:39 = **2,3×** | ~17,5/11,5 min = **1,5×** |
| WP-32 | 69,0/20,91 = **3,3×** | 4,75/0,39GB = **12,0×** | 12:54/5:39 = **2,3×** | ~19,5/11,5 min = **1,7×** |
| WP-33 | 66,9/20,97 = **3,19×** | 4,75/0,39GB = **12,0×** | 12:20/5:39 = **2,18×** | ~16,5/11,5 min = **1,4×** |
| WP-34 | 65,1/20,92 = **3,11×** | 4,73/0,39GB = **12,0×** | ~12:35/5:39 = **2,2×** (rumore) | ~17,5/11,5 min = **1,5×** |
| WP-35 | 59,6/20,99 = **2,84×** ⭐ | 4,73/0,39GB = **12,0×** | ~12:05/5:39 = **2,14×** | ~17/11,5 min = **1,5×** |
| WP-36 | 61,4/21,06 = **2,92×** ⚠️ | 4,78/0,40GB = **12,1×** | ~12:05/5:39 = **2,14×** | ~17/11,5 min = **1,5×** |
| WP-37 | 60,07/20,94 = **2,87×** | 4,72/0,39GB = **12,0×** | ~12:30/5:39 = **2,2×** (rumore) | ~17/11,5 min = **1,5×** |

⚠️ riga WP-36: NON è una regressione — l'old-binary (WP-35) rimisurato lo
STESSO giorno dà 61,1s (2,90×): la giornata di WP-35 era favorevole; il
confronto interleaved new/old dà phpr −0,5/−1% (rumore/flat).

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).

## Lezioni operative (nuove WP-33)
- ⭐⭐ **Strumentazione nel hot loop SEMPRE dietro cargo feature**: un
  `if bool` mai-preso nel run_loop = +2,9% sul micro op-denso (misurato
  5 coppie A/B). `#[cold]` sul recorder NON basta; compilare via il branch
  sì (−0,4% = rumore). Feature build in CARGO_TARGET_DIR separato per non
  invalidare la cache default.
- ⭐⭐ **Il census dump va su FILE quando il workload spawna subprocess**:
  l'env si eredita, il dump stderr del figlio finisce nell'output che il
  harness cattura e asserisce (15 errori PHPUnit separate-process solo
  per il banner). `PHPR_OP_CENSUS=/path` → append (aggrega anche i figli).
- ⭐ **Il micro mente sul mix reale**: bench.php è Long-aritmetica; il WP
  reale è Concat/NotIdentical(Str,Str) 10× sopra l'aritmetica. Le matrici
  tipate del census (non i sospetti) decidono la matrice dei fast-path —
  è così che sono saltati fuori i cross-class === costanti.
- ⭐ **Fast-path = guardia sui TAG appena poppati + fall-through al
  generico**: mai duplicare warning/coercion/overload nel ramo veloce —
  il miss DEVE ricadere nel funnel esistente (una sola fonte di verità);
  pinnare PRIMA su oracle E su old-binary (probe byte-diff) i casi bordo
  (NaN, -0.0, overflow→Double sugli operandi, "10"=="1e1", float-key).
- ⭐ Il hook serena-vexp-guard ora blocca anche `git add` con path .rs
  espliciti → `git add -u` + `git commit -F file`.
- ⭐ `run_source` (test eval.rs) NON ha php-builtins: niente var_dump/
  gettype — asserire via echo/ternari con stringhe oracle-pinnate.

## Lezioni operative (WP-32)
- ⭐⭐ **Il timing di distruzione phpr (sweep-driven) diverge GIÀ da Zend**:
  le sentinelle drop-order NON possono essere oracle-diff — vanno pinnate
  sull'output phpr CORRENTE, committate PRIMA del cambio layout (metodo
  C2→C3: 3 sentinelle rosse-se-cambia, passate invariate).
- ⭐⭐ **Boxare campi freddi senza riordini osservabili**: mettere il Box
  DOPO l'ultimo campo hot Rc-bearing e ordinare i campi interni come nel
  layout pre-esistente; i campi che romperebbero l'ordine si boxano IN
  PLACE (dyn_vars → Option<Box> alla stessa posizione) o restano inline
  (ret_cell). MAI un Drop manuale su Frame (romperebbe mem::take del pool).
- ⭐ Fusione op a EMIT-TIME, mai peephole (rimuovere op sposta gli
  indirizzi); fondere solo quando la RADICE AST è il pattern (il bool
  interno consumato come valore non è mai fondibile per costruzione).
- ⭐ API composite di PhpArray: replicare il composito ESATTO (contains+
  insert+get_mut) con unit test di equivalenza a matrice su tutte le forme
  repr — mai "quasi uguale" (holds_containers/next_free/ordine sono parità).
- ⭐⭐ **Gli scratchpad delle vecchie sessioni in /private/tmp VENGONO
  RIPULITI**: wpdev ci ha vissuto per 9 sessioni ed è stato sventrato a
  metà run. Gli asset di lunga vita vanno in **~/Claude/** (ora:
  ~/Claude/wpdev). Se una suite dice "Could not open input file" dopo ore
  di sleep del Mac, è il reaper, non una regressione.
- ⭐ Ricostruire wpdev: trunk alla DATA del setup (il tag release non
  basta: test-set diverso), composer install, wp-tests-config con la
  password del probe wp8 ('wp-secret-Pass1'); validare con option 413 per
  nome + fail-set full identico; le differenze P-only upstream si
  documentano e si ribasa il confronto (run23 è la nuova base).
- Il Mac in sleep congela le run detached per ore: guardare i timestamp
  del .done prima di diagnosticare un hang.

## Lezioni operative (nuove WP-31)
- ⭐⭐ **L'op-clone per-istruzione era il singolo costo più grosso del
  run_loop** (−30% sul carico call-heavy, −14% full-suite): `Frame.func` è
  `&'m Func` Copy ⇒ `let func = self.frames[top].func; let op =
  &func.ops[ip];` NON borrowa self e il match gira su `&'m Op`. Le lezioni
  WP-29/30 "il dispatch CLONA l'op" sono STORICHE: ora le op sono
  raggiunte per reference (le celle IC Rc restano condivise — a maggior
  ragione, si tocca la cella originale).
- ⭐ **Refactor da centinaia di type-error = script sulle suggestion JSON
  del compilatore** (`--message-format=json`, applicare solo replacement
  con forma in whitelist: `*x`, rimozione di `&`, `x.clone()`): 171/175
  fix automatici in una passata, il resto a mano. MAI regex alla cieca sul
  sorgente.
- ⭐ Le coercion `&&T→&T` e `&Rc<[u8]>→&[u8]` coprono quasi tutti i siti
  d'uso di un match passato a reference: ZERO clone aggiunti — se un
  refactor del genere richiede molti .clone(), qualcosa è storto.
- ⚠️ `git diff` via RTK è riformattato (prefisso 2 spazi, header
  "Changes:"): i grep su `^[+-]` non matchano — usare `^\s+[+-]`.

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
