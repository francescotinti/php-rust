<?php
// Fixture LAZY ghost (Stogov): la lettura di proprieta' inizializza PRIMA;
// un fast-path che leggesse lo slot del ghost non inizializzato leggerebbe
// spazzatura. ATTESA: init stampato UNA volta, alla PRIMA lettura.
class L {
    public $x = 0;
    public function __construct() { echo "init\n"; $this->x = 42; }
}
$r = new ReflectionClass(L::class);
$o = $r->newLazyGhost(function ($obj) { $obj->__construct(); });
echo "before\n";
echo $o->x, "\n";     // init 42 (l'accesso inizializza)
echo $o->x, "\n";     // 42 (niente doppia init)
$o->x = 7;            // scrittura post-init: normale
echo $o->x, "\n";     // 7
