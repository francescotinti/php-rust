<?php
// smoke147 — ogni famiglia di chiavi s147 viva almeno una volta:
// movimenti attribuiti a LoadSlot/LoadVar (s147mv), digrammi (s147dg),
// classi s145.clone_* e riga zvalcensus_s147 (arr/obj possono restare 0
// su uno script minimo: FUORI smoke, criterio p.3).
function mover($a, $s, $o) {
    $x = $a;        // lettura slot NON ultima: clone arr
    $y = $s;        // clone str
    $z = $o;        // clone obj (handle)
    $last_a = $a;   // ultimo uso di $a: candidato would_take arr
    $last_o = $o;   // ultimo uso di $o: candidato would_take obj
    return [$x, $y, $z, $last_a, $last_o];
}
$arr = [1, 2, 3, "k" => "v"];
$str = str_repeat("s", 8);
$obj = new stdClass();
$obj->p = 1;
$r = null;
for ($i = 0; $i < 50; $i++) {
    $r = mover($arr, $str, $obj);
}
echo count($r), "\n";
