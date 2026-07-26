# WP_SESSION_58 — Fase 3 tranche 2 "ARENA" per le entries di PhpArray: dieta header (−16B/array) + scan-mode small hashed + block-arena; peak fisico −1,05% (−17MB), media CPU FLAT; obj live-esatto (ultimo estimatore morto)

> ⚡ **WP-58 (2026-07-26, `8e54185`→`ff033ce`)** — Tranche 2 eseguita per
> decisione utente (banda onesta nota). **Pin (a) decisivo**: misurate le
> size reali (test-pin committato) — blocco heap `Rc<RefCell<PhpArray>>` =
> 104B → bin mimalloc 112; i buffer entries/index crescono a capacità pow2
> che cadono su bin ESATTI ⇒ **rounding lato entries ≈ 0**; la componente
> "−16B Rc hdr + rounding" della banda WP-57 vive nel blocco HEADER. Un'arena
> condivisa a handle u32 che restituisca `&/&mut` è IRRAGGIUNGIBILE in safe
> Rust (RULEBOOK §0: servirebbe unsafe o riscrittura closure-based della
> superficie VM — bocciata dalla roadmap). Materializzazione safe in TRE
> sotto-leve (tutte in array.rs, sentinelle probe56 4 assi + yieldfrom
> BYTE-ID dopo ciascuna):
> **A — dieta header** (`1c563a8`): `cursor`+`holds_containers` in una
> parola u32 (bit 31 = flag, sul word FREDDO) ⇒ PhpArray 80→72B, blocco
> 104→**96B = bin esatto**: −16B REALI su ogni array (≈−5MB al picco
> canale, −87MB/run churn media).
> **C — scan-mode small hashed** (`b0f7396`): tabelle ≤8 slot SENZA
> KeyIndex (lookup = scan lineare delle entries; indice materializzato al
> 9° slot, torna via `build` in compattazione). La popolazione dominante
> (hashed 1-4 el: 21,4k standing) non costruisce mai l'indice.
> **B — block arena ownership-transfer** (`14b8c4c`): shelf thread-local
> per-classe (pow2: entries/packed 4..256, index 8..1024, depth 16,
> ritenzione <700KB) per crescita, build/rebuild indice e Drop; blocco
> POSSEDUTO dall'array (zero indirection = zero parità); Drop esplicito con
> element-drop front-to-back identico al derivato.
> **GATE58 VERDE** (13:30-13:39): corpus **1421 IDENTICO** per nome · refl
> **290 IDENTICO** · ORM **3E/13F IDENTICO** · hk **0E/0F** · cargo
> **1645/0** (2 test nuovi). **GIUDICI**: ab58 6 round interleaved →
> **peak fisico −1,05% = −17MB medi (1,619→1,602G)** — dentro la banda
> WP-57 −10..−20MB — **CPU media −0,13% FLAT** (guardia pilota ≤+2%
> superata); mechanism-check census58 ALLA CIFRA (hashed b1 −32×11.682,
> b2 −32×9.755 esatti; recon 63.435==63.435 intatta con l'arena).
> **FULL stessa-sera**: run46 (new) e run46-old (phpr-wp57) ENTRAMBE
> 0E/2F/86W/73S, **fail-set 88 nomi BYTE-ID a run33 ×2 ⇒ il fix
> yield_from `e9a1679` è VALIDATO anche sul full** (chiusa la ⚠️ WP-57).
> ⚠️ CPU full: new ≈714,4s vs old ≈696,6s raw (+2,5%; correggendo il
> troncamento asimmetrico del campionatore ≈ +1,0..+2,5%) — regressione
> SOLO-FULL (media flat), TENUTA per direttiva no-revert e verbalizzata;
> cumulato Fase 3 sul full resta ampiamente negativo (WP-56 −2,66%).
> **Ob.2 (`ff033ce`)**: live-accounting ESATTO esteso al canale obj
> (Props accounted+sync+Drop; parte fissa al choke `next_id`; rare/proxy
> walk-only documentati) — validato ALLA CIFRA (probe: 1501==1501,
> 352.208==352.208; cum_n 6500 = costruzioni esatte). **Prima misura
> esatta di sempre: obj peak 56,1MB ≈ 3,7% del fisico** (l'estimatore
> era +28% anche sull'EOR). Canali valore al picco: arr 65,6 + str 62,3 +
> obj 56,1 ≈ **12% del fisico** — la leva GRANDE resta FUORI.
> Release: **phpr-wp58 (2d7efdf8…)**; census: phpr-memgc58 (4c9b8da3…),
> phpr-memgc58b con Ob.2 (bc47f237…).

## Dettaglio leve e siti

- A: `cur_holds: u32` (HOLDS_BIT = 1<<31, CURSOR_MASK), accessor privati
  `cursor()`/`set_cursor()` saturanti; 8 siti holds aggiornati; size-pin
  `wp58_layout_size_pins` asserisce blocco==96 nei build di parità.
- C: `SCAN_MAX=8`; `KeyIndex::{build,lookup,insert_new,remove}` con ramo
  scan (slots vuoto); materializzazione in `insert_new` skip-and-add
  (robusta a entrambi gli ordini push/index); TDD
  `small_hashed_scan_mode_elides_index` + churn model test già esistente
  copre il bordo nei due sensi.
- B: `mod pool` privato in array.rs; `grow_entries/grow_packed` #[cold]
  (guard `len==capacity` prima dei push nei 4 siti); take/put in
  KeyIndex::build/rebuild/materializzazione, compact, Drop; `to_hashed`/
  `Clone` restano collect esatto (miss al pool, footprint-first).
- Ob.2: `Props.accounted` (Cell, census-only) + Clone manuale (bilancio
  proprio, mai copiato); sync in testa a `set`/`replace` (unici mutatori
  di capacità); `memcensus::obj_fixed()`; `live_estimate` ora legge LIVE
  per tutti i canali; obj.peak significativo per la prima volta.

## Misure (giudici, stessa-sera)

- ab58 (media, 6 round): old 54,91s/1,619G → new 54,84s/1,602G =
  **CPU −0,13% · peak −1,05%**; oracle 21,04s/0,394G ⇒ **media CPU 2,61×
  · footprint 4,07×**.
- full: run46 ≈714,4s (11:54,4 ultimo campione) vs run46-old ≈696,6s
  (11:36,6) ⇒ **2,11×** di giornata; fail-set ×2 byte-id a run33.
- census58/58b: arr peak census 66,43→65,58MB (−0,85MB = leva C);
  obj peak esatto 56,1MB; proxy_peak 403,7MB (ora con obj esatto).

## ⭐ Lezioni

- ⭐⭐ **Il pin (a) su size MISURATE ridisegna la leva**: la banda "arena
  −10..−20MB" sopravviveva solo spostando il bersaglio dall'entries-arena
  (rounding ≈0, irraggiungibile safe) alla dieta dell'HEADER (−16B/array
  = bin 112→96). Stessa banda consegnata (−17MB), design diverso da
  quello immaginato: quotare PRIMA sui byte veri, non sull'idea.
- ⭐⭐ **Un canale può regredire su UN solo workload**: media FLAT e full
  +1..+2,5% insieme — il giudizio di una leva churn-shaped richiede
  ENTRAMBI i giudici; il full-only delta è ora un'attribuzione aperta
  (candidati: TLS+RefCell del pool sul Drop path, scan 5-8 slot).
- ⭐ Il campionatore ps a 20s tronca ASIMMETRICAMENTE il CPU finale dei
  full (fino a ~15 CPU-s): il raw ±1,3% del delta — leggere l'istante di
  morte vs ultimo campione prima di dichiarare un delta full <3%.
- ⭐ Il pattern accounted+sync+Drop si estende a strutture con parte
  fissa senza funnel di costruzione: si alloca la costante al choke
  dell'ID e si live-accounta solo la parte variabile (Props); le parti
  senza funnel (rare/proxy) si dichiarano walk-only invece di inseguirle.
- ⭐ Sentinelle a 5 assi ri-pinnate sulla release CORRENTE prima del
  layout = 3 leve landed in un pomeriggio senza un singolo diff di
  parità (corpus/refl/ORM/hk tutti IDENTICI al primo colpo).

## Prossimo (WP-59) — proposta di rotta

1. I metri esatti dicono: canali valore ≈12% del fisico; unit 222,6MB
   (14,5%); resto ~1,1GB FUORI (allocatore/COW/frammentazione/walk-
   unreached obj 46k). Candidato Ob.1: **attribuzione di seconda
   generazione del fisico fuori-canale** (strumenti Fase 0/54) prima di
   aprire altre tranche arena — decisione utente.
2. Attribuzione della regressione full-only +1..+2,5% (op/gc census sul
   full o probe churn-shaped) — anche solo per chiudere il verbale.
3. Backlog: panic census-only run.rs:478 (trappola armata); isset via
   `__get` annidato (WP-42).
