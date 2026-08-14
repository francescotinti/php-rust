<?php
// S-138 — fixtures semantiche leva FD1-ext RMW (parità BYTE vs oracle).
// Copre: hit caldo, chiave assente (warning), coercizioni chiave, entry-ref,
// overflow, op fuori perimetro (concat/div), incdec su stringa/float/max,
// nkeys≠1, hook/__set/readonly/visibilità, pre/post, base $this.
error_reporting(E_ALL);

class En { public int $id=0; public array $data=[]; }
$e = new En();

// 1: hit caldo += (chiave creata prima)
$e->data['k'] = 10; $e->data['k'] += 5; var_dump($e->data['k']);
// 2: chiave ASSENTE += (warning undefined key, null+1)
$r = $e->data['manca'] += 3; var_dump($r, $e->data['manca']);
// 3: chiave int-string normalizzata
$e->data['7'] = 1; $e->data[7] += 2; var_dump($e->data[7], isset($e->data['7']));
// 4: chiave float (deprecation/troncamento come oracle)
$e->data[3.7] = 100; $e->data[3.7] += 1; var_dump($e->data[3]);
// 5: entry per riferimento
$x = 50; $e->data['r'] = &$x; $e->data['r'] += 10; var_dump($x, $e->data['r']);
// 6: overflow int -> float
$e->data['o'] = PHP_INT_MAX; $e->data['o'] += 1; var_dump($e->data['o']);
// 7: concat .= (fuori perimetro)
$e->data['s'] = "ab"; $e->data['s'] .= "cd"; var_dump($e->data['s']);
// 8: div /= e per zero
$e->data['d'] = 10; $e->data['d'] /= 4; var_dump($e->data['d']);
try { $e->data['d'] /= 0; } catch (DivisionByZeroError $ex) { var_dump($ex->getMessage()); }
// 9: mul *= float
$e->data['m'] = 3; $e->data['m'] *= 2.5; var_dump($e->data['m']);
// 10: sub -=
$e->data['k'] -= 7; var_dump($e->data['k']);
// 11: ++/-- post e pre, valore d'uso
$e->data['i'] = 1; var_dump($e->data['i']++, $e->data['i'], ++$e->data['i'], $e->data['i']--);
// 12: ++ su chiave assente
var_dump($e->data['nuovo']++, $e->data['nuovo']);
// 13: ++ su stringa alfabetica e su float
$e->data['z'] = "az"; $e->data['z']++; var_dump($e->data['z']);
$e->data['f'] = 1.5; $e->data['f']++; var_dump($e->data['f']);
// 14: ++ a PHP_INT_MAX
$e->data['x'] = PHP_INT_MAX; $e->data['x']++; var_dump($e->data['x']);
// 15: nkeys=2 (annidato, fuori perimetro)
$e->data['n'] = ['a' => 1]; $e->data['n']['a'] += 4; var_dump($e->data['n']['a']);
// 16: prop tipata int (coercizione mantenuta) — attenzione: data è array, qui RMW su prop scalare via dim NON applicabile; copre il vicino: id += resta PropOpSet
$e->id += 2; var_dump($e->id);

// 17: classe con hook set sulla prop
class Hk { private array $b = [];
  public array $data { get => $this->b; set(array $v) { $this->b = $v; echo "HOOK-SET\n"; } }
}
$h = new Hk(); $arr = $h->data; // baseline
try { $h->data['k'] = 1; $h->data['k'] += 2; var_dump($h->data); } catch (Error $ex) { var_dump(get_class($ex), $ex->getMessage()); }

// 18: __set con prop assente (RMW su dim di prop inaccessibile)
class Ms { private array $hid = []; public array $seen = [];
  public function __get($n){ echo "GET:$n\n"; return $this->hid; }
  public function __set($n, $v){ echo "SET:$n\n"; $this->hid = $v; }
}
$m = new Ms();
$m->ghost['k'] = 5; var_dump($m->seen);
// 19: readonly con array
class Ro { public function __construct(public readonly array $data) {} }
$ro = new Ro(['k' => 1]);
try { $ro->data['k'] += 1; } catch (Error $ex) { var_dump($ex->getMessage()); }
// 20: base $this (dentro un metodo)
class Th { public array $data = ['k' => 5];
  public function bump(): int { $this->data['k'] += 3; $this->data['k']++; return $this->data['k']; }
}
var_dump((new Th())->bump());
// 21: due RMW sulla stessa statement-famiglia con IC condiviso per sito — classi DIVERSE sullo stesso sito (poly)
function poly(object $o): void { $o->data['k'] += 1; }
class P1 { public array $data = ['k' => 1]; }
class P2 { public array $data = ['k' => 100]; }
$p1 = new P1(); $p2 = new P2();
poly($p1); poly($p2); poly($p1); poly($p2);
var_dump($p1->data['k'], $p2->data['k']);
echo "FINE\n";
