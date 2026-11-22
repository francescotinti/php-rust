#!/usr/bin/env python3
"""s194-registro-op-classi.py — S-194: net instruction table per op class from the ledger dumps (main process = largest dump).
Net = attributed - K * count, K = per-op residual of the hook (s194-registro-op-taratura.out), shown for K in (50, 65, 80).
Output: s194-registro-op-classi.out. Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, re
W = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness")
CL = [("compilazione/include", r"Include|Eval|Declare|NewAnonDeferred"),
      ("builtin", r"CallBuiltin|CallHostBuiltin|CallNsBuiltin|Strlen$|Count$|CallArrayMultisort"),
      ("chiamate utente e ritorni", r"Ret$|Call|MethodCall|ThisMethodCall|StaticCall|InvokeMethod|InvokeCtor|CheckArity|FillDefault|CoerceParam|PushArgPlace|BindRef|MakeClosure|MakeFcc|HookCall|ClosureStatic|Yield|StaticGuard|StaticStore|StaticAlias"),
      ("oggetti e proprieta'", r"Prop|ThisPropGet|Field|Alloc|InitProps|StampThrowable|StaticProp|Clone|InstanceOf|ClassConst|ClassName|EnumCase|This$"),
      ("array", r"FetchDim|AssignPath|AssignOpPath|IssetPath|EmptyPath|UnsetPath|IncDecPath|Array|Iter|CoalesceFetchDim"),
      ("Sweep (pulizia dei temporanei)", r"Sweep"),
      ("pila e variabili", r"LoadVar|PushConst|Pop|Dup|StoreSlot|LoadSlot|PushRef|Swap|PushUndef|MakeRef|DerefTop|LoadGlobal|StoreGlobal|LoadSuperglobal|Nop"),
      ("salti, confronti, aritmetica", r"Jump|CmpJmp|Binary|Unary|Cast|IncDec|ConstFetch|DefineConst"),
      ("stringhe", r"Concat|Stringify|Echo|Print")]
def cls(n): return next((c for c, r in CL if re.match(r, n)), "altro")
out = []
for work, prod in (("orm", 246.754e9), ("wp", 3779.983e9)):
    t = (W / f"s194-registro-op-{work}.txt").read_text(errors="replace")
    d = max((x for x in re.split(r"(?m)^(?=== PHPR_OP_CENSUS: )", t) if x.startswith("== PHPR_OP_CENSUS")), key=lambda x: int(x.split()[2]))
    L = d.splitlines(); i0 = next(i for i, l in enumerate(L) if l.startswith("-- instructions by op")); i1 = next(i for i, l in enumerate(L) if l.startswith("-- instructions by builtin")); i2 = next(i for i, l in enumerate(L) if l.startswith("-- op counts"))
    ops = [(a[5], int(a[0]), int(a[2])) for a in (l.split() for l in L[i0 + 1:i1]) if len(a) == 6]
    N = sum(o[2] for o in ops); G = sum(o[1] for o in ops)
    out.append(f"== {work}: {N} op, attribuito lordo {G / 1e9:.1f} G; produzione braccio A {prod / 1e9:.1f} G ==")
    out.append("   " + L[1][:260])
    for K in (50, 65, 80):
        T = G - K * N
        out.append(f"-- K = {K}: netto {T / 1e9:.1f} G ({(T / prod - 1) * 100:+.1f} % sulla produzione), {T / N:.0f} per op")
        g = {}
        for n, s, c in ops:
            e = g.setdefault(cls(n), [0, 0]); e[0] += s - K * c; e[1] += c
        for c, (s, n) in sorted(g.items(), key=lambda kv: -kv[1][0]):
            out.append(f"   {c:32s} {s / 1e9:8.1f} G  {100 * s / T:5.1f} % istr  {100 * n / N:5.1f} % op  {s / n:10.0f} per op")
    K = 65; T = G - K * N
    out.append("-- i 25 op piu' costosi (K = 65): quota, netto per op, milioni di esecuzioni")
    for n, s, c in sorted(ops, key=lambda o: -(o[1] - K * o[2]))[:25]:
        out.append(f"   {n:20s} {100 * (s - K * c) / T:5.1f} %  {(s - K * c) / c:10.0f}  {c / 1e6:9.1f} M")
    out.append("-- builtin per nome (lordo: K pesa solo sui piu' leggeri): i primi 25")
    for a in [l.split() for l in L[i1 + 1:i2] if len(l.split()) == 4][:25]:
        out.append(f"   {a[3]:28s} {100 * int(a[0]) / T:5.1f} %  {float(a[2]):12.0f} per chiamata  {int(a[1]):10d} chiamate")
(W / "s194-registro-op-classi.out").write_text("\n".join(out) + "\n")
