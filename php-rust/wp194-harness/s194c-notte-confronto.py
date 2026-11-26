#!/usr/bin/env python3
"""s194c-notte-confronto.py — S-194 night b, tail c, criterion s194-criterio-notte-b.md p.11: same-night comparison of pin s190
(t31 ORM, t29 WordPress) with stash s186 (t32, t30) and stash s189 (t33, t31). Output: s194c-notte-confronto.out.
Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, re
W = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness")
def read(n): p = W / n; return p.read_text(errors="replace") if p.exists() else ""
def orm(n):
    m = re.search(r"rapporto net \(companion\): \[([0-9.]+); ([0-9.]+)\]", read(n)); return (float(m.group(1)), float(m.group(2))) if m else None
def wp(tag):
    m = re.search(r"mediana_%s=([0-9.]+)" % tag, read("s194-pair-verdetto-%s.out" % tag)); return float(m.group(1)) if m else None
O = {"s190": orm("s194b-orm-coppia-verdetto-t31.out"), "s186": orm("s194c-orm-coppia-verdetto-t32.out"), "s189": orm("s194c-orm-coppia-verdetto-t33.out")}
P = {"s190": wp("t29"), "s186": wp("t30"), "s189": wp("t31")}
out = ["== s194 notte b, coda c: confronto nella STESSA notte (criterio s194-criterio-notte-b.md p.11) =="]
for k in ("s186", "s189", "s190"):
    o = O[k]; out.append("%s: ORM %s · WordPress %s" % (k, "[%.3f; %.3f]" % o if o else "ASSENTE", "%.3f" % P[k] if P[k] is not None else "ASSENTE"))
def j_orm(new, old):
    a, b = O[new], O[old]
    if not a or not b: return "non giudicabile (verdetto assente)"
    return "CONFERMATA" if a[1] < b[0] else ("REGRESSIONE" if a[0] > b[1] else "NON DISTINGUIBILE")
def j_wp(new, old):
    a, b = P[new], P[old]
    if a is None or b is None: return "non giudicabile (verdetto assente)"
    d = a - b
    return ("CONFERMATA" if d < -0.010 else ("REGRESSIONE" if d > 0.010 else "NON DISTINGUIBILE")) + " (scarto %+.3f, banda 0,010)" % d
out.append("GIUDIZIO s190 contro s186: ORM %s · WordPress %s" % (j_orm("s190", "s186"), j_wp("s190", "s186")))
out.append("ripartizione s189 contro s186: ORM %s · WordPress %s" % (j_orm("s189", "s186"), j_wp("s189", "s186")))
out.append("ripartizione s190 contro s189: ORM %s · WordPress %s" % (j_orm("s190", "s189"), j_wp("s190", "s189")))
if O["s186"]: out.append("deriva fra notti, ORM s186: stanotte [%.3f; %.3f] contro 22/11 [6.075; 6.119]" % O["s186"])
if P["s186"] is not None: out.append("deriva fra notti, WordPress s186: stanotte %.3f contro 22/11 1.646 (scarto %+.3f)" % (P["s186"], P["s186"] - 1.646))
import glob
for tag, want in (("t29", "97e000aa00fe802e"), ("t30", "9977cc4b1cf19991"), ("t31", "2d42efd6606e8dd9")):
    seen = set()
    for f in glob.glob(str(W / "pair-out" / (tag + "-leg*-on") / "*")):
        try: seen |= set(re.findall(r"^phpr=([0-9a-f]{16})", open(f, errors="replace").read(), re.M))
        except (IsADirectoryError, OSError): pass
    out.append("binario delle gambe WordPress %s: %s (atteso %s) %s" % (tag, sorted(seen) or "NON TROVATO", want, "OK" if seen == {want} else "DA VERIFICARE"))
out.append("limite: s190 misurato prima di s186 e s189 (ordine non bilanciato)")
(W / "s194c-notte-confronto.out").write_text("\n".join(out) + "\n")
