#!/usr/bin/env python3
"""s192-bisezione-k0.py <tag> <ref-commit> <commit>... — S-192: WHERE does the K0 cost of pin s184 over s183 come from?

Every commit is built from a git ARCHIVE (never the tree) with the pin recipe, all in the SAME source dir and the SAME
target dir on the bundle (~/Claude/phpr-target/s192-srcB, s192-armB: identical paths, so the layout differs only by the
code). The first commit is the reference; each later binary is timed against it on ret-buf-dq-k0 (R rounds, ABAB
interleaved, user CPU of the child, floor = empty.php median of 3), estimator = median of the paired differences in
ns/Ret. DIRECTION and attribution only, never a verdict. Results: out/bisezione-<tag>.out, rc in out/bisezione-<tag>.rc.
Generato con assistenza AI (Claude Fable 5.2).
"""
import hashlib, os, pathlib, re, resource, shutil, statistics, subprocess, sys, time

P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
O = P / "wp192-harness/out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT = BT / "s192-srcB", BT / "s192-armB"
K0 = P / "wp186-harness/ret-buf-dq-k0.php"
EMPTY = P / "wp160-harness/empty.php"
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
tag, commits = sys.argv[1], sys.argv[2:]
R = int(os.environ.get("R", "5"))
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


def ucpu(binary, script):
    before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_utime
    subprocess.run([str(binary), str(script)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=900)
    return resource.getrusage(resource.RUSAGE_CHILDREN).ru_utime - before


src = K0.read_text()
N = int(re.search(r"\$i<(\d+)", src).group(1))
RPI = int(re.search(r"RETS_PER_ITER=(\d+)", src).group(1))
out.write_text(f"== s192 bisezione K0 tag={tag} N={N}x{RPI} Ret R={R} {time.strftime('%F %T')} (attribuzione, non verdetto) ==\n")
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
ref_c, ref = bins[0]
floor = {c: statistics.median(ucpu(b, EMPTY) for _ in range(3)) for c, b in bins}
for c, b in bins[1:]:
    ds, ta, tb = [], [], []
    for r in range(R):
        if r % 2 == 0:
            a = ucpu(ref, K0); x = ucpu(b, K0)
        else:
            x = ucpu(b, K0); a = ucpu(ref, K0)
        ta.append(a); tb.append(x)
        ds.append(((x - floor[c]) - (a - floor[ref_c])) / (N * RPI) * 1e9)
    note(f"{c} vs {ref_c}: ref=[{' '.join('%.2f' % v for v in ta)}] s · B=[{' '.join('%.2f' % v for v in tb)}] s · "
         f"D appaiato=[{' '.join('%+.2f' % v for v in ds)}] ns/Ret · mediana {statistics.median(ds):+.2f}")
note(f"fine {time.strftime('%F %T')}")
rcf.write_text("0\n")
