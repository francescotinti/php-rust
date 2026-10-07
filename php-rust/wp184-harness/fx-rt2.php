<?php
// FX-RT2 (S-184): presidio bilaterale della leva L-RT2 «Ret fuso» — oracle == phpr byte-id.
// Il rischio della leva è l'AMMISSIONE: il Ret fuso deve prendere SOLO la funzione libera di forma 0 (nessun hint di
// ritorno, non by-ref, nessun flag RET_*/INIT_PROPS/CLONE_INIT/IN_DESTRUCTOR, nessuna ret_cell, niente $this/foreach/ext/$$).
// Ogni riga è etichettata «RT2-<bit>» = il bit di ammissione che la presidia; il MUTANTE «ammissione senza i bit di forma»
// (shape/flags/ret_cell ignorati) DEVE cambiare almeno una riga RT2-* (hint non coerced, by-ref non alias, cella non scritta).
function h(): int { return "5"; }                       // RS_HINT: coercizione weak "5" -> 5
function hf(): int { return "x"; }                      // RS_HINT: TypeError
function hn(): ?string { return null; }                 // RS_HINT: nullable
function &w() { static $v = 1; return $v; }            // RS_WRAP/RET_DEREF: by-ref, alias in `$r = &w()`
function &d() { $a = [1, 2]; return $a; }               // by-ref in contesto di VALORE (RET_DEREF)
function plain($a, $b) { return $a + $b; }              // forma 0: il Ret fuso
function nested($a) { return plain($a, plain($a, 1)); } // forma 0 annidata
function nul() { }                                      // forma 0, stack vuoto: Null implicito
function many() { $x = 1; $y = [2]; $z = 'z'; return $x; } // forma 0 con locali da rilasciare
class C { const A = 2; public static $s = self::A * 3; public static $t = [self::A, 'k' => 4]; } // ret_cell: thunk statico
class P { public $q = C::A + 5; public array $r = [1]; }                                          // INIT_PROPS (cammino vecchio)
class T { function __toString(): string { return "tostr"; } }                                     // RET_STRINGIFY
class D { public function __construct(public string $n) {} function __destruct() { echo "RT2-dtor: {$this->n} ", plain(1, 1), "\n"; } }
echo "RT2-hint: "; var_dump(h());
try { hf(); echo "RT2-hint-fail: nessun errore\n"; } catch (TypeError $e) { echo "RT2-hint-fail: ", get_class($e), "\n"; }
echo "RT2-hint-null: "; var_dump(hn());
$r = &w(); $r++; echo "RT2-wrap: ", w(), "\n";
$x = d(); $x[] = 3; echo "RT2-deref: ", count($x), "\n";
echo "RT2-plain: ", plain(1, 2), " ", nested(3), "\n";
echo "RT2-null: "; var_dump(nul());
echo "RT2-many: ", many(), "\n";
echo "RT2-cell: ", C::$s, " ", count(C::$t), "\n";
$p = new P; echo "RT2-init: ", $p->q, " ", count($p->r), "\n";
echo "RT2-tostr: ", (string) new T, " ", strlen(new T), "\n";
function mkd($n) { $d = new D($n); return strlen($n); } $m = mkd('a'); echo "RT2-dtor-ret: $m\n";
echo "FX-RT2 DONE\n";
