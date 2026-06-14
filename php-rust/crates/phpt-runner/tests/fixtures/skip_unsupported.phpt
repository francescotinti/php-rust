--TEST--
unsupported construct -> skip
--FILE--
<?php enum E {} echo 1;
--EXPECT--
1
