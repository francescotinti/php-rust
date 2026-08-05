<?php
// Fixture MUTAZIONE INTERLACCIATA (Matsakis∪Stogov): $o->a + $o->mut() —
// il valore di $o->a e' osservabile PRE-mutazione: l'addref/copia lo
// garantisce, un prestito nudo dello slot NO. Il controesempio capitale
// di H-C1b: se il ricevitore viaggia in prestito, la mutazione dentro
// mut() non deve comunque toccare il valore GIA' LETTO.
// ATTESA: 1+100=101 (a letto PRIMA che mut() lo porti a 100); poi a=100.
class M {
    public $a = 1;
    public function mut() { $this->a = 100; return 100; }
}
$o = new M;
echo $o->a + $o->mut(), "\n";   // 101
echo $o->a, "\n";               // 100
// Stessa trappola sulla STRINGA (il prestito del buffer sarebbe UAF logico):
class MS {
    public $s = "pre";
    public function smash() { $this->s = "post"; return "-"; }
}
$m = new MS;
echo $m->s . $m->smash() . $m->s, "\n";  // pre-post
// E con l'ordine invertito (la scrittura prima della lettura):
$m2 = new MS;
echo $m2->smash() . $m2->s, "\n";        // -post
