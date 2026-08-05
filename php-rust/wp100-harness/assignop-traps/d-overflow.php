<?php
// Trappola (d) — overflow int→float SENZA wrap nel fast path fuso, anche
// dentro il loop stretto (la finestra che il pass riscrive).
$s = PHP_INT_MAX;
$s += 1;
var_dump($s);
$s = PHP_INT_MAX - 2;
for ($i = 0; $i < 5; $i++) { $s += 1; }
var_dump($s);
$n = PHP_INT_MIN;
$n -= 1;
var_dump($n);
$m = PHP_INT_MAX;
$m *= 2;
var_dump($m);
