# Verbale 5 — BAK (Concilio WP-105, fase 1)

Mandato: refutare S-103 e la bozza §S-104. Fonti: NEXT_SESSION, WP_SESSION_103,
hc-drop-census-s103.out, leva-nulla-verdetto.out, hc2-criterio.out; codice:
`crates/php-runtime/src/vm/run.rs` (Op::Pop, ~679-689),
`crates/php-runtime/src/vm/mod.rs` (`gc_note` 3914-3928),
`crates/php-types/src/zval.rs` (`Zval` 15-34, `is_gc_container` 215-237).

## VERDETTO

S-103 come sessione di prefissi: SOLIDA (census esatto, criterio scritto prima,
leva-nulla eseguita). Ma il criterio H-C2 contiene **un difetto capitale di
predicato** e la "banda-layout" è un campione, non una banda. La leva è
ADDITIVA rispetto a H-C1a — canale diverso — ma va emendata PRIMA dell'A/B.

## Refutazioni

**R1 (punto d — il canale).** H-C1a guarda SOLO la chiamata `gc_note`
(mod.rs:3925: `if v.is_gc_container() { gc_note_slow }`): un Long oggi paga
il predicato inline e nient'altro su quel ramo. Il fast-out H-C2 bersaglia il
**drop glue Rust** alla morte del valore (Pop: `stack.pop()` → glue a fine
scope; overwrite nei siti Binary*/IncDec) — canale che H-C1a NON tocca.
Quindi ADDITIVO, non ridondante. MA: implementato ingenuamente aggiunge una
SECONDA valutazione del predicato per morte (una in gc_note, una nel fast-out)
più il branch del glue: tre rami dove oggi ce ne sono due. Senza fusione in
un `dispose(v: Zval)` a predicato UNICO, il Δ atteso può essere ~0 o negativo.

**R2 (capitale — predicato sbagliato per il drop).** `hc2-criterio.out` pinna
«fast-out SOLO via is_gc_container». Ma `is_gc_container` è `false` per
`Str(Rc<PhpStr>)`, `Resource`, `Generator` (zval.rs:227-233), che SONO
refcounted: un fast-out che su `!is_gc_container` faccia forget/skip del glue
**LEAKA ogni stringa poppata**. Il giudice prop (quasi solo Long) non lo
vedrebbe; stringhe/corpus sì. Il predicato giusto per il canale drop è
"non-refcounted" (Undef/Null/Bool/Long/Double) — DISTINTO da quello del canale
note. A-HO-104-5 protegge i container, non le Str.

**R3 (punto a).** La banda-layout è R=1 SUL LAYOUT: una sola perturbazione
(−308 B), un solo binario B. 0,67 ns/iter (0,4% di ~162 ns/iter) è un
CAMPIONE di una distribuzione a coda pesante (Mytkowicz; Stabilizer): code al
2-5% (3-8 ns/iter) sono documentate. Il pavimento 4 ns/iter (2,5%) NON è
provatamente sopra la coda layout.

**R4 (punto b).** R=5, spread 2,7-3,3 ns/iter ⇒ risoluzione sul Δ delle
mediane ~±1,5-2 ns/iter. Un Δ=4 ns/iter è ~2σ: promozione fragile nella
zona [4,8). Inoltre la soglia di CADUTA a 0,67 è sotto la risoluzione del
rumore: tra 0,67 e ~3 si "registra" una cifra senza significato.

**R5 (punto c).** Il census conferma il DENOMINATORE (11 DropS), non il
numeratore: [8,22] presuppone 0,7-2,0 ns/drop, mai misurato. Gli 11 sono
quasi tutti Long: glue = branch sul discriminante — costa ~2 ns solo se
`drop_in_place::<Zval>` è OUTLINED nei siti caldi (mai verificato). Se LLVM
lo inlinea, l'attesa realistica è 2-4 ns/iter: sotto/al pavimento. Dire
«banda confermata dal contato» (NEXT_SESSION r.45) è una sovradichiarazione.

## Emendamenti

- **A-BA-105-1**: la leva si implementa come dispose a predicato UNICO per
  morte (scalare→forget; refcounted-non-container→glue; container→note+glue);
  il predicato drop è `is_refcounted`/`needs_drop`, NON `is_gc_container`.
- **A-BA-105-2**: prefisso da 30' prima dell'A/B: disassembly dei siti Pop/
  Binary* per verificare se il glue è outlined; l'esito RINOMINA la banda.
- **A-BA-105-3**: banda-layout = max|Δ| su N≥3 perturbazioni (pad diversi,
  es. 64/192/448 B); CADUTA sotto max(banda-layout, ~3 ns rumore).
- **A-BA-105-4**: promozione con Δ∈[4,8): R≥9 + segno stabile su tutte le
  coppie ABAB; Δ≥8: basta R=5.

## Kill-switch

- **KS-BA-105-1**: fast-out del glue chiavato su `!is_gc_container` = REJECT
  senza appello (leak di Str/Resource); serve fixture stringhe-in-Pop nel gate.
- **KS-BA-105-2**: due valutazioni del predicato sulla stessa morte = reject
  in code review (il check di gc_note si fonde nel dispose).
- **KS-BA-105-3**: A/B lanciato senza il verdetto disasm di A-BA-105-2 = VOID.

## Priorità S-104

1. Verdetto A/B peak R=7 (invariato). 2. A-BA-105-2 (disasm, 30').
3. Leva H-C2 nella forma A-BA-105-1, gate pieno + fixture KS-BA-105-1,
coppia WP (salda il debito). 4. H-D SiteTag invariato. 5. Banda-layout N≥3
solo se il Δ cade in zona marginale — non prima (timebox).
