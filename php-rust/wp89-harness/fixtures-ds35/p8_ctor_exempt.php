<?php
class P { public function __construct(int $x) {} }
class C extends P { public function __construct(string $y, $z = null) { parent::__construct(1); } }
new C("a");
echo "ctor-exempt";
