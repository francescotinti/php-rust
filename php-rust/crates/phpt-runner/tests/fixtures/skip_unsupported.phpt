--TEST--
unsupported construct -> skip
--FILE--
<?php function f() { return 1; } echo f();
--EXPECT--
1
