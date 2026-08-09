# s122-istruttoria-prop.md — prop-cloni sito-per-sito (NEXT_SESSION §S-122 p.2)

Loop caldo di prop.php (dump op pin s120): `CmpJmpSC → PropGetSlotRecv(y) →
BinaryTCPropSetPop(Add,x) → PropGetSlot(x) → BinarySTDst(Add,$s) → IncDecSlotJmp`.

## Attribuzione dei 5,000 cloni/iter (census C-lite s119) — SOMMA ESATTA

| # | Sito | file:riga | Rc? |
|---|------|-----------|-----|
| 1 | `read_slot` del recv `$o` in PropGetSlotRecv (sentiero storico) | run.rs:4285 → arrays.rs:841 | **SÌ** (unico Rc) |
| 2 | `deref_clone` di `$o->y` (IC-hit PropGetSlotRecv) | run.rs:4300 | no (Long) |
| 3 | `value.clone()` in `prop_set_entry` IC-hit (store `x`) | run.rs:712 | no (Long) |
| 4 | `deref_clone` di `$o->x` (IC-hit PropGetSlot) | run.rs:4179 | no (Long) |
| 5 | `read_slot` di `$s` in BinarySTDst | run.rs:1727 → arrays.rs:841 | no (Long) |

CmpJmpSC e IncDecSlotJmp: 0 cloni (borrow/in-place, verificato).

## ⚠️ SCOPERTA A VERBALE (cambia la voce 2 della classifica)

Il census DISATTIVA il peephole fuso di PropGetSlotRecv (`run.rs:4282-4283`:
`#[cfg(any(feature="zval-census", feature="op-census"))] let fused = false;`
— verificato al sorgente). La firma «5 cloni/iter di cui 1 Rc» descrive il
SENTIERO STORICO; il binario RELEASE percorre il ramo FUSO (run.rs:4214-4281):
`deref_clone` a 4240, store per MOVE a 4271, recv per borrow ⇒ i siti #1
(clone Rc) e #3 (store clone) NON girano sul release. Residuo release stimato:
~3 copie Long/iter (#2-fuso, #4, #5) — copie di enum, NON allocazioni.

## Conseguenza per la leva

- La voce «prop-cloni 5/iter, 1 Rc» della classifica s119 SOVRASTIMA il
  bersaglio release; il margine dai soli cloni è di copie Long (pochi ns).
- Ogni candidato è ≤10 ns/iter ⇒ VINCOLATO alla banda-LAYOUT (misura in corso
  S-122); criterio di leva solo DOPO la banda, con bersaglio prop e banda-v2
  3,33 confrontata con BANDA_LAYOUT_prop.
- Candidati NOMINATI (facilità decrescente): (a) `run.rs:712` clone inutile
  quando `DISCARD=true` — morde solo il sentiero storico/multi-shape, sul
  micro release non gira: utile ma NON misurabile sul micro; (b) borrow-fast
  di `read_slot` in BinarySTDst (#5) quando lo slot non è `Ref`; (c) il gap
  prop 5,5× resta indiziato al di FUORI dei cloni (IC probe + doppio
  `borrow()` RefCell per iter) — istruttoria futura con census per-sito che
  NON spenga la fusione (contatori nel ramo fuso).
