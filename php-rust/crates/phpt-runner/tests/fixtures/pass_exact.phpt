--TEST--
exact match
--FILE--
<?php $a = [1, 2, 3]; foreach ($a as $v) { echo $v; }
--EXPECT--
123
