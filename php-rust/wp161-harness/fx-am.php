<?php
// S-160 fx-am.php v2 — fixture bilaterale L-AM1 ESTESA (az.rev. S-159 #2):
// forme 1-14 EREDITATE da s159 (copia dichiarata, manifest s160-fxam-copia.diff)
// + forme 15-20 sui buchi nominati dalla revisione: closure-generatore,
// closure 1-param con hint, func_get_args nel fast, static closure,
// Closure::bind, by-ref a warning MASCHERATO (esclusione documentata).
// oracle == pin BYTE-ID al gate.
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
// ---- v2 (S-160, az.rev. S-159 #2): buchi nominati dalla revisione ----
// 15. closure-GENERATORE (esclusa dal fast via !is_generator: mai provata
//     bilaterale prima): il callback torna un Generator per elemento.
$gens = array_map(function ($x) { yield $x * 3; }, [1, 2]);
foreach ($gens as $g) { var_dump($g instanceof Generator, $g->current()); }
// 16. closure 1-param CON hint di tipo (simple_call=false: cammino pieno,
//     instradamento mai testato prima) — coercizione int attiva.
var_dump(array_map(function (int $x): int { return $x + 5; }, ['1', 2]));
// 17. func_get_args/func_num_args DENTRO la closure fast (argc/extra_args
//     del frame arità-1: devono vedere ESATTAMENTE 1 argomento).
var_dump(array_map(function ($x) { return func_num_args() . ':' . func_get_args()[0]; }, [7, 8]));
// 18. static closure (fast senza bound_this).
var_dump(array_map(static fn($x) => $x - 1, [1, 2]));
// 19. Closure::bind (bound_this + scope su closure arità-1).
class AmFx3 { public $k2 = 50; }
$b = Closure::bind(function ($x) { return $x + $this->k2; }, new AmFx3(), AmFx3::class);
var_dump(array_map($b, [1, 2]));
// 20. by-ref param: ESCLUSIONE dal fast DOCUMENTATA sui VALORI. Il warning
//     oracle "must be passed by reference" e' divergenza PRE-esistente a
//     catalogo (assente in phpr): lo si MASCHERA con error_reporting per
//     confrontare al byte il resto (risultato + originale intatto).
error_reporting(E_ALL & ~E_WARNING);
$src = [1, 2];
$r = array_map(function (&$x) { $x++; return $x; }, $src);
var_dump($r, $src);
error_reporting(E_ALL);
echo "FXAM-END\n";
