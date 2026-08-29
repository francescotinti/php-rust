<?php
// S-160 leva p.4 — giudice micro array_filter (caso dominante: 1 array +
// closure anonima arità-1, mode=0). 10.000 chiamate x 1.000 elementi = 10M
// invocazioni-elemento: il segnale della leva è il plumbing PER-ELEMENTO
// (vec![v] + dispatch call_callable rifatto a ogni elemento); il corpo della
// closure, to_bool e la costruzione dell'array di output sono common-mode.
// Parità: AF-OK 5000000 (500 elementi pari tenuti per ognuna delle 10k chiamate).
$f = function ($x) { return ($x % 2) === 0; };
$a = range(0, 999);
$acc = 0;
for ($i = 0; $i < 10000; $i++) {
    $r = array_filter($a, $f);
    $acc += count($r);
}
echo "AF-OK $acc\n";
