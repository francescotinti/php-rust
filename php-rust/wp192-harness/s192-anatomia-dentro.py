#!/usr/bin/env python3
"""s192-anatomia-dentro.py <tag> <substr> — S-192: what a function spends its time on, in a profile recorded by s192-anatomia.py.

For the function whose mangled symbol contains <substr> (outermost occurrence on each stack): (1) its SELF samples by source
line of the VM file (through inlined frames); (2) the INCLUSIVE samples of each direct callee symbol. Profile = direction.
Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, pathlib, re, subprocess, sys
O = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp192-harness/out")
tag, key = sys.argv[1], sys.argv[2]
BIN = str(O / f"anatomia-{tag}.bin" / "phpr")
raw = (O / f"anatomia-{tag}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
ext = {}
side = O / f"anatomia-{tag}.prof.syms.json"
if side.exists():
    sj = json.loads(side.read_text())
    strs = sj.get("string_table", [])
    for lib in sj.get("data", []):
        ext[lib.get("debug_name")] = sorted((e["rva"], e.get("size") or 0, strs[e["symbol"]]) for e in lib.get("symbol_table", []))
def ext_name(lib, a):
    for rva, size, nm in ext.get(lib, ()):
        if a is not None and rva <= a < rva + max(size, 1):
            return nm
    return lib
chains = collections.Counter()
total = 0
sys.setrecursionlimit(100000)
for th in d["threads"]:
    stt, ft, fu, rt = th["stackTable"], th["frameTable"], th["funcTable"], th["resourceTable"]
    memo = {}
    def chain(s):
        if s is None:
            return ()
        if s not in memo:
            fr = stt["frame"][s]
            res = fu["resource"][ft["func"][fr]]
            lib = libs[rt["lib"][res]] if res is not None and res >= 0 and rt["lib"][res] is not None else "?"
            memo[s] = ((lib, ft["address"][fr]),) + chain(stt["prefix"][s])
        return memo[s]
    for s in th["samples"]["stack"]:
        if s is not None:
            total += 1
            chains[chain(s)] += 1
addrs = sorted({a for ch in chains for (lib, a) in ch if lib == "phpr"})
af = O / f"anatomia-{tag}.dentro.addrs"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
nm = dict(zip(addrs, subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()))
def sym(fr):
    lib, a = fr
    return nm[a].split(" (in ")[0] if lib == "phpr" else f"({ext_name(lib, a)})"
def short(s):
    ids = re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]{2,})", s)
    return "::".join(ids[-2:]) if ids else s[-40:]
cache = {}
def line(a):
    if a not in cache:
        inl = subprocess.run(["atos", "-i", "-o", BIN, hex(0x100000000 + a)], capture_output=True, text=True).stdout
        ls = re.findall(r"\(([\w.]+:\d+)\)", inl)
        cache[a] = ls[-1] if ls else "?"
    return cache[a]
self_line, callee, incl = collections.Counter(), collections.Counter(), 0
for ch, n in chains.items():
    idx = [i for i, fr in enumerate(ch) if fr[0] == "phpr" and key in sym(fr)]
    if not idx:
        continue
    incl += n
    i = idx[0]  # innermost occurrence: what THIS activation is doing
    if i == 0:
        self_line[line(ch[0][1])] += n
    else:
        callee[short(sym(ch[i - 1])) if ch[i - 1][0] == "phpr" else sym(ch[i - 1])] += n
print(f"== dentro *{key}* tag={tag}: inclusivo {incl} campioni ({100 * incl / total:.1f} % di {total}); proprio {sum(self_line.values())} ==")
print("  PROPRIO per riga:")
for k, n in self_line.most_common(14):
    print(f"    {n:6d} {100 * n / total:4.1f}%  {k}")
print("  CHIAMATI diretti (inclusivo):")
for k, n in callee.most_common(16):
    print(f"    {n:6d} {100 * n / total:4.1f}%  {k}")
