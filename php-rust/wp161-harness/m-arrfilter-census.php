<?php
// S-161 sonda AF1 — driver CONTEGGI census: m-arrfilter.php (wp160) con
// chiamate 10000→200 (adattamento DICHIARATO; i conteggi scalano esatti).
// 200 chiamate x 1.000 elementi = 200.000 invocazioni-elemento.
// Parità: AF-OK 100000 (500 elementi pari tenuti x 200 chiamate).
$f = function ($x) { return ($x % 2) === 0; };
$a = range(0, 999);
$acc = 0;
for ($i = 0; $i < 200; $i++) {
    $r = array_filter($a, $f);
    $acc += count($r);
}
echo "AF-OK $acc\n";
