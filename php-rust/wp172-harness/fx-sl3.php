<?php
// fx-sl3.php — fixture BILATERALE (oracle == pin byte-id) per la leva L-SL2 fetta 3 = prop
// residuo (S-173, criterio p.6): P3 = peephole `$s OP= $o->x` (PropGetSlot + BinarySTDst,
// nessun push/pop), P4 = borrow unico nel probe sigillato `$o->x = $o->y OP C` quando il
// ricevitore È l'oggetto letto (recv == slot). Ogni cammino che ESCE dal dominio deve cadere al
// sentiero storico: $s non-Long/Ref/typed-ref, prop Double/Ref/assente/__get/hook/lazy/enum,
// overflow, shift, Div/Mod/Pow/Concat; recv ≠ slot (P4 non preso, P1 sì); PropGetSlot NON
// seguito da BinarySTDst (controllo negativo del peephole). OGNI forma con IC gira in un loop a
// 2 iterazioni (1ª riempie la IC, 2ª entra nel probe: lezione S-172). SENZA diagnostici (quelli
// stanno in fx-sl3-div.php se emergono).
function show($label, $v) { echo $label, ': '; var_dump($v); }
class P { public $x = 0; public $y = 1; }
class Q { public $x = 5; public $y = 2; }
class T { public int $x = 0; public int $y = 1; public float $f = 1.5; }
class RO { public function __construct(public readonly int $x = 0, public int $y = 1) {} }
class G { public $y = 3; private $d = ['x' => 4]; public function __get($n) { return $this->d[$n] ?? null; } }
class HG { public int $x = 6 { get => $this->x * 2; } public int $y = 1; }
class PV { private $x = 0; public $y = 1; public function run() { for ($k = 0; $k < 2; $k++) { $this->x = $this->y + 1; } return $this->x; } }
enum E: int { case A = 7; }
function g($v) { return $v; }

