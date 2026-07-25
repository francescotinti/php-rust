# WP_SESSION_52 — (IN CORSO) residuo classify: in-node marks + Fase 1.3 cold-box Object

## Ob.1 — QUOTA della leva in-node marks (registrata PRIMA di scrivere codice)

**Fatti misurati (census-B WP-51, binario phpr-memgc51, fuso):**
- classify_ms master = 109.383ms su 303 round (146 round "big" = 105,3s);
  152 call, roots Σ 9,72M, ctr Σ 7,97M, live_roots Σ 1,215M, freed Σ 14,235M.
- Costo per hash-op/arco calibrato dall'A/B WP-51: la fusione (3→2 hash/arco
  + rimozione rehash-cascade) = −33s misurati, con il rehash ≈ metà del
  guadagno (lezione WP-51) ⇒ **1 hash-op/arco ≈ 16-17s di full run**.
- Sanity primi-principi: walk big ≈ ~1M nodi × fanout 2-4 ⇒ ~3M archi/round
  × 146 round ≈ 400-500M archi; ×2 passate ≈ 0,9-1G hash-op; a 15-25ns
  effettivi (FxHash + probe su mappa multi-MB) ≈ 15-25s — coerente.

**Design (epoch+slot, zero hash su Obj/Arr/Clo):**
- Mark in-node da 8B = `{walk_epoch: Cell<u32>, walk_slot: Cell<u32>}`:
  epoch-based ⇒ nessun reset walk-to-walk (stale = epoch≠corrente);
  lo stato mutabile del walk (handle, in_edges, live) vive in una
  `Vec<NodeRec>` walk-local indicizzata da `walk_slot` (contigua, niente
  mappa). Worklist pass-1 = cursore sulla Vec (stesso ordine FIFO di
  scoperta di oggi). Epoch della Vm incrementato a ogni gc_classify.
- Sedi: Object → dentro `GcMark` esistente (8→16B); PhpArray → campo nuovo
  (+8B, Clone COW = mark fresco); Closure → campo nuovo (+8B, ~6 siti).
- **Ref = ibrido**: `Zval::Ref(Rc<RefCell<Zval>>)` non ha una struct dove
  mettere il mark — cambiare la variante è superficie enorme (fuori quota).
  I nodi Ref restano su una FxHashMap piccola (solo-Ref). Quota invariata:
  gli archi dominanti sono arr/obj.
- ⚠️ ordine dell'external-check: da iterazione HashMap (random) a ordine di
  scoperta (deterministico). Il live-SET risultante è order-independent;
  l'ordine dei whites arr/clo può cambiare (obj restano sortati) — parità
  sorvegliata dal gate per nome + full BYTE-ID.

**Predizioni (da verificare coi CONTEGGI census, mai i ms):**
1. Conservazione ESATTA: collects 303, calls 152, roots Σ, freed Σ 14.235.479,
   whites, reroot, live_roots tutti INVARIATI (stessa sequenza di scoperta).
2. classify_ms census: 109,4s → **~75-85s** (−2 hash-op/arco ≈ −25..−34s).
3. Full run giudice: 782,7s → **~750-760s** (−3..−4%; 2,31×→~2,21-2,24×).
4. Media: CPU dentro guardia ≤+0,5%; footprint peak **+7-8MB ≈ +0,4%**
   (+8B × arr.live_n ~770k + 8B × obj ~136k + clo; sotto guardia ≤+2%).

**Sentinelle drop-order (WP-28) pinnate PRIMA del tocco ai layout:**
i campi aggiunti sono `Cell<u32>` senza Drop ⇒ nessun cambio d'ordine di
drop strutturale; sentinelle = baseline WP-51 già verde (corpus 1421 per
nome `gate-out-wp46-archived/corpus.fails`, fail-set full run33 88 nomi,
cargo 1639/0) — ogni divergenza del gate = rottura di parità da fixare
in avanti.

## Ob.2 — Fase 1.3 cold-box Object (QUOTA registrata PRIMA dei giudici)

