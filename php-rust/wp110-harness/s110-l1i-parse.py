#!/usr/bin/env python3
# s110-l1i-parse.py — dal MetricTable esportato alle quote mediane per motore/giudice
# (criterio v2 9ff53cf). Statistica: mediana delle finestre ~1ms del processo
# bersaglio, poi mediana delle R=3 corse. Emette il verbale ascii-nudo con gli N.
import sys, os, glob, statistics as st
import xml.etree.ElementTree as ET

OUT = "/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp110-harness/l1i-out/coll"
NAMES = ["Instruction Delivery Bottleneck", "Discarded Bottleneck",
         "Instruction Processing Bottleneck", "Useful"]
SHORT = {"Instruction Delivery Bottleneck": "delivery",
         "Discarded Bottleneck": "discarded",
         "Instruction Processing Bottleneck": "processing",
         "Useful": "useful"}
TARGET = {"oracle": "php", "phpr": "phpr"}

def rows_for(path, procname):
    # risolve la compressione id/ref dell'export xctrace; le DEFINIZIONI possono
    # stare annidate (es. process dentro thread), quindi l'indice id->elemento
    # va costruito con una pre-passata sull'INTERO albero, non sui figli diretti
    root = ET.parse(path).getroot()
    byid = {}
    for el in root.iter():
        if el.get("id") is not None:
            byid[el.get("id")] = el
    def resolve(el):
        if el.get("ref") is not None:
            return byid.get(el.get("ref"))
        return el
    per_name = {n: [] for n in NAMES}
    n_win = 0
    for row in root.iter("row"):
        kids = [resolve(k) for k in row]
        name = proc = val = None
        for k in kids:
            if k is None: continue
            if k.tag == "string" and k.get("fmt") in NAMES:
                name = k.get("fmt")
            elif k.tag == "process":
                proc = (k.get("fmt") or "")
            elif k.tag == "fixed-decimal":
                val = float(k.text)
        if name and val is not None and proc is not None and proc.startswith(procname + " "):
            per_name[name].append(val)
            n_win += 1
    return per_name, n_win

def main():
    print("s110-l1i-verdetto: quote top-down bilaterali (criterio v2 9ff53cf)")
    print("grade=VERDICT  # mediana finestre ~1ms del processo bersaglio, poi mediana R=3")
    print("formato=ascii-nudo")
    agg = {}
    for side in ("oracle", "phpr"):
        for j in ("arith", "prop", "arr"):
            runs = sorted(glob.glob(f"{OUT}/{side}-{j}-r*.xml"))
            per_run = {SHORT[n]: [] for n in NAMES}
            wins = []
            for f in runs:
                per_name, n_win = rows_for(f, TARGET[side])
                wins.append(n_win)
                for n in NAMES:
                    if per_name[n]:
                        per_run[SHORT[n]].append(st.median(per_name[n]))
            for n in NAMES:
                s = SHORT[n]
                med = st.median(per_run[s]) if per_run[s] else float("nan")
                agg[(side, j, s)] = med
            print(f"{side}_{j}: R={len(runs)} finestre_per_run={wins} " +
                  " ".join(f"{SHORT[n]}={agg[(side,j,SHORT[n])]:.4f}" for n in NAMES))
    print("-- contrasti phpr/oracle (quota su quota, per giudice) --")
    for j in ("arith", "prop", "arr"):
        parts = []
        for s in ("delivery", "discarded", "processing", "useful"):
            o, p = agg[("oracle", j, s)], agg[("phpr", j, s)]
            parts.append(f"{s}={p/o:.2f}x" if o and o == o else f"{s}=n/d")
        print(f"contrasto_{j}: " + " ".join(parts))

if __name__ == "__main__":
    main()