// --- P3: $s OP= $o->x (dominio Long e uscite; ogni forma in loop 2 = IC calda) ---
$o = new P; $o->x = 7;
$s = 10; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-add', $s);
$s = 10; for ($k = 0; $k < 2; $k++) { $s -= $o->x; } show('p3-sub', $s);
$s = 10; for ($k = 0; $k < 2; $k++) { $s *= $o->x; } show('p3-mul', $s);
$s = 70; for ($k = 0; $k < 2; $k++) { $s &= $o->x; } show('p3-and', $s);
$s = 8; for ($k = 0; $k < 2; $k++) { $s |= $o->x; } show('p3-or', $s);
$s = 3; for ($k = 0; $k < 2; $k++) { $s ^= $o->x; } show('p3-xor', $s);
$s = 3; for ($k = 0; $k < 2; $k++) { $s <<= $o->x; } show('p3-shl', $s);
$s = 1000; for ($k = 0; $k < 2; $k++) { $s >>= $o->x; } show('p3-shr', $s);
$o->x = 64; $s = -3; for ($k = 0; $k < 2; $k++) { $s <<= $o->x; } show('p3-shl-64', $s);
$s = -3; for ($k = 0; $k < 2; $k++) { $s >>= $o->x; } show('p3-shr-64', $s);
$o->x = 5; $s = -7; for ($k = 0; $k < 2; $k++) { $s <<= $o->x; } show('p3-shl-neg-l', $s);
$s = -7; for ($k = 0; $k < 2; $k++) { $s >>= $o->x; } show('p3-shr-neg-l', $s);
$o->x = -1; for ($k = 0; $k < 2; $k++) { try { $s = 3; $s <<= $o->x; } catch (ArithmeticError $e) { echo "p3-shl-neg#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$o->x = 3;
$s = 10; for ($k = 0; $k < 2; $k++) { $s /= $o->x; } show('p3-div', $s);
$s = 9; for ($k = 0; $k < 2; $k++) { $s /= $o->x; } show('p3-div-exact', $s);
$s = 10; for ($k = 0; $k < 2; $k++) { $s %= $o->x; } show('p3-mod', $s);
$s = 2; for ($k = 0; $k < 2; $k++) { $s **= $o->x; } show('p3-pow', $s);
$s = "ab"; for ($k = 0; $k < 2; $k++) { $s .= $o->x; } show('p3-concat', $s);
$o->x = 0; for ($k = 0; $k < 2; $k++) { try { $s = 1; $s /= $o->x; } catch (DivisionByZeroError $e) { echo "p3-div-zero#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$o->x = 1; $s = PHP_INT_MAX - 1; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-add-overflow', $s);
$o->x = PHP_INT_MIN; $s = 5; for ($k = 0; $k < 2; $k++) { $s -= $o->x; } show('p3-sub-overflow', $s);
$o->x = PHP_INT_MAX; $s = 3; for ($k = 0; $k < 2; $k++) { $s *= $o->x; } show('p3-mul-overflow', $s);
$o->x = 2;
$s = 1.5; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-double-dst', $s);
$s = "10"; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-numstr-dst', $s);
$s = null; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-null-dst', $s);
$s = true; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-bool-dst', $s);
// -each (S-173 corsa 2): il dst torna non-Long a OGNI iterazione (senza reset, null/bool/numstr diventano Long dopo la 1ª: in dominio dalla 2ª)
for ($k = 0; $k < 2; $k++) { $s = null; $s += $o->x; } show('p3-null-dst-each', $s);
for ($k = 0; $k < 2; $k++) { $s = true; $s += $o->x; } show('p3-bool-dst-each', $s);
for ($k = 0; $k < 2; $k++) { $s = "10"; $s += $o->x; } show('p3-numstr-dst-each', $s);
$o->x = 2.5; $s = 1; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-double-src', $s);
$o->x = "7"; $s = 1; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-numstr-src', $s);
$o->x = null; $s = 1; for ($k = 0; $k < 2; $k++) { $s += $o->x; } show('p3-null-src', $s);
$o->x = 3; $ds = 1; $rr = &$ds; for ($k = 0; $k < 2; $k++) { $ds += $o->x; } show('p3-dst-ref', $ds); show('p3-dst-ref-alias', $rr); unset($rr);
$o2 = new P; $o2->x = 4; $q = &$o2->x; $s2 = 1; for ($k = 0; $k < 2; $k++) { $s2 += $o2->x; } show('p3-src-ref', $s2); unset($q);
$tp = new T; $tp->x = 1; $tr = &$tp->x; for ($k = 0; $k < 2; $k++) { $tr += $o->x; } show('p3-typed-ref', $tp->x);
$tp->x = PHP_INT_MAX; for ($k = 0; $k < 2; $k++) { try { $tr += $o->x; show("p3-typed-ref-overflow#$k", $tp->x); } catch (TypeError $e) { echo "p3-typed-ref-overflow#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } } unset($tr);
$t = new T; $t->x = 5; $s3 = 1; for ($k = 0; $k < 2; $k++) { $s3 += $t->x; } show('p3-typed-src', $s3);
$g = new G; $s4 = 1; for ($k = 0; $k < 2; $k++) { $s4 += $g->x; } show('p3-magic-get-src', $s4);
$s5 = 1; for ($k = 0; $k < 2; $k++) { $s5 += $g->y; } show('p3-magic-class-plain-src', $s5);
$hg = new HG; $s6 = 1; for ($k = 0; $k < 2; $k++) { $s6 += $hg->x; } show('p3-hook-get-src', $s6);
$d = new stdClass; $d->x = 3; $s7 = 1; for ($k = 0; $k < 2; $k++) { $s7 += $d->x; } show('p3-dynamic-src', $s7);
$s8 = 1; for ($k = 0; $k < 2; $k++) { $s8 += E::A->value; } show('p3-enum-src', $s8);
$o->x = 4; $s9 = 0; for ($i = 0; $i < 100; $i++) { $s9 += $o->x; } show('p3-loop100', $s9);
$o->x = 5; $s10 = PHP_INT_MAX - 10; for ($i = 0; $i < 4; $i++) { $s10 += $o->x; if ($i === 1) show('p3-loop-overflow-step', $s10); } show('p3-loop-overflow', $s10);
$o->x = 6; $s11 = 0; for ($k = 0; $k < 2; $k++) { $t11 = $o->x; $s11 += $t11; } show('p3-no-peephole', $s11); // PropGetSlot NON seguito da BinarySTDst
$o->x = 6; $s12 = 0; for ($k = 0; $k < 2; $k++) { $s12 += $o->x + 1; } show('p3-rhs-expr', $s12);          // BinaryTC fra i due op: niente peephole
$o->x = 6; $s13 = 0; for ($k = 0; $k < 2; $k++) { $s13 += $o->x; $s13 += $o->x; } show('p3-twice', $s13);
$os = new Q; for ($k = 0; $k < 2; $k++) { try { $os += $os->x; show("p3-dst-is-obj#$k", $os); } catch (TypeError $e) { echo "p3-dst-is-obj#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }

// --- P4: $o->x = $o->y OP C sullo STESSO oggetto (recv == slot) e controlli con oggetti diversi ---
$o = new P; $o->y = 7; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 3; } show('p4-same', $o->x);
$o->y = 7; for ($k = 0; $k < 2; $k++) { $o->x = $o->y * 3; } show('p4-same-mul', $o->x);
$o->y = -7; for ($k = 0; $k < 2; $k++) { $o->x = $o->y >> 1; } show('p4-same-shr-neg', $o->x);
$o->x = 5; $o->y = 7; for ($k = 0; $k < 2; $k++) { $o->x = $o->x + $o->y; } show('p4-self-rhs', $o->x); // reset di x (S-173 corsa 2: cascata dal blocco precedente)
$o->x = 5; for ($k = 0; $k < 2; $k++) { $o->x = $o->x + 1; } show('p4-self', $o->x);
$o->y = 7; for ($k = 0; $k < 2; $k++) { $o->y = $o->y + 1; } show('p4-self-y', $o->y);
$o->y = PHP_INT_MAX; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p4-overflow', $o->x);
$o->y = 7; $o->x = 1.5; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p4-dst-double', $o->x);
$o->x = "s"; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p4-dst-str', $o->x);
for ($k = 0; $k < 2; $k++) { $o->x = 1.5; $o->x = $o->y + 1; } show('p4-dst-double-each', $o->x); // -each (S-173 corsa 2): dst non-Long a ogni iterazione
for ($k = 0; $k < 2; $k++) { $o->x = "s"; $o->x = $o->y + 1; } show('p4-dst-str-each', $o->x);
$o->x = 0; $r = &$o->x; for ($k = 0; $k < 2; $k++) { $o->x = $o->y + 1; } show('p4-dst-ref', $o->x); show('p4-dst-ref-alias', $r); unset($r);
$o3 = new P; $q3 = &$o3->y; $o3->y = 4; for ($k = 0; $k < 2; $k++) { $o3->x = $o3->y + 1; } show('p4-src-ref', $o3->x); unset($q3);
$o4 = new P; $o4->y = 2.5; for ($k = 0; $k < 2; $k++) { $o4->x = $o4->y + 1; } show('p4-src-double', $o4->x);
$a = new P; $b = new P; $b->y = 9; for ($k = 0; $k < 2; $k++) { $a->x = $b->y + 1; } show('p4-two-objs', [$a->x, $b->x]);
$c = new Q; for ($k = 0; $k < 2; $k++) { $c->x = $c->y + 10; } show('p4-other-class', $c->x);
$t = new T; $t->y = 5; for ($k = 0; $k < 2; $k++) { $t->x = $t->y + 1; } show('p4-typed', $t->x);
$t->y = 3; for ($k = 0; $k < 2; $k++) { $t->f = $t->y + 1; } show('p4-typed-float-dst', $t->f);
$ro = new RO(5, 7); for ($k = 0; $k < 2; $k++) { try { $ro->x = $ro->y + 1; show("p4-readonly#$k", $ro->x); } catch (Error $e) { echo "p4-readonly#$k: ", get_class($e), ' ', $e->getMessage(), "\n"; } }
$pv = new PV; show('p4-this-private', $pv->run());
$d = new stdClass; $d->y = 3; for ($k = 0; $k < 2; $k++) { $d->x = $d->y + 1; } show('p4-dynamic', $d->x);
for ($k = 0; $k < 2; $k++) { try { $e = E::A; $e->x = $e->value + 1; show("p4-enum#$k", $e->x); } catch (Error $ex) { echo "p4-enum#$k: ", get_class($ex), ' ', $ex->getMessage(), "\n"; } }
$o = new P; $s = 0; for ($i = 0; $i < 1000; $i++) { $o->x = $o->y + 1; $s += $o->x; } show('prop-micro-1000', $s);
echo "FX-SL3 DONE\n";
