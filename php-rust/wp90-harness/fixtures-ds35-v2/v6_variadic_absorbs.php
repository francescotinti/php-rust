<?php
class P { public function m($a, $b, $c) { return $a; } }
class C extends P { public function m(...$rest) { return $rest[0]; } }
echo (new C)->m(40, 1, 2) + 2;
