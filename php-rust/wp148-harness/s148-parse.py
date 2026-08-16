#!/usr/bin/env python3
# s148-parse.py — parser del census ATTRIBUZIONE per TAG (criterio
# s148-criterio-attrib.md p.3-4-7). Collaudato da s148-parse-golden-test.sh
# PRIMA del run. Uso: s148-parse.py <dir con census-mem-r*.txt>
# SOLO conteggi: mai cifre di tempo come record (p.7: soglia-conteggio INDIZIO).
import glob
import re
import sys

out = sys.argv[1]
MKEYS = [
    "str.cum_n", "arr.cum_n", "obj.cum_n",
    "s143.arrbuf_n", "s143.propsbuf_n", "s143.galloc_n", "s143.gfree_n",
    "s144.rczval_n", "s144.rczval_prop_n", "s144.vecargs_n",
    "s144.grealloc_n", "s144.objsynth_n",
]
# riferimento s144 (wp144-harness/s144-census-tranche2.out r1) per il
# confronto p.4 «workload identico» (scarti >1% DICHIARATI, non gate):
S144_REF = {
    "str.cum_n": 129933714, "arr.cum_n": 28431596, "obj.cum_n": 3265745,
    "s143.arrbuf_n": 15818775, "s143.propsbuf_n": 3219798,
    "s143.galloc_n": 471263413, "s144.rczval_n": 4738276,
    "s144.vecargs_n": 13141064,
}
TAGS = ["none", "frame", "hostcall", "gc", "arrgrow"]
BUCKETS = ["<=16", "<=32", "<=48", "<=64", "<=96", "<=128", "<=256",
           "<=512", "<=1024", "<=4096", ">4096"]
SOGLIA_INDIZIO = 24_800_000  # p.7: 0,293 s / 11,8 ns (prezzo pair ALTO, INDIZIO mai budget)

per_rep, tag_rep, sum_rep = {}, {}, {}
for f in sorted(glob.glob(f"{out}/census-mem-r*.txt")):
    rep = re.search(r"census-mem-(r\d)", f).group(1)
    tot = dict.fromkeys(MKEYS, 0)
    tags = {t: {"n": 0, "attr": 0, "b": 0, "hist": [0] * 11} for t in TAGS}
    ident = {"galloc_n": 0, "sum_n": 0}
    for line in open(f, errors="replace"):
        if line.startswith("s148tag "):
            kv = dict(re.findall(r"(\w+)=(\S+)", line))
            t = kv["name"]
            if t not in tags:
                continue
            tags[t]["n"] += int(kv["n"])
            tags[t]["attr"] += int(kv["attr"])
            tags[t]["b"] += int(kv["b"])
            for i, h in enumerate(kv["hist"].split(",")):
                tags[t]["hist"][i] += int(h)
            continue
        if line.startswith("s148sum "):
            kv = dict(re.findall(r"(\w+)=(\d+)", line))
            ident["galloc_n"] += int(kv["galloc_n"])
            ident["sum_n"] += int(kv["sum_n"])
            continue
        m = re.search(r"\btag=(\S+)", line)
        if not m or m.group(1) != "exit":
            continue
        kv = dict(re.findall(r"([\w.]+)=(-?\d+)", line))
        for k in MKEYS:
            if k in kv:
                tot[k] += int(kv[k])
    per_rep[rep] = tot
    tag_rep[rep] = tags
    sum_rep[rep] = ident

for rep, tot in sorted(per_rep.items()):
    print(f"{rep}: " + " ".join(f"{k}={tot[k]}" for k in MKEYS))
for rep, ident in sorted(sum_rep.items()):
    ok = "OK" if ident["galloc_n"] == ident["sum_n"] else "VIOLATA"
    print(f"{rep}_identita: galloc_n={ident['galloc_n']} sum_tag_n={ident['sum_n']} {ok}")

# p.4: confronto col riferimento s144 (workload identico; dichiara, non gate)
reps = [per_rep[r] for r in sorted(per_rep)]
if reps:
    for k, ref in S144_REF.items():
        a = reps[0][k]
        if ref and abs(a - ref) / ref > 0.01:
            print(f"DICHIARA scarto>1% vs s144 su {k}: ora={a} s144={ref}")

# p.4: replica r1==r2 <=1% per chiave aggregata
tag_list = [tag_rep[r] for r in sorted(tag_rep)]
if len(tag_list) == 2:
    worst = (0.0, "-")
    for t in TAGS:
        for k in ("n", "attr", "b"):
            a, b = tag_list[0][t][k], tag_list[1][t][k]
            m = max(a, b)
            d = abs(a - b) / m if m else 0.0
            if d > worst[0]:
                worst = (d, f"{t}.{k}")
            print(f"replica {t}.{k}: r1={a} r2={b} delta={100*d:.3f}%")
    print(f"replica_worst: {worst[1]} {100*worst[0]:.3f}%")
    if worst[0] > 0.01:
        print("DICHIARA r1!=r2 >1%: replica dovuta (criterio p.9)")

# tabella per tag (media r1/r2) + ranking other
if tag_list:
    n_reps = len(tag_list)
    print("--- partizione galloc_n per TAG (media repliche; other = n - attr) ---")
    galloc = max(1, sum(s["galloc_n"] for s in sum_rep.values()) // max(1, len(sum_rep)))
    rows = []
    for t in TAGS:
        n = sum(x[t]["n"] for x in tag_list) // n_reps
        attr = sum(x[t]["attr"] for x in tag_list) // n_reps
        b = sum(x[t]["b"] for x in tag_list) // n_reps
        other = n - attr
        rows.append((t, n, attr, other, b))
        print(f"tag {t:<9} n={n:>12} ({100*n/galloc:5.2f}%) attr={attr:>12} "
              f"other={other:>12} ({100*other/galloc:5.2f}%) bytes={b}")
    print("--- ranking OTHER per tag (bersagli tranche-3; soglia INDIZIO p.7 = "
          f"{SOGLIA_INDIZIO} eventi) ---")
    for t, n, attr, other, b in sorted(rows, key=lambda r: -r[3]):
        verdict = ("CANDIDATA sonda-prezzo" if other >= SOGLIA_INDIZIO
                   else "NON-bersaglio-solo (sotto soglia INDIZIO)")
        print(f"other {t:<9} {other:>12} ({100*other/galloc:5.2f}% di galloc_n)  {verdict}")
    print("--- istogramma taglie per tag (media repliche) ---")
    for t in TAGS:
        h = [sum(x[t]["hist"][i] for x in tag_list) // n_reps for i in range(11)]
        if sum(h) == 0:
            continue
        top = sorted(range(11), key=lambda i: -h[i])[:3]
        tops = " ".join(f"{BUCKETS[i]}:{h[i]}" for i in top if h[i])
        print(f"hist {t:<9} " + ",".join(str(v) for v in h) + f"  top: {tops}")
