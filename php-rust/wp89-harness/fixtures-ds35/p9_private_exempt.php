<?php
class P { private function m(): string { return "p"; } public function call() { return $this->m(); } }
class C extends P { private function m(): int { return 9; } }
echo (new P)->call();
echo "|private-exempt";