**Leva**: `readonly_init` + `readonly_clone_writable` + `typed_unset`
(3 Vec = 72B) → `rare: Option<Box<ObjRare>>` (8B) = **−64B/istanza** sul
comune oggetto senza feature rare.
**Deviazione dichiarata dalla lettera della roadmap**: `dyn_entries` NON
boxato — è LO storage delle proprietà di stdClass/dynamic props (WP ne fa
uso pesante): +1 deref per accesso su un fast-path = rischio CPU classe
WP-44 per −16B; si prende l'80% del canale (64 di 80B) a rischio ~zero.
**Predizioni**:
1. Peak fisico media: −64B × obj.live_n peak 140.156 ≈ **−9,0MB ≈ −0,5%**
   (census-media WP-50: max obj.live_n=140.156, max arr.live_n=769.897 —
   che quota anche il +8B/nodo di Ob.1 a +7,3MB: nette **−1..−2MB**).
2. Canale census obj: `size_of::<Object>()` −64B visibile in obj.live.
3. Guardia CPU media ≤+0,5%: i path readonly/typed-unset/clone sono
   freddi; accessors predicano su `rare` senza allocare.
`ARR_OVERHEAD` census 40→48 (il WalkMark Ob.1 entra nell'header PhpArray).

## Stato

- [x] Ob.1 implementazione in-node marks (`1a52712`)
- [x] Ob.2 cold-box Object (`ebd44f3`)
- [x] Gate albero Ob.1: corpus **1421 IDENTICO per nome** + cargo 1639/0
- [x] Cargo albero Ob.2: 1639/0; smoke readonly/clone/typed-unset BYTE-ID
      con l'oracle; gc_smoke identico
- [x] Gate corpus albero Ob.2: **1421 IDENTICO per nome** (vs set WP-51)
- [x] **Census mechanism-check (phpr-memgc52, sha 79c9ed34…) — ENTRAMBE le
      leve mechanism-backed:**
  - **classify_ms 109.383 → 62.678 = −46,7s (−42,7%)** — OLTRE la
    predizione (−25..−34s): il surplus viene dal `get` per-nodo dei pop di
    pass-2, dall'iterazione contigua (Vec vs mappa) di external-check e
    whites, e dall'azzeramento del costo-mappa residuo. Il calo è NEL
    contatore bersaglio.
  - Conservazione: collects **303=303**, calls **152=152**, freed
    14.235.463 vs 14.235.479 (Δ−16, 10⁻⁶), roots 9.718.875 vs 9.718.980,
    notes Δ−2,5k su 4,109G — stessa banda di micro-rumore del check WP-51.
  - Fail-set census **BYTE-ID a run33 (88 nomi)**; 30.472 test 2F/86W/73S.
  - Ob.2 alla cifra: `obj.live_n` exit IDENTICO 525.258=525.258;
    `obj.live` −29,41MB = **−56,0B/oggetto** (predetto −64B = 87,5%,
    ≥70% ✓; il residuo = i Box ObjRare di chi usa le feature).
    `arr.live` +14,0MB = il +8B/nodo Ob.1 sugli array vivi al teardown.
    **Netto exit −15,4MB.** obj.cum −2,02GB di churn (36,13M alloc).
- [x] **Full giudice stesso-giorno (coppia run40/run40-old)**: new
      **725,1s** (ultimo sample 12:05.10, morte +19s) vs old phpr-wp51b
      **787,4s** (13:07.36, morte +10s) = **−7,9% raw / ~−6,7% corretto**
      — mechanism-backed (classify −46,7s ≈ −5,9% + churn obj −2,02GB del
      cold-box). Entrambi fail-set **BYTE-ID a run33 (88 nomi)**,
      30.472 test 2F/86W/73S. Old di giorno 787s vs old di notte 782,7
      (run39): ambiente diurno ~+1%, la coppia stesso-giorno è il giudice.
      ⚠️ RSS telemetrico (ps, accounting MADV): new max 4024 vs old 2897
      stesso-giorno; entrambi gonfi vs notte (1945/1700) — verdetto
      footprint rimandato al peak FISICO di ab52 (metrica ufficiale).
- [ ] ab52 in corso (media 6 round interleaved + 2 oracle)
