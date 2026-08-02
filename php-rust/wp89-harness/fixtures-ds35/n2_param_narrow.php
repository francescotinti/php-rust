<?php
class P { public function m($x) { return $x; } }
class C extends P { public function m(int $x) { return $x; } }
echo "alive";
