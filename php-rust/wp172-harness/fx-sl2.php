<?php
// fx-sl2.php — fixture BILATERALE (oracle == pin byte-id) per la leva L-SL2 «forma
// sigillata Long» fetta 2 = prop (S-172, criterio p.6): P1 = bigramma fuso
// `$o->x = $o->y OP C` (PropGetSlotRecv + BinaryTCPropSetPop), P2 = `$s OP= $o->x`
// (PropGetSlot + BinarySTDst). Ogni cammino che ESCE dal dominio Long deve cadere al
// corpo esatto: overflow, shift fuori range, Div/Mod/Pow/Concat, prop/slot Double,
// stringa numerica, null, bool, Ref, typed int/float, readonly, hook set, __set,
// private in scope, ereditata, dinamica.
// OGNI forma P1 gira in un loop a 2 iterazioni: la 1ª riempie l'IC del sito (get e set),
// la 2ª entra nel probe sigillato (revisione S-172 rilievo 4: single-shot = IC fredda =
// presidio vuoto). P2 non ha IC: single-shot basta.
// SENZA forme che emettono diagnostici (quelle stanno in fx-sl2-div.php).
function show($label, $v) { echo $label, ': '; var_dump($v); }
class P { public $x = 0; public $y = 1; }
class Sub extends P {}
class T { public int $x = 0; public int $y = 1; public float $f = 1.5; }
class T2 { public float $f = 1; public int $y = 3; }
class RO { public function __construct(public readonly int $x = 0, public int $y = 1) {} }
class RO3 { public readonly int $x; public int $y = 1; public function set() { $o = $this; $o->x = $o->y + 1; } }
class PV { private int $x = 0; public int $y = 1; public function run() { $o = $this; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } return $o->x; } }
class H { public int $log = 0; public int $y = 1; public int $x = 0 { set { $this->log++; $this->x = $value * 2; } } }
class MS { public $y = 1; private $d = []; public function __set($n, $v) { $this->d[$n] = $v * 10; } public function __get($n) { return $this->d[$n] ?? null; } }
function g($v) { return $v; }

