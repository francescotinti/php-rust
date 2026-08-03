<?php
class P { public function m(): static { return $this; } }
class C extends P { public function m(): self { return $this; } }
echo "unreachable";
