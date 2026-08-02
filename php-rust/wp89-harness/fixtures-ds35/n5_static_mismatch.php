<?php
class P { public function m(): int { return 1; } }
class C extends P { public static function m(): int { return 2; } }
echo "alive";
