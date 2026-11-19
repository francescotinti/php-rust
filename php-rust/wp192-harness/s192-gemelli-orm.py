#!/usr/bin/env python3
"""s192-gemelli-orm.py <tag> <commit-Z> <commit-L> — S-192 ter: twin judge on ONE whole ORM leg (criterion s192-criterio-hasher.md p.5).

Z and L are built from git ARCHIVES (never the tree) with the pin recipe, in the SAME source and target directories on the
bundle (s192-srcB / s192-armB, `tar -m`), so the two binaries differ only by the commit. Then R rounds, interleaved
Z L L Z …, each a fresh untar of orm-work.tgz + phpunit; the figure is the child's USER CPU (rusage). Estimator = median of
the paired differences in % of Z. Direction, never a verdict. Outputs: s192-gemelli-orm-<tag>.out, out/gemelli-orm-<tag>.rc
(0 = L−Z ≤ −1.0 %, 1 = otherwise, 7 = build/validity). Generato con assistenza AI (Claude Fable 5.2).
"""
import hashlib, os, pathlib, resource, shutil, statistics, subprocess, sys, tarfile, time

P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
H = P / "wp192-harness"
O = H / "out"
BT = pathlib.Path.home() / "Claude/phpr-target"
SRC, TGT, SP = BT / "s192-srcB", BT / "s192-armB", BT / "s192-orm"
GATES = pathlib.Path("/Volumes/Extreme Pro/Claude/wp9-harness/gates")
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
tag, cz, cl = sys.argv[1:4]
R = int(os.environ.get("R", "5"))
out = H / f"s192-gemelli-orm-{tag}.out"
rcf = O / f"gemelli-orm-{tag}.rc"
O.mkdir(parents=True, exist_ok=True)
rcf.unlink(missing_ok=True)
out.write_text("")


def note(s):
    with out.open("a") as f:
        f.write(s + "\n")


def fin(rc):
    rcf.write_text(f"{rc}\n")
    sys.exit(rc)


def build(c, letter):
    shutil.rmtree(SRC, ignore_errors=True)
    SRC.mkdir(parents=True)
    paths = ["php-rust/" + n for n in ("crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo")]
    ar = subprocess.Popen(["git", "-C", str(P.parent), "archive", c] + paths, stdout=subprocess.PIPE)
    subprocess.run(["tar", "-x", "-m", "-C", str(SRC)], stdin=ar.stdout, check=True)
    if ar.wait() != 0:
        return None
    env = dict(os.environ, SOURCE_DATE_EPOCH="0", CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(TGT))
    with (O / f"gemelli-orm-{tag}-build{letter}.log").open("w") as lg:
        r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli"], cwd=SRC / "php-rust", env=env, stdout=lg, stderr=lg)
    if r.returncode != 0:
        return None
    dst = O / f"phpr-orm-{tag}-{letter}"
    shutil.copy2(TGT / "release/phpr", dst)
    note(f"braccio {letter} commit {c} hash {hashlib.sha256(dst.read_bytes()).hexdigest()[:16]} {time.strftime('%T')}")
    return dst


def leg(binary, label):
    shutil.rmtree(SP / "orm-work", ignore_errors=True)
    SP.mkdir(parents=True, exist_ok=True)
    with tarfile.open(GATES / "orm-work.tgz") as t:
        t.extractall(SP)
    before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_utime
    with (O / f"gemelli-orm-{tag}-{label}.txt").open("w") as so:
        subprocess.run([str(binary), "vendor/bin/phpunit", "--no-coverage"], cwd=SP / "orm-work", stdout=so,
                       stderr=subprocess.DEVNULL, env=dict(os.environ, MIMALLOC_PURGE_DELAY="0"))
    u = resource.getrusage(resource.RUSAGE_CHILDREN).ru_utime - before
    txt = (O / f"gemelli-orm-{tag}-{label}.txt").read_bytes().replace(b"\0", b"").decode("utf-8", "replace")
    ok = any(l.startswith("Tests: 3484,") for l in txt.splitlines())
    return u, ok


while subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0 or \
        not FEED.read_text().strip().splitlines()[-1].startswith("DONE"):
    time.sleep(30)
note(f"== s192 gemelli ORM tag={tag} Z={cz} L={cl} R={R} {time.strftime('%F %T')} (direzione, non verdetto) ==")
z = build(cz, "Z")
l = build(cl, "L")
if z is None or l is None:
    note("build FALLITA")
    fin(7)
leg(z, "warm")  # unjudged warm-up (file cache)
tz, tl, d = [], [], []
for r in range(R):
    order = [("Z", z), ("L", l)] if r % 2 == 0 else [("L", l), ("Z", z)]
    got = {}
    for name, b in order:
        u, ok = leg(b, f"{name}{r}")
        if not ok:
            note(f"gamba {name}{r} NON valida (Tests: 3484 assente)")
            fin(7)
        got[name] = u
    tz.append(got["Z"]); tl.append(got["L"])
    d.append(100 * (got["L"] - got["Z"]) / got["Z"])
shutil.rmtree(SP / "orm-work", ignore_errors=True)
med = statistics.median(d)
note(f"user Z=[{' '.join('%.2f' % v for v in tz)}] s · L=[{' '.join('%.2f' % v for v in tl)}] s")
note(f"L−Z appaiato=[{' '.join('%+.2f' % v for v in d)}] % · mediana {med:+.2f} % · dispersione di Z (max−min)/mediana {100 * (max(tz) - min(tz)) / statistics.median(tz):.2f} %")
note(f"DIREZIONE: {'L−Z ≤ −1,0 %: attesa rispettata' if med <= -1.0 else 'attesa NON rispettata'} · fine {time.strftime('%F %T')}")
fin(0 if med <= -1.0 else 1)
