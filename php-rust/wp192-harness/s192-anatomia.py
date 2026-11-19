#!/usr/bin/env python3
"""s192-anatomia.py <tag> — S-192 bis: per-op anatomy of the interpreter loop's SELF time (criterion s192-criterio-anatomia.md).

Builds the tree with the pin recipe plus line tables only (text unchanged, dSYM produced) in ~/Claude/phpr-target/s192-armD,
runs one whole ORM leg (orm-work.tgz, phpunit) under `samply record` at 1 kHz (macOS `sample` collapses the leaf addresses of a
symbol into one truncated row, so it cannot serve), then attributes every sample whose deepest frame
is run_loop to the `Op::X` arm containing its source line (atos -i on the dSYM). Profile = direction, never a verdict.
Outputs: s192-anatomia-<tag>.out, rc in out/anatomia-<tag>.rc. Generato con assistenza AI (Claude Fable 5.2).
"""
import collections, os, pathlib, re, shutil, subprocess, sys, tarfile, time

P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
H = P / "wp192-harness"
O = H / "out"
BT = pathlib.Path.home() / "Claude/phpr-target"
TGT = BT / "s192-armD"
SP = BT / "s192-orm"
GATES = pathlib.Path("/Volumes/Extreme Pro/Claude/wp9-harness/gates")
FEED = pathlib.Path("/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log")
RUN_RS = P / "crates/php-runtime/src/vm/run.rs"
tag = sys.argv[1]
work = sys.argv[2] if len(sys.argv) > 2 else "orm"  # orm | wp (WordPress phpunit --group media, uploads under the guard)
WPDEV = pathlib.Path.home() / "Claude/wpdev"
GUARD = "/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
out = H / f"s192-anatomia-{tag}.out"
rcf = O / f"anatomia-{tag}.rc"
O.mkdir(parents=True, exist_ok=True)
rcf.unlink(missing_ok=True)
lines_out = []


def say(s=""):
    lines_out.append(s)
    out.write_text("\n".join(lines_out) + "\n")


def fin(rc):
    rcf.write_text(f"{rc}\n")
    sys.exit(rc)


def busy():
    return subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0


while busy() or not FEED.read_text().strip().splitlines()[-1].startswith("DONE"):
    time.sleep(30)
env = dict(os.environ, CARGO_TARGET_DIR=str(TGT), CARGO_PROFILE_RELEASE_DEBUG="line-tables-only",
           CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO="packed")
with (O / f"anatomia-{tag}-build.log").open("w") as lg:
    r = subprocess.run(["cargo", "build", "--release", "-p", "php-cli"], cwd=P, env=env, stdout=lg, stderr=lg)
if r.returncode != 0:
    fin(7)
BIN = TGT / "release/phpr"
DSYM = TGT / "release/phpr.dSYM"
if not DSYM.exists():
    say("dSYM assente")
    fin(7)

# one ORM leg under samply
prof = O / f"anatomia-{tag}.prof.json"
REUSE = os.environ.get("ANALYZE_ONLY") == "1" and prof.exists()  # re-read an existing profile, no new run
if not REUSE:
    prof.unlink(missing_ok=True)


def record(cwd, args):
    with (O / f"anatomia-{tag}.txt").open("w") as so, (O / f"anatomia-{tag}.err").open("w") as se:
        return subprocess.run(["samply", "record", "--save-only", "-o", str(prof), "--rate", "1000", "--", str(BIN)] + args,
                              cwd=cwd, stdout=so, stderr=se, env=dict(os.environ, MIMALLOC_PURGE_DELAY="0"))


t0 = time.time()
if REUSE:
    proc = subprocess.CompletedProcess([], -1)
elif work == "orm":
    shutil.rmtree(SP / "orm-work", ignore_errors=True)
    SP.mkdir(parents=True, exist_ok=True)
    with tarfile.open(GATES / "orm-work.tgz") as t:
        t.extractall(SP)
    proc = record(SP / "orm-work", ["vendor/bin/phpunit", "--no-coverage"])
else:
    # the WordPress leg of the pair harness (s191-pair.sh): uploads guard, fresh wptests, phpunit --group media
    up = WPDEV / "src/wp-content/uploads"
    with (O / f"anatomia-{tag}-guard.log").open("w") as gl:
        if subprocess.run([GUARD, "backup-wipe"], stdout=gl, stderr=gl).returncode != 0:
            say("uploads-guard backup-wipe FALLITO: niente gamba WP")
            fin(4)
        try:
            r = subprocess.run(["mysql", "-h", "127.0.0.1", "-u", "root", "-e",
                                "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';"],
                               stdout=gl, stderr=gl)
            if r.returncode != 0:
                say("reset di wptests FALLITO")
                fin(3)
            proc = record(WPDEV, ["vendor/bin/phpunit", "--group", "media"])
        finally:
            subprocess.run(["find", str(up), "-mindepth", "1", "-delete"], stdout=gl, stderr=gl)
            g = subprocess.run([GUARD, "restore"], stdout=gl, stderr=gl)
            say(f"uploads-guard restore rc={g.returncode}")
