<?php
class P { public function m(): int { return 1; } }
class C extends P { public function m(): string { return "x"; } }
echo "alive";
