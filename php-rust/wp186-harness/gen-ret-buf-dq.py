#!/usr/bin/env python3
"""gen-ret-buf-dq.py — generates ret-buf-dq-k{0,4,16}.php (S-186 guard, criterio s186-criterio-guardia.md p.2).

One statement per iteration: K `new A` born FIRST (seeded in gc_buf by gc_track and kept alive by
pin()'s static array: strong_count 3 = created + buffer + keep, so they stay in the buffer until the
statement-end sweep), then M=64 inline `$o->m($i,k)` method calls. Every Ret of the statement (pin's
plus the 64 methods) makes the tree's gc_sweep_after_ret scan the K live entries: O(K) per Ret on the
tree (cure 22/10), nothing on pin s183. RETS_PER_ITER is emitted in the source for the judge.
Generato con assistenza AI (Claude Fable 5).
"""
import os, sys
H = os.path.dirname(os.path.abspath(__file__))
N = 1000000
M = 64
for K in (0, 4, 16):
    news = ", ".join(["new A"] * K)
    calls = " + ".join(f"$o->m($i,{k})" for k in range(1, M + 1))
    src = (
        "<?php class A {} class O { function m($a,$b){ return $a+$b; } }\n"
        "function pin(...$xs){ static $keep=[]; foreach($xs as $x) $keep[]=$x; if (count($keep) >= 4096) $keep=[]; return 0; }\n"
        f"// RETS_PER_ITER={M + 1} (pin + {M} metodi) · K={K} oggetti vivi in gc_buf per statement\n"
        "$o=new O; $s=0;\n"
        f"for($i=0;$i<{N};$i++){{ $s = pin({news}) + {calls}; }} echo $s,\"\\n\";\n"
    )
    p = os.path.join(H, f"ret-buf-dq-k{K}.php")
    with open(p, "w") as f:
        f.write(src)
    assert open(p).read().count(f"$i<{N}") == 1
    assert open(p).read().count("new A") == K
    print(p)
