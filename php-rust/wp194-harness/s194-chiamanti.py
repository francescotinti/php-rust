#!/usr/bin/env python3
"""s194-chiamanti.py <fixture> <pattern>... — S-194 call-cost investigation: for the leaf samples whose symbol contains a pattern,
histogram of the nearest caller frames inside the phpr image (symbol file:line), from the samply profile already recorded by
s194-profilo-fixture.py. Read-only on the profile. Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, os, pathlib, re, subprocess, sys
H = pathlib.Path(__file__).resolve().parent
BIN = os.environ.get("S194_BIN", os.path.expanduser("~/Claude/phpr-target/s192-armD/release/phpr"))
fx, pats = sys.argv[1], sys.argv[2:]
raw = (H / "primitive" / f"{fx}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
chains = collections.Counter()
for th in d["threads"]:
    stt, ft, fu, rt = th["stackTable"], th["frameTable"], th["funcTable"], th["resourceTable"]
    def lib_of(fr):
        res = fu["resource"][ft["func"][fr]]
        return libs[rt["lib"][res]] if res is not None and res >= 0 and rt["lib"][res] is not None else "?"
    for s in th["samples"]["stack"]:
        if s is None:
            continue
        ch = []
        while s is not None and len(ch) < 4:
            fr = stt["frame"][s]
            ch.append((lib_of(fr), ft["address"][fr]))
            s = stt["prefix"][s]
        chains[tuple(ch)] += 1
addrs = sorted({a for ch in chains for (lib, a) in ch if lib == "phpr" and a is not None and a >= 0})
af = H / "primitive" / f"{fx}.addrs"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
nm = dict(zip(addrs, subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()))
def name(lib, a):
    if lib != "phpr" or a not in nm:
        return f"({lib})"
    s = nm[a].split(" (in ")[0]
    ids = re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]{2,})", s)
    sym = ids[-1] if ids else s[-40:]
    if sym.startswith("h") and len(ids) > 1 and len(sym) == 17:
        sym = ids[-2]
    if "drop_glue" in s:
        sym = "drop_glue<" + (ids[-2] if len(ids) > 1 else "?") + ">"
    ln = re.search(r"\(([\w.]+):(\d+)\)", nm[a])
    return sym + (f" {ln.group(1)}:{ln.group(2)}" if ln else "")
total = sum(chains.values())
out = [f"== chiamanti fixture {fx}: {total} campioni, binario {BIN} (direzione) =="]
for p in pats:
    sub = collections.Counter()
    for ch, k in chains.items():
        names = [name(*x) for x in ch]
        if p in names[0]:
            sub[" <- ".join(names[:3])] += k
    n = sum(sub.values())
    out.append(f"-- foglia ~ '{p}': {n} campioni ({100 * n / total:.1f} %)")
    for c, k in sub.most_common(int(os.environ.get('S194_ROWS', '8'))):
        out.append(f"  {k:6d} {100 * k / total:5.1f}%  {c}")
(H / f"s194-chiamanti-{fx}.out").write_text("\n".join(out) + "\n")
