#!/usr/bin/env python3
# s144-census-parse.py — parser v3 del census tranche-2 (criterio
# s144-criterio-tranche2.md p.4-5). Collaudato da s144-parser-golden-test.sh
# (fixture exit+exit_mi: il tag si confronta ESATTO, lezione S-143).
# Uso: s144-census-parse.py <dir con census-mem-r*.txt e census-zval-r*.txt>
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
per_rep, zval_rep, zsize = {}, {}, None
for f in sorted(glob.glob(f"{out}/census-mem-r*.txt")):
    rep = re.search(r"census-mem-(r\d)", f).group(1)
    tot = dict.fromkeys(MKEYS, 0)
    for line in open(f, errors="replace"):
        m = re.search(r"\btag=(\S+)", line)
        if not m or m.group(1) != "exit":
            continue
        kv = dict(re.findall(r"([\w.]+)=(-?\d+)", line))
        for k in MKEYS:
            if k in kv:
                tot[k] += int(kv[k])
        if "s143.zval_size" in kv:
            zsize = int(kv["s143.zval_size"])
    per_rep[rep] = tot
for f in sorted(glob.glob(f"{out}/census-zval-r*.txt")):
    rep = re.search(r"census-zval-(r\d)", f).group(1)
    tot = {"galloc_bytes": 0, "gfree_bytes": 0}
    for line in open(f, errors="replace"):
        if not line.startswith("alloccensus "):
            continue
        kv = dict(re.findall(r"(\w+)=(\d+)", line))
        for k in tot:
            if k in kv:
                tot[k] += int(kv[k])
    zval_rep[rep] = tot
for rep, tot in sorted(per_rep.items()):
    print(f"{rep}: " + " ".join(f"{k}={tot[k]}" for k in MKEYS))
for rep, tot in sorted(zval_rep.items()):
    print(f"{rep} bytes: galloc={tot['galloc_bytes']} gfree={tot['gfree_bytes']}")
print(f"zval_size={zsize}")
reps = [per_rep[r] for r in sorted(per_rep)]
if len(reps) == 2:
    for k in MKEYS:
        a, b = reps[0][k], reps[1][k]
        m = max(a, b)
        if m and abs(a - b) / m > 0.01:
            print(f"DICHIARA scarto>1% su {k}: r1={a} r2={b} (criterio p.6.ii)")
if reps:
    t = reps[0]
    g = t["s143.galloc_n"] or 1
    q_obj = 100 * (t["obj.cum_n"] + t["s143.propsbuf_n"]) / g
    q_arr = 100 * (t["arr.cum_n"] + t["s143.arrbuf_n"]) / g
    q_str = 100 * t["str.cum_n"] / g
    q_rcz = 100 * t["s144.rczval_n"] / g
    q_rczp = 100 * t["s144.rczval_prop_n"] / g
    q_va = 100 * t["s144.vecargs_n"] / g
    q_attr = q_obj + q_arr + q_str + q_rcz + q_va
    strict = q_obj + q_rczp
    loose = q_obj + q_rcz
    print(f"quota_obj={q_obj:.2f}% (banda dyn_entries +0/-0,69pp DICHIARATA, criterio p.3)")
    print(f"quota_arr={q_arr:.2f}%  quota_str={q_str:.2f}%")
    print(f"quota_rczval={q_rcz:.2f}% (prop {q_rczp:.2f}%)  quota_vecargs={q_va:.2f}%")
    print(f"quota_obj_max_strict={strict:.2f}%  quota_obj_max_loose={loose:.2f}%")
    print(f"quota_attribuita={q_attr:.2f}%  residuo_other={100-q_attr:.2f}%")
    print(f"conteggi FUORI denominatore (dichiarati, p.3): grealloc_n={t['s144.grealloc_n']} objsynth_n={t['s144.objsynth_n']}")
    print("REGOLA p.5 (pre-registrata; arbitra = regola S-143 p.3 sul maggiorante):")
    if loose < 25:
        print(f"ESITO REGOLA: quota_obj_max_loose {loose:.1f}% < 25% => B sola / B-poi-A CONFERMATA PER MISURA (numeratore chiuso)")
    elif strict >= 40:
        print(f"ESITO REGOLA: quota_obj_max_strict {strict:.1f}% >= 40% => A-poi-B (regola S-143)")
    elif strict >= 25:
        print(f"ESITO REGOLA: quota_obj_max_strict {strict:.1f}% in 25-40% => RICONVOCA")
    else:
        print(f"ESITO REGOLA: loose {loose:.1f}% >= 25% con strict {strict:.1f}% < 25% => RICONVOCA con tranche-3 (flag su make_cell)")
