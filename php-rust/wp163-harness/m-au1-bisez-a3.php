<?php // closure SIMPLE 1 param (AL2 fast atteso: ~0)
spl_autoload_register(function ($c) { /* no-op */ });
$cn = 'X\\Miss'; $acc=0; for ($i=0;$i<200000;$i++){ if(!class_exists($cn)){$acc++;} } echo "A3-OK $acc\n";
