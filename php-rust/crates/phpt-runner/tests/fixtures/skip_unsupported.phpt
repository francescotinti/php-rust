--TEST--
unsupported construct -> skip
--FILE--
<?php trait T {} echo 1;
--EXPECT--
1
