--TEST--
unsupported construct -> skip
--FILE--
<?php function f(...$a){} echo 1;
--EXPECT--
1
