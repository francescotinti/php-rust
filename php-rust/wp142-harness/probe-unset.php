<?php
class D { public $n; function __construct($n){$this->n=$n;} function __destruct(){ echo "d:", $this->n, "\n"; } }
function f1(){ $x = new D("var"); unset($x); echo "after-var\n"; }
function f2(){ $a = ["k"=>new D("arrel")]; unset($a["k"]); echo "after-arrel\n"; }
function f3(){ $a = [new D("packel"), 2]; unset($a[0]); echo "after-packel\n"; }
f1(); f2(); f3(); echo "fine\n";
