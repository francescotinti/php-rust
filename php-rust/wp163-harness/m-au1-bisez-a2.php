<?php // closure BY-REF 1 param (non-simple, arita' combacia)
spl_autoload_register(function (&$c) { /* no-op */ });
$cn = 'X\\Miss'; $acc=0; for ($i=0;$i<200000;$i++){ if(!class_exists($cn)){$acc++;} } echo "A2-OK $acc\n";
