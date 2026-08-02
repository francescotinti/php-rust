<?php
echo "out\n";
class P { public function m(): string { return "s"; } }
class C extends P { public function m(): int { return 1; } }
