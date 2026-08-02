<?php
class P { public function m(): self { return $this; } }
class C extends P { public function m(): static { return $this; } }
echo get_class((new C)->m());
