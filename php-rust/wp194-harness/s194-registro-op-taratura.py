#!/usr/bin/env python3
"""s194-registro-op-taratura.py — S-194: calibrates the per-op residual K of the instruction ledger on the instrumentation binary itself.
For each probe fixture: (a) true instructions per iteration of the census build with the hook DISARMED (/usr/bin/time -l, differential
over two iteration counts), (b) ledger-attributed instructions and ops per iteration with PHPR_OP_INSTR=1. K = (b - a) / ops per iteration.
Output: s194-registro-op-taratura.out. Generato con assistenza AI (Claude Fable 5.2)."""
import os, pathlib, re, subprocess
W = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness"); O = W / "out"
B = pathlib.Path.home() / "Claude/phpr-target/s194-armCensus/release/phpr"
N1, N2 = 1000000, 2000000
env0 = {k: v for k, v in os.environ.items() if not k.startswith("PHPR_")}
def true_instr(fx, n):
    r = subprocess.run(["/usr/bin/time", "-l", str(B), str(W / f"primitive/{fx}.php"), str(n)], capture_output=True, text=True, env=env0)
    return int(re.search(r"(\d+)\s+instructions retired", r.stderr).group(1))
def ledger(fx, n):
    f = O / f"taratura-{fx}-{n}.txt"; f.unlink(missing_ok=True)
    subprocess.run([str(B), str(W / f"primitive/{fx}.php"), str(n)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                   env=dict(env0, PHPR_OP_CENSUS=str(f), PHPR_OP_INSTR="1"))
    t = f.read_text(errors="replace")
    return int(t.split()[2]), int(re.search(r"attributed (\d+)", t).group(1))
rows = ["== s194 taratura del registro: residuo K per op (binario di strumentazione, aggancio spento contro registro) =="]
ks = []
for fx in ("noop", "assign_const", "add", "builtin_strlen", "prop_read", "call_f2", "arr_int_read", "concat"):
    if not (W / f"primitive/{fx}.php").exists(): rows.append(f"{fx}: fixture assente"); continue
    a = (true_instr(fx, N2) - true_instr(fx, N1)) / (N2 - N1)
    (o1, l1), (o2, l2) = ledger(fx, N1), ledger(fx, N2)
    ops, b = (o2 - o1) / (N2 - N1), (l2 - l1) / (N2 - N1)
    k = (b - a) / ops; ks.append(k)
    rows.append(f"{fx:16s} op/giro {ops:5.2f}  vero {a:8.1f}  registro {b:8.1f}  K = {k:6.1f}")
rows.append(f"K min {min(ks):.1f}  max {max(ks):.1f}  media {sum(ks) / len(ks):.1f}")
(W / "s194-registro-op-taratura.out").write_text("\n".join(rows) + "\n")
