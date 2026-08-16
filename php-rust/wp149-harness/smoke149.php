<?php
// smoke149 — accende i tag s148 (frame/hostcall/arrgrow) EREDITATI e i nomi
// s149 del criterio p.5: str_repeat e sprintf (builtin che allocano di certo).
function s149_smoke($seed) {
    $a = [];
    for ($i = 0; $i < 200; $i++) {
        $a["k$i"] = $i + $seed;
    }
    $s = str_repeat("x", 32);
    $t = sprintf("v=%d", $seed);
    return count($a) + strlen($s) + strlen($t);
}
$tot = 0;
for ($r = 0; $r < 3; $r++) {
    $tot += s149_smoke($r);
}
echo "smoke149 tot=", $tot, "\n";
