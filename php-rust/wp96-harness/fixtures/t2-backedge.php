<?php
// Negativo che DEVE mordere: lo slot e' vivo sul back-edge del loop, quindi
// la lettura NON e' un ultimo uso e non va contata.
function backedge() {
    $acc = "a";
    for ($i = 0; $i < 3; $i++) {
        $acc = $acc . "b";   // legge $acc, che vivra' ancora al giro dopo
    }
    echo $acc, "\n";
}
backedge();
