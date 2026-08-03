<?php
class P { public function m(int &$x) { return $x; } }
class C extends P { public function m(int|string &$x) { return $x; } }
$v = 41; echo (new C)->m($v) + 1;
