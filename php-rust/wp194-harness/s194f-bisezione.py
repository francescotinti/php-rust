#!/usr/bin/env python3
"""s194f-bisezione.py <tag> <commit>... — S-194, criterion s194-criterio-notte-b.md p.19: WHICH step of pin s190 brought the
cycles-per-method-call cost seen by the guard? Declared copy of wp192-harness/s192-bisezione-k0.py (see gen-bisezione-s194f.py):
same build half (git ARCHIVE, pin recipe, one source dir and one target dir on the bundle), new measuring half: cycles and
instructions per call (/usr/bin/time -l) on wp185-harness/mcalls-dq.php, every binary in rotation, R rounds, median; the stash
s189 copy and the canonical s190 run in the same rotation as controls. Attribution only, never a verdict.
Results: out/bisezione-<tag>.out, rc in out/bisezione-<tag>.rc. Generato con assistenza AI (Claude Fable 5.2).
"""
import hashlib, os, pathlib, re, resource, shutil, statistics, subprocess, sys, time

P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
O = P / "wp194-harness/out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT = BT / "s194f-src", BT / "s194f-arm"
FX = P / "wp185-harness/mcalls-dq.php"
EMPTY = P / "wp160-harness/empty.php"
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
tag, commits = sys.argv[1], sys.argv[2:]
R = int(os.environ.get("R", "9"))
out = O / f"bisezione-{tag}.out"
rcf = O / f"bisezione-{tag}.rc"
O.mkdir(parents=True, exist_ok=True)
rcf.unlink(missing_ok=True)


def note(s):
    with out.open("a") as f:
        f.write(s + "\n")


def cargo_busy():
    return subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0


def build(c):
    shutil.rmtree(SRC, ignore_errors=True)
    SRC.mkdir(parents=True)
    paths = ["php-rust/" + n for n in ("crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo")]
    ar = subprocess.Popen(["git", "-C", str(P.parent), "archive", c] + paths, stdout=subprocess.PIPE)
    # -m: mtime = now. An archive carries the COMMIT time, and cargo (mtime fingerprints, same paths) would keep a stale
    # rlib of an OLDER commit built after a newer one (run b1: 901515b6 failed to link against the 483e970f php-types).
    subprocess.run(["tar", "-x", "-m", "-C", str(SRC)], stdin=ar.stdout, check=True)
    assert ar.wait() == 0
    env = dict(os.environ, SOURCE_DATE_EPOCH="0", CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(TGT))
    with (O / f"bisezione-{tag}-build-{c}.log").open("w") as lg:
        r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
    if r.returncode != 0:
        return None
    dst = O / f"phpr-bis-{tag}-{c}"
    shutil.copy2(TGT / "release/phpr", dst)
    return dst


def count(binary):
    r = subprocess.run(["/usr/bin/time", "-l", str(binary), str(FX)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, timeout=900)
    return (int(re.search(r"(\d+)\s+instructions retired", r.stderr).group(1)), int(re.search(r"(\d+)\s+cycles elapsed", r.stderr).group(1)))


N = int(re.search(r"\$i<(\d+)", FX.read_text()).group(1))
out.write_text(f"== s194f bisezione mcalls-dq tag={tag} N={N} R={R} {time.strftime('%F %T')} (attribuzione, non verdetto) ==\n")
while cargo_busy() or not FEED.read_text().strip().splitlines()[-1].startswith("DONE"):
    time.sleep(30)
bins = []
for c in commits:
    b = build(c)
    if b is None:
        note(f"build FAILED {c}")
        rcf.write_text("7\n")
        sys.exit(7)
    h = hashlib.sha256(b.read_bytes()).hexdigest()[:16]
    note(f"build {c} hash {h} {time.strftime('%T')}")
    bins.append((c, b))
PINS = [("stash-s189", pathlib.Path("/private/tmp/phpr-s194c-bin/phpr-s189"), "2d42efd6606e8dd9"),
        ("canon-s190", pathlib.Path.home() / "Claude/php-rust-output/release/phpr", "97e000aa00fe802e")]
for n, b, h in PINS:
    if hashlib.sha256(b.read_bytes()).hexdigest()[:16] != h:
        note(f"hash inatteso {n}"); rcf.write_text("7\n"); sys.exit(7)
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
note(f"fine {time.strftime('%F %T')}")
rcf.write_text("0\n")
