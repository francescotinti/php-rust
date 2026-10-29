<?php class A {}
$N = 200000;
$a = array_map(fn($i) => new A, range(1, $N)); echo count($a), "\n";
