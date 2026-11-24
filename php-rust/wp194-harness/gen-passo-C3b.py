#!/usr/bin/env python3
"""gen-passo-C3b.py — S-194 C3b: s194-passo-C3b.sh as a declared copy of s194-passo-C3.sh. ONLY adaptations: three fixtures added to the
list (mcall_private, this_private0, this_protected0) and the default base reading (the pin s187 on the same list). Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, subprocess
W = pathlib.Path(__file__).resolve().parent
s = (W / "s194-passo-C3.sh").read_text()
def sub(a, b):
    global s
    assert s.count(a) == 1, (a, s.count(a)); s = s.replace(a, b)
sub(",call_default,mcall_default", ",call_default,mcall_default,mcall_private,this_private0,this_protected0")
sub("s194-primitive-C3base.out", "s194-primitive-C3bbase.out")
sub("# s194-passo-C3.sh <tag> [base.out] — S-194 C3 (declared copy of s194-passo-C.sh, see gen-passo-C3.py)", "# s194-passo-C3b.sh <tag> [base.out] — S-194 C3b (declared copy of s194-passo-C3.sh, see gen-passo-C3b.py)")
sub("against the pin s187 reading C3base", "against the pin s187 reading C3bbase")
(W / "s194-passo-C3b.sh").write_text(s)
d = subprocess.run(["diff", str(W / "s194-passo-C3.sh"), str(W / "s194-passo-C3b.sh")], capture_output=True, text=True).stdout
(W / "s194-passo-C3b-copia.diff").write_text(d)
print("ok")
