<?php
class P { public function m(): mixed { return 1; } }
class C extends P { public function m(): int { return 42; } }
echo (new C)->m();
