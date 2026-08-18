<?php
// S-153 sonda p.1 — free via sweep per statement: temp scartata, niente __destruct.
// Atteso (piano): Sweep.borrow k=4-6 (cand_id + unbuffer + class_id + lazy + cascade).
class TdD { public $a = 1; }
$n = (int)(getenv('TDN') ?: 100000);
for ($i = 0; $i < $n; $i++) {
    new TdD;
}
echo "TD-DIE-OK n=$n\n";
