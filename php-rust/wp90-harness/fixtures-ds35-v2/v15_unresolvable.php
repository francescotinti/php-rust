<?php
class P { public function m(): A { return new A; } }
class C extends P { public function m(): B { return new B; } }
echo "alive";
