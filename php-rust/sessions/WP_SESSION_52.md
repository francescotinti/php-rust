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

## Ob.2 — Fase 1.3 cold-box Object (da eseguire dopo Ob.1)

Pattern WP-32 (`Option<Box<RareObj>>`) su `readonly_init`,
`readonly_clone_writable`, `typed_unset` (+ eventuale `dyn_entries`):
~96B×istanze. Predizione dal canale census PRIMA della leva.

## Stato

- [ ] Ob.1 implementazione in-node marks
- [ ] Gate classe GC (corpus per nome + cargo + full BYTE-ID)
- [ ] Census mechanism-check (conteggi + classify_ms)
- [ ] Ob.2 cold-box Object
- [ ] Giudici notte: full new vs old phpr-wp51b + A/B media 6 round
