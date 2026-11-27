#!/usr/bin/env python3
"""gen-bisezione-s194f.py — S-194 inquiry step 2: s194f-bisezione.py as a declared copy of wp192-harness/s192-bisezione-k0.py.
ONLY adaptations: paths (harness, source and target dirs), the docstring, and the measuring half (cycles and instructions per
method call from /usr/bin/time -l on the guard fixture mcalls-dq, all binaries in rotation, plus the two pinned binaries as
controls) in place of the paired user-CPU timing of ret-buf-dq-k0. The build half (git archive, pin recipe, same source and
target dir for every commit) is untouched. Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, subprocess
W = pathlib.Path(__file__).resolve().parent
SRCF = W.parent / "wp192-harness/s192-bisezione-k0.py"
s = SRCF.read_text()
def sub(a, b):
    global s
    assert s.count(a) == 1, (a, s.count(a)); s = s.replace(a, b)
d0 = s.index('"""'); doc = s[d0:s.index('"""', d0 + 3) + 3]
sub(doc, '"""s194f-bisezione.py <tag> <commit>... — S-194, criterion s194-criterio-notte-b.md p.19: WHICH step of pin s190 brought the\n'
    'cycles-per-method-call cost seen by the guard? Declared copy of wp192-harness/s192-bisezione-k0.py (see gen-bisezione-s194f.py):\n'
    'same build half (git ARCHIVE, pin recipe, one source dir and one target dir on the bundle), new measuring half: cycles and\n'
    'instructions per call (/usr/bin/time -l) on wp185-harness/mcalls-dq.php, every binary in rotation, R rounds, median; the stash\n'
    's189 copy and the canonical s190 run in the same rotation as controls. Attribution only, never a verdict.\n'
    'Results: out/bisezione-<tag>.out, rc in out/bisezione-<tag>.rc. Generato con assistenza AI (Claude Fable 5.2).\n"""')
sub('O = P / "wp192-harness/out"', 'O = P / "wp194-harness/out"')
sub('SRC, TGT = BT / "s192-srcB", BT / "s192-armB"', 'SRC, TGT = BT / "s194f-src", BT / "s194f-arm"')
sub('K0 = P / "wp186-harness/ret-buf-dq-k0.php"', 'FX = P / "wp185-harness/mcalls-dq.php"')
sub('R = int(os.environ.get("R", "5"))', 'R = int(os.environ.get("R", "9"))')
i = s.index("def ucpu(binary, script):"); j = s.index("while cargo_busy()")
s = s[:i] + '''def count(binary):
    r = subprocess.run(["/usr/bin/time", "-l", str(binary), str(FX)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, timeout=900)
    return (int(re.search(r"(\\d+)\\s+instructions retired", r.stderr).group(1)), int(re.search(r"(\\d+)\\s+cycles elapsed", r.stderr).group(1)))


N = int(re.search(r"\\$i<(\\d+)", FX.read_text()).group(1))
out.write_text(f"== s194f bisezione mcalls-dq tag={tag} N={N} R={R} {time.strftime('%F %T')} (attribuzione, non verdetto) ==\\n")
''' + s[j:]
k = s.index("ref_c, ref = bins[0]"); e = s.index('note(f"fine ')
s = s[:k] + '''PINS = [("stash-s189", pathlib.Path("/private/tmp/phpr-s194c-bin/phpr-s189"), "2d42efd6606e8dd9"),
        ("canon-s190", pathlib.Path.home() / "Claude/php-rust-output/release/phpr", "97e000aa00fe802e")]
for n, b, h in PINS:
    if hashlib.sha256(b.read_bytes()).hexdigest()[:16] != h:
        note(f"hash inatteso {n}"); rcf.write_text("7\\n"); sys.exit(7)
    bins.append((n, b))
d = {c: [] for c, _ in bins}
for r in range(R):
    order = bins if r % 2 == 0 else bins[::-1]
    for c, b in order:
        d[c].append(count(b))
def drop1(v):
    return max(abs(statistics.median(v[:q] + v[q + 1:]) - statistics.median(v)) for q in range(len(v)))
med = {}
for c, _ in bins:
    ins = [x[0] / N for x in d[c]]; cyc = [x[1] / N for x in d[c]]
    med[c] = (statistics.median(ins), statistics.median(cyc))
    note(f"{c}: istr {med[c][0]:.2f} · cicli {med[c][1]:.2f} (min {min(cyc):.2f} max {max(cyc):.2f} drop-1 {drop1(cyc):.2f}) · corse=[{' '.join('%.1f' % v for v in cyc)}]")
names = [c for c, _ in bins[:-2]]
for a, b in zip(names, names[1:]):
    note(f"passo {a} -> {b}: istr {med[b][0] - med[a][0]:+.2f} · cicli {med[b][1] - med[a][1]:+.2f}")
note(f"BRACCIO ultimo - primo: istr {med[names[-1]][0] - med[names[0]][0]:+.2f} · cicli {med[names[-1]][1] - med[names[0]][1]:+.2f}")
note(f"PIN canon-s190 - stash-s189: istr {med['canon-s190'][0] - med['stash-s189'][0]:+.2f} · cicli {med['canon-s190'][1] - med['stash-s189'][1]:+.2f}")
''' + s[e:]
sub("tag, commits = sys.argv[1], sys.argv[2:]", "tag, commits = sys.argv[1], sys.argv[2:]")
assert "K0" not in s and "ucpu" not in s and "floor" not in s and "RPI" not in s, "leftover of the timing half"
(W / "s194f-bisezione.py").write_text(s)
d = subprocess.run(["diff", str(SRCF), str(W / "s194f-bisezione.py")], capture_output=True, text=True).stdout
(W / "s194f-bisezione-copia.diff").write_text(d)
print("ok", len(d.splitlines()), "righe di manifest")
