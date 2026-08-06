<?php
// probe-312 — CENSIMENTO del perimetro §3.12 (typed-ref: AssignOp fallito —
// Zend azzera il referente, phpr lo conserva). A-ST-104-2.
class T { public int $i = 1; public static int $s = 2; }

echo "case-a:ref-a-prop-typed\n";
$t = new T();
$r = &$t->i;
try { $r += "abc"; } catch (\TypeError $e) { echo "TE\n"; }
echo "t-i=", $t->i, "\n";

echo "case-b:ref-a-static-typed\n";
$q = &T::$s;
try { $q += "abc"; } catch (\TypeError $e) { echo "TE\n"; }
echo "T-s=", T::$s, "\n";

echo "case-c:prop-typed-diretta-controllo\n";
$t2 = new T();
try { $t2->i += "abc"; } catch (\TypeError $e) { echo "TE\n"; }
echo "t2-i=", $t2->i, "\n";

echo "case-d:ref-passata-per-funzione\n";
function bump(&$x) { try { $x += "abc"; } catch (\TypeError $e) { echo "TE\n"; } }
$t3 = new T();
bump($t3->i);
echo "t3-i=", $t3->i, "\n";

echo "fine\n";
