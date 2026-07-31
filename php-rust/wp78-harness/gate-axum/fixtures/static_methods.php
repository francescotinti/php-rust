<?php
// A-DS9 fixture (S-78.1.5, Council WP-79): statics in METHODS and in INHERITED
// methods (PHP 8.1+ semantics: a child that merely inherits the method SHARES
// the parent's static). The oracle defines the expected body; every request
// must reproduce it from scratch (per-request isolation by Vm death).
class Base {
    public static function tick() { static $n = 0; return ++$n; }
}
class Child extends Base {}

$parts = [];
$parts[] = "base=" . Base::tick() . Base::tick();
$parts[] = "child=" . Child::tick();
$parts[] = "again=" . Base::tick();
echo "SM:" . implode(";", $parts) . "\n";
