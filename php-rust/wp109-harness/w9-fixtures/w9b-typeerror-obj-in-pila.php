<?php
// W9b BinaryTCPropSetPop (azione-3 revisore S-108): TypeError nel funnel
// col ricevitore ANCORA in pila — l'unwind deve pulirla e il programma
// continua. Giudice = byte-parity oracle<->phpr nei due modi.
class NoStr {}
class P { public $p = 'init'; }
$o = new P;
$x = new NoStr;
try {
    $o->p = $x . "s";
} catch (Error $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
var_dump($o->p);
$o->p = "fine";
echo $o->p, "\n";
