<?php
// S-159 fx-am.php — fixture bilaterale L-AM1 (criterio p.7): tutte le forme di
// array_map, fast path E cammino pieno; oracle == candidato BYTE-ID al gate.
error_reporting(E_ALL);

// 1. closure anonima 1-param (FAST): valori + chiavi preservate
$r = array_map(fn($x) => $x * 2, ['a' => 1, 'b' => 2, 7 => 3]);
var_dump($r);
// 2. closure arità 2 su 1 array (mismatch: cammino pieno, arg mancante)
try {
    $r = array_map(function ($x, $y) { return [$x, $y]; }, [1, 2]);
    var_dump($r);
} catch (Throwable $e) { echo get_class($e), "\n"; } // solo classe: il TESTO diverge pre-leva (clausola "in ... on line" phpr; divergenza a catalogo, fuori perimetro L-AM1)
// 3. string callable
$r = array_map('strtoupper', ['x' => 'ab', 'y' => 'cd']);
var_dump($r);
// 4. [obj, metodo] callable
class AmFx { public function twice(int $n): int { return $n + $n; } }
$o = new AmFx();
var_dump(array_map([$o, 'twice'], [3, 4]));
// 5. null callback, 1 array (identità)
var_dump(array_map(null, ['k' => 9, 10]));
// 6. null callback, multi array (zip)
var_dump(array_map(null, [1, 2], ['a', 'b', 'c']));
// 7. multi-array con closure (re-index, code NULL)
var_dump(array_map(fn($a, $b) => "$a-$b", [1, 2, 3], ['x', 'y']));
// 8. closure variadica (cammino pieno)
var_dump(array_map(function (...$v) { return count($v); }, [5, 6]));
// 9. string callable statico "Classe::metodo" (cammino pieno)
// (la forma by-ref-param è RIMOSSA dal gate: warning "must be passed by
// reference" presente nell'oracle e assente in phpr PRE-leva — divergenza a
// catalogo, fuori perimetro L-AM1)
class AmFx2 { public static function neg(int $n): int { return -$n; } }
var_dump(array_map('AmFx2::neg', [7, 8]));
// 10. array vuoto (fast: zero elementi)
var_dump(array_map(fn($x) => $x, []));
// 11. eccezione dentro il callback (unwind attraverso il builtin)
try {
    array_map(function ($x) { if ($x === 2) { throw new RuntimeException("boom$x"); } return $x; }, [1, 2, 3]);
} catch (RuntimeException $e) { echo "caught: ", $e->getMessage(), "\n"; }
// 12. closure con default sul 2o param (n_params>1: cammino pieno)
var_dump(array_map(function ($x, $y = 10) { return $x + $y; }, [1, 2]));
// 13. capture nella closure (fast: use)
$k = 100;
var_dump(array_map(function ($x) use ($k) { return $x + $k; }, [1, 2]));
// 14. closure che ritorna by-value un oggetto (Rc nel canale)
var_dump(array_map(fn($x) => new AmFx(), [1])[0] instanceof AmFx);
echo "FXAM-END\n";
