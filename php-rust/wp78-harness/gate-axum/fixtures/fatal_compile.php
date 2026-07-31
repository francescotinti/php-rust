<?php
// A-TH8 fixture (S-78.1.4): compile-time fatal (goto to undefined label —
// LowerError::Fatal in phpr, E_COMPILE_ERROR in Zend). Body must be
// byte-identical to the oracle CLI stdout (banner + "Stack trace:\n#0 {main}"),
// status 500. NOTE: the echo must NOT appear (compile precedes execution).
echo "pre\n";
goto nope;
