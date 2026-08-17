#!/bin/bash
# smoke151-check.sh — verificatore MECCANICO delle guardie dichiarate in
# census-prep/smoke-atteso.md (scritte PRIMA di ogni lettura di output).
# Uso: CENSUS_PHPR=<probe> smoke151-check.sh <outdir>
# Esito: una riga `GUARDIA <nome>: PASS/FAIL <dettaglio>` per guardia su
# stdout; rc=0 solo se TUTTE PASS. MAI cifre di tempo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
CENSUS_PHPR="${CENSUS_PHPR:?binario probe richiesto}"
OUT="${1:?outdir richiesto}"
mkdir -p "$OUT"

rm -f "$OUT/smoke151-mem.txt" "$OUT/smoke149-mem.txt"
PHPR_MEM_CENSUS="$OUT/smoke151-mem.txt" \
  "$CENSUS_PHPR" "$REPO/wp151-harness/census-prep/smoke151.php" \
  > "$OUT/smoke151.out" 2> "$OUT/smoke151.err"
RC_A=$?
PHPR_MEM_CENSUS="$OUT/smoke149-mem.txt" \
  "$CENSUS_PHPR" "$REPO/wp149-harness/smoke149.php" \
  > "$OUT/smoke149.out" 2> "$OUT/smoke149.err"
RC_B=$?

python3 - "$OUT" "$RC_A" "$RC_B" <<'PY'
import sys, re, collections
out, rc_a, rc_b = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
fails = []
def guard(name, ok, detail=""):
    print(f"GUARDIA {name}: {'PASS' if ok else 'FAIL'} {detail}".rstrip())
    if not ok:
        fails.append(name)

mem = open(f"{out}/smoke151-mem.txt", "rb").read().replace(b"\0", b"").decode("utf-8", "replace")
stdout = open(f"{out}/smoke151.out", "rb").read()

# 1. rc + stdout esatto
guard("A1-rc-stdout", rc_a == 0 and stdout == b"SMOKE151 r=42\n",
      f"rc={rc_a} stdout={stdout!r}")

sites = collections.defaultdict(list)   # channel -> [(site, n)]
tots = {}
kv = {}
cons = {}
n6 = {}
props = {}
phist, dhist = {}, {}
for line in mem.splitlines():
    t = line.split()
    if not t: continue
    d = dict(p.split("=", 1) for p in t[1:] if "=" in p)
    if t[0] == "s151site":
        sites[d["channel"]].append((d["site"], int(d["n"])))
    elif t[0] == "s151tot":
        tots[d["channel"]] = int(d["n"])
    elif t[0] == "s151overlap":
        kv["overlap"] = int(d["n"])
    elif t[0] == "s151clsovf":
        kv["clsovf"] = int(d["n"])
    elif t[0] == "s151snap":
        kv["snap"] = int(d["taken"])
    elif t[0] == "s151cons":
        cons[d["class"]] = {k: int(v) for k, v in d.items() if k != "class"}
    elif t[0] == "s151n6":
        n6[d["class"]] = {k: int(v) for k, v in d.items() if k != "class"}
    elif t[0] == "s151props":
        props = d
    elif t[0] == "s151propshist":
        phist[int(d["bucket"])] = int(d["n"])
    elif t[0] == "s151dynhist":
        dhist[int(d["bucket"])] = int(d["n"])

# 2. identita' per canale
for ch in ["c1", "c2", "c3", "c4", "c5"]:
    s = sum(n for _, n in sites.get(ch, []))
    t = tots.get(ch, None)
    guard(f"A2-identita-{ch}", t is not None and s == t, f"sum_siti={s} tot={t}")

# 3. overlap / clsovf / snap
guard("A3-overlap", kv.get("overlap") == 0, f"overlap={kv.get('overlap')}")
guard("A3-clsovf", kv.get("clsovf") == 0, f"clsovf={kv.get('clsovf')}")
guard("A3-snap", kv.get("snap") == 1, f"snap={kv.get('snap')}")

# 4. conservazione classe K
K = cons.get("K", {})
b, c, dd, l = (K.get(k) for k in ("births", "clones", "drops", "live_end"))
guard("A4-K-births", b == 9, f"births={b}")
guard("A4-K-conservazione", None not in (b, c, dd, l) and b + c == dd + l,
      f"b={b} c={c} d={dd} l={l} (b+c={0 if b is None else b+(c or 0)} d+l={0 if dd is None else dd+(l or 0)})")
