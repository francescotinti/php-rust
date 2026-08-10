<?php
// log_errors OFF: il canale log CLI->stderr e' una divergenza SEPARATA
// (phpr non lo emette — a catalogo); qui si confronta il canale DISPLAY.
@ini_set('log_errors','0');
// §3.19-bis: la famiglia processo risolta come callable DINAMICO.
$f = 'exec';
$last = $f('echo dyn-exec');
echo "var-exec: ", $last, "\n";
echo "cuf-exec: ", call_user_func('exec', 'echo cuf-exec'), "\n";
function wrap(callable $c, ...$a) { return $c(...$a); }
echo "wrap-system: "; $rc = wrap('system', 'echo wrap-system'); echo " rc-val=", var_export($rc, true), "\n";
// out-param fornito BY VALUE: warna e non scrive (semantica call_user_func).
$out = 'untouched';
$r = call_user_func('exec', 'echo warned', $out);
echo "exec-out: r=", $r, " out=", var_export($out, true), "\n";
// passthru dinamico (stampa direttamente).
call_user_func('passthru', 'echo dyn-passthru');
// function_exists coerente con la chiamata.
var_dump(function_exists('exec'), is_callable('exec'));
