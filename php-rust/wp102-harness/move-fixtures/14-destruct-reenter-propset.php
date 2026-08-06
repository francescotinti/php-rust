<?php
// Fixture 14 — DESTRUCT-REENTER-PROPSET (A-ST-103-1, Stogov).
// Il __destruct del VECCHIO valore rientra nel VM e rilegge/riscrive la
// STESSA proprieta' in volo. Semantica Zend garbage-poi-dtor: il nuovo
// valore e' gia' nello slot QUANDO il dtor parte — il dtor vede il NUOVO.
// Con l'handle MOSSO (H-C1b) il ricevitore in volo deve restare vivo e
// coerente per tutta la catena rientrante.
// ATTESA (scritta PRIMA; l'oracle arbitra l'ordine esatto):
//   begin
//   dtor(A): sees obj(B)          <- garbage-poi-dtor
//   dtor(B): sees 'from-dtor-A'   <- il write del dtor(A) uccide B, nested
//   after: 'from-dtor-B'          <- l'ultimo write vince
//   end / dtor residui dell'holder a fine script
class Holder {
    public $slot = null;
    public function __destruct() { echo "dtor(holder)\n"; }
}
class Old {
    public $h;
    public $tag;
    public function __construct($h, $tag) { $this->h = $h; $this->tag = $tag; }
    public function __destruct() {
        echo "dtor({$this->tag}): sees " . describe($this->h->slot) . "\n";
        // Riscrive la STESSA proprieta' in volo (re-entrancy sincrona).
        $this->h->slot = "from-dtor-{$this->tag}";
    }
}
function describe($v) {
    if (is_object($v)) { return "obj(" . $v->tag . ")"; }
    return var_export($v, true);
}
$h = new Holder();
$h->slot = new Old($h, "A");
echo "begin\n";
$h->slot = new Old($h, "B");   // dtor(A) parte QUI, dentro il braccio PropSet
echo "after: " . var_export($h->slot, true) . "\n";
$h->slot = null;
echo "end\n";
