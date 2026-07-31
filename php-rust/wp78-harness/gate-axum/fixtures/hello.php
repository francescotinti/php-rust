<?php
// G-APERTURA-2 hello fixture (A-SK2): deterministic output, byte-compared across
// sequential requests on the same server (persistent RetainSet per worker).
echo "hello axum\n";
echo "sum=" . (2 + 40) . "\n";
