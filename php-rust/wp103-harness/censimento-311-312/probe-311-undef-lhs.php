<?php
// probe-311 — CENSIMENTO del perimetro §3.11 (AssignOp con lhs indefinito:
// warning «Undefined variable/property/array key» mancante). A-ST-104-2.
// Ogni caso è marcato; il diff oracle<->phpr mostra QUALI warning mancano.
echo "case-a:toplevel-var-plus\n";
$u1 += 1; echo "u1=$u1\n";

echo "case-b:function-var-plus\n";
(function () { $u2 += 2; echo "u2=$u2\n"; })();

echo "case-c:dim-su-var-indefinita\n";
$a1['k'] += 3; echo "a1k={$a1['k']}\n";

echo "case-d:array-key-indefinita\n";
$a2 = ['x' => 1];
$a2['y'] += 4; echo "a2y={$a2['y']}\n";

echo "case-e:concat-assign\n";
$u3 .= "x"; echo "u3=$u3\n";

echo "case-f:coalesce-assign-controllo\n";
$u4 ??= 5; echo "u4=$u4\n";   // per design NIENTE warning: controllo positivo

echo "case-g:post-incr\n";
$u5++; echo "u5=", var_export($u5, true), "\n";

echo "case-h:pre-incr\n";
++$u6; echo "u6=", var_export($u6, true), "\n";

echo "case-i:minus-assign\n";
$u7 -= 7; echo "u7=$u7\n";

echo "fine\n";
