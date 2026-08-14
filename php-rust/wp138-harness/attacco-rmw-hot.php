<?php
error_reporting(E_ALL);
class E { public array $data = []; }

// A: entry per riferimento, += a caldo (write-through?)
$e = new E(); $x = 50; $e->data['r'] = &$x;
for ($i=0; $i<4; $i++) { $e->data['r'] += 10; }
var_dump($x, $e->data['r']);

// B: entry per riferimento, ++ a caldo
$e2 = new E(); $y = 1; $e2->data['r'] = &$y;
for ($i=0; $i<4; $i++) { $e2->data['r']++; }
var_dump($y, $e2->data['r']);

// C: valore d'uso post-inc / pre-dec a caldo
$e3 = new E(); $e3->data['i'] = 0;
for ($i=0; $i<4; $i++) { var_dump($e3->data['i']++); }
for ($i=0; $i<4; $i++) { var_dump(--$e3->data['i']); }

// D: overflow += a caldo (int -> float alla 3a iter)
$e4 = new E(); $e4->data['o'] = PHP_INT_MAX - 2;
for ($i=0; $i<4; $i++) { $e4->data['o'] += 1; var_dump($e4->data['o']); }
// D2: overflow ++ a caldo
$e4->data['p'] = PHP_INT_MAX - 2;
for ($i=0; $i<4; $i++) { $e4->data['p']++; var_dump($e4->data['p']); }

// E: aliasing rhs = stessa entry
$e5 = new E(); $e5->data['k'] = 1;
for ($i=0; $i<4; $i++) { $e5->data['k'] += $e5->data['k']; }
var_dump($e5->data['k']);

// F: chiave Long su entry creata con chiave string '7'
$e6 = new E(); $e6->data['7'] = 1;
for ($i=0; $i<4; $i++) { $e6->data[7] += 2; }
var_dump($e6->data);

// G: *= float a caldo (old diventa Double dalla 2a iter)
$e7 = new E(); $e7->data['m'] = 3;
for ($i=0; $i<4; $i++) { $e7->data['m'] *= 2.5; }
var_dump($e7->data['m']);

// H: -= con rhs Double
$e9 = new E(); $e9->data['s'] = 10;
for ($i=0; $i<4; $i++) { $e9->data['s'] -= 0.5; }
var_dump($e9->data['s']);

// I: rhs muta la prop durante la valutazione (ordine di lettura di old)
class F2 { public array $data = ['k' => 1]; }
function evil(F2 $o): int { $o->data = ['k' => 100]; return 5; }
$e8 = new F2();
for ($i=0; $i<4; $i++) { $e8->data['k'] += evil($e8); var_dump($e8->data['k']); }

// J: rhs trasforma l'entry in ref durante la valutazione
class F3 { public array $data = ['k' => 1]; }
$gz = 1000;
function evil2(F3 $o): int { global $gz; $o->data['k'] = &$gz; return 7; }
$ea = new F3();
for ($i=0; $i<4; $i++) { $ea->data['k'] += evil2($ea); }
var_dump($gz, $ea->data['k']);

// K: ++ su Double a caldo
$eb = new E(); $eb->data['f'] = 1.25;
for ($i=0; $i<4; $i++) { var_dump($eb->data['f']++); }

// L: COW: copia dell'array prop viva durante il loop caldo
$ec = new E(); $ec->data['k'] = 1; $snap = [];
for ($i=0; $i<4; $i++) { $snap[] = $ec->data; $ec->data['k'] += 1; }
var_dump($ec->data['k'], $snap[0]['k'], $snap[3]['k']);
echo "END\n";
