<?php
// G-STATEFUL (A-DS3, Council WP-78): function statics, class static properties,
// closure statics and define() MUST be per-request (PHP-FPM semantics: today
// enforced by Vm death, NOT by request_end()). Any cross-request drift in this
// output is KS-DS-78-1 → FATAL, blocks merge.
//
// Within ONE request statics DO accumulate (each probe runs twice → "12");
// across requests every probe must restart from scratch.
function g3_fn() { static $n = 0; return ++$n; }

class G3C {
    public static $p = 0;
    public static function bump() { return ++self::$p; }
}

$g3_closure = function () { static $m = 0; return ++$m; };

$parts = [];
$parts[] = "fn=" . g3_fn() . g3_fn();
$parts[] = "prop=" . G3C::bump() . G3C::bump();
$parts[] = "closure=" . $g3_closure() . $g3_closure();
$parts[] = "const=" . (defined('G3_CONST') ? "LEAK" : "fresh");
define('G3_CONST', 1);
$parts[] = "constval=" . G3_CONST;

echo "G3:" . implode(";", $parts) . "\n";
