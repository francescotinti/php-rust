<?php
class A {}
class B extends A {}
class P { public function m(): A { return new A; } }
class C extends P { public function m(): B { return new B; } }
$r = (new C)->m();
echo get_class($r);
