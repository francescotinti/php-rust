<?php
// S-159 leva p.3 — giudice micro array_map (caso dominante: 1 array + closure
// anonima). 10.000 chiamate x 1.000 elementi = 10M invocazioni-elemento: il
// segnale della leva è il plumbing PER-ELEMENTO (vec![v] + dispatch
// invoke_value rifatto a ogni elemento); il corpo della closure e la
// costruzione dell'array di output sono common-mode sui due bracci.
// Parità: AM-OK 10000000 (r[999]=1000 per ognuna delle 10k chiamate).
$f = function ($x) { return $x + 1; };
$a = range(0, 999);
$acc = 0;
for ($i = 0; $i < 10000; $i++) {
    $r = array_map($f, $a);
    $acc += $r[999];
}
echo "AM-OK $acc\n";
