<?php
// S-136 FD1 — fixture di fedeltà dim-write su proprietà (criterio
// s136-criterio-dimwrite.md p.8). UNO statement per riga. Parità attesa:
// candidato==stash BYTE-ID; vs oracle valgono SOLO le famiglie §3.21.
echo "f1 fast puro (untyped e ripetuto: fill al primo giro, hit dal secondo)\n";
class A1 { public $d = []; }
$a = new A1;
$a->d['k'] = 1;
$a->d['k'] = 2;
$a->d[5] = 'x';
$a->d[] = 'app';
print_r($a->d);
echo "f2 prop TIPATA array\n";
class A2 { public array $d = []; }
$b = new A2;
$b->d['k'] = 7;
$b->d['k'] = 8;
print_r($b->d);
echo "f3 unset prop dichiarata -> dynamic-route, poi re-set\n";
class A3 { public $d = []; }
$c = new A3;
$c->d['k'] = 1;
unset($c->d);
$c->d['k'] = 2;
print_r($c->d);
echo "f4 readonly: drill-in = errore (anche DOPO un giro su prop normale)\n";
class A4 { public readonly array $r; public array $ok = [];
  public function __construct() { $this->r = ['x' => 1]; }
}
$d = new A4;
$d->ok['k'] = 1;
try {
    $d->r['x'] = 2;
} catch (Error $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
print_r($d->r);
echo "f5 asym private(set): drill-in da FUORI = errore, da DENTRO ok\n";
class A5 { public private(set) array $s = [];
  public function w() { $this->s['in'] = 1; }
}
$e5 = new A5;
$e5->w();
try {
    $e5->s['out'] = 2;
} catch (Error $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
print_r($e5->s);
echo "f6 prop con Ref dentro e scrittura attraverso\n";
class A6 { public $d = []; }
$f = new A6;
$f->d = [0];
$rr = &$f->d[0];
$f->d[0] = 9;
var_dump($rr);
unset($rr);
echo "f7 prop stringa: offset-write resta al pieno\n";
class A7 { public $s = 'abc'; }
$g = new A7;
$g->s[1] = 'X';
var_dump($g->s);
echo "f8 nested 2 chiavi (fuori perimetro)\n";
class A8 { public $d = []; }
$h = new A8;
$h->d['a']['b'] = 3;
print_r($h->d);
echo "f9 classe con __get/__set: prop dichiarata NON li consulta, assente sì\n";
class A9 { public $d = [];
  private $bag = [];
  public function __get($n) { echo "GET $n\n"; return $this->bag[$n] ?? null; }
  public function __set($n, $v) { echo "SET $n\n"; $this->bag[$n] = $v; }
}
$i9 = new A9;
$i9->d['k'] = 1;
$i9->d['k'] = 2;
print_r($i9->d);
$i9->assente = 5;
echo "f10 scope privato: mangling mai nella IC (key!=name al fill)\n";
class A10 { private array $p = [];
  public function w($v) { $this->p['k'] = $v; return $this->p; }
}
$j = new A10;
print_r($j->w(1));
print_r($j->w(2));
echo "f11 chiave illegale su prop-array (TypeError condiviso)\n";
class A11 { public $d = []; }
$k11 = new A11;
$k11->d['k'] = 1;
try {
    $k11->d[[]] = 2;
} catch (TypeError $t) {
    echo $t->getMessage(), "\n";
}
print_r($k11->d);
echo "f12 stessa cella IC, classe DIVERSA al secondo giro (miss pulito)\n";
class B12a { public $d = []; }
class B12b { public $d = []; }
function w12($o) { $o->d['k'] = get_class($o); return $o->d; }
print_r(w12(new B12a));
print_r(w12(new B12a));
print_r(w12(new B12b));
echo "f13 distruttore dell'elemento sovrascritto (timing)\n";
class D13 { public $n;
  public function __construct($n) { $this->n = $n; }
  public function __destruct() { echo "dtor {$this->n}\n"; }
}
class A13 { public $d = []; }
$m = new A13;
$m->d[0] = new D13('primo');
echo "prima\n";
$m->d[0] = new D13('secondo');
echo "dopo\n";
unset($m);
echo "f14 base \$this (FieldBase::This)\n";
class A14 { public $d = [];
  public function fill() { $this->d['a'] = 1; $this->d['a'] = 2; return $this->d; }
}
$n14 = new A14;
print_r($n14->fill());
echo "f15 valore d'espressione e chiave float (diag §3.21c)\n";
class A15 { public $d = []; }
$o15 = new A15;
var_dump($o15->d['k'] = 41);
$o15->d[1.7] = 'f';
var_dump($o15->d);
echo "fine\n";
