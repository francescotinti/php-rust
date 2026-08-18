<?php
// S-149 — bench leva BT1 «debug_backtrace onora options/limit»: la forma
// Doctrine\Deprecations (DEBUG_BACKTRACE_IGNORE_ARGS, 2) su pila profonda —
// il cammino che il census tranche-4 nomina (275,0M alloc = 81,9% del tag
// hostcall su ORM). Args nei frame perché IGNORE_ARGS abbia qualcosa da
// saltare; profondità 48 ~ PHPUnit+ORM.
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
echo bt_recurse(48, 600000, str_repeat("x", 32), [1, 2, 3]), "\n";
