#!/usr/bin/env python3
"""s194-profilo-orm.py — S-194: samply profile of ONE whole ORM leg (phpunit on a fresh untar of orm-work.tgz). DECLARED COPY of
s194-profilo-fixture.py (manifest s194-profilo-orm-copia.diff): only the profiled command, its working directory, the output names
and the length of the two histograms change. Original header: samply profile of one primitive fixture on the
profile binary kept by the s194w anatomy (same source as the tree, line tables + dSYM), leaf histogram by symbol and, for the
interpreter loop, by source line (with the line's text). Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, os, pathlib, re, subprocess, sys
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
O = P / "wp192-harness/out"
H = P / "wp194-harness"
BIN = os.environ.get("S194_BIN", str(O / "anatomia-s194w.bin" / "phpr"))  # S194_BIN / S194_SUF: another profile binary and an output suffix
fx, n = "orm", "leg"
import shutil, tarfile
SP = pathlib.Path.home() / "Claude/phpr-target/s192-orm"
shutil.rmtree(SP / "orm-work", ignore_errors=True); SP.mkdir(parents=True, exist_ok=True)
with tarfile.open("/Volumes/Extreme Pro/Claude/wp9-harness/gates/orm-work.tgz") as t: t.extractall(SP)
prof = H / "primitive" / f"{fx}.prof.json"
prof.unlink(missing_ok=True)
with open(os.devnull, "w") as dn:
    rc = subprocess.run(["samply", "record", "--save-only", "--unstable-presymbolicate", "-o", str(prof), "--rate", "4000", "--",
                         BIN, "vendor/bin/phpunit", "--no-coverage"], stdout=dn, stderr=dn, cwd=SP / "orm-work").returncode
raw = prof.read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
leaf = collections.Counter()
for th in d["threads"]:
    stt, ft, fu, rt = th["stackTable"], th["frameTable"], th["funcTable"], th["resourceTable"]
    for s in th["samples"]["stack"]:
        if s is None:
            continue
        fr = stt["frame"][s]
        res = fu["resource"][ft["func"][fr]]
        lib = libs[rt["lib"][res]] if res is not None and res >= 0 and rt["lib"][res] is not None else "?"
        leaf[(lib, ft["address"][fr])] += 1
total = sum(leaf.values())
addrs = sorted(a for (lib, a) in leaf if lib == "phpr" and a is not None and a >= 0)
af = H / "primitive" / f"{fx}.addrs"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
nm = dict(zip(addrs, subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()))
syms, lines = collections.Counter(), collections.Counter()
for (lib, a), k in leaf.items():
    if lib == "phpr" and a in nm:
        s = nm[a].split(" (in ")[0]
        ids = re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]{2,})", s)
        sym = ids[-1] if ids else s[-40:]
        if sym.startswith("h") and len(ids) > 1 and len(sym) == 17:
            sym = ids[-2]
        syms[sym] += k
        ln = re.search(r"\(([\w.]+):(\d+)\)", nm[a])
        if ln:
            lines[(sym, ln.group(1), int(ln.group(2)))] += k
    else:
        syms[f"({lib})"] += k
src = {}
def text(fname, no):
    if fname not in src:
        hits = list((P / "crates").rglob(fname))
        src[fname] = hits[0].read_text(errors="replace").splitlines() if len(hits) == 1 else None
    t = src[fname]
    return t[no - 1].strip()[:100] if t and no <= len(t) else ""
out = [f"== profilo fixture {fx} N={n}: {total} campioni, samply rc={rc} (direzione) =="]
for k, c in syms.most_common(70):
    out.append(f"  {c:6d} {100 * c / total:5.1f}%  {k}")
out.append("  righe (simbolo file:riga):")
for (sym, f, no), c in lines.most_common(110):
    out.append(f"  {c:6d} {100 * c / total:5.1f}%  {sym} {f}:{no}  | {text(f, no)}")
(H / f"s194-profilo-{fx}{os.environ.get('S194_SUF', '')}.out").write_text("\n".join(out) + "\n")
