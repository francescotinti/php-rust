<?php
// A-TH8 fixture (S-78.1.4): exit() is a CLEAN termination — HTTP 200, body =
// captured output up to the exit (FPM semantics; the exit code has no HTTP
// channel). Byte-compared against the oracle CLI stdout.
echo "out-before-exit\n";
exit(3);
echo "never";
