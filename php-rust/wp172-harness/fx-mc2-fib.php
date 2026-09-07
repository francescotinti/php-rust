<?php
// fx-mc2-fib.php — caso 5 SEPARATO (phpr-only, DICHIARATO): in PHP vero
// `Fiber` e' FINAL e il subclassing e' fatale a compile ⇒ il caso del
// revisore e' irrealizzabile nell'oracle (soundness IC rafforzata dal
// linguaggio); qui si prova solo l'identita' fast==funnel sul sito condiviso.
error_reporting(E_ALL);
class P { function add($a,$b){ return $a+$b; } }
$o = new P;
for ($i = 0; $i < 3; $i++) { $o->add(1, 2); }
echo "C5: ";
class MyFib extends Fiber {}
$fib = new MyFib(function () { return 1; });
foreach ([$o, $fib, $o] as $t) {
    try { echo $t->add(1, 2), " "; }
    catch (\Error $e) { echo "EF:", $e->getMessage(), " "; }
}
echo "\n";
echo "done\n";
