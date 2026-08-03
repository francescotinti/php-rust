<?php
class Città {
    public function m(): int { return 1; }
}
class Provincia extends Città {
    public function m(): string { return "a"; }
}
