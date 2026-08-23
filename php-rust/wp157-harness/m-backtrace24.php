<?php
// S-156 — guardia backtrace RI-RISOLUTA (REGOLE §3, az.rev. S-154: N≥2,4M
// sui bracci di record, tick 10 ms/N ≈ 4,2 ns ≤ soglia/4). DERIVATO
// DICHIARATO di wp149-harness/m-backtrace.php: SOLO 150000→2400000.
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
echo bt_recurse(48, 2400000, str_repeat("x", 32), [1, 2, 3]), "\n";
