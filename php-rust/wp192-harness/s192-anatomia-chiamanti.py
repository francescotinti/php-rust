#!/usr/bin/env python3
"""s192-anatomia-chiamanti.py <tag> — S-192 bis: second reading of a profile recorded by s192-anatomia.py.

(1) self samples grouped by coarse family (demangled symbol prefix / library); (2) for the leaf families that are
callee-side helpers (libsystem_platform = memmove/memcpy/bzero, SipHash, hashbrown, Zval clone/drop, malloc/free),
the nearest phpr CALLER that is not itself in the family — who pays. Profile = direction. Output:
s192-anatomia-chiamanti-<tag>.out. Generato con assistenza AI (Claude Fable 5.2).
"""
import collections, gzip, json, pathlib, re, subprocess, sys

P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
O = P / "wp192-harness/out"
BIN = str(pathlib.Path.home() / "Claude/phpr-target/s192-armD/release/phpr")
tag = sys.argv[1]
raw = (O / f"anatomia-{tag}.prof.json").read_bytes()
d = json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
libs = [l.get("name") for l in d["libs"]]

stacks = collections.Counter()  # tuple of (lib, addr) leaf-first, depth-limited
for th in d["threads"]:
    stt, ft, fu, rt = th["stackTable"], th["frameTable"], th["funcTable"], th["resourceTable"]

    def frame(fr):
        res = fu["resource"][ft["func"][fr]]
        lib = libs[rt["lib"][res]] if res is not None and res >= 0 and rt["lib"][res] is not None else "?"
        return (lib, ft["address"][fr])

    for sidx in th["samples"]["stack"]:
        if sidx is None:
            continue
        chain, s = [], sidx
        while s is not None and len(chain) < 12:
            chain.append(frame(stt["frame"][s]))
            s = stt["prefix"][s]
        stacks[tuple(chain)] += 1
total = sum(stacks.values())
addrs = sorted({a for ch in stacks for (lib, a) in ch if lib == "phpr" and a is not None and a >= 0})
af = O / f"anatomia-{tag}.calleraddrs.txt"
af.write_text("\n".join(hex(0x100000000 + a) for a in addrs) + "\n")
names = subprocess.run(["atos", "-o", BIN, "-f", str(af)], capture_output=True, text=True).stdout.splitlines()
assert len(names) == len(addrs), (len(names), len(addrs))


def lite(m):
    """v0 mangling, readable enough: the length-prefixed identifiers joined by `::` (no rustfilt on this machine)."""
    out, i = [], 0
    while i < len(m):
        if m[i].isdigit():
            j = i
            while j < len(m) and m[j].isdigit():
                j += 1
            n = int(m[i:j])
            if m[j:j + 1] == "_":
                j += 1
            ident = m[j:j + n]
            if 0 < n < 60 and len(ident) == n and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", ident):
                out.append(ident)
                i = j + n
                continue
        i += 1
    return "::".join(out) or m


pretty = [lite(n.split(" (in ")[0]) for n in names]
sym = dict(zip(addrs, pretty))


def name(fr):
    lib, a = fr
    return sym.get(a, "?") if lib == "phpr" else f"({lib})"


def family(n):
    if n.startswith("("):
        return n
    for key, fam in (("run_loop", "vm: run_loop"), ("sip", "hash: SipHash"), ("hashbrown", "hash: hashbrown"),
                     ("drop_glue", "zval/drop glue"), ("Zval::core::clone", "zval clone"), ("mi_", "alloc: mimalloc"),
                     ("gc_", "vm: gc"), ("collect_cycles", "vm: gc"), ("php_runtime::vm", "vm: altro"),
                     ("php_runtime", "runtime: altro"), ("php_types", "php_types"), ("php_builtins", "php_builtins"),
                     ("regex", "regex"), ("alloc::", "std alloc/vec"), ("core::", "std core")):
        if key in n:
            return fam
    return "altro"


fam = collections.Counter()
for ch, n in stacks.items():
    fam[family(name(ch[0]))] += n
lines = [f"== s192 anatomia/chiamanti tag={tag} campioni {total} (profilo: direzione) ==", "", "FAMIGLIE per campioni propri:"]
for f, n in fam.most_common(18):
    lines.append(f"  {n:7d}  {100 * n / total:5.1f} %  {f}")
for title, pred in (("(libsystem_platform.dylib) — memmove/memcpy/bzero", lambda n: n == "(libsystem_platform.dylib)"),
                    ("SipHash + hashbrown", lambda n: family(n).startswith("hash:")),
                    ("Zval clone + drop glue", lambda n: family(n) in ("zval clone", "zval/drop glue")),
                    ("(libsystem_kernel.dylib)", lambda n: n == "(libsystem_kernel.dylib)"),
                    ("mimalloc", lambda n: family(n) == "alloc: mimalloc")):
    callers = collections.Counter()
    tot = 0
    for ch, n in stacks.items():
        if not pred(name(ch[0])):
            continue
        tot += n
        who = next((name(fr) for fr in ch[1:] if not pred(name(fr)) and not name(fr).startswith("(")
                    and family(name(fr)) not in ("std core", "std alloc/vec", "hash: hashbrown", "hash: SipHash")), "?")
        callers[who[-95:]] += n
    lines += ["", f"CHI PAGA {title}: {tot} campioni ({100 * tot / total:.1f} %) — primo chiamante phpr:"]
    for c, n in callers.most_common(12):
        lines.append(f"  {n:7d}  {100 * n / total:5.1f} %  {c}")
(P / f"wp192-harness/s192-anatomia-chiamanti-{tag}.out").write_text("\n".join(lines) + "\n")
