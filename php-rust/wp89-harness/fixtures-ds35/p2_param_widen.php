<?php
class P { public function m(int $x) { return $x; } }
class C extends P { public function m($x = 0) { return $x + 1; } }
echo (new C)->m(41);
