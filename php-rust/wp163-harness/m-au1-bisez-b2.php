<?php // [obj,m] con metodo BY-REF (non-simple, generico)
class B2 { public function m(&$c) { /* no-op */ } }
$l = new B2(); spl_autoload_register([$l,'m']);
$cn = 'X\\Miss'; $acc=0; for ($i=0;$i<200000;$i++){ if(!class_exists($cn)){$acc++;} } echo "B2-OK $acc\n";
