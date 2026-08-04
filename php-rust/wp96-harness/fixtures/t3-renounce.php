<?php
// I tre modi in cui PHP vede i locali fuori dal flusso: ognuno toglie
// esattamente i suoi siti dal perimetro F2.
function con_compact() { $x = "x"; $y = "y"; return compact('x', 'y'); }
function con_global()  { global $G; $l = "l"; echo $l, $G, "\n"; }
function con_useref()  { $c = "c"; $f = function () use (&$c) { $c .= "!"; }; $f(); echo $c, "\n"; }
$G = "g";
print_r(con_compact());
con_global();
con_useref();
