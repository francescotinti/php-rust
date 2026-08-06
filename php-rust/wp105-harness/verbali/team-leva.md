# Team-LEVA (Hoare + Bak) — Concilio WP-105, fase 2

Tema: leva H-C2 (drop fast-out) e suo criterio. Fonti: verbale-1-hoare.md, verbale-5-bak.md. I verbali individuali restano vincolanti.

## (a) Convergenze

Colpo indipendente sullo STESSO punto capitale (R-HO-105-1 ≡ R2-Bak): `is_gc_container` distingue container/non-container, NON trivial-drop/drop-con-lavoro. `Str(Rc<PhpStr>)`, `Resource`, `Generator` rispondono `false` ma hanno Drop non banale: un fast-out letterale su `!is_gc_container` LEAKA ogni stringa poppata, invisibile al giudice prop (quasi solo Long) fino a corpus/WordPress. Il criterio `hc2-criterio.out` («fast-out SOLO via is_gc_container», eredità A-HO-104-5) va emendato PRIMA dell'implementazione, pena VOID (KS-HO-105-2).

**Predicato emendato**: nome `is_trivial_drop` (Hoare) ≡ «non-refcounted» (Bak, `is_refcounted`/`needs_drop`). Varianti true SOLO per: Undef/Null/Bool/Long/Double (+ eventuali specie provate Copy-like — ArgPlace/WeakHandle da classificare ESPLICITAMENTE, match esaustivo, niente wildcard). Denti che lo sigillano: `debug_assert!(is_trivial_drop(v) ⇒ !v.is_gc_container())` (A-HO-105-1) + fixture stringhe-in-Pop nel gate (KS-BA-105-1) + kill-switch congiunto: fast-out chiavato su `!is_gc_container` = reject senza appello (KS-HO-105-1 ≡ KS-BA-105-1).

## (b) Conflitti

Nessun conflitto sul predicato. Complementarità, non attrito:
- **Bak aggiunge la FORMA**: `dispose(v: Zval)` a predicato UNICO per morte (scalare→forget; refcounted-non-container→glue; container→note+glue); due valutazioni del predicato sulla stessa morte = reject in review (KS-BA-105-2). Hoare non lo contraddice; il team lo adotta.
- **Bak aggiunge il prefisso disasm** (A-BA-105-2, 30'): verificare se `drop_in_place::<Zval>` è outlined nei siti Pop/Binary*; A/B senza verdetto disasm = VOID (KS-BA-105-3). Hoare tace sul punto: nessuna obiezione.
- Solo Hoare: assert nested-Ref issato (A-HO-105-2), doc `is_gc_container` (A-HO-105-3), braccio rosso 19a/19b (A-HO-105-4) — igiene, non leva.

## (c) Bande

- **Banda-layout**: R=1 (una perturbazione, −308 B) è un CAMPIONE, non una banda; pavimento 4 ns non provatamente sopra la coda layout. Emenda: max|Δ| su N≥3 pad (64/192/448 B), CADUTA sotto max(banda-layout, ~3 ns rumore) (A-BA-105-3). In S-104 SOLO se il Δ cade in zona marginale — timeboxata, non prima.
- **Banda [8,22]**: il census conferma il denominatore (11 DropS), MAI il numeratore (0,7-2,0 ns/drop non misurato); «banda confermata dal contato» = sovradichiarazione. Il disasm la RINOMINA prima dell'A/B. Promozione: Δ∈[4,8) ⇒ R≥9 + segno stabile ABAB; Δ≥8 ⇒ R=5 (A-BA-105-4).

## (d) Priorità proposte S-104 (fronte leva)

1. Verdetto A/B peak R=7 (invariato).
2. Atto zero della leva: emenda criterio = A-HO-105-1 + forma A-BA-105-1 (predicato `is_trivial_drop`, dispose unico) — ~30'.
3. Prefisso disasm A-BA-105-2 (30'); l'esito rinomina la banda.
4. Leva H-C2: gate pieno + fixture KS-BA-105-1 + coppia WP (salda il debito).
5. H-D SiteTag invariato; banda-layout N≥3 solo se Δ marginale; A-HO-105-2/3/4 nell'igiene timeboxata (105-4 prima di usare 19a/19b come gate di promozione).
