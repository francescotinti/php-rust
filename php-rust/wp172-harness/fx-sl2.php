<?php
// fx-sl2.php — fixture BILATERALE (oracle == pin byte-id) per la leva L-SL2 «forma
// sigillata Long» fetta 2 = prop (S-172, criterio p.6): P1 = bigramma fuso
// `$o->x = $o->y OP C` (PropGetSlotRecv + BinaryTCPropSetPop), P2 = `$s OP= $o->x`
// (PropGetSlot + BinarySTDst). Ogni cammino che ESCE dal dominio Long deve cadere al
// corpo esatto: overflow, shift fuori range, Div/Mod/Pow/Concat, prop/slot Double,
// stringa numerica, null, bool, Ref, typed int/float, readonly, hook set, dinamica.
// SENZA forme che emettono diagnostici (quelle stanno in fx-sl2-div.php).
function show($label, $v) { echo $label, ': '; var_dump($v); }
class P { public $x = 0; public $y = 1; }
class T { public int $x = 0; public int $y = 1; public float $f = 1.5; }
class RO { public function __construct(public readonly int $x = 0, public int $y = 1) {} }
class H { public int $log = 0; public int $y = 1; public int $x = 0 { set { $this->log++; $this->x = $value * 2; } } }
function g($v) { return $v; }

// --- P1: $o->x = $o->y OP C (bigramma fuso, dominio Long e uscite) ---
$o = new P; for ($i = 0; $i < 100; $i++) { $o->x = $o->y + 1; $o->y = $o->x + 1; } show('p1-loop100', [$o->x, $o->y]);
$o = new P; $o->y = PHP_INT_MAX - 3; for ($i = 0; $i < 5; $i++) { $o->x = $o->y + 1; $o->y = $o->x; } show('p1-loop-overflow', $o->y);
$o = new P; $o->y = 7;
$o->x = $o->y + 3; show('p1-add', $o->x);
$o->x = $o->y - 10; show('p1-sub', $o->x);
$o->x = $o->y * 3; show('p1-mul', $o->x);
$o->x = $o->y & 3; show('p1-and', $o->x);
$o->x = $o->y | 8; show('p1-or', $o->x);
$o->x = $o->y ^ 5; show('p1-xor', $o->x);
$o->x = $o->y << 3; show('p1-shl', $o->x);
$o->x = $o->y >> 1; show('p1-shr', $o->x);
$o->x = $o->y << 64; show('p1-shl-64', $o->x);
$o->x = $o->y >> 70; show('p1-shr-70', $o->x);
$o->x = $o->y << 63; show('p1-shl-63', $o->x);
$o->y = -7; $o->x = $o->y << 5; show('p1-shl-neg-l', $o->x); $o->x = $o->y >> 2; show('p1-shr-neg-l', $o->x); $o->x = $o->y >> 70; show('p1-shr-70-neg', $o->x);
$o->y = 7;
$o->x = $o->y / 2; show('p1-div-inexact', $o->x);
$o->x = $o->y / 7; show('p1-div-exact', $o->x);
$o->x = $o->y % 4; show('p1-mod', $o->x);
$o->x = $o->y ** 2; show('p1-pow', $o->x);
$o->x = $o->y ** 70; show('p1-pow-overflow', $o->x);
$o->x = $o->y . 'a'; show('p1-concat', $o->x);
try { $o->x = $o->y / 0; } catch (DivisionByZeroError $e) { echo "p1-div-zero: ", get_class($e), ' ', $e->getMessage(), "\n"; }
try { $o->x = $o->y % 0; } catch (DivisionByZeroError $e) { echo "p1-mod-zero: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$o->y = PHP_INT_MIN; $o->x = $o->y % -1; show('p1-mod-min-neg1', $o->x);
$o->y = PHP_INT_MAX; $o->x = $o->y + 1; show('p1-add-overflow', $o->x);
$o->x = $o->y * 2; show('p1-mul-overflow', $o->x);
$o->y = PHP_INT_MIN; $o->x = $o->y - 1; show('p1-sub-overflow', $o->x);
$o->y = 2.5; $o->x = $o->y + 1; show('p1-double-src', $o->x);
$o->y = "5"; $o->x = $o->y + 1; show('p1-numstr-src', $o->x);
$o->y = null; $o->x = $o->y + 1; show('p1-null-src', $o->x);
$o->y = true; $o->x = $o->y + 1; show('p1-bool-src', $o->x);
$o->y = 4; $o->x = 1.5; $o->x = $o->y + 1; show('p1-double-dst', $o->x);
$o->x = "s"; $o->x = $o->y + 1; show('p1-str-dst', $o->x);
$o->x = null; $o->x = $o->y + 1; show('p1-null-dst', $o->x);
$o->x = [1]; $o->x = $o->y + 1; show('p1-arr-dst', $o->x);
$o->x = 0; $r = &$o->x; $o->x = $o->y + 1; show('p1-dst-ref', $o->x); show('p1-dst-ref-alias', $r); unset($r);
$o2 = new P; $q = &$o2->y; $o2->y = 4; $o2->x = $o2->y + 1; show('p1-src-ref', $o2->x); unset($q);
$o3 = new P; $o3->x = 5; $o3->x = $o3->x + 1; show('p1-self', $o3->x);
$a = new P; $b = new P; $b->y = 9; $a->x = $b->y + 1; show('p1-two-objs', [$a->x, $b->x]);
$t = new T; $t->y = 5; $t->x = $t->y + 1; show('p1-typed-int', $t->x);
$t->y = PHP_INT_MAX; try { $t->x = $t->y + 1; show('p1-typed-int-overflow', $t->x); } catch (TypeError $e) { echo "p1-typed-int-overflow: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$t->y = 3; $t->f = $t->y + 1; show('p1-typed-float-dst', $t->f);
$t->f = 2.0; $t->x = $t->f + 1; show('p1-typed-int-from-float', $t->x);
$ro = new RO(5, 7); try { $ro->x = $ro->y + 1; show('p1-readonly', $ro->x); } catch (Error $e) { echo "p1-readonly: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$h = new H; $h->y = 5; $h->x = $h->y + 1; show('p1-hook-set', [$h->x, $h->log]);
$d = new stdClass; $d->y = 3; $d->x = $d->y + 1; show('p1-dynamic', $d->x);
$d->x = $d->x + 1; show('p1-dynamic-self', $d->x);

// --- P2: $s OP= $o->x (PropGetSlot + BinarySTDst) ---
$o = new P; $o->x = 7; $s = 10;
$s += $o->x; show('p2-add', $s);
$s -= $o->x; show('p2-sub', $s);
$s *= $o->x; show('p2-mul', $s);
$s &= $o->x; show('p2-and', $s);
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
$o->x = 5; $s = PHP_INT_MAX - 10; for ($i = 0; $i < 4; $i++) { $s += $o->x; } show('p2-loop-overflow', $s);
// --- micro prop.php in piccolo: le due forme insieme ---
$o = new P; $s = 0; for ($i = 0; $i < 1000; $i++) { $o->x = $o->y + 1; $s += $o->x; } show('prop-micro-1000', $s);
echo "FX-SL2 DONE\n";
