#!/usr/bin/env python3
"""Legge un profilo samply (Firefox Profiler format, .json.gz) e stampa la
classifica per SELF-TIME. Il self-time e cio che una funzione esegue DA SOLA:
e l'unica metrica che dice dove va la CPU, mentre il total-time dice solo chi
sta piu in alto nello stack.
"""
import gzip, json, sys, collections

path = sys.argv[1]
TOP = int(sys.argv[2]) if len(sys.argv) > 2 else 30

with gzip.open(path, "rt", encoding="utf-8") as fh:
    prof = json.load(fh)

shared = prof.get("shared", {})
strings = shared.get("stringArray") or prof.get("stringArray") or []

def sname(i):
    try:
        return strings[i]
    except Exception:
        return f"<str#{i}>"

total_samples = 0
self_by_func = collections.Counter()
# per attribuire ogni campione alla funzione IN CIMA allo stack
for thread in prof.get("threads", []):
    tname = thread.get("name", "?")
    samples = thread["samples"]
    stack_tbl = thread["stackTable"]
    frame_tbl = thread["frameTable"]
    func_tbl = thread["funcTable"]
    st_frame = stack_tbl["frame"]
    fr_func = frame_tbl["func"]
    fn_name = func_tbl["name"]
    weights = samples.get("weight") or [1] * len(samples["stack"])
    for si, w in zip(samples["stack"], weights):
        if si is None:
            continue
        total_samples += w or 1
        fi = st_frame[si]
        fu = fr_func[fi]
        self_by_func[(tname, fn_name[fu])] += (w or 1)

print(f"campioni totali = {total_samples}")
print(f"{'self%':>7}  {'campioni':>9}  thread / funzione")
for (tname, nidx), c in self_by_func.most_common(TOP):
    pct = 100.0 * c / total_samples if total_samples else 0
    print(f"{pct:7.2f}  {c:9d}  [{tname}] {sname(nidx)}")
