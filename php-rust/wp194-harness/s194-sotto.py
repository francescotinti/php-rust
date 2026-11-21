#!/usr/bin/env python3
"""s194-sotto.py <tag> <substr> — S-194: for the samples of a profile recorded by wp192-harness/s192-anatomia.py that have a
function whose mangled symbol contains <substr> anywhere on the stack, the SELF histogram of the leaf symbol (phpr symbols
with the innermost VM source line; system libraries by library). Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import collections, gzip, json, pathlib, re, subprocess, sys
O = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp192-harness/out")
tag, key = sys.argv[1], sys.argv[2]
BIN = str(O / f"anatomia-{tag}.bin" / "phpr")
raw = (O / f"anatomia-{tag}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]
sys.setrecursionlimit(100000)
stacks = collections.Counter()
total = 0
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
            stacks[chain(s)] += 1
addrs = sorted({a for ch in stacks for (lib, a) in ch if lib == "phpr" and a is not None and a >= 0})
af = O / f"anatomia-{tag}.sotto.addrs"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
nm = dict(zip(addrs, subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()))
def tail(a):
    s = nm[a].split(" (in ")[0]
    ids = re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]{2,})", s)
    ln = re.search(r"\(([\w.]+:\d+)\)", nm[a])
    return (ids[-1] if ids else s[-30:]), (ln.group(1) if ln else "?")
out, lines, n_all = collections.Counter(), collections.Counter(), 0
for ch, n in stacks.items():
    if not any(lib == "phpr" and a in nm and key in nm[a].split(" (in ")[0] for lib, a in ch):
        continue
    n_all += n
    lib, a = ch[0]
    if lib == "phpr" and a in nm:
        sym, ln = tail(a)
        out[sym] += n
        if sym == "run_loop":
            lines[ln] += n
    else:
        out[f"({lib})"] += n
print(f"== sotto *{key}* tag={tag}: {n_all} campioni ({100 * n_all / total:.1f} % di {total}) — foglie per campioni propri ==")
for k, n in out.most_common(28):
    print(f"  {n:6d} {100 * n / total:4.1f}%  {k}")
print("  run_loop per riga:")
for k, n in lines.most_common(12):
    print(f"  {n:6d} {100 * n / total:4.1f}%  {k}")
