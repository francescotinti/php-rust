<?php
// S-194 P1 behaviour tooth: isset()/empty() of mixed property/index paths, every site warm. Expected = the oracle's output.
// Generato con assistenza AI (Claude Fable 5.2).
class AA implements ArrayAccess { public $d = ['k' => ['z' => 1], 'n' => null];
  function offsetExists($o): bool { echo "[exists $o]"; return isset($this->d[$o]); } function offsetGet($o): mixed { echo "[get $o]"; return $this->d[$o] ?? null; }
  function offsetSet($o, $v): void {} function offsetUnset($o): void {} }
#[AllowDynamicProperties] class P { public $arr = ['a' => 1, 'n' => null, 'z' => 0, 's' => '0', 'e' => [], 'f' => 0.0, 'sub' => ['x' => ['y' => 2]], 'str' => 'hello'];
  public $q; public $nul = null; public ?int $typed; public int $ti = 5; private $priv = ['k' => 1]; protected $prot = ['k' => 1]; public $aa; public $obj;
  function inPriv($k) { return [isset($this->priv[$k]), empty($this->priv[$k]), isset($this->prot[$k]), isset($this->q->priv[$k])]; } }
class Kid extends P { private $priv = ['other' => 1]; function kidPriv($k) { return [isset($this->priv[$k]), isset($this->prot[$k])]; } }
class Mg { public $decl = ['k' => 1]; public $gone = ['k' => 1]; private $hid = ['k' => 1]; public $data = ['virt' => ['k' => 7], 'nul' => null];
  function __isset($n) { echo "[__isset $n]"; return isset($this->data[$n]); } function __get($n) { echo "[__get $n]"; return $this->data[$n] ?? null; } }
class G { public $decl = ['k' => 1]; function __get($n) { echo "[__get $n]"; return ['k' => 1]; } }
class H { public array $h = ['k' => 1] { get { return $this->h; } } public $plain = ['k' => 1]; }
function b($v) { return var_export($v, true); }
function row($l, ...$v) { echo $l, ': ', implode(' ', array_map('b', $v)), "\n"; }
$p = new P; $p->q = new P; $p->q->q = new Kid; $p->aa = new AA; $p->obj = new stdClass; $p->obj->dyn = ['k' => 1]; $p->dynp = ['k' => null, 'j' => 2];
$m = new Mg; unset($m->gone); $g = new G; $h = new H; $r = ['k' => 1]; $p->ref = &$r; $n = null; $arr = ['o' => $p];
for ($i = 0; $i < 3; $i++) {
  row('plain', isset($p->arr['a']), isset($p->arr['n']), isset($p->arr['missing']), isset($p->arr['sub']['x']['y']), isset($p->arr['sub']['x']['w']), isset($p->arr['a']['b']));
  row('empty', empty($p->arr['a']), empty($p->arr['n']), empty($p->arr['z']), empty($p->arr['s']), empty($p->arr['e']), empty($p->arr['f']), empty($p->arr['missing']), empty($p->arr['sub']['x']));
  row('nested', isset($p->q->arr['a']), isset($p->q->q->arr['a']), isset($p->q->nul), isset($p->q->nul->x), isset($p->nul['k']), isset($p->q->q->q->arr), isset($p->nope->x), isset($p->nope['k']));
  row('typed', isset($p->typed), isset($p->q->typed), empty($p->q->typed), isset($p->q->ti), empty($p->q->ti), isset($p->ti['k']));
  row('vis-out', isset($p->priv['k']), empty($p->priv['k']), isset($p->prot['k']), isset($p->q->priv['k']), isset($p->q->prot));
  row('vis-in', ...$p->inPriv('k'), ...$p->inPriv('other'));
  row('vis-kid', ...$p->q->q->kidPriv('k'), ...$p->q->q->kidPriv('other'), ...$p->q->q->inPriv('k'));
  row('dyn', isset($p->dynp['k']), isset($p->dynp['j']), empty($p->dynp['j']), isset($p->obj->dyn['k']), isset($p->obj->nope['k']), empty($p->obj->dyn));
  row('ref', isset($p->ref['k']), isset($p->ref['x']), empty($p->ref['k']));
  row('str', isset($p->arr['str'][1]), isset($p->arr['str'][9]), isset($p->arr['str']['x']), empty($p->arr['str'][0]));
  row('key', isset($p->arr[true]), isset($p->arr['sub'][2.0]), isset($p->arr[2]), isset($p->arr['2']));
  row('aa1', isset($p->aa['k']), isset($p->aa['n']), empty($p->aa['k']));
  row('aa2', isset($p->aa['k']['z']), isset($p->aa['k']['w']), empty($p->aa['k']['z']));
  row('magic-decl', isset($m->decl['k']), isset($m->decl['x']), empty($m->decl['k']));
  row('magic-virt', isset($m->virt['k']), isset($m->virt['x']), empty($m->virt['k']), isset($m->nul['k']), isset($m->none['k']), isset($m->virt), isset($m->none));
  row('magic-gone', isset($m->gone['k']), isset($m->hid['k']), empty($m->hid['k']));
  row('get-only', isset($g->decl['k']), isset($g->virt['k']), isset($g->virt), empty($g->virt['k']), empty($g->virt));
  row('hook', isset($h->h['k']), isset($h->plain['k']), empty($h->h['k']));
  row('nonobj', isset($n->a['k']), isset($arr['o']->arr['a']), isset($arr['x']->arr['a']), empty($n->a->b));
  row('leaf', isset($p->q->q), empty($p->q->q), empty($p->q->obj), isset($p->q->arr), empty($p->q->arr));
}
