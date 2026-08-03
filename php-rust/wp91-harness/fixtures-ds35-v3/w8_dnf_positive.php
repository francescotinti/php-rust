<?php
interface A {}
interface B {}
class X implements A, B {}
class P { public function m(): (A&B)|string { return "p"; } }
class C extends P { public function m(): string { return "c"; } }
$c = new C();
echo $c->m(), "|alive";
