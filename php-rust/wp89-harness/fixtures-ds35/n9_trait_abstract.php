<?php
trait T { abstract public function f(): string; }
class C { use T; public function f(): int { return 1; } }
echo "alive";
