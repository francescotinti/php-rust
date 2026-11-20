#!/usr/bin/env python3
"""s194-chi-chiama-riga.py <tag> <substr> [depth] — S-194, DECLARED COPY of wp192-harness/s192-anatomia-chi-chiama.py; one change:
the call site prints the WHOLE inline chain of file:line (innermost first, max 4), because the outermost line alone hides the
VM arm when run_loop is inlined into its driver. Original docstring: S-192: for the samples of a recorded profile that have a function whose
mangled symbol contains <substr> on the stack, the chain of the next [depth] phpr callers above it (symbol tail + the VM
source line of the call, through inlined frames). Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, pathlib, re, subprocess, sys
O = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp192-harness/out")
tag, key = sys.argv[1], sys.argv[2]
depth = int(sys.argv[3]) if len(sys.argv) > 3 else 2
BIN = str(O / f"anatomia-{tag}.bin" / "phpr")
raw = (O / f"anatomia-{tag}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
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
            memo[s] = ((ft["address"][fr],) if lib == "phpr" else ()) + chain(stt["prefix"][s])
        return memo[s]
    for s in th["samples"]["stack"]:
        if s is not None:
            total += 1
            chains[chain(s)] += 1
addrs = sorted({a for ch in chains for a in ch})
af = O / f"anatomia-{tag}.cc.addrs"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
nm = dict(zip(addrs, subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()))
def tail(a):
    s = nm[a].split(" (in ")[0]
    ids = re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]{2,})", s)
    return ids[-1] if ids else s[-30:]
cache = {}
def site(a):
    if a not in cache:
        inl = subprocess.run(["atos", "-i", "-o", BIN, hex(0x100000000 + a)], capture_output=True, text=True).stdout
        ls = re.findall(r"\(([\w.]+:\d+)\)", inl)
        cache[a] = f"{tail(a)}@{'<'.join(ls[:4]) if ls else '?'}"
    return cache[a]
out = collections.Counter()
n_all = 0
for ch, n in chains.items():
    idx = [i for i, a in enumerate(ch) if key in nm[a].split(" (in ")[0]]
    if not idx:
        continue
    n_all += n
    up = [a for a in ch[idx[-1] + 1:] if key not in nm[a]][:depth]
    out[" <- ".join(site(a) for a in up)] += n
print(f"== chi chiama *{key}* tag={tag}: {n_all} campioni ({100 * n_all / total:.1f} % di {total}) ==")
for k, n in out.most_common(22):
    print(f"  {n:6d} {100 * n / total:4.1f}%  {k}")
