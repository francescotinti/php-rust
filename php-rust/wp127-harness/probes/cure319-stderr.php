<?php
// log_errors OFF: il canale log CLI->stderr e' una divergenza SEPARATA
// (phpr non lo emette — a catalogo); qui si confronta il canale DISPLAY.
@ini_set('log_errors','0');
// §3.19-ter: display_errors=stderr = DESTINAZIONE (fatal e warning su stderr).
@ini_set('display_errors', 'stderr');
echo "stdout-vivo\n";
var_dump(ini_get('display_errors'));
trigger_error('warn-su-stderr', E_USER_WARNING);
echo "dopo-warning\n";
throw new Error('fatal-su-stderr');