wall = time.time() - t0
txt = (O / f"anatomia-{tag}.txt").read_bytes().replace(b"\0", b"").decode("utf-8", "replace")
tests = [l for l in txt.splitlines() if l.startswith("Tests:") or l.startswith("OK")]
shutil.rmtree(SP / "orm-work", ignore_errors=True)
say(f"carico: {work}")
git = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=P, capture_output=True, text=True).stdout.strip()
say(f"== s192 anatomia tag={tag} git={git} wall={wall:.1f}s samply rc={proc.returncode} {time.strftime('%F %T')} (profilo: direzione) ==")
say(tests[-1] if tests else "phpunit: nessuna riga Tests")
if not prof.exists():
    say("samply non ha scritto il profilo")
    fin(7)

# gecko profile: leaf frame of every sample -> (lib, relative address)
import gzip, json
raw = prof.read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
leaf = collections.Counter()  # (lib name, rel address) -> samples
for th in d["threads"]:
    stt, ft, fu, rt = th["stackTable"], th["frameTable"], th["funcTable"], th["resourceTable"]
    for sidx in th["samples"]["stack"]:
        if sidx is None:
            continue
        fr = stt["frame"][sidx]
        res = fu["resource"][ft["func"][fr]]
        lib = libs[rt["lib"][res]] if res is not None and res >= 0 and rt["lib"][res] is not None else "?"
        leaf[(lib, ft["address"][fr])] += 1
total = sum(leaf.values())
mine = {a: n for (lib, a), n in leaf.items() if lib == "phpr" and a is not None and a >= 0}
DW = str(BIN)  # atos finds the packed dSYM next to the binary (its DWARF file carries a hash suffix)
addrs = sorted(mine)
af = O / f"anatomia-{tag}.addrs.txt"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
plain = subprocess.run(["atos", "-o", DW, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()
if len(plain) != len(addrs):
    say(f"atos: {len(plain)} righe per {len(addrs)} indirizzi")
    fin(7)
self_sym = collections.Counter()
for (lib, a), n in leaf.items():
    if lib != "phpr":
        self_sym[f"({lib})"] += n
loop_addr = {}
line_of = {}
for a, ln in zip(addrs, plain):
    sym = ln.split(" (in ")[0]
    if "run_loop" in sym:
        loop_addr[a] = mine[a]
        m = re.search(r"\(run\.rs:(\d+)\)", ln)
        if m:
            line_of[a] = int(m.group(1))
    else:
        self_sym[sym[:90]] += mine[a]
loop_total = sum(loop_addr.values())
say(f"campioni totali {total} · in phpr {sum(mine.values())} · run_loop proprio {loop_total} ({100 * loop_total / max(total, 1):.1f} %) · indirizzi distinti nel loop {len(loop_addr)}")
if not loop_total:
    say("niente da attribuire")
    fin(1)
# inlined code from other files: ask atos -i for the chain and keep the LAST frame that sits in the loop's own file
for a in loop_addr:
    if a in line_of:
        continue
    chain = subprocess.run(["atos", "-i", "-o", DW, hex(0x100000000 + a)], capture_output=True, text=True).stdout
    for fr in chain.splitlines():
        m = re.search(r"\(run\.rs:(\d+)\)", fr)
        if m:
            line_of[a] = int(m.group(1))
src = RUN_RS.read_text().splitlines()
arm_at = {}
cur = "testata(dispatch)"
start = next(i for i, l in enumerate(src) if "fn run_loop" in l)
end = next(i for i in range(start + 1, len(src)) if re.match(r"^    (pub(\(\w+\))? )?fn |^    #\[", src[i]) and i > start + 200)
for i in range(start, end):
    m = re.match(r"^ {16}(?:\| )?Op::(\w+)", src[i])
    if m:
        cur = "Op::" + m.group(1)
    arm_at[i + 1] = cur
by_arm = collections.Counter()
unattributed = 0
for a, n in loop_addr.items():
    if a in line_of:
        by_arm[arm_at.get(line_of[a], "run_loop(altro)")] += n
    else:
        unattributed += n
        by_arm["non attribuito"] += n
say(f"attribuiti a una riga {100 * (loop_total - unattributed) / loop_total:.1f} % dei campioni di run_loop")
say("")
say("BRACCI di run_loop per campioni PROPRI (quota del proprio di run_loop · cumulata · quota del totale):")
cum = 0
for arm, n in by_arm.most_common(25):
    cum += n
    say(f"  {arm:34s} {n:7d}  {100 * n / loop_total:5.1f} %  {100 * cum / loop_total:5.1f} %  {100 * n / total:5.1f} %")
say("")
say("FUORI dal loop, primi 15 simboli per campioni propri (quota del totale):")
for sym, n in self_sym.most_common(15):
    say(f"  {n:7d}  {100 * n / total:5.1f} %  {sym}")
ok = bool(tests) and (work != "orm" or tests[-1].startswith("Tests: 3484")) and (loop_total - unattributed) / loop_total >= 0.95
fin(0 if ok else 1)
