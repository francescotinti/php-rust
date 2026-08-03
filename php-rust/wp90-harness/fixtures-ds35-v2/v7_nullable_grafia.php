<?php
class P { public function m(?int $x): ?int { return $x; } }
class C extends P { public function m(int|null $x): int|null { return $x; } }
echo (new C)->m(42);
