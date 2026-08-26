<?php
// S-157 az.rev. S-156 #3 — sonda oracle<->pin: debug_backtrace() DENTRO
// l'autoloader innescato da class_exists (contratto di visibilita' storico
// HD2: in Zend il frame di class_exists mostra gli args). Solo semantica.
spl_autoload_register(function ($c) {
    echo "AL:$c\n";
    $bt = debug_backtrace();
    foreach ($bt as $i => $f) {
        $cls = isset($f['class']) ? $f['class'] . ($f['type'] ?? '') : '';
        $fn  = $f['function'] ?? '?';
        $na  = isset($f['args']) ? count($f['args']) : -1;
        $a0  = (isset($f['args'][0]) && is_string($f['args'][0])) ? $f['args'][0] : '-';
        echo "  #$i {$cls}{$fn} nargs=$na a0=$a0\n";
    }
});
var_dump(class_exists('Nope\\ProbeClass'));
var_dump(class_exists('\\Nope\\ProbeClass2'));
var_dump(interface_exists('Nope\\ProbeIface'));
