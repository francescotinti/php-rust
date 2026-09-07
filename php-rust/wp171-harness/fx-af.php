<?php
// S-160 fx-af.php — fixture bilaterale L-AF1 (array_filter): forme fast E
// cammino pieno; oracle == candidato BYTE-ID al gate.
error_reporting(E_ALL);

// 1. closure anonima arità-1 (FAST): chiavi preservate, truthy misto
var_dump(array_filter(['a' => 1, 'b' => 0, 7 => 2, 'c' => 3], fn($x) => $x > 1));
// 2. nessun callback (truthy; falsy vari eliminati)
var_dump(array_filter([1, 0, '', '0', 'x', null, [], false, 0.0, '00']));
// 3. mode=ARRAY_FILTER_USE_KEY (cammino pieno)
var_dump(array_filter(['aa' => 1, 'b' => 2, 'ccc' => 3], fn($k) => strlen($k) > 1, ARRAY_FILTER_USE_KEY));
// 4. mode=ARRAY_FILTER_USE_BOTH (cammino pieno)
var_dump(array_filter(['a' => 5, 'bb' => 1, 'c' => 7], fn($v, $k) => $v > 2 && strlen($k) === 1, ARRAY_FILTER_USE_BOTH));
// 5. string callable (cammino pieno)
var_dump(array_filter(['1', 'abc', '0', '2x'], 'is_numeric'));
// 6. closure 1-param CON hint (simple_call=false: pieno)
var_dump(array_filter([1, 2, 3, 4], function (int $x): bool { return $x % 2 === 1; }));
// 7. static closure (fast senza bound_this)
var_dump(array_filter([1, 2, 3], static fn($x) => $x !== 2));
// 8. eccezione dentro il callback (unwind attraverso il fast)
try {
    array_filter([1, 2, 3], function ($x) { if ($x === 2) { throw new RuntimeException("boom$x"); } return true; });
} catch (RuntimeException $e) { echo "caught: ", $e->getMessage(), "\n"; }
// 9. array vuoto (fast: zero elementi)
var_dump(array_filter([], fn($x) => true));
// 10. capture nella closure (fast: use)
$soglia = 2;
var_dump(array_filter([1, 2, 3, 4], function ($x) use ($soglia) { return $x > $soglia; }));
// 11. Closure::bind (bound_this su arità-1)
class AfFx { public $lim = 2; }
$b = Closure::bind(function ($x) { return $x <= $this->lim; }, new AfFx(), AfFx::class);
var_dump(array_filter([1, 2, 3], $b));
// 12. ritorno non-bool dal callback fast (to_bool sul risultato)
var_dump(array_filter([0, 1, 2, 3], fn($x) => $x % 3));
// 13. chiavi miste preservate dal fast (int sparse + string)
var_dump(array_filter([9 => 'a', 'k' => '', 3 => 'b', 'z' => 0, 11 => 'c'], fn($v) => $v !== ''));
echo "FXAF-END\n";
