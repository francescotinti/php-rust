<?php
class P { public function m(): void {} }
class C extends P { public function m(): int { return 1; } }
echo "alive";
