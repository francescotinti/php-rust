# Verbale LEIJEN — bozza indipendente S-167

## VERDETTO: GO-CONDIZIONATO

**Rotta in una riga:** campagna SÌ ma ordinata R1 (front-end/dispatch) dopo sonda discriminante su arith; R3-arena declassata (classe alloc refutata da AL3), R2 inline-payload sotto veto footprint.

## Refutazioni dal mio angolo

1. **R3-arena non paga in alloc-cost.** L-AL3 (S-164) è verdetto di classe: alloc mimalloc su fast path ≈0 ns, census esatto. Un'arena può pagare SOLO in località — ma frame e operandi sono già slot diretti (H-D/HD2, Frame 176B): working set L1-resident. Indizi: borrow ≤1 ns (L-TD1) = dati caldi in L1; H-C2 caduta **icache**-bound = la memoria che morde è il CODICE, non i dati. R3 va riletta come riduzione traffico push/pop, cioè territorio R1/ARCO REGISTRI.
2. **Su arith la località dati NON può essere il canale.** Il driver `$s += $i*3 − ($i>>2)` tocca una manciata di Zval scalari: dopo la prima iterazione tutto sta in poche linee L1. L'invarianza per forma di prop (~300-340 ns) e la banda-layout MC1 (±5-8 ns per +45 bl) puntano al **front-end CPU** (taglia del match, branch-miss, icache), non a D-cache.
3. **Anti-R2 (footprint).** Zval è pinnata a 16B da const-assert (KS-HE-104-1, `array.rs:93`) con niche `Option<Zval>`==Zval (`array.rs:88`) che regge il Bucket packed di PhpArray. Ingrassarla a 24/32B per inline-payload = +50-100% su OGNI bucket packed e slot props: il margine RAM conquistato (−70,6MB unit, slot_names −51,63MB, Frame 400→176B) verrebbe restituito sui DATI, che scalano col workload. Margine per una Zval grassa: NON c'è senza gate dedicato.

## Emendamenti

- **E-LE-1**: ordine R1 → (R3 solo-come-traffico) → R2; nessuna fetta "allocatore/arena" senza census a eventi con alloc/op > 0 sul cammino bersaglio (se 0, fetta vacua per costruzione).
- **E-LE-2**: ogni fetta R2 che cambia `size_of::<Zval>` è VOID salvo porti: nuovo const-assert + gate `vmmap` Physical footprint WP ≤ +5% pre-registrato.
- **E-LE-3**: sonda discriminante PRIMA delle leve: (a) gemello data-stride su arith (stessa dispatch, operandi su linee cache distinte) — D≈0 ⇒ memoria-dati refutata; (b) CPU Counters (IPC, branch-miss/op, L1I-miss) phpr vs oracle — il canale si NOMINA, non si presume.

## Kill-switch

**KS-LE-167-1**: se la sonda non separa (stride muto E counter phpr≈oracle), la campagna su arith NON parte ⇒ istruttoria R4.

## PRIMA FETTA

Sonda discriminante arith. Giudice: `run-micro.sh` R=5 + counter per-op. Soglie: braccio stride |D| ≤ 0,5 ns ⇒ memoria-dati esclusa; branch-miss/op phpr ≥ 2× oracle ⇒ front-end FIRMATO ⇒ prima leva R1 (riduzione taglia match / predecode).
