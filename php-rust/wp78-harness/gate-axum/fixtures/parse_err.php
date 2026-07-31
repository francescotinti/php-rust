<?php
// A-TH8 fixture (S-78.1.4): hard parse failure. Status 500; the ENVELOPE is
// asserted ("\nParse error: " prefix) but the message text is a REGISTERED
// divergence (PHPR_DIVERGENCES_FROM_PHP.md 3.9) — no oracle cmp, no parity
// claim (KS-DS-78-5).
echo "pre\n";
function f( {
