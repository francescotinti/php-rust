--TEST--
unsupported construct -> skip
--FILE--
<?php class C { function __construct(public int $x){} } echo 1;
--EXPECT--
1
