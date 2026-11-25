<?php
// S-194 P3 behaviour tooth: writes to private / protected properties through warm sites. Expected = the oracle's output.
// Generato con assistenza AI (Claude Fable 5.2).
class Par { private $p = 'par'; protected $q = 'q'; private $arr = ['a' => 1]; public $pub = 0;
  function setP($v) { $this->p = $v; return $this; } function getP() { return $this->p; }
  function setQ($v) { $this->q = $v; } function getQ() { return $this->q; }
  function put($k, $v) { $this->arr[$k] = $v; } function add($k, $v) { $this->arr[$k] += $v; } function inc($k) { return $this->arr[$k]++; } function app($v) { $this->arr[] = $v; }
  function arr() { return $this->arr; } function setOn($o, $v) { $o->p = $v; } }
class Kid extends Par { private $p = 'kid'; function kidSetP($v) { $this->p = $v; } function kidGetP() { return $this->p; } function kidSetQ($v) { $this->q = $v; } }
class Ty { private int $n = 0; private ?string $s = null; private array $a = []; private float $f = 0.0;
  function setN($v) { $this->n = $v; return $this->n; } function setS($v) { $this->s = $v; return $this->s; } function setF($v) { $this->f = $v; return $this->f; }
  function putA($k, $v) { $this->a[$k] = $v; return $this->a; } function dropS() { unset($this->s); } function hasS() { return isset($this->s); } }
class Killer { function __construct(public $victim) {} function __toString() { $this->victim->dropS(); return 'killed'; } }
class Mg { private $hid = 1; public $vis = 2; public $log = [];
  function __set($n, $v) { $this->log[] = "__set($n)"; } function inSet($v) { $this->hid = $v; return $this->hid; } function inNew($v) { $this->fresh = $v; } }
class Ro { function __construct(private readonly int $x, public private(set) int $y = 0) {} function again($v) { $this->x = $v; } function setY($v) { $this->y = $v; return $this->y; } }
function t($f) { try { return $f(); } catch (Throwable $e) { return get_class($e) . ': ' . $e->getMessage(); } }
function b($v) { return is_string($v) ? $v : var_export($v, true); }
function row($l, ...$v) { echo $l, ': ', implode(' | ', array_map('b', $v)), "\n"; }
$par = new Par; $kid = new Kid; $ty = new Ty; $mg = new Mg;
for ($i = 0; $i < 3; $i++) {
  foreach ([$par, $kid, $par, $kid] as $o) { $o->setP("p$i"); $o->setQ("q$i"); $o->put("k$i", $i); $o->add('a', 10); $o->inc('a'); $o->app($i); }
  row('inside', $par->getP(), $kid->getP(), $kid->kidGetP(), $par->getQ(), $kid->getQ(), json_encode($par->arr()), json_encode($kid->arr()));
  $kid->kidSetP("kp$i"); $kid->kidSetQ("kq$i"); row('kid', $kid->getP(), $kid->kidGetP(), $kid->getQ());
  $par->setOn($par, "on$i"); $par->setOn($kid, "onk$i"); row('other', $par->getP(), $kid->getP(), $kid->kidGetP());
  row('outside', t(function () use ($par) { $par->p = 1; return 'no error'; }), t(function () use ($par) { $par->q = 1; return 'no error'; }), t(function () use ($par) { $par->arr['x'] = 1; return 'no error'; }));
  $w = function ($v) { $this->p = $v; return $this->p; };
  foreach ([[$par, Par::class], [$kid, Kid::class], [$kid, Par::class], [$par, Par::class]] as [$o, $sc]) { echo t(fn() => Closure::bind($w, $o, $sc)("b$i")), ' '; } echo $par->getP(), ' ', $kid->getP(), ' ', $kid->kidGetP(), "\n";
  row('typed', $ty->setN('5'), $ty->setN(7.0), t(fn() => $ty->setN('x')), $ty->setS(12), $ty->setS(null), $ty->setF(3), t(fn() => $ty->putA('k', $i)), t(fn() => $ty->setN(null)));
  row('kill', t(fn() => $ty->setS(new Killer($ty))), $ty->hasS(), $ty->setS('back'), $ty->hasS());
  $ty->dropS(); row('unset', $ty->hasS(), $ty->setS("again$i"), $ty->hasS());
  $mg->hid = 9; $mg->vis = 3; $mg->other = 4; row('magic', $mg->inSet($i), $mg->vis, json_encode($mg->log)); $mg->log = [];
  $ro = new Ro($i, 1); row('readonly', t(function () use ($ro) { $ro->again(5); return 'no error'; }), $ro->setY(8), t(function () use ($ro) { $ro->y = 9; return 'no error'; }));
}
$r = (function &() { return $this->p; })->call($par); $r = 'via-ref'; $par->setP('direct'); row('ref', $r, $par->getP());
$mg->inNew(1); row('magic-in', json_encode($mg->log));
