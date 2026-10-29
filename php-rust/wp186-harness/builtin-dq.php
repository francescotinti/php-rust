<?php
$s = 0;
for ($i = 0; $i < 10000000; $i++) { $s += abs($i - 5); }
echo $s, "\n";
