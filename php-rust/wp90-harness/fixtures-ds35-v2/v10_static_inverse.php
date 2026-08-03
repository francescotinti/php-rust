<?php
class P { public static function m() { return 1; } }
class C extends P { public function m() { return 2; } }
echo "alive";
