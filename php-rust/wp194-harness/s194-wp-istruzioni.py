#!/usr/bin/env python3
"""s194-wp-istruzioni.py <label>=<commit|path> ... — S-194: retired instructions of ONE whole WordPress leg (phpunit full, the
"full" leg of wp109-harness/pair109.sh, phpr side only) per binary. DECLARED COPY of s194-orm-istruzioni.py (manifest
s194-wp-istruzioni-copia.diff): only the workload (wpdev + fresh wptests + uploads guard), per-label reps (label*2=...), names.
A commit is built from a git ARCHIVE (never the tree) with the pin recipe in the twin source/target dirs of the bundle (same
recipe as s192-gemelli-orm.py); a path is used as it is. Each binary runs its legs under /usr/bin/time -l. Instructions are load-independent: direction on the WORK done, never a time verdict.
Output: s194-wp-istruzioni.out, out/wp-istruzioni.rc. Generato con assistenza AI (Claude Fable 5.2)."""
import hashlib, os, pathlib, re, shutil, statistics, subprocess, sys, time
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
W = P / "wp194-harness"; O = W / "out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT = BT / "s192-srcB", BT / "s192-armB"
WPDEV = pathlib.Path.home() / "Claude/wpdev"; UP = WPDEV / "src/wp-content/uploads"
GUARD = "/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
out = W / "s194-wp-istruzioni.out"; rcf = O / "wp-istruzioni.rc"; rcf.unlink(missing_ok=True)
def note(s):
    with out.open("a") as f: f.write(s + "\n")
def build(c, label):
    shutil.rmtree(SRC, ignore_errors=True); SRC.mkdir(parents=True)
    paths = ["php-rust/" + n for n in ("crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo")]
    ar = subprocess.Popen(["git", "-C", str(P.parent), "archive", c] + paths, stdout=subprocess.PIPE)
    subprocess.run(["tar", "-x", "-m", "-C", str(SRC)], stdin=ar.stdout, check=True)
    if ar.wait() != 0: return None
    env = dict(os.environ, SOURCE_DATE_EPOCH="0", CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(TGT))
    with (O / f"wp-istruzioni-build-{label}.log").open("w") as lg:
        r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
    if r.returncode != 0: return None
    dst = O / f"phpr-orm-istr-{label}"; shutil.copy2(TGT / "release/phpr", dst); return dst
def wipe():
    for p in UP.iterdir(): shutil.rmtree(p) if p.is_dir() and not p.is_symlink() else p.unlink()
def leg(binary, label, r):
    subprocess.run(["mysql", "-h", "127.0.0.1", "-u", "root", "-e",
                    "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';"], check=True)
    wipe()
    so = O / f"wp-istruzioni-{label}-{r}.txt"
    with so.open("w") as f:
        p = subprocess.run(["/usr/bin/time", "-l", str(binary), "vendor/bin/phpunit"], cwd=WPDEV, stdout=f,
                           stderr=subprocess.PIPE, env=dict(os.environ, MIMALLOC_PURGE_DELAY="0"))
    err = p.stderr.decode("latin-1")
    ins = int(re.search(r"(\d+)\s+instructions retired", err).group(1)); cyc = int(re.search(r"(\d+)\s+cycles elapsed", err).group(1))
    usr = float(re.search(r"([\d.]+) user", err).group(1))
    txt = so.read_bytes().replace(b"\0", b"").decode("utf-8", "replace")
    tests = next((l.strip() for l in reversed(txt.splitlines()) if l.startswith(("Tests: ", "OK ("))), "nessuna riga Tests")
    return ins, cyc, usr, tests
while subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0 or not FEED.read_text().strip().splitlines()[-1].startswith("DONE"):
    time.sleep(30)
note(f"== s194 WordPress (phpunit full) istruzioni ritirate per gamba, {time.strftime('%F %T')} (direzione sul lavoro, non verdetto di tempo) ==")
bins, rc = [], 0
for a in sys.argv[1:]:
    label, what = a.split("=", 1); label, _, n = label.partition("*")
    b = pathlib.Path(os.path.expanduser(what)) if "/" in what else build(what, label)
    if b is None or not b.exists(): note(f"{label}: build FALLITA"); rc = 7; continue
    note(f"binario {label} = {what} sha {hashlib.sha256(b.read_bytes()).hexdigest()[:16]}"); bins.append((label, b, int(n or 1)))
base = None; first = None
with (O / "wp-istruzioni-guard.log").open("w") as gl:
    if subprocess.run([GUARD, "backup-wipe"], stdout=gl, stderr=gl).returncode != 0:
        note("uploads-guard backup-wipe FALLITO: nessuna gamba"); rcf.write_text("4\n"); sys.exit(4)
try:
  for label, b, reps in bins:
    rows = [leg(b, label, r) for r in range(reps)]
    ins = statistics.median(x[0] for x in rows); sp = (max(x[0] for x in rows) - min(x[0] for x in rows)) / ins * 100
    if base is None: base = ins
    note(f"{label:10s} istruzioni {ins / 1e9:9.3f} G (dispersione {sp:.2f} %)  vs {bins[0][0]} {100 * (ins / base - 1):+6.2f} %  cicli {statistics.median(x[1] for x in rows) / 1e9:8.3f} G  user {statistics.median(x[2] for x in rows):6.2f} s  | {rows[0][3][:90]}")
    if first is None: first = rows[0][3]
    if not all(x[3] == first and x[3].startswith(("Tests: ", "OK (")) for x in rows): rc = max(rc, 1)
finally:
    wipe()
    with (O / "wp-istruzioni-guard.log").open("a") as gl: g = subprocess.run([GUARD, "restore"], stdout=gl, stderr=gl).returncode
    note(f"uploads-guard restore rc={g}"); rc = max(rc, 2 if g else 0)
note(f"fine rc={rc} {time.strftime('%F %T')}"); rcf.write_text(f"{rc}\n")
