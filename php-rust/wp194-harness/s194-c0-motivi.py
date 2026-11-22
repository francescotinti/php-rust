#!/usr/bin/env python3
"""s194-c0-motivi.py <commit> <orm|wp> ... — S-194 package C, step C0a: how many calls and returns leave the short path on the REAL workloads, and why (census-time + h2 reasons).
Derived from s194-censimento-op.py (manifest s194-c0-motivi-copia.diff). Only adaptations: output names, feature census-time with
PHPR_CENSUS_TIME=<file>, and the report sums the dumps of all processes by key.
Builds <commit> from a git ARCHIVE with the pin recipe plus --features op-census in the twin dirs of the bundle (never the tree,
never the pin), then runs one whole leg per workload with PHPR_OP_CENSUS=<file>. Counting only: no time, no verdict.
Output: s194-c0-motivi-<workload>.txt (raw dump), s194-c0-motivi.out (head), out/c0-motivi.rc.
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
out = W / "s194-c0-motivi.out"; rcf = O / "c0-motivi.rc"; rcf.unlink(missing_ok=True)
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
with (O / "c0-motivi-build.log").open("w") as lg:
    r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli", "--features", "census-time"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
note(f"== s194 C0a motivi di uscita dal cammino corto (census-time), sorgente {c}, build rc={r.returncode}, {time.strftime('%F %T')} ==")
if ar.wait() != 0 or r.returncode != 0: rcf.write_text("7\n"); sys.exit(7)
B = TGT / "release/phpr"; rc = 0
for work in sys.argv[2:]:
    raw = W / f"s194-c0-motivi-{work}.txt"; raw.unlink(missing_ok=True)
    e = dict(os.environ, MIMALLOC_PURGE_DELAY="0", PHPR_CENSUS_TIME=str(raw))
    so = O / f"c0-motivi-{work}-stdout.txt"
    if work == "orm":
        shutil.rmtree(SP / "orm-work", ignore_errors=True); SP.mkdir(parents=True, exist_ok=True)
        with tarfile.open(GATES / "orm-work.tgz") as t: t.extractall(SP)
        with so.open("w") as f: subprocess.run([str(B), "vendor/bin/phpunit", "--no-coverage"], cwd=SP / "orm-work", stdout=f, stderr=subprocess.DEVNULL, env=e)
    else:
        with (O / "c0-motivi-guard.log").open("w") as gl:
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
        cls, h2 = {}, {}
        for l in raw.read_text(errors="replace").splitlines():
            a = l.split("\t")
            if a[0] == "census-time" and len(a) == 5: cls[a[1]] = cls.get(a[1], 0) + int(a[2])
            elif a[0] == "census-h2" and len(a) == 4: h2[(a[1], a[2])] = h2.get((a[1], a[2]), 0) + int(a[3])
        for k, v in cls.items(): note(f"   classe {k:16s} {v:>14d}")
        for (k, r), v in sorted(h2.items(), key=lambda kv: -kv[1])[:45]: note(f"   motivo {k:10s} {v:>13d}  {r[:150]}")
    else: rc = max(rc, 1)
note(f"fine rc={rc} {time.strftime('%F %T')}"); rcf.write_text(f"{rc}\n")
