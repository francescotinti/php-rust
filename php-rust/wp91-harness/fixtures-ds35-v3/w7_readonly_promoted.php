<?php
class P { public function __construct(public readonly int $x) {} }
class C extends P { public function __construct(public int $x) { parent::__construct($x); } }
echo "live";
