<?php
// Fixture ORDINE __destruct (Matsakis R2, riusa il pattern della trappola f
// S-100): un cambio di aliasing a runtime sposta il momento in cui
// strong_count tocca 1 — QUALUNQUE cambio d'ordine dei distruttori = reject
// (KS-MA-102-3). ATTESA: l'ordine ESATTO delle stampe qui sotto.
class D {
    public function __construct(public string $tag) {}
    public function __destruct() { echo "~", $this->tag, "\n"; }
}
// 1) il temporaneo di un'espressione muore A FINE STATEMENT:
echo "s1\n";
(new D("tmp"))->tag;
echo "s2\n";                    // atteso: s1 ~tmp s2
// 2) la lettura di proprieta' che tiene vivo l'oggetto: il valore letto
//    (handle in $v) POSTICIPA la morte fino alla riassegnazione di $v.
class Holder { public $inner; }
$h = new Holder;
$h->inner = new D("held");
$v = $h->inner;                 // seconda handle
unset($h);                      // l'oggetto Holder muore, "held" NO (in $v)
echo "after-unset-h\n";         // atteso: after-unset-h (niente ~held qui)
$v = null;                      // ULTIMA handle: ~held ORA
echo "after-null-v\n";          // ~held poi after-null-v
// 3) sovrascrittura di proprieta': il VECCHIO muore alla scrittura (sweep
//    di fine statement), PRIMA della stampa successiva.
$h2 = new Holder;
$h2->inner = new D("old");
$h2->inner = new D("new");      // ~old a fine statement
echo "after-overwrite\n";       // atteso: ~old after-overwrite
unset($h2);                     // ~new
echo "end\n";
