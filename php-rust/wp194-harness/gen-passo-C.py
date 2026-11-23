#!/usr/bin/env python3
"""gen-passo-C.py — S-194 package C: s194-passo-C.sh as a declared copy of s194-passo.sh. ONLY adaptations: the fixture list (calls and
returns instead of builtins/strings) and the default base reading (the pin s186 on the same list). Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, subprocess
W = pathlib.Path(__file__).resolve().parent
s = (W / "s194-passo.sh").read_text()
def sub(a, b):
    global s
    assert s.count(a) == 1, (a, s.count(a)); s = s.replace(a, b)
ONLY = "noop,ns_noop,call_user0,call_user2,call_typed,call_method,call_static,call_closure,call_this_inc,new_ctor,ns_user2,ret_typed_only,ret_typed_class,call_typed_class,call_typed_iface,mcall_typed,mcall_typed_class"
old = s[s.index("--only ") + 7:s.index("\n", s.index("--only "))]
sub("--only " + old, "--only " + ONLY)
sub("s194-primitive-D.out", "s194-primitive-C0.out")
sub("# s194-passo.sh <tag> [base.out] — S-194 D-bis, one measured step", "# s194-passo-C.sh <tag> [base.out] — S-194 package C (declared copy of s194-passo.sh, see gen-passo-C.py), one measured step")
sub("on the builtin/string fixtures against the D reading", "on the call/return fixtures against the pin reading C0")
(W / "s194-passo-C.sh").write_text(s)
d = subprocess.run(["diff", str(W / "s194-passo.sh"), str(W / "s194-passo-C.sh")], capture_output=True, text=True).stdout
(W / "s194-passo-C-copia.diff").write_text(d)
print("ok", ONLY.count(",") + 1, "fixture")
