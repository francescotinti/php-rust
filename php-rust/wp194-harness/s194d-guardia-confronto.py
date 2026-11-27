#!/usr/bin/env python3
"""s194d-guardia-confronto.py — S-194 night b, tail d, criterion s194-criterio-notte-b.md p.15: reads guard verdicts b7, b7b
(B = pin s190) and b7c (B = stash s189) and applies the pre-registered rule on mcalls-dq. Output: s194d-guardia-confronto.out.
Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, re
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
def g(tag):
    t = (P / ("wp186-harness/s186-guardia-verdetto-%s.out" % tag)); t = t.read_text(errors="replace") if t.exists() else ""
    r = {}
    for name in ("mcalls-dq", "calls-dq", "ret-buf-dq-K0"):
        m = re.search(r"GIUDICE " + re.escape(name) + r": .*? B=([0-9.]+) .*?D=B−A colonna=([+-][0-9.]+) appaiata=([+-][0-9.]+) soglia=([0-9.]+) \(rumore drop-1 ([0-9.]+)", t)
        if m: r[name] = tuple(float(x) for x in m.groups())
    return r
out = ["== s194 notte b, coda d: istruttoria della guardia (criterio s194-criterio-notte-b.md p.15) =="]
G = {t: g(t) for t in ("b7", "b7b", "b7c")}
for t, lab in (("b7", "B=s190, 18:42"), ("b7b", "B=s190, ripetizione"), ("b7c", "B=s189")):
    for name, v in G[t].items():
        out.append("%s (%s) %s: B=%.2f ns · D colonna %+.2f appaiata %+.2f · rumore drop-1 %.2f" % (t, lab, name, v[0], v[1], v[2], v[4]))
b, c = G["b7b"].get("mcalls-dq"), G["b7c"].get("mcalls-dq")
if not b: out.append("GIUDIZIO: b7b assente ⇒ NON DECISO, s190 resta non confermato sulla guardia")
elif b[1] > 6.00:
    where = "non localizzato (b7c assente)" if not c else ("già in s189 (s187–s189)" if c[1] > 6.00 else "in s190 (C4, N1, N2, T1, N3)")
    out.append("GIUDIZIO: b7b mcalls-dq D %+.2f > 6,00 ⇒ costo REALE sulle chiamate di metodo, entrato %s; s190 NON confermato sulla guardia" % (b[1], where))
elif b[4] < 2.00:
    out.append("GIUDIZIO: b7b mcalls-dq D %+.2f <= 6,00 con rumore %.2f < 2,00 ⇒ b7 attribuita alla finestra rumorosa, apertura INVARIATA ⇒ s190 CONFERMATO di notte" % (b[1], b[4]))
else:
    out.append("GIUDIZIO: b7b mcalls-dq D %+.2f <= 6,00 ma rumore %.2f >= 2,00 ⇒ NON DECISO, s190 resta non confermato sulla guardia" % (b[1], b[4]))
(P / "wp194-harness/s194d-guardia-confronto.out").write_text("\n".join(out) + "\n")
