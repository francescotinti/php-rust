<?php
// Trappola (a) — timing RHS-first: l'AssignOp valuta il RHS PRIMA di
// leggere il lhs (lezione WP-1..28 «assign composto RHS-first»); un fold
// che leggesse il lhs in anticipo cambierebbe questi valori.
$x = 10;
$x += ($x = 5);
echo "self:", $x, "\n";
function mut(&$v) { $v = 3; return 4; }
$a = 10;
$a += mut($a);
echo "byref:", $a, "\n";
$b = 100;
$b -= ($b = 7);
echo "sub:", $b, "\n";
$m = 2;
$m *= ($m = 6);
echo "mul:", $m, "\n";
