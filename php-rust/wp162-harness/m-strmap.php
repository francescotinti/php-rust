<?php
// S-162 L-AM2 — giudice micro array_map string-callable UTENTE: 200 chiamate
// array_map('inc1', $arr) su array di 50.000 interi = 10.000.000 elementi.
// Esercita il bundle per-elemento del loop generico: cb.clone + scan "::" +
// name to_vec + find_fn_ci + args-Vec + bind_params. N letterale (200x50000).
// Parita': marcatore dalla somma. Funzione UTENTE ammessa: 1 param senza
// hint/by_ref/variadic/default => simple_call.
function inc1($x) { return $x + 1; }
$arr = range(0, 49999);
$acc = 0;
for ($i = 0; $i < 200; $i++) {
    $r = array_map('inc1', $arr);
    $acc += $r[0] + $r[49999];
}
echo "SM-OK $acc\n";
