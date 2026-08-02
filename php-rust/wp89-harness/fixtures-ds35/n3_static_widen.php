<?php
class P { public function m(): static { return $this; } }
class C extends P { public function m(): P { return $this; } }
echo "alive";