// --- P1: $o->x = $o->y OP C (bigramma fuso, dominio Long e uscite; 2 iterazioni per forma) ---
$o = new P; for ($i = 0; $i < 100; $i++) { $o->x = $o->y + 1; $o->y = $o->x + 1; } show('p1-loop100', [$o->x, $o->y]);
$o = new P; $o->y = PHP_INT_MAX - 3; for ($i = 0; $i < 5; $i++) { $o->x = $o->y + 1; if ($i === 1) show('p1-loop-overflow-step', $o->x); $o->y = $o->x; } show('p1-loop-overflow', $o->y); // -step (S-173): il float finale satura, il +1 del mutante non si vede
$o = new P; $o->y = 7;
for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 3; } show('p1-add', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y - 10; } show('p1-sub', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y * 3; } show('p1-mul', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y & 3; } show('p1-and', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y | 8; } show('p1-or', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y ^ 5; } show('p1-xor', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y << 3; } show('p1-shl', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y >> 1; } show('p1-shr', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y << 64; } show('p1-shl-64', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y >> 70; } show('p1-shr-70', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y << 63; } show('p1-shl-63', $o->x);
$o->y = -7;
for ($k = 0; $k < 2; $k++) { $o->x = $o->y << 5; } show('p1-shl-neg-l', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y >> 2; } show('p1-shr-neg-l', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y >> 70; } show('p1-shr-70-neg', $o->x);
$o->y = 7;
for ($k = 0; $k < 2; $k++) { $o->x = $o->y / 2; } show('p1-div-inexact', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y / 7; } show('p1-div-exact', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y % 4; } show('p1-mod', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y ** 2; } show('p1-pow', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y ** 70; } show('p1-pow-overflow', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y . 'a'; } show('p1-concat', $o->x);
for ($k = 0; $k < 2; $k++) { try { $o->x = $o->y / 0; } catch (DivisionByZeroError $e) { echo "p1-div-zero#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
for ($k = 0; $k < 2; $k++) { try { $o->x = $o->y % 0; } catch (DivisionByZeroError $e) { echo "p1-mod-zero#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$o->y = PHP_INT_MIN; for ($k = 0; $k < 2; $k++) { $o->x = $o->y % -1; } show('p1-mod-min-neg1', $o->x);
$o->y = PHP_INT_MAX; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-add-overflow', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = $o->y * 2; } show('p1-mul-overflow', $o->x);
$o->y = PHP_INT_MIN; for ($k = 0; $k < 2; $k++) { $o->x = $o->y - 1; } show('p1-sub-overflow', $o->x);
$o->y = 2.5; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-double-src', $o->x);
$o->y = "5"; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-numstr-src', $o->x);
$o->y = null; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-null-src', $o->x);
$o->y = true; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-bool-src', $o->x);
$o->y = 4; $o->x = 1.5; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-double-dst', $o->x);
$o->x = "s"; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-str-dst', $o->x);
$o->x = null; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-null-dst', $o->x);
$o->x = [1]; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-arr-dst', $o->x);
$o->x = 0; $r = &$o->x; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p1-dst-ref', $o->x); show('p1-dst-ref-alias', $r); unset($r);
$o2 = new P; $q = &$o2->y; $o2->y = 4; for ($k = 0; $k < 2; $k++) { $o2->x = $o2->y + 1; } show('p1-src-ref', $o2->x); unset($q);
$o3 = new P; $o3->x = 5; for ($k = 0; $k < 2; $k++) { $o3->x = $o3->x + 1; } show('p1-self', $o3->x);
$a = new P; $b = new P; $b->y = 9; for ($k = 0; $k < 2; $k++) { $a->x = $b->y + 1; } show('p1-two-objs', [$a->x, $b->x]);
$sb = new Sub; $sb->y = 8; for ($k = 0; $k < 2; $k++) { $sb->x = $sb->y + 2; } show('p1-inherited', $sb->x);
$t = new T; $t->y = 5; for ($k = 0; $k < 2; $k++) { $t->x = $t->y + 1; } show('p1-typed-int', $t->x);
$t->y = PHP_INT_MAX; for ($k = 0; $k < 2; $k++) { try { $t->x = $t->y + 1; show("p1-typed-int-overflow#$k", $t->x); } catch (TypeError $e) { echo "p1-typed-int-overflow#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$t->y = 3; for ($k = 0; $k < 2; $k++) { $t->f = $t->y + 1; } show('p1-typed-float-dst', $t->f);
$t->f = 2.0; for ($k = 0; $k < 2; $k++) { $t->x = $t->f + 1; } show('p1-typed-int-from-float', $t->x);
$t2 = new T2; for ($k = 0; $k < 2; $k++) { $t2->f = $t2->y + 1; } show('p1-typed-float-from-long', $t2->f); // il default `float $f = 1` (int(1) vs float(1)) sta in fx-sl2-div (§3.30)
$ro = new RO(5, 7); for ($k = 0; $k < 2; $k++) { try { $ro->x = $ro->y + 1; show("p1-readonly#$k", $ro->x); } catch (Error $e) { echo "p1-readonly#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$ro3 = new RO3; for ($k = 0; $k < 2; $k++) { try { $ro3->set(); show("p1-readonly-inscope#$k", $ro3->x); } catch (Error $e) { echo "p1-readonly-inscope#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$pv = new PV; show('p1-private-inscope', $pv->run());
for ($k = 0; $k < 2; $k++) { try { $pv->x = $pv->y + 1; show("p1-private-outscope#$k", 'written'); } catch (Error $e) { echo "p1-private-outscope#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$h = new H; $h->y = 5; for ($k = 0; $k < 2; $k++) { $h->x = $h->y + 1; } show('p1-hook-set', [$h->x, $h->log]);
$ms = new MS; for ($k = 0; $k < 2; $k++) { $ms->x = $ms->y + 1; } show('p1-magic-set', $ms->x);
$d = new stdClass; $d->y = 3; for ($k = 0; $k < 2; $k++) { $d->x = $d->y + 1; } show('p1-dynamic', $d->x);
for ($k = 0; $k < 2; $k++) { $d->x = $d->x + 1; } show('p1-dynamic-self', $d->x);

// --- P2: $s OP= $o->x (PropGetSlot + BinarySTDst) ---
$o = new P; $o->x = 7; $s = 10;
$s += $o->x; show('p2-add', $s);
$s -= $o->x; show('p2-sub', $s);
$s *= $o->x; show('p2-mul', $s);
$s = 70; $s &= $o->x; show('p2-and', $s); // reset (S-173): 85&7+1 == 70&7 coincidenza del mutante
$s |= $o->x; show('p2-or', $s);
$s ^= $o->x; show('p2-xor', $s);
$s = 3; $s <<= $o->x; show('p2-shl', $s);
$s >>= $o->x; show('p2-shr', $s);
$o->x = 64; $s = -3; $s <<= $o->x; show('p2-shl-64', $s); $s = -3; $s >>= $o->x; show('p2-shr-64', $s);
$o->x = 5; $s = -7; $s <<= $o->x; show('p2-shl-neg-l', $s); $s = -7; $s >>= $o->x; show('p2-shr-neg-l', $s);
$o->x = -1; try { $s = 3; $s <<= $o->x; } catch (ArithmeticError $e) { echo "p2-shl-neg: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$o->x = 3; $s = 10; $s /= $o->x; show('p2-div', $s); $s = 9; $s /= $o->x; show('p2-div-exact', $s); $s = 10; $s %= $o->x; show('p2-mod', $s); $s = 2; $s **= $o->x; show('p2-pow', $s);
$s = "ab"; $s .= $o->x; show('p2-concat', $s);
$o->x = 0; try { $s = 1; $s /= $o->x; } catch (DivisionByZeroError $e) { echo "p2-div-zero: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$o->x = 1; $s = PHP_INT_MAX; $s += $o->x; show('p2-add-overflow', $s);
$o->x = PHP_INT_MIN; $s = 5; $s -= $o->x; show('p2-sub-overflow', $s);
$o->x = PHP_INT_MAX; $s = 3; $s *= $o->x; show('p2-mul-overflow', $s);
$o->x = 2; $s = 1.5; $s += $o->x; show('p2-double-dst', $s);
$s = "10"; $s += $o->x; show('p2-numstr-dst', $s);
$s = null; $s += $o->x; show('p2-null-dst', $s);
$s = true; $s += $o->x; show('p2-bool-dst', $s);
$o->x = 2.5; $s = 1; $s += $o->x; show('p2-double-src', $s);
$o->x = "7"; $s = 1; $s += $o->x; show('p2-numstr-src', $s);
$o->x = null; $s = 1; $s += $o->x; show('p2-null-src', $s);
$o->x = 3; $ds = 1; $rr = &$ds; $ds += $o->x; show('p2-dst-ref', $ds); show('p2-dst-ref-alias', $rr); unset($rr);
$tp = new T; $tp->x = 1; $tr = &$tp->x; $tr += $o->x; show('p2-typed-ref', $tp->x);
$tp->x = PHP_INT_MAX; try { $tr += $o->x; show('p2-typed-ref-overflow', $tp->x); } catch (TypeError $e) { echo "p2-typed-ref-overflow: ", get_class($e), ' ', $e->getMessage(), "\n"; } unset($tr);
$o->x = 4; $s = 0; for ($i = 0; $i < 100; $i++) { $s += $o->x; } show('p2-loop100', $s);
$arr = [5]; $s = 1; $s += $arr[0]; show('p2-arr-src', $s);
$s = 1; $s += g(3); show('p2-call-src', $s);
$s = 1; $s += g(2.5); show('p2-call-double-src', $s);
$o->x = 5; $s = PHP_INT_MAX - 10; for ($i = 0; $i < 4; $i++) { $s += $o->x; if ($i === 0) show('p2-loop-overflow-step', $s); } show('p2-loop-overflow', $s); // -step (S-173): il float finale satura
// --- micro prop.php in piccolo: le due forme insieme ---
$o = new P; $s = 0; for ($i = 0; $i < 1000; $i++) { $o->x = $o->y + 1; $s += $o->x; } show('prop-micro-1000', $s);
echo "FX-SL2 DONE\n";
