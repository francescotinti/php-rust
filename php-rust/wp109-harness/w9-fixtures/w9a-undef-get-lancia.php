<?php
// W9a PropGetSlotRecv (azione-4 revisore S-108): ordine warning/eccezione.
// caso A: __get che lancia sul read RMW di una proprieta' non dichiarata.
class G {
    public function __get($n) { echo "get($n)\n"; throw new RuntimeException("boom:$n"); }
}
$o = new G;
try {
    $o->q = $o->q + 1;
} catch (RuntimeException $e) {
    echo "caught ", $e->getMessage(), "\n";
}
// caso B: ricevitore undef — l'ordine warning/Error deve essere l'oracle.
try {
    $u->p = $u->p + 1;
} catch (Error $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
echo "done\n";
