<?php
// fx-backtrace — fixture bilaterale leva BT1 (criterio s149-criterio-bt1.md
// p.2): stessa uscita byte-id su phpr e oracle 8.5.7. Stampa CHIAVI ordinate
// (presenza di args/object/type) + function + conteggio frame per ciascuna
// combinazione options/limit — mai il contenuto di object (instabile).
function fx_show(array $bt): void {
    foreach ($bt as $i => $f) {
        $keys = array_keys($f);
        sort($keys);
        echo "#$i ", implode(",", $keys), " fn=", $f['function'] ?? '?', "\n";
    }
    echo "n=", count($bt), "\n";
}
class FxB {
    public function m(int $x, string $s): void {
        echo "-- default\n";      fx_show(debug_backtrace());
        echo "-- options=0\n";    fx_show(debug_backtrace(0));
        echo "-- ignore_args\n";  fx_show(debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS));
        echo "-- limit=1\n";      fx_show(debug_backtrace(DEBUG_BACKTRACE_PROVIDE_OBJECT, 1));
        echo "-- limit=2\n";      fx_show(debug_backtrace(DEBUG_BACKTRACE_PROVIDE_OBJECT, 2));
        echo "-- ia+limit2\n";    fx_show(debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 2));
    }
}
function fx_outer(FxB $o): void { $o->m(7, "abc"); }
fx_outer(new FxB);
