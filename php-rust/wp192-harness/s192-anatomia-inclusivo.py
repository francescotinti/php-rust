#!/usr/bin/env python3
"""s192-anatomia-inclusivo.py <tag> <substr>... — S-192: INCLUSIVE share of a profile recorded by s192-anatomia.py for every
function whose (mangled) symbol contains one of the given substrings: samples with that function anywhere on the stack.
Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, pathlib, subprocess, sys
O = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp192-harness/out")
BIN = str(pathlib.Path.home() / "Claude/phpr-target/s192-armD/release/phpr")
tag, keys = sys.argv[1], sys.argv[2:]
_keep = O / f"anatomia-{tag}.bin" / "phpr"
if _keep.exists():  # the profile's own binary (see s192-anatomia.py)
    BIN = str(_keep)
raw = (O / f"anatomia-{tag}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
chains = collections.Counter()
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
    sys.setrecursionlimit(100000)
    for s in th["samples"]["stack"]:
        if s is not None:
            chains[frozenset(chain(s))] += 1
total = sum(chains.values())
addrs = sorted({a for ch in chains for a in ch if a is not None})
af = O / f"anatomia-{tag}.incl.addrs"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
nm = dict(zip(addrs, subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()))
print(f"== inclusivo tag={tag} campioni {total} ==")
for k in keys:
    hit = {a for a in addrs if k in nm[a].split(" (in ")[0]}
    n = sum(c for ch, c in chains.items() if ch & hit)
    print(f"  {n:7d}  {100 * n / total:5.1f} %  *{k}*")
