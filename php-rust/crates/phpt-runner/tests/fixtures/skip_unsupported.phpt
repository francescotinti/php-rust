--TEST--
unsupported construct -> skip
--FILE--
<?php class C {} echo 1;
--EXPECT--
1
