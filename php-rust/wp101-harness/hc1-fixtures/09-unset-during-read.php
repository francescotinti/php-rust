<?php
// Fixture UNSET DURANTE LA LETTURA (Stogov): la proprieta' sparisce a meta'
// espressione — il valore gia' letto sopravvive (copia), la lettura
// successiva prende il warning undef + null (via __get assente).
// ATTESA: 5 + unset + warning sulla SECONDA lettura, somma 5; poi isset false.
class U {
    public $x = 5;
    public function kill() { unset($this->x); return 0; }
}
$o = new U;
$r = $o->x + $o->kill() + (int)@$o->x;   // il primo x letto vale 5; il terzo e' undef->0
echo $r, "\n";                           // 5
var_dump(isset($o->x));                  // false
// Warning osservabile senza silenziatore (ordine del diagnostico):
$o2 = new U;
$o2->kill();
echo $o2->x ?? "gone", "\n";             // gone (?? sopprime il warning)
$v = $o2->x;                             // Warning: Undefined property: U::$x
var_dump($v);                            // NULL
