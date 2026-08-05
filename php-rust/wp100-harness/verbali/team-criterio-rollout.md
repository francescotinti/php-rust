# Team «criterio-rollout» — Concilio WP-100 (relatore)

**Perimetro**: punto 3 dell'ordine S-99 (rollout specializzazione Add nelle forme registro flag-on) e il suo criterio. Fonti vincolanti: verbale-1-hoare, verbale-2-matsakis, verbale-5-bak, verbale-8-stogov. Questa nota riconcilia, non sostituisce.

## Convergenze (tutte e quattro le sedie)

1. **D=6,07 ns/occ NON è ereditabile come àncora per le forme registro.** D misura il plumbing del percorso PILA (call `binary_value_ab` con due Zval owned da 24B, pop/push, marshalling/sret); `BinarySS/SC/Dst` prendono gli slot per PRESTITO e chiamano `binary_fast` `#[inline(always)]`: quel plumbing lì in gran parte non esiste. Un criterio copiato da D è miscalibrato per costruzione (Hoare A-HO-100-3; Matsakis R-1/A-MA-100-1; Bak R3/KS-BA-100-1; Stogov R1/A-ST-100-1).
2. **Il criterio va PRE-REGISTRATO da un controfattuale statico NUOVO del percorso registro**, scritto PRIMA del codice: il residuo rimovibile in BinarySS è solo carico+match del payload `BinOp` (+ eventuale bordo di call), classe di costo plausibilmente ≪6 ns.
3. **MISS = funnel owned ATTUALE, intoccato**: guardia Undef|Ref conservata, deref-funnel, `binary_value_ab`, ordine diags byte-identico; vietato l'in-place su slot attraverso un `Zval::Ref` (KS-HO-100-1 ≡ KS-ST-100-1).
4. La forma del handler `BinaryAdd` pila REGGE (nessuna sedia la refuta); è il CRITERIO del rollout a cadere.

## Conflitti / posizioni per sedia

- **Bak vs Matsakis — compatibili, non identici.** Matsakis (R-1): BinarySS già inlinea `binary_fast`, quindi il residuo è solo il match del payload — argomento STATICO sul codice. Bak (R3/A-BA-100-1): D non è mai stato decomposto (è 2,2–4,3× il suo tetto statico) e chiede una build intermedia di MISURA che separi prologo+marshalling (i)+(ii) da pop/push (iii), quota trasferibile stimata 50–70% ma ignota. Non c'è contraddizione: Matsakis dice DOVE il residuo vive, Bak esige che la sua GRANDEZZA sia misurata, non stimata. La build di Bak resta utile anche se BinarySS non chiama il funnel nel hit-path: spiega D e tara il controfattuale.
- **Chi chiede cosa PRIMA del rollout**: Bak → build di decomposizione di D (A-BA-100-1). Matsakis → tetto nuovo misurato flag-on sul giudice registro (A-MA-100-1) + chiusura wildcard `bin_op_of` (A-MA-100-2, R-2 latente: Sub/Mul compilerebbero senza fondersi, perf-only invisibile al corpus). Hoare → census dump-based dei `Binary(Add)` residui flag-off (A-HO-100-4) + decadenza del claim «flag-on bit-identico» al primo cambio di emissione (KS-HO-100-2). Stogov → sette fixture-trappola A-ST-99-3 come PRECONDIZIONE nominata (A-ST-100-2) + fixture in-tree (A-ST-100-3); coda AssignOp da disambiguare (R2).

## Proposta del team per il punto 3 di S-99 (forma finale)

1. **Pre-misura obbligatoria** (prima di ogni codice): (a) controfattuale statico del corpo BinarySS (residuo = match payload) con banda pre-registrata; (b) build intermedia A-BA-100-1 che decompone D; (c) baseline flag-on del giudice registro.
2. **Criterio**: soglia in ns/occ derivata da (a)+(b), MAI da D=6,07 (KS-MA-100-1, KS-BA-100-1); pubblicata come banda se sotto 2× il pavimento sonda (KS-BA-100-2).
3. **Precondizioni**: test/chiusura `bin_op_of` (KS-MA-100-2); sette trappole A-ST-99-3 scritte e verdi per NOME (KS-ST-100-2); census flag-off (A-HO-100-4).
4. **Gate di spedizione**: MISS=funnel intatto (KS-HO-100-1/KS-ST-100-1); corpus flag-ON per NOME + census ri-pinnati stessa sessione (KS-HO-100-2).

**Priorità**: 1) controfattuale statico registro; 2) chiusura `bin_op_of`; 3) build decomposizione D; 4) trappole A-ST-99-3; 5) solo poi il codice del rollout.
