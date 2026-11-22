#!/usr/bin/env python3
"""s194-registro-op.py <commit> <orm|wp> ... — S-194: retired instructions per VM op type on the REAL workloads (instruction ledger, PHPR_OP_INSTR).
Derived from s194-censimento-op.py (manifest s194-registro-op-copia.diff). Only adaptations: output names, PHPR_OP_INSTR=1,
a probe sanity run before the workloads, and the report picks the MAIN process dump (the largest) instead of the first.
Builds <commit> from a git ARCHIVE with the pin recipe plus --features op-census in the twin dirs of the bundle (never the tree,
never the pin), then runs one whole leg per workload with PHPR_OP_CENSUS=<file>. Counting only: no time, no verdict.
Output: s194-registro-op-<workload>.txt (raw dump), s194-registro-op.out (head), out/registro-op.rc.
Generato con assistenza AI (Claude Fable 5.2)."""
import os, pathlib, re, shutil, subprocess, sys, tarfile, time
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
W = P / "wp194-harness"; O = W / "out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT, SP = BT / "s192-srcB", BT / "s194-armCensus", BT / "s192-orm"
GATES = pathlib.Path("/Volumes/Extreme Pro/Claude/wp9-harness/gates")
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
WPDEV = pathlib.Path.home() / "Claude/wpdev"; UP = WPDEV / "src/wp-content/uploads"
GUARD = "/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
out = W / "s194-registro-op.out"; rcf = O / "registro-op.rc"; rcf.unlink(missing_ok=True)
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
with (O / "registro-op-build.log").open("w") as lg:
    r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli", "--features", "op-census"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
note(f"== s194 registro delle istruzioni per op, sorgente {c}, build rc={r.returncode}, {time.strftime('%F %T')} ==")
if ar.wait() != 0 or r.returncode != 0: rcf.write_text("7\n"); sys.exit(7)
B = TGT / "release/phpr"; rc = 0
def dumps(p):
    return sorted((d for d in re.split(r"(?m)^(?=== PHPR_OP_CENSUS: )", p.read_text(errors="replace")) if d.startswith("== PHPR_OP_CENSUS")),
                  key=lambda d: -int(d.split()[2]))
att = []
for n in (1000000, 2000000):
    sf = O / f"registro-op-sonda-{n}.txt"; sf.unlink(missing_ok=True)
    subprocess.run([str(B), str(W / "primitive/assign_const.php"), str(n)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                   env=dict(os.environ, PHPR_OP_CENSUS=str(sf), PHPR_OP_INSTR="1"))
    d = dumps(sf)[0]; att.append((int(d.split()[2]), int(re.search(r"attributed (\d+)", d).group(1))))
dop, din = att[1][0] - att[0][0], att[1][1] - att[0][1]
note(f"-- sanita' sulla sonda assign_const: {dop / 1e6:.2f} op per giro, {din / 1e6:.1f} istruzioni attribuite per giro (produzione braccio A: 331; banda 248-414)")
if not 248 <= din / 1e6 <= 414: rc = 3
for work in sys.argv[2:]:
    raw = W / f"s194-registro-op-{work}.txt"; raw.unlink(missing_ok=True)
    e = dict(os.environ, MIMALLOC_PURGE_DELAY="0", PHPR_OP_CENSUS=str(raw), PHPR_CENSUS_HITS="0", PHPR_OP_INSTR="1")
    so = O / f"registro-op-{work}-stdout.txt"
    if work == "orm":
        shutil.rmtree(SP / "orm-work", ignore_errors=True); SP.mkdir(parents=True, exist_ok=True)
        with tarfile.open(GATES / "orm-work.tgz") as t: t.extractall(SP)
        with so.open("w") as f: subprocess.run([str(B), "vendor/bin/phpunit", "--no-coverage"], cwd=SP / "orm-work", stdout=f, stderr=subprocess.DEVNULL, env=e)
    else:
        with (O / "registro-op-guard.log").open("w") as gl:
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
        ds = dumps(raw); kids = sum(int(re.search(r"attributed (\d+)", d).group(1)) for d in ds[1:])
        note(f"   {len(ds)} dump; processo principale qui sotto; sottoprocessi: {sum(int(d.split()[2]) for d in ds[1:])} op, {kids} istruzioni attribuite")
        m = ds[0].splitlines(); cut = next(i for i, l in enumerate(m) if l.startswith("-- op counts"))
        nm = next(i for i, l in enumerate(m) if l.startswith("-- instructions by builtin name"))
        for l in m[:min(cut, 63)] + m[nm:nm + 46]: note("   " + l[:200])
    else: rc = max(rc, 1)
note(f"fine rc={rc} {time.strftime('%F %T')}"); rcf.write_text(f"{rc}\n")
