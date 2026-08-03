<?php
interface A {}
interface B {}
class P {
    public function m(): A&B { throw new RuntimeException("x"); }
}
class C extends P {
    public function m(): A { throw new RuntimeException("x"); }
}
