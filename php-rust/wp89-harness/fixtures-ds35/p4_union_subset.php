<?php
class P { public function m(): int|string|null { return 1; } }
class C extends P { public function m(): ?int { return 7; } }
var_dump((new C)->m());
