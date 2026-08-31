<?php
// S-165 — guardia backtrace RI-RISOLUTA (REGOLE §3, az.rev. S-154: guardia su
// giudice quantizzato si ri-risolve a N con tick ≤ soglia/4 sui bracci di
// record; soglia 4 ⇒ tick ≤ 1 ns ⇒ N=9600000, tick 10 ms/N ≈ 1,04 ns).
// DERIVATO DICHIARATO di wp158-harness/m-backtrace24.php: SOLO 2400000→9600000.
function bt_leaf(int $n): int {
    $tot = 0;
    for ($i = 0; $i < $n; $i++) {
        $bt = debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 2);
        $tot += count($bt);
    }
    return $tot;
}
function bt_recurse(int $d, int $n, string $pad, array $extra): int {
    if ($d > 0) { return bt_recurse($d - 1, $n, $pad, $extra); }
    return bt_leaf($n);
}
echo bt_recurse(48, 9600000, str_repeat("x", 32), [1, 2, 3]), "\n";
