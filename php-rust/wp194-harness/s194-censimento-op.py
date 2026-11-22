#!/usr/bin/env python3
"""s194-censimento-op.py <commit> <orm|wp> ... — S-194: which VM ops the REAL workloads execute (op census, instrumentation build).
Builds <commit> from a git ARCHIVE with the pin recipe plus --features op-census in the twin dirs of the bundle (never the tree,
never the pin), then runs one whole leg per workload with PHPR_OP_CENSUS=<file>. Counting only: no time, no verdict.
Output: s194-censimento-op-<workload>.txt (raw dump), s194-censimento-op.out (head), out/censimento-op.rc.
Generato con assistenza AI (Claude Fable 5.2)."""
import os, pathlib, shutil, subprocess, sys, tarfile, time
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
W = P / "wp194-harness"; O = W / "out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT, SP = BT / "s192-srcB", BT / "s194-armCensus", BT / "s192-orm"
GATES = pathlib.Path("/Volumes/Extreme Pro/Claude/wp9-harness/gates")
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
WPDEV = pathlib.Path.home() / "Claude/wpdev"; UP = WPDEV / "src/wp-content/uploads"
GUARD = "/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
out = W / "s194-censimento-op.out"; rcf = O / "censimento-op.rc"; rcf.unlink(missing_ok=True)
def note(s):
    with out.open("a") as f: f.write(s + "\n")
def wipe():
    for p in UP.iterdir(): shutil.rmtree(p) if p.is_dir() and not p.is_symlink() else p.unlink()
while subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0 or not FEED.read_text().strip().splitlines()[-1].startswith("DONE"):
    time.sleep(30)
c = sys.argv[1]
shutil.rmtree(SRC, ignore_errors=True); SRC.mkdir(parents=True)
paths = ["php-rust/" + n for n in ("crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo")]
ar = subprocess.Popen(["git", "-C", str(P.parent), "archive", c] + paths, stdout=subprocess.PIPE)
subprocess.run(["tar", "-x", "-m", "-C", str(SRC)], stdin=ar.stdout, check=True)
env = dict(os.environ, SOURCE_DATE_EPOCH="0", CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(TGT))
with (O / "censimento-op-build.log").open("w") as lg:
    r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli", "--features", "op-census"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
note(f"== s194 censimento degli op, sorgente {c}, build rc={r.returncode}, {time.strftime('%F %T')} ==")
if ar.wait() != 0 or r.returncode != 0: rcf.write_text("7\n"); sys.exit(7)
B = TGT / "release/phpr"; rc = 0
for work in sys.argv[2:]:
    raw = W / f"s194-censimento-op-{work}.txt"; raw.unlink(missing_ok=True)
    e = dict(os.environ, MIMALLOC_PURGE_DELAY="0", PHPR_OP_CENSUS=str(raw), PHPR_CENSUS_HITS="0")
    so = O / f"censimento-op-{work}-stdout.txt"
    if work == "orm":
        shutil.rmtree(SP / "orm-work", ignore_errors=True); SP.mkdir(parents=True, exist_ok=True)
        with tarfile.open(GATES / "orm-work.tgz") as t: t.extractall(SP)
        with so.open("w") as f: subprocess.run([str(B), "vendor/bin/phpunit", "--no-coverage"], cwd=SP / "orm-work", stdout=f, stderr=subprocess.DEVNULL, env=e)
    else:
        with (O / "censimento-op-guard.log").open("w") as gl:
            if subprocess.run([GUARD, "backup-wipe"], stdout=gl, stderr=gl).returncode != 0: note("guard FALLITA"); rc = 4; continue
            try:
                subprocess.run(["mysql", "-h", "127.0.0.1", "-u", "root", "-e",
                                "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';"], check=True)
                wipe()
                with so.open("w") as f: subprocess.run([str(B), "vendor/bin/phpunit"], cwd=WPDEV, stdout=f, stderr=subprocess.DEVNULL, env=e)
            finally:
                wipe(); g = subprocess.run([GUARD, "restore"], stdout=gl, stderr=gl).returncode
                note(f"uploads-guard restore rc={g}"); rc = max(rc, 2 if g else 0)
    txt = so.read_bytes().replace(b"\0", b"").decode("utf-8", "replace")
    tests = next((l.strip() for l in reversed(txt.splitlines()) if l.startswith(("Tests: ", "OK ("))), "nessuna riga Tests")
    note(f"-- {work}: {tests[:90]} | dump {raw.name} {'presente' if raw.exists() else 'ASSENTE'}")
    if raw.exists():
        for l in raw.read_text(errors="replace").splitlines()[:70]: note("   " + l[:200])
    else: rc = max(rc, 1)
note(f"fine rc={rc} {time.strftime('%F %T')}"); rcf.write_text(f"{rc}\n")
