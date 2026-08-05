<?php
// Trappola (f) — timing di __destruct del valore spiazzato/temporaneo:
// l'ordine delle stampe e' il giudice, un fold che tenesse vivo un borrow
// oltre il punto di rilascio lo sposterebbe.
class D {
  public $tag;
  function __construct($t) { $this->tag = $t; }
  function __destruct() { echo "destruct:", $this->tag, "\n"; }
}
$x = new D('displaced');
$x = 0;
echo "after-assign\n";
$x += 1;
echo "x:", $x, "\n";
$y = 1;
$y += (function () { $tmp = new D('temp'); return 2; })();
echo "y:", $y, "\n";
