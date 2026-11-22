#!/usr/bin/env python3
"""s194-notte-confronto.py — S-194 night, criterion p.5: reads the three verdicts and applies the pre-registered rule against the
night of 20/11 (t29, t27, b5b). Output: s194-notte-confronto.out. Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, re
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"); W = P / "wp194-harness"
out = ["== s194 notte: confronto con la notte del 20/11 (criterio s194-criterio-notte.md p.5) =="]
def read(p): return p.read_text(errors="replace") if p.exists() else ""
t = read(W / "s194-orm-coppia-verdetto-t30.out"); m = re.search(r"rapporto net \(companion\): \[([0-9.]+); ([0-9.]+)\]", t)
if m:
    a, b = float(m.group(1)), float(m.group(2))
    v = "CONFERMATA" if b < 6.449 else ("REGRESSIONE: ritorno a s185 sul tavolo dell'utente" if a > 6.576 else "NON DISTINGUIBILE da t29")
    out.append(f"ORM t30 [{a:.3f}; {b:.3f}] contro t29 [6.449; 6.576]: {v}")
else: out.append("ORM t30: verdetto assente o senza riga del rapporto netto")
t = read(W / "s194-pair-verdetto-t28.out"); m = re.search(r"mediana_t28=([0-9.]+)", t)
if m:
    x = float(m.group(1))
    v = "CONFERMATA" if x < 1.683 else ("REGRESSIONE: ritorno a s185 sul tavolo dell'utente" if x > 1.707 else "NON DISTINGUIBILE da t27")
    out.append(f"WordPress t28 mediana {x:.3f} contro t27 1.705 [1.683; 1.707]: {v}")
else: out.append("WordPress t28: verdetto assente o senza mediana")
t = read(P / "wp186-harness/s186-guardia-verdetto-b6.out")
out.append("guardia b6 (righe con D, da leggere contro b5b: K0 -0,46 · mcalls-dq +4,33; soglie del criterio p.5):")
out += ["   " + l[:220] for l in t.splitlines() if re.search(r"\bD\b|rc=", l)][:14] or ["   verdetto assente"]
(W / "s194-notte-confronto.out").write_text("\n".join(out) + "\n")
