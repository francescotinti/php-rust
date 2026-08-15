<?php
// S-142 az.rev. #3 — sonda profondità teardown array annidati.
// argv: [1]=profondità N, [2]=repr ("p"=packed, "h"=hashed).
// Costruzione ITERATIVA (O(1) per livello); la ricorsione è solo nel DROP.
$n = (int)($argv[1] ?? 1000);
$repr = $argv[2] ?? "p";
$a = 1;
for ($i = 0; $i < $n; $i++) {
    if ($repr === "h") { $a = ["k" => $a, "pad" => $i]; }
    else               { $a = [$a, $i]; }
}
echo "built:", $n, ":", $repr, "\n";
unset($a);            // teardown QUI (ricorsivo in profondità n)
echo "dropped\n";
