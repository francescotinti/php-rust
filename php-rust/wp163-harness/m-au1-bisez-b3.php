<?php // forma STATICA per stringa ['Cls','m'] (generico, dispatch statico)
class B3 { public static function m($c) { /* no-op */ } }
spl_autoload_register(['B3','m']);
$cn = 'X\\Miss'; $acc=0; for ($i=0;$i<200000;$i++){ if(!class_exists($cn)){$acc++;} } echo "B3-OK $acc\n";
