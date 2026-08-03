<?php
class P { public function m(int|string &$x) { return $x; } }
class C extends P { public function m(int &$x) { return $x; } }
echo "alive";
