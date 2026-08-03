# design91 — delibere S-91.0 p7 (Concilio WP-92): design v9 residui + status delibere ereditate

Ordine Concilio WP-92 §Sintesi p7 + residui p5/p6 dichiarati per NOME.
NESSUNA riga di runtime cambia con questo documento; l'attuazione di
ogni item è la prima sessione che ne consuma l'esito.

## Canale iterazione 3 — residui p5 per NOME (dopo A-PP-63 ATTUATO)

A-PP-63 è VIVO sul canale (riga `tag=mi_quiesce ... outstanding=N` sui
checkpoint, smoke-morso in S-91.0; binario mem-census nuovo). Restano,
nell'ordine di potere del team-misura:

- **A-DL-52 census ON-THREAD alla barriera (P1 team-misura, potere
  massimo)**: snapshot per-worker eseguito SUL thread del worker alla
  barriera di picco (`mi_heap_get_default()+mi_heap_visit_blocks`
  chiamate dal worker stesso, snapshot su canale); due thr con lo
  stesso heap ptr ⇒ REFUSE (A-DL31 resta il dente, KS-DL-92-1). La
  barriera È il testimone (si compone con A-PP-63). Fallback in ordine:
  heap espliciti per worker (`mi_heap_new` + allocazioni VM
  heap-scoped), poi merge stats tld at-exit (patch al crate).
- **A-DL-53**: lettura read-only purged/purge_calls a t0/t1 dello span
  sul worker path (chiude A-DL50).
- **A-DL-54**: braccio overlap FORZATO (doppia richiesta a barriera) —
  il negativo controllato della legge clamp⇔overlap 10/10 (A-BG61,
  dichiarata in MEASURE90 emendato; KS-DL-92-2: claim sul clamp senza
  riga overlap-status ⇒ VOID).
- **A-DL-55**: current (non total) di malloc_huge + conteggio pagine
  abandoned + classi per-chunk in mi_arena_chunk_bins (chiude il
  bilancio di coverage: l'arena si cammina senza TLS).
- **A-DL-56**: braccio bare-thread (W thread che toccano mimalloc, zero
  init phpr) — scompone b_boot in thread-runtime + stato phpr (b_boot
  resta *quanto* senza *di che cosa* finché non gira).
- **A-PP-60/61**: assert_server_gone con escalation via ensure_gone +
  server_gone= ledgerato prima di fail; head_unmoved instradato su fail
  (vietato l'exit muto) — nel measure91-campaign alla nascita.
- **A-PP-62**: `-f` su TUTTI i curl census + pid-echo x-phpr-pid
  asserito almeno su pwork e self. **A-PP-64**: `LC_ALL=C` nel prologo.
- **Grammatica campaign v2** (design91-ledger.md): A-AH60 campaign_sha
  sulle verdict, A-BG58 reason=requalify autosufficiente, A-PP-65 riga
  phase=authorize per ogni cambio judge_sha post-start (KS-PP-92-2) —
  NESSUNA campagna m91 può partire con la grammatica v1.
- **Promozione b_work a verdict-grade** = tre condizioni CONGIUNTE
  (team-misura): A-PP-63 ✓ attuato · stimatore non-tautologico ✓
  (repair90) · census per-worker senza collisione (A-DL-52, aperto).

## Roadmap A-DS51 — fasi 1-3 (fase 0 CHIUSA in S-91.0)

Fase 0 ✓: fixture v3 (A-DS53, 8 oracle-morse; vincolo di modellistica
NUOVO da w5: il messaggio hook è `C::$x::get(): string must be
compatible with P::$x::get(): int`) + pin v3 length-prefixed (A-DS54)
committati. Restano, ordine VINCOLANTE (A-DS57):
1. **Fase 1**: checker LSP PURO (nessun wiring) + unit sui messaggi —
   45 pin, solo `cargo test --release`. Vincoli di modellistica:
   `self` si stampa RISOLTO (w4: `C::m(): C`), hook come
   `Class::$prop::get()` (w5), unioni nell'ordine canonico Zend.
2. **Fase 2**: wiring nel lowering HOISTED (lower/class.rs, dopo
   flatten trait, accanto al final-check) con le ESENZIONI nello STESSO
   commit (ctor-plain-class, private, RTWC, tentative-Deprecated
   A-DS46; il ctor NON è esente per interfacce/abstract, v13/v14) →
   gate ORM 3E/13F + hk 1665 + corpus per NOME. t3 exit 0, t4 sul
   braccio PERSIST (KS-DS-92-3). **Rischio dichiarato: la fase 2 può
   solo AGGIUNGERE fatal su ORM/hk — ogni esenzione mancante è
   regressione.**
3. **Fase 3**: bind-registry per condizionali/dinamiche (t2) + gate di
   nuovo. Gate di merge: KS-DS-88-3 + KS-DS-89-3 + KS-DS-91-1..3 +
   KS-DS-92-1..3 (fixture A-DS53 per NOME nel gate o REJECT).

## A-MS-51/52+54/53/55 — probe/census engine (Matsakis WP-92, design)

- **A-MS-51**: guardia `len()==MAX_THEAPS` (non capacity) o
  `Box<[TheapAgg]>` a lunghezza fissa — il cap dichiarato deve essere
  quello che morde (KS-MS-92-1: heaps_total>MAX_THEAPS con
  heap_overflow=0 ⇒ census VOID).
- **A-MS-52+54**: tag di sito nell'abort (arm porta `&'static str`) +
  dente anti-nesting in arm() (`debug_assert!(!flag)` o contatore) —
  NELLA STESSA DELIBERA (KS-MS-92-2/3). ⚠️ Vincolo di sequenza
  RISPETTATO in S-91.0: A-TH-57 è in forma PREFISSO, il cambio firma
  non lo romperà; il pin narm va aggiornato nello stesso commit del
  cambio firma.
- **A-MS-53**: `heap=<ptr>` sulle righe mi_theap_pages (join
  anti-collisione con mi_bin_thr_sum — serve a VSELF iter-3).
- **A-MS-55**: nessun campione REQ_NS per hit `/__census_self`.

## A-MS46 / A-TH55 / A-BB60 / A-PP49 — invariati

A-MS46 (`mod probe` annidato, design90) · A-TH55 (ordine canonico KIND
intra-putord, design90) · A-BB60/A-PP49 (design89) — invariati, nessuna
sessione ne ha consumato l'esito.

## Deferred invariati

A-MS27 (rustc giudice — resta la chiusura vera del lane A-TH53, il cui
pin Cargo.toml è ATTUATO in S-91.0) · A-PP18 · A-PP27 · A-AH38 —
backlog. KS-DS-80-3 invariata (mai innescata).