guard("A4-K-live_end", l == 3, f"live_end={l}")

# 5. N6
k6 = n6.get("K", {})
guard("A5-K-objects", k6.get("objects") == 1, f"objects={k6.get('objects')}")
guard("A5-K-deaths", k6.get("deaths") == 9, f"deaths={k6.get('deaths')}")

# 6. distribuzione props
ok6 = (props.get("n") == "9" and props.get("sum") == "27" and props.get("p50") == "3"
       and props.get("p90") == "3" and props.get("p99") == "3"
       and props.get("le4_pct") == "100.00" and props.get("le8_pct") == "100.00"
       and props.get("dyn_objs") == "0" and props.get("dyn_entries") == "0"
       and phist == {3: 9} and dhist == {0: 9})
guard("A6-props-N2", ok6, f"props={props} phist={phist} dhist={dhist}")

# 7. C3 alloc_instance
c3 = {s: n for s, n in sites.get("c3", [])}
guard("A7-C3-alloc_instance", c3.get("alloc_instance", 0) >= 18,
      f"alloc_instance={c3.get('alloc_instance')} (bounded >=18)")

# 8. C5 scalar
sdrop = sum(n for s, n in sites.get("c5", []) if s.endswith(".scalar.drop"))
sclone = sum(n for s, n in sites.get("c5", []) if s.endswith(".scalar.clone"))
guard("A8-C5-scalar-drop", sdrop == 1, f"scalar.drop={sdrop}")
guard("A8-C5-scalar-clone", sclone >= 1, f"scalar.clone={sclone} (bounded >=1)")

# 9. mint == 0
mint = sum(n for s, n in sites.get("c1", []) if s.endswith(".mint"))
guard("A9-C1-mint", mint == 0, f"mint={mint}")

# 10./11. C2, C4 presenza
guard("A10-C2-tot", tots.get("c2", 0) >= 1, f"c2={tots.get('c2')} (bounded >=1)")
guard("A11-C4-tot", tots.get("c4", 0) >= 1, f"c4={tots.get('c4')} (bounded >=1)")

# 12. eredita' s148/s149 nel run A
def s148_ok(text):
    ga = re.search(r"^s148sum pid=\d+ galloc_n=(\d+) sum_n=(\d+)$", text, re.M)
    return ga and ga.group(1) == ga.group(2)
def s149_vals(text):
    m = re.search(r"^s149sum pid=\d+ hostcall_n=(\d+) sum_name_n=(\d+) unnamed_n=(\d+) overflow=(\d+)$", text, re.M)
    return [int(x) for x in m.groups()] if m else None
v = s149_vals(mem)
guard("A12-eredita-run-A", bool(s148_ok(mem)) and v is not None
      and v[0] == v[1] + v[2] and v[2] == 0 and v[3] == 0, f"s149sum={v}")

# 13./14. run B (smoke149 ereditato)
memb = open(f"{out}/smoke149-mem.txt", "rb").read().replace(b"\0", b"").decode("utf-8", "replace")
tags = dict(re.findall(r"^s148tag pid=\d+ tag=\d+ name=(\w+) n=(\d+)", memb, re.M))
okb = rc_b == 0 and all(int(tags.get(t, 0)) >= 1 for t in ("frame", "hostcall", "arrgrow"))
guard("B13-s148tag", okb and bool(s148_ok(memb)), f"rc={rc_b} tags={tags}")
vb = s149_vals(memb)
names = dict(re.findall(r"^s149name pid=\d+ name=(\w+) n=(\d+)", memb, re.M))
guard("B14-s149", vb is not None and vb[0] == vb[1] + vb[2] and vb[2] == 0 and vb[3] == 0
      and int(names.get("str_repeat", 0)) >= 1 and int(names.get("sprintf", 0)) >= 1,
      f"s149sum={vb} str_repeat={names.get('str_repeat')} sprintf={names.get('sprintf')}")

print(f"SMOKE151 VERDETTO: {'PASS' if not fails else 'FAIL'} ({len(fails)} guardie fallite: {fails})")
sys.exit(0 if not fails else 1)
PY
exit $?
