# WP_SESSION_40 — archivio storico della sessione WP-40

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-40 (2026-07-23, gated `4c8de21`+`2f00d36`, 2 commit)** — **demote
> churn GC chiuso: marks in-object al posto delle strutture hash per-id =
> media −2,5% (2,68×), full −2,4%.** Eseguito il piano del handoff WP-39 in
> due step, entrambi A/B-misurati lo stesso giorno:
> **(1) `GcMark` su Object + buffer unico** (`4c8de21`): la tripla
> `gc_roots: HashMap<u32,Rc>` + `gc_queue: VecDeque` + `gc_birth: HashSet`
> sostituita da `gc_buf: Vec<Option<Rc>>` + cursore `gc_buf_head` sul Vm e
> marks nell'oggetto (`php_types::GcMark`: `pos: Cell<u32>` con MAX=assente
> + bitfield). Dedup = lettura Cell (vacant-entry semantics conservate: una
> release note non tocca il BIRTH di un entry esistente); ⭐ rimozione a
> metà buffer O(1) via slot `None` con indici stabili — il clone droppa
> NELLO STESSO ISTANTE di prima in tutti e 4 i siti (demote, unbuffer del
> candidato, consume del birth-seed nella cascata WP-28, unbuffer nel
> free-loop di collect); ⭐ `gc_buf_head` è un campo Vm, non un locale: il
> break di scheduling-destructor riprende dove era rimasto; fast-path
> empty-sweep WP-39 rimappato sul cursore; verify-mode su flag in-object
> (mismatch ⇒ il candidato si riparcheggia nel suo slot = la vecchia map).
> Esito A/B: −0,76% — il probe hashmap era solo parte del canale.
> **(2) flag-guard** (`2f00d36`): i set per-id RESTANO AUTORITATIVI, GcMark
> porta tre bit specchio esatti per gli oggetti vivi — DESTRUCTED
> sostituisce il probe hashset in gc_note (177M/run; set+flag aggiornati
> nei 4 insert e nel remove di host_reflect, path-throw incluso; unico sito
> di lettura del flag), CYCLE_ROOT/LIGHT_DEMOTED guardano gli insert
> ridondanti del demote (95% dei 47,5M ri-demota membri esistenti). ⭐⭐ La
> regola d'oro dei mirror: il flag va azzerato a OGNI svuotamento del set
> (re-seed `mem::take` per tutti i vivi anche se già bufferizzati, roots
> drain del collect via `created`, `gc_release_child`; id-reuse = flag
> morto con l'oggetto) — un flag true su set vuoto = under-insert =
> destructor perso. gc_note ora a UN borrow (insert inline).
> **Esito complessivo: old stesso-giorno 57,52 → new 56,05 user = −2,5%
> (sys −6%); oracle 20,95 ⇒ 2,68× (old = 2,75×); full run31 master-CPU
> ~11:39 (−2,4% vs 11:56)** — al tetto alto della forchetta 1,5-2,5%
> stimata dal canale (lezione WP-36 applicata). ⭐⭐ gc-census come PROVA DI
> PARITÀ del refactor: inserted/freed/demoted/collect/dtor IDENTICI alla
> baseline WP-39 in entrambi gli step (47.017.314 / 2.085.632 / 47.468.514
> / 1 / 1001; le note variano di ±62 su 177M = nondeterminismo del
> workload) — i conteggi di ingresso in sweep_impl NON sono confrontabili
> col census WP-39 (quello era pre-fast-path). **Gate/run**: sentinelle
> drop-order 9ed457b verdi PRIMA e DOPO ogni step; probe battery
> (dtor39/sent_engine/sent_builtin/probe_wp39) old==new byte-id; gate22
> TUTTO verde per nome (corpus 1447/sess 28/date 351/refl 290 IDENTICI,
> ORM 3E/13F, hk 0E/0F, gd/mysqli/media BYTE-ID, http 16 DIFF attesi,
> option/restapi identici); **run31 = run30 PER NOME** (30.472,
> 0E/2F/86W/73S, 88 righe fail identiche); cargo **1636** invariato.
> **Riprofilo (`wp40-harness/gc-out/new-wp40.sample`, stessa finestra
> t=35s)**: gc_sweep_impl **38→19**, il probe hashmap sparito; gc_note
> residuo **86** = il WALK stesso (borrow + match + discesa
> Ref/Array/Closure a strong_count==1) — ridurre il VOLUME delle note è il
> prossimo bersaglio del canale gc; dominano di nuovo drop/clone Zval
> (132+116) + memmove 108, poi resolve_prop_access 36.
> **→ PROSSIMA SESSIONE**: (a) canale drop/clone Zval + memmove (il collo
> #1 costante da WP-36 — servirà attribuzione per-chiamante prima, metodo
> WP-26/39), oppure (b) arco bytecode-a-registri (unica "leva lunga"
> approvata, cfr. verdetti Gemini post-WP-38 — census WP-33 alla mano per
> il tetto), oppure (c) volume gc_note (177M chiamate: elidere note
> provabilmente ridondanti per costruzione — es. slot già Undef, scalari —
> SEMPRE con census di parità prima/dopo). Il footprint (12,0×) resta il
> fronte non aggredito; Object +8B/istanza (GcMark) = trascurabile
> (2,5M oggetti ⇒ ~20MB teorici sul picco multi-GB).
