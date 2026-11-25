<?php
// S-194 P4 behaviour tooth: readonly properties written through warm sites. Expected = the oracle's output.
// Generato con assistenza AI (Claude Fable 5.2).
class Pt { function __construct(public readonly int $x, protected readonly ?string $s = null, private readonly array $a = []) {}
  function again($v) { $this->x = $v; } function s() { return $this->s; } function a() { return $this->a; } function setS($v) { $this->s = $v; } }
class Kid extends Pt { public readonly float $f; function __construct(int $x, $f = 1.5) { parent::__construct($x, "k$x", [$x]); $this->f = $f; } function kidAgain() { $this->s = 'no'; } }
class Late { public readonly int $n; public readonly string $t; function init($n) { $this->n = $n; return $this; } function drop() { unset($this->n); return $this; }
  function setT($v) { $this->t = $v; return $this; } }
class Cl { function __construct(public readonly array $items, public readonly int $gen = 0) {} function __clone() { $this->gen = $this->gen + 1; }
  function twice() { $c = clone $this; return $c; } }
class Cl2 { function __construct(public readonly int $v) {} function with($v) { $c = clone $this; $c->v = $v; return $c; } }
class Bad { function __clone() { $this->v = 1; $this->v = 2; } function __construct(public readonly int $v = 0) {} }
readonly class RC { function __construct(public int $a, public string $b) {} }
class Av { function __construct(public private(set) readonly int $p, public public(set) readonly int $q = 0) {} function setP($v) { $this->p = $v; } }
class Str { function __toString() { return 'str'; } }
function t($f) { try { $r = $f(); return is_object($r) ? 'ok' : $r; } catch (Throwable $e) { return get_class($e) . ': ' . $e->getMessage(); } }
function b($v) { return is_string($v) ? $v : json_encode($v); }
function row($l, ...$v) { echo $l, ': ', implode(' | ', array_map('b', $v)), "\n"; }
for ($i = 0; $i < 4; $i++) {
  $p = new Pt($i, "s$i", [$i]); $k = new Kid($i, $i); $q = new Pt("7");
  row('ctor', $p->x, $p->s(), $p->a(), $k->x, $k->s(), $k->a(), $k->f, $q->x, $q->s(), $q->a());
  row('again', t(fn() => $p->again(9)), t(fn() => $k->again(9)), t(fn() => $k->kidAgain()), t(function () use ($p) { $p->x = 1; }), t(function () use ($k) { $k->f = 2.0; }), $p->x);
  row('outside', t(function () { $l = new Late; $l->n = 5; return $l->n; }), t(fn() => (new Late)->init($i)->n), t(fn() => (new Late)->init($i)->init(2)->n), t(fn() => (new Late)->drop()->init($i)->n), t(fn() => (new Late)->init(1)->drop()));
  row('coerce', t(fn() => (new Late)->init('12')->n), t(fn() => (new Late)->init('x')), t(function () use ($i) { $l = new Late; try { $l->init('bad'); } catch (TypeError $e) {} return $l->init($i)->n; }), t(fn() => (new Late)->setT(new Str)->t), t(fn() => (new Late)->setT(5)->t), t(fn() => new Pt([])));
  $c = new Cl([$i]); $d = $c->twice(); $e = $d->twice(); row('clone', $c->gen, $d->gen, $e->gen, $d->items, t(function () use ($d) { $d->gen = 9; }));
  $w = new Cl2($i); row('clone2', t(fn() => $w->with(5)), $w->v, t(fn() => clone new Bad));
  $r = new RC($i, "b$i"); row('roclass', $r->a, $r->b, t(function () use ($r) { $r->a = 1; }));
  $a = new Av($i); row('aviz', $a->p, t(fn() => $a->setP(3)), t(function () use ($a) { $a->p = 4; }), t(function () { $a = new Av(1); return 'q' . $a->q; }), t(function () use ($a) { $a->q = 5; }));
}
