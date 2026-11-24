#!/usr/bin/env python3
"""gen-passo-C3.py — S-194 C3: s194-passo-C3.sh as a declared copy of s194-passo-C.sh. ONLY adaptations: two fixtures added to the list
(call_default, mcall_default) and the default base reading (the pin s187 on the same list). Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, subprocess
W = pathlib.Path(__file__).resolve().parent
s = (W / "s194-passo-C.sh").read_text()
def sub(a, b):
    global s
    assert s.count(a) == 1, (a, s.count(a)); s = s.replace(a, b)
sub(",mcall_typed,mcall_typed_class", ",mcall_typed,mcall_typed_class,call_default,mcall_default")
sub("s194-primitive-C0.out", "s194-primitive-C3base.out")
sub("# s194-passo-C.sh <tag> [base.out] — S-194 package C (declared copy of s194-passo.sh, see gen-passo-C.py)", "# s194-passo-C3.sh <tag> [base.out] — S-194 C3 (declared copy of s194-passo-C.sh, see gen-passo-C3.py)")
sub("against the pin reading C0", "against the pin s187 reading C3base")
(W / "s194-passo-C3.sh").write_text(s)
d = subprocess.run(["diff", str(W / "s194-passo-C.sh"), str(W / "s194-passo-C3.sh")], capture_output=True, text=True).stdout
(W / "s194-passo-C3-copia.diff").write_text(d)
print("ok")
