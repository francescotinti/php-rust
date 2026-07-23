# WP_SESSION_35 — archivio storico della sessione WP-35

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

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
