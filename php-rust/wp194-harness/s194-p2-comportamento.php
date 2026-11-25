<?php
// S-194 P2 behaviour tooth: isset()/empty()/assignment with ONE key on an array variable. Expected = the oracle's output.
// Generato con assistenza AI (Claude Fable 5.2).
class AA implements ArrayAccess { public $d = ['k' => 1, 'n' => null];
  function offsetExists($o): bool { echo "[exists $o]"; return isset($this->d[$o]); } function offsetGet($o): mixed { echo "[get $o]"; return $this->d[$o] ?? null; }
  function offsetSet($o, $v): void { echo "[set " . var_export($o, true) . "]"; $this->d[$o] = $v; } function offsetUnset($o): void {} }
class D { function __construct(public $n) {} function __destruct() { echo "[dtor {$this->n}]"; } }
function b($v) { return var_export($v, true); }
function row($l, ...$v) { echo $l, ': ', implode(' ', array_map('b', $v)), "\n"; }
$G = ['g' => 1, 'n' => null, 5 => 'five'];
function useGlobal($k) { global $G; $r = [isset($G[$k]), empty($G[$k])]; $G['w' . $k] = $k; return $r; }
function useStatic($k) { static $S = ['s' => 1]; $r = isset($S[$k]); $S[$k] = true; return $r; }
function byRef(array &$a, $k) { $r = [isset($a[$k]), empty($a[$k])]; $a[$k] = 'set'; return $r; }
function run() {
  $a = ['a' => 1, 'n' => null, 'z' => 0, 's' => '0', 'e' => [], 5 => 'five', '' => 'empty', 1 => 'one', 0 => 'zero', 'obj' => new stdClass];
  $x = 7; $a['r'] = &$x; $nul = null; $y = null; $a['rn'] = &$y; $str = 'hello'; $int = 5; $aa = new AA;
  for ($i = 0; $i < 3; $i++) {
    row('isset', isset($a['a']), isset($a['n']), isset($a['missing']), isset($a[5]), isset($a['5']), isset($a[6]), isset($a['r']), isset($a['rn']), isset($a['obj']), isset($a['']));
    row('empty', empty($a['a']), empty($a['n']), empty($a['z']), empty($a['s']), empty($a['e']), empty($a['missing']), empty($a['r']), empty($a['rn']), empty($a['obj']));
    row('keys', isset($a[true]), isset($a[false]), isset($a['1']), isset($a['01']), isset($a[1.0]), empty($a[true]), isset($a[$i]), isset($a["$i"]));
    row('bases', isset($nul['k']), empty($nul['k']), isset($str[1]), isset($str[9]), empty($str[0]), isset($int['k']), isset($undef['k']), empty($undef['k']));
    row('aa', isset($aa['k']), isset($aa['n']), empty($aa['k']));
    row('super', isset($_SERVER['argv']), isset($_SERVER['nope']), empty($_GET['x']), isset($GLOBALS['G']));
    row('global', ...useGlobal('g'), ...useGlobal('q' . $i), ...useGlobal(5));
    row('static', useStatic('s'), useStatic('t' . $i), useStatic('t' . $i));
    $arr = ['p' => 1]; $r1 = byRef($arr, 'p'); $r2 = byRef($arr, 'new'); row('byref', $r1, $r2, $arr);
  }
  $c = $a; $a['a'] = 2; $a['new'] = 3; $a[5] = 'FIVE'; $a['5'] = 'again'; $a[7] = 'seven'; $a['r'] = 99; row('cow', $c['a'], $a['a'], isset($c['new']), $a[5], $x, $c['r'], count($a) - count($c));
  $v = ($a['chain'] = 'val'); row('value', $v, $a['chain']);
  $n = null; $n['k'] = 1; $u['k'] = 2; $f = false; row('vivify', $n, $u);
  $aa['w'] = 5; echo "\n"; row('aa-set', $aa->d['w']);
  $d = ['o' => new D(1)]; $d['o'] = new D(2); echo "|"; $d['o'] = 3; echo "|\n";
  $s = 'abc'; $s[1] = 'X'; row('stroff', $s);
  $m = [1, 2, 3]; foreach ($m as $k => $val) { $m[$k] = $val * 2; } row('loop', $m);
  $t = [[1]]; $t[0] = 'flat'; $w = ['a' => ['b' => 1]]; $w2 = $w; $w['a'] = 'x'; row('nest', $t, $w2['a']['b'], $w);
}
run();
$top = ['k' => 1]; row('top', isset($top['k']), empty($top['k']), isset($top['x'])); $top['x'] = 2; row('top2', $top);
