<?php
// smoke148 — accende i tre tag del criterio p.5: frame (chiamata funzione),
// hostcall (builtin), arrgrow (insert con chiavi stringa => hashed + grow).
function s148_smoke($seed) {
    $a = [];
    for ($i = 0; $i < 200; $i++) {
        $a["k$i"] = $i + $seed;
    }
    $s = str_repeat("x", 32);
    return count($a) + strlen($s);
}
$tot = 0;
for ($r = 0; $r < 3; $r++) {
    $tot += s148_smoke($r);
}
echo "smoke148 tot=", $tot, "\n";
