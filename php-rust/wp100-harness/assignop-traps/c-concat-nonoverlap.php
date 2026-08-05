<?php
// Trappola (c) — `.=` NON si sovrappone al fold aritmetico: viaggia su
// ConcatAssignSlot e il pass non deve toccarlo ne' scambiarlo per un
// AssignOp numerico (loop stretto = finestra candidata).
$s = "a";
for ($i = 0; $i < 5; $i++) { $s .= $i; }
echo $s, "\n";
$t = "x";
$t .= "y" . $t;
echo $t, "\n";
$n = "1";
$n .= 2;      // concat, NON addizione: "12"
$n += 3;      // ora numerico: 15
echo $n, "\n";
