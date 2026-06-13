--TEST--
expectf wildcards
--FILE--
<?php echo "n=", 7 * 6, " f=", 1.0 / 4;
--EXPECTF--
n=%d f=%s
