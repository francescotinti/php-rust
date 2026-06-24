--TEST--
unsupported construct -> skip
--FILE--
<?php global $$name; echo 1;
--EXPECT--
1
