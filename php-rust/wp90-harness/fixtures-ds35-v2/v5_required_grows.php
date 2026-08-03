<?php
class P { public function m($a) { return $a; } }
class C extends P { public function m($a, $b) { return $a; } }
echo "alive";
