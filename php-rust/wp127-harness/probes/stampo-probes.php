<?php
// Sonde funzionali L-OL1-F1 «stampo»: default condivisi COW, thunk-once,
// unserialize/reflection, retry su default che lancia. Confronto byte-a-byte
// con l'oracle 8.5.7.

// 1) COW: il default [] condiviso non deve accoppiare le istanze.
class A1 { public array $d = []; public $s = 'x'; public ?A1 $r = null; }
$a = new A1; $b = new A1;
$a->d['k'] = 1;
var_dump($a->d, $b->d);

// 2) Default non-costante (thunk): array literal con chiavi + const espressione.
class B1 { const K = 5; public $arr = [1, 'due', 3.0]; public $n = self::K + 2; }
$x = new B1; $y = new B1;
$x->arr[] = 99;
var_dump($x->arr, $y->arr, $y->n);

// 3) get_class_vars sui default valutati.
var_dump(get_class_vars('B1'));

// 4) unserialize di istanza vuota: i default (anche non-costanti) presenti.
$u = unserialize('O:2:"B1":0:{}');
var_dump($u->arr, $u->n);

// 5) newInstanceWithoutConstructor: default presenti, ctor saltato.
class C1 { public array $d = ['seed']; public function __construct() { $this->d = ['ctor']; } }
$rc = new ReflectionClass('C1');
$w = $rc->newInstanceWithoutConstructor();
var_dump($w->d, (new C1)->d);

// 6) Ordine di dichiarazione e uninitialized conservati.
class D1 { public int $t; public $u = 'v'; public array $w = []; }
$d = new D1;
var_dump($d);

// 7) Ereditarietà: il figlio ridefinisce il default del padre.
class E1 { public $p = ['padre']; }
class E2 extends E1 { public $p = ['figlio']; public $q = [7]; }
var_dump((new E2)->p, (new E1)->p, (new E2)->q);

// 8) Molte istanze dopo lo stampo: valori stabili.
$sum = 0;
for ($i = 0; $i < 1000; $i++) { $o = new B1; $sum += $o->n; }
var_dump($sum);

// 9) Serializzazione di un'istanza con default condiviso non toccato.
var_dump(serialize(new B1));

// 10) var_export / json su default condivisi.
var_dump(json_encode(new B1));
