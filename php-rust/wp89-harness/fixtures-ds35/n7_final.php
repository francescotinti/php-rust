<?php
class P { final public function m(): int { return 1; } }
class C extends P { public function m(): int { return 2; } }
echo "alive";
