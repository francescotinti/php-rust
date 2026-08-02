<?php
class P { public function m(): string { return "s"; } }
class C extends P { public function m(): never { throw new Exception("boom"); } }
echo "declared";
