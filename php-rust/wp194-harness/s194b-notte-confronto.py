#!/usr/bin/env python3
"""s194b-notte-confronto.py — S-194 night b, criterion s194-criterio-notte-b.md p.5: reads the three verdicts and applies the
pre-registered rule against the night of 22/11 (t30, t28, b6). Output: s194b-notte-confronto.out.
Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, re
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"); W = P / "wp194-harness"
out = ["== s194 notte b: pin s190 contro la notte del 22/11 su s186 (criterio s194-criterio-notte-b.md p.5) =="]
def read(p): return p.read_text(errors="replace") if p.exists() else ""
bad = False
t = read(W / "s194b-orm-coppia-verdetto-t31.out"); m = re.search(r"rapporto net \(companion\): \[([0-9.]+); ([0-9.]+)\]", t)
if m:
    a, b = float(m.group(1)), float(m.group(2))
    v = "CONFERMATA" if b < 6.075 else ("REGRESSIONE: ritorno a s186 sul tavolo dell'utente" if a > 6.119 else "NON DISTINGUIBILE da t30")
    bad |= a > 6.119
    out.append(f"ORM t31 [{a:.3f}; {b:.3f}] contro t30 [6.075; 6.119]: {v}")
else: out.append("ORM t31: verdetto assente o senza riga del rapporto netto"); bad = True
t = read(W / "s194-pair-verdetto-t29.out"); m = re.search(r"mediana_t29=([0-9.]+)", t)
if m:
    x = float(m.group(1))
    v = "CONFERMATA" if x < 1.640 else ("REGRESSIONE: ritorno a s186 sul tavolo dell'utente" if x > 1.652 else "NON DISTINGUIBILE da t28")
    bad |= x > 1.652
    out.append(f"WordPress t29 mediana {x:.3f} contro t28 1.646 [1.640; 1.652]: {v}")
else: out.append("WordPress t29: verdetto assente o senza mediana"); bad = True
t = read(P / "wp186-harness/s186-guardia-verdetto-b7.out")
def col(name):
    m = re.search(r"GIUDICE " + re.escape(name) + r":.*?D=B−A colonna=([+-][0-9.]+)", t)
    return float(m.group(1)) if m else None
mc, k0 = col("mcalls-dq"), col("ret-buf-dq-K0")
rc = re.search(r"ESITO rc=(\d+)", t)
if rc is None or mc is None or k0 is None:
    out.append("guardia b7: verdetto assente o incompleto"); bad = True
elif rc.group(1) == "0":
    out.append(f"guardia b7 rc=0 (mcalls-dq D {mc:+.2f}, K0 D {k0:+.2f}): aperture di costo CHIUSE")
elif mc <= 6.00 and k0 <= 4.0:
    out.append(f"guardia b7 rc={rc.group(1)} (mcalls-dq D {mc:+.2f} <= 6,00; K0 D {k0:+.2f} <= +4): apertura INVARIATA, ereditata (b6: +5,00 / -2,00)")
else:
    out.append(f"guardia b7 rc={rc.group(1)} (mcalls-dq D {mc:+.2f}, K0 D {k0:+.2f}): COSTO AGGIUNTO alle chiamate, da istruire"); bad = True
out.append("pin s190: " + ("NON confermato (vedi sopra)" if bad else "CONFERMATO di notte secondo la regola del p.5"))
out.append("guardia b7, righe del giudice:")
out += ["   " + l[:220] for l in t.splitlines() if re.search(r"\bD\b|rc=", l)][:14] or ["   verdetto assente"]
(W / "s194b-notte-confronto.out").write_text("\n".join(out) + "\n")
