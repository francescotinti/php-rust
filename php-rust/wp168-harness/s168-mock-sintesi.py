#!/usr/bin/env python3
"""s168-mock-sintesi.py — sintesi MECCANICA del criterio s168-criterio-mock.md p.4:
legge i verdetti s168-{m0,m1,m2,m3,m123}-verdetto.out (+ disasm), calcola nominati,
chiusura, kill (Σ mock < 10) e additività; stampa il blocco da appendere a
s168-mock-verdetto.out. Solo esiti e numeri (REGOLE §8)."""
import re, sys, os
H = os.path.dirname(os.path.abspath(__file__))
def parse(tag):
    p = f"{H}/s168-{tag}-verdetto.out"
    if not os.path.exists(p): return None
    t = open(p).read()
    g = re.search(r"GIUDICE arith-dq: A=([\d.]+) B=([\d.]+) ns/iter D=A−B=([+-][\d.]+) soglia=([\d.]+) \(rumore drop-1 A'=([\d.]+) B'=([\d.]+)\) -> (\w+)", t)
    e = re.search(r"E2 loop nudo: A=([\d.]+) B=([\d.]+) oracle=([\d.]+) ns/iter; guardia \|e2B−e2A\|=([\d.]+) ≤ ([\d.]+) -> (\w+)", t)
    s = re.search(r"statement \(dq−e2\): A=([\d.]+) oracle=([\d.]+) gap-statement=([\d.]+); controllo-loop: A=([\d.]+) oracle=([\d.]+) gap-loop=([\d.]+); riferimento finestra: dq oracle=([\d.]+) gap totale=([\d.]+)", t)
    rc = re.search(r"ESITO rc=(\d+)", t)
    if not (g and e and s and rc): return {"tag": tag, "err": "verdetto incompleto"}
    return {"tag": tag, "A": float(g[1]), "B": float(g[2]), "D": float(g[3]), "thr": float(g[4]), "nA": float(g[5]), "nB": float(g[6]), "nom": g[7],
            "e2A": float(e[1]), "e2B": float(e[2]), "e2O": float(e[3]), "ge2": e[6], "stA": float(s[1]), "stO": float(s[2]), "gapst": float(s[3]),
            "gaploop": float(s[6]), "dqO": float(s[7]), "gap": float(s[8]), "rc": int(rc[1])}
def bl(tag):
    p = f"{H}/ab-out/disasm-{tag}.out"
    if not os.path.exists(p): return "n/d"
    v = re.findall(r"bl=(\d+)", open(p).read()); return f"A={v[0]} B={v[1]} Δ={int(v[1])-int(v[0]):+d}" if len(v) == 2 else "n/d"
out = ["== S-168 MOCK sottrattivi F0 — SINTESI MECCANICA (criterio s168-criterio-mock.md p.4; A=m0 braccio nullo; giudice arith-dq N=250M; R=5) =="]
R = {t: parse(t) for t in ("m0", "m1", "m2", "m3", "m123")}
for t, r in R.items():
    if r is None: out.append(f"{t}: verdetto ASSENTE"); continue
    if "err" in r: out.append(f"{t}: {r['err']}"); continue
    out.append(f"{t}: A={r['A']:.2f} B={r['B']:.2f} D={r['D']:+.2f} soglia={r['thr']:.2f} rumore(A'={r['nA']:.2f},B'={r['nB']:.2f}) -> {r['nom']}; e2 A={r['e2A']:.2f} B={r['e2B']:.2f} guardia {r['ge2']}; bl run_loop {bl(t)}; rc={r['rc']}")
m0 = R.get("m0")
if m0 and "err" not in m0:
    out.append(f"braccio NULLO m0 vs pin: D={m0['D']:+.2f} (soglia {m0['thr']:.2f}) -> {'ricetta/copia NEUTRA (in banda)' if abs(m0['D']) < m0['thr'] else 'ricetta/copia NON neutra: banda-layout ricetta = ' + format(abs(m0['D']), '.2f')}")
mocks = [R[t] for t in ("m1", "m2", "m3") if R.get(t) and "err" not in R[t]]
if len(mocks) == 3:
    ref = mocks[0]
    named = [(m["tag"], m["D"]) for m in mocks if m["D"] >= m["thr"] and m["ge2"] == "ok"]
    sum_named = sum(d for _, d in named); sum_raw = sum(m["D"] for m in mocks)
    e2 = ref["e2A"]; dq = ref["A"]
    closure = (e2 + sum_named) / dq
    out.append(f"controllo-loop (E2 su A=m0) = {e2:.2f} ns/iter ({e2/dq*100:.1f}% di dq {dq:.2f}); oracle: e2={ref['e2O']:.2f} dq={ref['dqO']:.2f}; gap totale={ref['gap']:.2f} (statement {ref['gapst']:.2f} + loop {ref['gaploop']:.2f})")
    out.append(f"mock NOMINATI (≥soglia, guardia e2 ok): {', '.join(f'{t}={d:+.2f}' for t, d in named) or 'nessuno'}; Σ nominati={sum_named:.2f} (Σ grezza m1+m2+m3={sum_raw:+.2f})")
    out.append(f"CHIUSURA = (e2 + Σ mock nominati)/dq = ({e2:.2f}+{sum_named:.2f})/{dq:.2f} = {closure*100:.1f}% -> " + ("≥90%: GO F1/F2" if closure >= 0.9 else ("85–90%: decomposizione da completare in UNA sessione prima di codice" if closure >= 0.85 else "<85%: NESSUN codice di leva")))
    out.append(f"KILL ⚖️ regola 4 (Σ mock < 10 ns/iter): Σ={sum_named:.2f} -> " + ("SCATTA: R1 «interno-handler via compilatore» ESAURITO su arith ⇒ delibera R4 al concilio" if sum_named < 10 else "non scatta"))
    m123 = R.get("m123")
    if m123 and "err" not in m123:
        noise = max(m["nB"] for m in mocks + [m123]); band = max(4.0, noise)
        dd = m123["D"] - sum_raw
        out.append(f"ADDITIVITÀ: D_m123={m123['D']:+.2f} vs Σ grezza={sum_raw:+.2f} |diff|={abs(dd):.2f} banda={band:.2f} -> " + ("componenti INDIPENDENTI" if abs(dd) <= band else ("SOVRAPPOSIZIONE/sinergia dichiarata: si conta D_m123, mai la somma" )))
print("\n".join(out))
