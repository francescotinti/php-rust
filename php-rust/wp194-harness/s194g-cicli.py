#!/usr/bin/env python3
"""s194g-cicli.py <tag> <name=path>... — S-194 C5: cycles and instructions per call (/usr/bin/time -l) on the guard fixture
wp185-harness/mcalls-dq.php, the given binaries in rotation (order reversed on odd rounds), R rounds (env R, default 9), median.
Outcome: s194g-cicli-<tag>.out, out/s194g-cicli-<tag>.done. Generato con assistenza AI (Claude Fable 5.2)."""
import os, pathlib, re, statistics as st, subprocess, sys, time
W = pathlib.Path(__file__).resolve().parent
FX = W.parent / "wp185-harness/mcalls-dq.php"
tag = sys.argv[1]; bins = [a.split("=", 1) for a in sys.argv[2:]]; R = int(os.environ.get("R", "9"))
out = W / f"s194g-cicli-{tag}.out"; done = W / f"out/s194g-cicli-{tag}.done"; done.unlink(missing_ok=True)
N = int(re.search(r"\$i<(\d+)", FX.read_text()).group(1))
def count(b):
    r = subprocess.run(["/usr/bin/time", "-l", b, str(FX)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, timeout=900)
    return (int(re.search(r"(\d+)\s+instructions retired", r.stderr).group(1)) / N, int(re.search(r"(\d+)\s+cycles elapsed", r.stderr).group(1)) / N)
d = {n: [] for n, _ in bins}
for r in range(R):
    for n, b in (bins if r % 2 == 0 else bins[::-1]):
        d[n].append(count(b))
def drop1(v):
    return max(abs(st.median(v[:q] + v[q + 1:]) - st.median(v)) for q in range(len(v)))
L = [f"== s194g cicli mcalls-dq tag={tag} N={N} R={R} {time.strftime('%F %T')} =="]
med = {}
for n, _ in bins:
    ins = [x[0] for x in d[n]]; cyc = [x[1] for x in d[n]]; med[n] = (st.median(ins), st.median(cyc))
    L.append(f"{n}: istr {med[n][0]:.2f} · cicli {med[n][1]:.2f} (min {min(cyc):.2f} max {max(cyc):.2f} drop-1 {drop1(cyc):.2f}) · corse=[{' '.join('%.1f' % v for v in cyc)}]")
ref = bins[0][0]
for n, _ in bins[1:]:
    L.append(f"D {n} - {ref}: istr {med[n][0] - med[ref][0]:+.2f} · cicli {med[n][1] - med[ref][1]:+.2f}")
out.write_text("\n".join(L) + "\n"); done.write_text("0\n")
