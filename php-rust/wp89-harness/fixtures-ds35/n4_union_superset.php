<?php
class P { public function m(): int|string { return 1; } }
class C extends P { public function m(): int|string|float { return 1; } }
echo "alive";
