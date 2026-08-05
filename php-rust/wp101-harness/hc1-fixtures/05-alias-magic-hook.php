<?php
// Fixture __get/HOOK (Stogov): il valore e' un TEMPORANEO, non c'e' slot da
// prestare. Qualunque fast-path H-C1x deve MANCARE questi e cadere nel
// percorso generale.
// ATTESA: __get chiamato a ogni lettura (2 volte), hook get/set eseguiti,
// backed hook legge/scrive il backing store.
class G {
    private $data = ['v' => 10];
    public function __get($n) { echo "get:", $n, "\n"; return $this->data[$n]; }
    public function __set($n, $v) { echo "set:", $n, "=", $v, "\n"; $this->data[$n] = $v; }
}
$g = new G;
echo $g->v + $g->v, "\n";     // get:v get:v 20
$g->v = 5;                    // set:v=5
echo $g->v, "\n";             // get:v 5
// Property hooks (8.4): get virtuale + set con backing.
class H {
    public $base = 2;
    public $twice { get => $this->base * 2; }
    public $clamped = 0 { set => min(10, $value); }
}
$h = new H;
echo $h->twice, "\n";         // 4
$h->base = 5;
echo $h->twice, "\n";         // 10
$h->clamped = 99;
echo $h->clamped, "\n";       // 10
$h->clamped = 3;
echo $h->clamped, "\n";       // 3
