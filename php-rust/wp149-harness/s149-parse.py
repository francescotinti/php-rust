#!/usr/bin/env python3
# s149-parse.py — parser del census tranche-4 per-NOME-builtin (criterio
# s149-criterio-tr4.md p.3-4-7). Collaudato da s149-parse-golden-test.sh
# PRIMA del run. Uso: s149-parse.py <dir con census-mem-r*.txt>
# SOLO conteggi: mai cifre di tempo come record (p.7: soglia INDIZIO).
import glob
import re
import sys

out = sys.argv[1]
SOGLIA_INDIZIO = 24_800_000  # p.7 EREDITATA s148: 0,293 s / 11,8 ns (INDIZIO mai budget)
TOP = 30  # teste stampate; il resto aggregato in una riga
# riferimento s148 (wp148-harness/s148-attrib-verdetto.out, media repliche)
# per la comparabilita' p.4 (scarti >1% DICHIARATI, non gate):
S148_HOSTCALL_REF = {"n": 325_416_908, "attr": 159_833_568, "other": 165_583_340}

name_rep, sum_rep, taghost_rep = {}, {}, {}
for f in sorted(glob.glob(f"{out}/census-mem-r*.txt")):
    rep = re.search(r"census-mem-(r\d)", f).group(1)
    names = {}
    ident = {"hostcall_n": 0, "sum_name_n": 0, "unnamed_n": 0, "overflow": 0}
    tagh = {"n": 0, "attr": 0}
    for line in open(f, errors="replace"):
        if line.startswith("s149name "):
            kv = dict(re.findall(r"(\S+)=(\S+)", line))
            d = names.setdefault(kv["name"], {"n": 0, "attr": 0, "b": 0})
            d["n"] += int(kv["n"])
            d["attr"] += int(kv["attr"])
            d["b"] += int(kv["b"])
        elif line.startswith("s149sum "):
            kv = dict(re.findall(r"(\w+)=(\d+)", line))
            for k in ident:
                ident[k] += int(kv[k])
        elif line.startswith("s148tag ") and " name=hostcall " in line:
            kv = dict(re.findall(r"(\w+)=(\S+)", line))
            tagh["n"] += int(kv["n"])
            tagh["attr"] += int(kv["attr"])
    name_rep[rep] = names
    sum_rep[rep] = ident
    taghost_rep[rep] = tagh

# p.4: identita' per replica (Σ nomi + unnamed == hostcall.n, ESATTA)
for rep in sorted(sum_rep):
    i = sum_rep[rep]
    ok = "OK" if i["sum_name_n"] + i["unnamed_n"] == i["hostcall_n"] else "VIOLATA"
    print(f"{rep}_identita: hostcall_n={i['hostcall_n']} sum_name_n={i['sum_name_n']} "
          f"unnamed_n={i['unnamed_n']} overflow={i['overflow']} {ok}")
    xok = "OK" if i["hostcall_n"] == taghost_rep[rep]["n"] else "DIVERGE"
    print(f"{rep}_crosscheck_s148tag: hostcall.n={taghost_rep[rep]['n']} {xok}")

# p.4: comparabilita' col perimetro s148 (dichiara, non gate)
for rep in sorted(taghost_rep):
    t = taghost_rep[rep]
    other = t["n"] - t["attr"]
    for k, now in (("n", t["n"]), ("attr", t["attr"]), ("other", other)):
        ref = S148_HOSTCALL_REF[k]
        d = abs(now - ref) / ref if ref else 0.0
        flag = " DICHIARA scarto>1% vs s148" if d > 0.01 else ""
        print(f"{rep}_hostcall_{k}: ora={now} s148={ref} delta={100*d:.3f}%{flag}")

reps = sorted(name_rep)
merged = {}
for rep in reps:
    for nm, d in name_rep[rep].items():
        m = merged.setdefault(nm, {r: None for r in reps})
        m[rep] = d

# p.4: replica r1==r2 <=1% su n/attr/b delle TESTE (per other medio)
def other_of(nm):
    tot = 0
    for rep in reps:
        d = merged[nm][rep]
        if d:
            tot += d["n"] - d["attr"]
    return tot // max(1, len(reps))

heads = sorted(merged, key=lambda nm: -other_of(nm))[:TOP]
if len(reps) == 2:
    worst = (0.0, "-")
    for nm in heads:
        for k in ("n", "attr", "b"):
            a = (merged[nm][reps[0]] or {}).get(k, 0)
            b = (merged[nm][reps[1]] or {}).get(k, 0)
            m = max(a, b)
            d = abs(a - b) / m if m else 0.0
            if d > worst[0]:
                worst = (d, f"{nm}.{k}")
    print(f"replica_worst_head: {worst[1]} {100*worst[0]:.3f}%")
    if worst[0] > 0.01:
        print("DICHIARA r1!=r2 >1% su una testa: replica dovuta (criterio p.8)")

# tabella per NOME (media repliche) + ranking other
host_n = max(1, sum(i["hostcall_n"] for i in sum_rep.values()) // max(1, len(sum_rep)))
host_other_ref = max(1, sum(t["n"] - t["attr"] for t in taghost_rep.values())
                     // max(1, len(taghost_rep)))
nrep = max(1, len(reps))
rows = []
for nm in merged:
    n = sum((merged[nm][r] or {}).get("n", 0) for r in reps) // nrep
    at = sum((merged[nm][r] or {}).get("attr", 0) for r in reps) // nrep
    b = sum((merged[nm][r] or {}).get("b", 0) for r in reps) // nrep
    rows.append((nm, n, at, n - at, b))
rows.sort(key=lambda r: -r[3])
print(f"--- partizione hostcall.n per NOME (media repliche; other = n - attr; "
      f"nomi totali={len(rows)}, stampati top {TOP} per other) ---")
for nm, n, at, ot, b in rows[:TOP]:
    print(f"name {nm:<28} n={n:>11} attr={at:>11} other={ot:>11} "
          f"({100*ot/host_other_ref:5.2f}% di hostcall.other) b={b}")
rest = rows[TOP:]
if rest:
    rn = sum(r[1] for r in rest)
    ra = sum(r[2] for r in rest)
    ro = sum(r[3] for r in rest)
    rb = sum(r[4] for r in rest)
    print(f"name RESTO({len(rest)} nomi)            n={rn:>11} attr={ra:>11} "
          f"other={ro:>11} ({100*ro/host_other_ref:5.2f}% di hostcall.other) b={rb}")
print(f"--- ranking OTHER per NOME (teste tranche-4; soglia INDIZIO p.7 = "
      f"{SOGLIA_INDIZIO} eventi) ---")
for nm, n, at, ot, b in rows[:TOP]:
    verdict = ("CANDIDATA sonda-prezzo" if ot >= SOGLIA_INDIZIO
               else "sotto soglia (solo per FAMIGLIA)")
    print(f"other {nm:<28} {ot:>11} ({100*ot/host_n:5.2f}% di hostcall.n)  {verdict}")
