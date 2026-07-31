<?php
// A-BB22 (Council WP-82, rewritten in the SAME COMMIT as the A-BB6 lever —
// the old comment said "a2(bare) = the fixed per-request cost that no cache
// HIT can remove", FALSE by 30x under the lever: the cached unit is the
// MAIN, so on HIT a2 goes away TOGETHER with a1).
//
// What bare.php IS: the smallest measurable program. On a MISS its row shows
// the cost the HIT REMOVES (a1+a2 of an almost-empty body); on a steady HIT
// its a_calls IS the measured ex-post floor of the lever — probe
// (canonicalize+stat+source-hash) + lookup + bookkeeping, nothing else
// (A-BB23: prediction <=200 call/req, delta hello-bare on HIT ~0; KB-82-3:
// a_calls(bare,HIT) > 200 = the ex-ante floor was wrong, re-derive BY NAME).
// Floor claims are bounds on ALLOCATIONS, never CPU (A-BB25: the hash/stat
// cost is alloc-invisible — CPU is judged only by the two-N slope).
