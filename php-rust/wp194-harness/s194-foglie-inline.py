#!/usr/bin/env python3
"""s194-foglie-inline.py <tag> <substr> — S-194: SELF samples of the function whose mangled symbol contains <substr>, grouped by the
full inline chain (file:line, innermost first, max 5) of the leaf address. Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, pathlib, re, subprocess, sys
O = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp192-harness/out")
tag, key = sys.argv[1], sys.argv[2]
BIN = str(O / f"anatomia-{tag}.bin" / "phpr")
raw = (O / f"anatomia-{tag}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
leaf = collections.Counter()
total = 0
for th in d["threads"]:
    stt, ft, fu, rt = th["stackTable"], th["frameTable"], th["funcTable"], th["resourceTable"]
    for s in th["samples"]["stack"]:
        if s is None:
            continue
        total += 1
        fr = stt["frame"][s]
        res = fu["resource"][ft["func"][fr]]
        lib = libs[rt["lib"][res]] if res is not None and res >= 0 and rt["lib"][res] is not None else "?"
        if lib == "phpr":
            leaf[ft["address"][fr]] += 1
out = collections.Counter()
for a, n in leaf.most_common(4000):
    if n < 3:
        break
    inl = subprocess.run(["atos", "-i", "-o", BIN, hex(0x100000000 + a)], capture_output=True, text=True).stdout
    fl = inl.splitlines()
    if not fl or key not in fl[-1].split(" (in ")[0]:
        continue
    ls = re.findall(r"\(([\w.]+:\d+)\)", inl)
    out["<".join(ls[:5])] += n
print(f"== foglie inline *{key}* tag={tag}: {sum(out.values())} campioni propri ({100 * sum(out.values()) / total:.1f} % di {total}) ==")
for k, n in out.most_common(16):
    print(f"  {n:6d} {100 * n / total:4.1f}%  {k}")
