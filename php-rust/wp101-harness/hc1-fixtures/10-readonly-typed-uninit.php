<?php
// Fixture READONLY + TYPED-UNINIT (Stogov): l'Undef tipizzato e' FATALE in
// lettura (mai null-degradato), il readonly rifiuta la seconda scrittura.
// Un fast-path che leggesse lo slot Undef come valore sarebbe unsound.
// ATTESA: Error "must not be accessed before initialization"; Error
// "Cannot modify readonly"; il valore resta.
class T { public int $n; }
$t = new T;
try { echo $t->n; } catch (\Error $e) { echo "E1:", $e->getMessage(), "\n"; }
$t->n = 3;
echo $t->n, "\n";      // 3
class RO { public function __construct(public readonly int $v) {} }
$r = new RO(8);
echo $r->v, "\n";      // 8
try { $r->v = 9; } catch (\Error $e) { echo "E2:", $e->getMessage(), "\n"; }
echo $r->v, "\n";      // 8
