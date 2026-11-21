#!/usr/bin/env python3
"""s194-orm-istruzioni.py <label>=<commit|path> ... — S-194: retired instructions of ONE whole ORM leg (phpunit, 3484 tests) per binary.
A commit is built from a git ARCHIVE (never the tree) with the pin recipe in the twin source/target dirs of the bundle (same
recipe as s192-gemelli-orm.py); a path is used as it is. Each binary runs REPS legs on a fresh untar of orm-work.tgz under
/usr/bin/time -l. Instructions are load-independent: direction on the WORK done, never a time verdict.
Output: s194-orm-istruzioni.out, out/orm-istruzioni.rc. Generato con assistenza AI (Claude Fable 5.2)."""
import hashlib, os, pathlib, re, shutil, statistics, subprocess, sys, tarfile, time
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
W = P / "wp194-harness"; O = W / "out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT, SP = BT / "s192-srcB", BT / "s192-armB", BT / "s192-orm"
GATES = pathlib.Path("/Volumes/Extreme Pro/Claude/wp9-harness/gates")
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
REPS = int(os.environ.get("REPS", "2"))
out = W / "s194-orm-istruzioni.out"; rcf = O / "orm-istruzioni.rc"; rcf.unlink(missing_ok=True)
def note(s):
    with out.open("a") as f: f.write(s + "\n")
def build(c, label):
    shutil.rmtree(SRC, ignore_errors=True); SRC.mkdir(parents=True)
    paths = ["php-rust/" + n for n in ("crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo")]
    ar = subprocess.Popen(["git", "-C", str(P.parent), "archive", c] + paths, stdout=subprocess.PIPE)
    subprocess.run(["tar", "-x", "-m", "-C", str(SRC)], stdin=ar.stdout, check=True)
    if ar.wait() != 0: return None
    env = dict(os.environ, SOURCE_DATE_EPOCH="0", CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(TGT))
    with (O / f"orm-istruzioni-build-{label}.log").open("w") as lg:
        r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
    if r.returncode != 0: return None
    dst = O / f"phpr-orm-istr-{label}"; shutil.copy2(TGT / "release/phpr", dst); return dst
def leg(binary, label, r):
    shutil.rmtree(SP / "orm-work", ignore_errors=True); SP.mkdir(parents=True, exist_ok=True)
    with tarfile.open(GATES / "orm-work.tgz") as t: t.extractall(SP)
    so = O / f"orm-istruzioni-{label}-{r}.txt"
    with so.open("w") as f:
        p = subprocess.run(["/usr/bin/time", "-l", str(binary), "vendor/bin/phpunit", "--no-coverage"], cwd=SP / "orm-work", stdout=f,
                           stderr=subprocess.PIPE, env=dict(os.environ, MIMALLOC_PURGE_DELAY="0"))
    err = p.stderr.decode("latin-1")
    ins = int(re.search(r"(\d+)\s+instructions retired", err).group(1)); cyc = int(re.search(r"(\d+)\s+cycles elapsed", err).group(1))
    usr = float(re.search(r"([\d.]+) user", err).group(1))
    txt = so.read_bytes().replace(b"\0", b"").decode("utf-8", "replace")
    tests = next((l.strip() for l in txt.splitlines() if l.startswith("Tests: ")), "nessuna riga Tests")
    return ins, cyc, usr, tests
while subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0 or not FEED.read_text().strip().splitlines()[-1].startswith("DONE"):
    time.sleep(30)
note(f"== s194 ORM istruzioni ritirate per gamba, REPS={REPS}, {time.strftime('%F %T')} (direzione sul lavoro, non verdetto di tempo) ==")
bins, rc = [], 0
for a in sys.argv[1:]:
    label, what = a.split("=", 1)
    b = pathlib.Path(os.path.expanduser(what)) if "/" in what else build(what, label)
    if b is None or not b.exists(): note(f"{label}: build FALLITA"); rc = 7; continue
    note(f"binario {label} = {what} sha {hashlib.sha256(b.read_bytes()).hexdigest()[:16]}"); bins.append((label, b))
base = None
for label, b in bins:
    rows = [leg(b, label, r) for r in range(REPS)]
    ins = statistics.median(x[0] for x in rows); sp = (max(x[0] for x in rows) - min(x[0] for x in rows)) / ins * 100
    if base is None: base = ins
    note(f"{label:10s} istruzioni {ins / 1e9:9.3f} G (dispersione {sp:.2f} %)  vs {bins[0][0]} {100 * (ins / base - 1):+6.2f} %  cicli {statistics.median(x[1] for x in rows) / 1e9:8.3f} G  user {statistics.median(x[2] for x in rows):6.2f} s  | {rows[0][3][:60]}")
    if not all("Tests: 3484," in x[3] for x in rows): rc = max(rc, 1)
note(f"fine rc={rc} {time.strftime('%F %T')}"); rcf.write_text(f"{rc}\n")
