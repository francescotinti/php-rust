# s129-modello-tempo.md — L-OL1 seg.3: il MODELLO DEL TEMPO è chiuso; forma nominata

Catena (criterio s129-criterio-tempo.md, ordine p.9 rispettato): census identità
(22/22) → sonde pin R=5 → per-passo (chiusura 95-96%) → disasm → QUESTA lettura.

## I tre fatti
1. **Identità dei residui CHIUSA (census 22/22)**: i ~2 alloc/statement ignoti di
   seg.2 sono i `n.clone()` (Box del nome) in `byref_hook_root` (mod.rs:14333) e
   `field_lazy_root` (mod.rs:12876); la variante name-borrow li toglie ESATTI
   (−2 su ogni statement FieldAssign, 11 categorie). Corollari chiusi: overwrite
   +4 = keys+2 cloni+dropped-push; 2° insert +3 = keys+2 cloni+push-in-capacità.
   La mono-passo `$o->p=v` è `Op::PropSet` e NON paga nulla di tutto ciò.
2. **Il costo per-statement è FISSO, non da lavoro-array** (pin, R=5, bilaterale):
   insert 340,0 · 2° insert 316,6 · overwrite 300,0 · append 300,0 ns/iter contro
   oracle 23–37 (rapporti 9–13×). La famiglia locale (`$a['k']=`) fa 170 ns: il
   sovrapprezzo prop-rooted è ~+150 ns/statement.
3. **La torta (per-passo, build emendata, quote=MODELLO; TOT 297, chiusura 96%)**:
   **E1 (dispatch+prop_step: ~5 `resolve_prop_access` + 3 lookup props) ≈ 155 ns (52%)**
   · **preludio B+C+D ≈ 73 ns (25%)** (byref 36 + lazy 30 + indirect 7,5; disasm:
   395/317/inlined ins, MAI necessari su classe senza hook e oggetto non-lazy)
   · E2 walk interno (COW `make_mut`+coerce+insert) ≈ 48 ns (16%) · keys 9 · resto 12.
   Le quote sono stabili per forma (p5/p6 entro pochi ns) — coerente col punto 2.

## Rank delle forme (upper-bound dal modello, ns/iter sul giudice objdatains)
1. **resolve-once in prop_step** (E1, UB ~155 ma quota obbligata ignota) — la più
   grossa e la più invasiva: cache di UNA resolve per statement attraverso guardia,
   contains, get_mut. RINVIATA a S-130: chiede progettazione dei borrow.
2. **F4 «prelude-gate»** (B+C+D, **UB = 73**) — un SOLO check (classe senza
   property-hook ∧ oggetto non-lazy/proxy ∧ base Object diretto) salta il trio i
   cui esiti sono None/no-op PER COSTRUZIONE sotto quelle condizioni; ingloba il
   guadagno name-borrow (i 2 cloni cadono col trio). Sostituto = 1 flag di classe
   precomputato + 1 borrow — NON è alloc-removal cieco: il costo sostitutivo è
   nominato e ~2 ordini sotto l'UB (lezione F2 onorata).
3. insert-path (E2, UB 48) e name-borrow da sola (~10-15) — dominati da 1-2.

## FORMA NOMINATA per l'A/B: **L-OL1-F4 prelude-gate** (criterio p.7 istanziato,
commit PRIMA del codice): giudice objdatains N=3e6, R=5 ABAB netto-pavimento,
segno CALA, soglia max(4 ns/iter, rumore R=5, banda-layout prop), smoke R=2
early-stop; guardie SOLO-REGRESSIONE sei micro + objalloc/objchurn/objmap (bande
«default 4 ns»+spread); census arbitro PRIMA dell'A/B con predizioni = colonna B
del s129-census-verdetto (identiche a name-borrow: datains 10, churn 11, p2 9,
p4 10, p5 11, p6 12, invariati gli altri); disasm prima/dopo (bl-count arm).
PREREQUISITO di REGOLE §4: gate micro R=5 (calls 4,9*) PRIMA dell'A/B.
