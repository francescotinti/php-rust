# S-142 — disasm A/B L-RD1 agli atti (az.rev. S-141 #2)

Fonte: binari del verdetto S-141 — A = stash `phpr-s140` (f2708b75) · B =
`phpr-rd1-target/keep/phpr-s141-rd1` (f751ef5b). Dump integrali in scratchpad
di sessione; estratti riprodotti con `objdump -d` + awk per corpo-simbolo.

## B: `<PhpArray as Drop>::drop` (h1f34bb4c5974c444, 204 istr — conferma il verbale)
- **bl sul cammino NORMALE (pre-`ret`): 19, TUTTI slow-path condizionali** raggiunti
  solo a refcount 0: 14× `Rc::drop_slow` (7 varianti ×2 rami Packed/Hashed),
  3× `zstr_drop_slow`, 2× `_mi_free` (buffer entries/index). Nessuna call
  incondizionata per-elemento: dispatch per variante via jump-table (`br x9`),
  dec Rc/zstr inline (`subs/str` + `b.ne` di rientro nel loop).
- **bl residuo verso `drop_in_place<Zval>` (10056eecc): DOPO la `ret` (10056eec0)**
  = landing pad di unwind (segue `panic_in_cleanup`). **NON è sul cammino Str**
  né su alcun cammino eseguito: la domanda del revisore è chiusa — firma PIENA,
  non parziale. Str sul cammino normale = dec inline + `zstr_drop_slow` outlined
  (10056ed1c/10056eea0/10056eea8), il costo per-elemento incomprimibile.
- Wrapper `drop_in_place<PhpArray>` nuovo (h5bf83466): bl → `PhpArray::drop` +
  bl → `drop_in_place<Repr>` (cammino post-drain: repr già svuotata).

## A: glue `drop_in_place<PhpArray>` (h980901d, 119 istr · hb9d5657, 90 istr)
- h980901d: **4 bl sul cammino normale** — 2× `drop_in_place<Zval>` (per-elemento,
  incondizionati nel loop), 1× `zstr_drop_slow`, 1× `_mi_free`; +9 unwind.
- hb9d5657: 3 bl normali (1× glue Zval, 1× `drop_in_place<Option<(Key,Zval)>>`,
  1× `_mi_free`); +8 unwind.

## Verdetto dell'atto
Controllo positivo del criterio p.4 SODDISFATTO e ARCHIVIATO: in B le call
per-elemento della glue spariscono dal cammino Repr-drop eseguito; il «bl
residuo» del verbale S-141 è unwind-only. La qualifica «firma parziale se il
residuo è sul cammino Str» è REFUTATA dai fatti del disasm.
