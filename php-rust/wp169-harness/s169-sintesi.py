#!/usr/bin/env python3
"""s169-sintesi.py — sintesi MECCANICA criterio s169-criterio.md p.4 dai verdetti s169-{m5,m7,m4b9,m39}."""
import re, os
H = os.path.dirname(os.path.abspath(__file__))
def parse(tag):
    p = f"{H}/s169-{tag}-verdetto.out"
    if not os.path.exists(p): return None
    t = open(p).read()
    g = re.search(r"GIUDICE arith-dq: A=([\d.]+) B=([\d.]+) ns/iter D=A−B=([+-][\d.]+) soglia=([\d.]+) \(rumore drop-1 A'=([\d.]+) B'=([\d.]+)\)", t)
    d = re.search(r"soglia_dec=([\d.]+) -> ([^(]+)\(", t)
    e = re.search(r"E2 loop nudo: A=([\d.]+) B=([\d.]+) oracle=([\d.]+) ns/iter; guardia \|e2B−e2A\|=([\d.]+) ≤ ([\d.]+) -> (\w+)", t)
    if not (g and d and e): return {"err": "verdetto incompleto"}
    return {"A": float(g[1]), "B": float(g[2]), "D": float(g[3]), "thr": float(g[4]), "n": max(float(g[5]), float(g[6])), "dec": float(d[1]), "dir": d[2].strip(), "e2A": float(e[1]), "e2O": float(e[3]), "ge2": e[6]}
out = ["== S-169 az.rev. — SINTESI MECCANICA (criterio s169-criterio.md p.4; A=m0; arith-dq N=250M) =="]
R = {t: parse(t) for t in ("m5", "m7", "m4b9", "m39")}
for t, r in R.items():
    out.append(f"{t}: " + ("ASSENTE" if r is None else (r["err"] if "err" in r else f"A={r['A']:.2f} B={r['B']:.2f} D={r['D']:+.2f} rumore={r['n']:.2f} pavimento={r['thr']:.2f} soglia_dec={r['dec']:.2f} -> {r['dir']}; guardia e2 {r['ge2']}")))
m5 = R.get("m5"); e2src = R.get("m4b9") or R.get("m39")  # emenda p.6: e2 di m5/m7 invalida, si usa quella di m4b9
if m5 and "err" not in m5 and e2src:
    m5 = dict(m5, e2A=e2src["e2A"], e2O=e2src["e2O"])
    disp = -m5["D"]/8; dn = m5["n"]/8
    body = (m5["e2A"] - 2*disp)/2
    sw = R["m4b9"]["D"] if R.get("m4b9") else 3.0
    out.append(f"DISPATCH puro per-op (8 Nop): {disp:.2f} ns/op (±{dn:.2f}); controllo-loop e2={m5['e2A']:.2f} = 2 dispatch ({2*disp:.2f}) + 2 corpi ⇒ corpo medio CmpJmpSC/IncDecSlotJmp = {body:.2f} ns/op; oracle e2={m5['e2O']:.2f} ⇒ {m5['e2O']/2:.2f} ns/op TOTALI")
    out.append(f"statement 32,0 (S-168, dq−e2) = Sweep {sw:.2f} (dispatch {disp:.2f} + corpo {sw-disp:.2f}) + dispatch BinarySCSCDst {disp:.2f} + corpo BinarySCSCDst ≈ {32.0 - sw - disp:.1f} ns/iter — di cui nominati m1 4,72 + m2 5,20 (m7 5,60) ⇒ corpo NON nominato ≈ {32.0 - sw - disp - 4.72 - 5.20:.1f} (guardie Undef/Ref, read_slot clone, 4 funnel, reg_store_slot+gc_note, bounds)")
    out.append(f"per-op phpr vs oracle: dispatch 1,75 vs (oracle totale {m5['e2O']/2:.2f}/op); handler banale (CmpJmpSC/IncDecSlotJmp) corpo {body:.2f} ⇒ {disp+body:.2f} ns/op contro {m5['e2O']/2:.2f}: il rapporto per-op è {(disp+body)/(m5['e2O']/2):.1f}×; il corpo del handler fuso ({32.0 - sw - disp:.1f}) è {(32.0 - sw - disp)/(disp+body):.1f}× un handler banale")
m7 = R.get("m7")
if m7 and "err" not in m7:
    out.append(f"GUARDIA tupla m2: D_m7 − D_m2 = {m7['D']:+.2f} − 5,20 = {m7['D']-5.20:+.2f} ns/iter (rumore {m7['n']:.2f}) -> m2 «pieno» (BinOp cotto senza guardia) = {m7['D']:+.2f}")
for t, lab in (("m4b9", "m4b (Sweep) R=9"), ("m39", "m3 (hoist, forma cambiata) R=9")):
    r = R.get(t)
    if r and "err" not in r: out.append(f"{lab}: D={r['D']:+.2f} rumore={r['n']:.2f} soglia_dec={r['dec']:.2f} -> {r['dir']} (S-168 R=5: {'+2,96' if t=='m4b9' else '−2,28'})")
print("\n".join(out))
