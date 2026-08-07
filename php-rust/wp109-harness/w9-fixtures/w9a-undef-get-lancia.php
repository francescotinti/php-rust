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
// (caso B — ricevitore undef — PARCHEGGIATO: catalogo §3.16, riga errata
// nel warning undef-var del ricevitore, bilaterale on≡off; repro in
// parked-w9a-caso-b-receiver-undef.php.txt)
echo "done\n";
