<?php
error_reporting(E_ALL);
class E { public array $data = []; }

// chiavi string non-canoniche: restano string in PHP
$e = new E();
$e->data['07'] = 1; $e->data['-0'] = 1; $e->data[' 7'] = 1; $e->data['1e2'] = 1; $e->data['9223372036854775808'] = 1;
for ($i=0; $i<4; $i++) {
  $e->data['07'] += 1; $e->data['-0'] += 1; $e->data[' 7'] += 1; $e->data['1e2'] += 1; $e->data['9223372036854775808'] += 1;
}
var_dump($e->data);

// chiave canonica negativa e Long negativo
$e2 = new E(); $e2->data['-5'] = 10;
for ($i=0; $i<4; $i++) { $e2->data[-5] += 3; }
var_dump($e2->data);

// base $this a caldo
class Th { public array $data = ['k' => 5];
  public function bump(): int {
    for ($i=0; $i<4; $i++) { $this->data['k'] += 3; $this->data['k']--; }
    return $this->data['k'];
  }
}
var_dump((new Th())->bump());

// base globale dentro funzione
$g = new E(); $g->data['k'] = 0;
function fg(): void { global $g; for ($i=0; $i<4; $i++) { $g->data['k'] += 2; } }
fg(); fg();
var_dump($g->data['k']);

// prop privata con scope corretto, a caldo
class Pv { private array $d = ['k' => 1];
  public function run(): int { for ($i=0; $i<4; $i++) { $this->d['k'] *= 2; } return $this->d['k']; }
}
var_dump((new Pv())->run());

// eredita: sito condiviso sottoclasse (stessa classe-id? no: figlio) — mono-IC re-fill
class Sub extends E {}
function touch2(E $o): void { $o->data['k'] += 1; }
$p = new E(); $p->data['k'] = 0; $s = new Sub(); $s->data['k'] = 100;
for ($i=0; $i<3; $i++) { touch2($p); }
for ($i=0; $i<3; $i++) { touch2($s); }
var_dump($p->data['k'], $s->data['k']);
echo "END\n";
