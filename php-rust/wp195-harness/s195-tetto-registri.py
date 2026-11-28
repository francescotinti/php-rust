#!/usr/bin/env python3
"""s195-tetto-registri.py — S-195: measured ceiling of the register model (criterion s195-criterio-tetto-registri.md p.2).
Reads the whole-workload instruction counts of the pin in both modes (labels ON/OFF in wp194-harness/s194-{orm,wp}-istruzioni.out,
last block) and the op census in both modes (s195-censimento-op-<load>-<mode>.txt, the dump with the most ops = the main process).
Prints E, the op delta, e = E / delta, the movement ops M left with the pass on, the residual R the census can count, the ceiling.
Counting only. Output: s195-tetto-registri.out. Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib, re, sys
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
W4, W5 = P / "wp194-harness", P / "wp195-harness"
MOVE = ["LoadVar", "LoadSlot", "PushConst", "LoadVarPushConst", "StoreSlot", "Pop", "Dup", "Swap"]
LOADS = ["LoadVar", "LoadSlot", "PushConst", "LoadVarPushConst"]
CALLS = ["Call", "CallValue", "CallNsFallback", "CallNsBuiltin", "CallBuiltin", "CallHostBuiltin", "CallHostBuiltinRef", "CallHostBuiltinOut",
         "CallBuiltinRef", "MethodCall", "ThisMethodCall", "MethodCallDynamic", "StaticCall", "StaticCallDynamic", "InvokeCtor", "InvokeMethod",
         "Strlen", "Count", "HookCall"]

def instr(load):
    lines = (W4 / f"s194-{load}-istruzioni.out").read_text().splitlines()
    start = max(i for i, l in enumerate(lines) if l.startswith("== "))
    got = {}
    for l in lines[start:]:
        m = re.match(r"(ON|OFF)\s+istruzioni\s+([\d.]+) G", l)
        if m: got[m.group(1)] = float(m.group(2)) * 1e9
    return got

def census(path):
    txt = pathlib.Path(path).read_text(errors="replace")
    blocks = re.split(r"(?m)^(?=== PHPR_OP_CENSUS: )", txt)
    best = max((b for b in blocks if b.startswith("== PHPR_OP_CENSUS")), key=lambda b: int(re.match(r"== PHPR_OP_CENSUS: (\d+)", b).group(1)))
    total = int(re.match(r"== PHPR_OP_CENSUS: (\d+)", best).group(1))
    ops, big, sec = {}, {}, None
    for l in best.splitlines()[1:]:
        if l.startswith("-- "):
            sec = "ops" if l.startswith("-- op counts") else "big" if l.startswith("-- bigrams (top") else None
            continue
        if sec == "ops":
            m = re.match(r"\s*(\d+)\s+[\d.]+%\s+(\w+)", l)
            if m: ops[m.group(2)] = int(m.group(1))
        elif sec == "big":
            m = re.match(r"\s*(\d+)\s+(\w+) -> (\w+)", l)
            if m: big[(m.group(2), m.group(3))] = int(m.group(1))
    return total, ops, big

def main(files=None):
    out = []
    for load in ("orm", "wp"):
        on = census(files[load]["on"] if files else W5 / f"s195-censimento-op-{load}-on.txt")
        M = sum(on[1].get(o, 0) for o in MOVE)
        assign = sum(n for (a, b), n in on[2].items() if a in LOADS and b == "StoreSlot")
        lastarg = sum(n for (a, b), n in on[2].items() if a in LOADS and b in CALLS)
        R = assign + lastarg
        out.append(f"== {load}: op a modo acceso {on[0]/1e6:.1f} M; movimento M = {M/1e6:.1f} M ({100*M/on[0]:.1f} % degli op); residuo contabile R = {R/1e6:.1f} M (assegnazioni nude {assign/1e6:.1f} + ultimo argomento nudo {lastarg/1e6:.1f}) ==")
        missing = [o for o in MOVE if o not in on[1]]
        if missing: out.append(f"   fuori dai primi 40 (contati 0): {' '.join(missing)}")
        if files and "off" not in files[load]: continue
        off = census(files[load]["off"] if files else W5 / f"s195-censimento-op-{load}-off.txt")
        dop = off[0] - on[0]
        ins = instr(load)
        if "ON" not in ins or "OFF" not in ins: out.append("   istruzioni ON/OFF assenti"); continue
        E = ins["OFF"] - ins["ON"]; e = E / dop if dop else float("nan")
        ceil = e * (M - R) / ins["ON"] * 100
        out.append(f"   istruzioni: spento {ins['OFF']/1e9:.3f} G, acceso {ins['ON']/1e9:.3f} G, E = {E/1e9:.3f} G ({100*E/ins['OFF']:.2f} % dello spento)")
        out.append(f"   op: spento {off[0]/1e6:.1f} M, acceso {on[0]/1e6:.1f} M, delta = {dop/1e6:.1f} M; e = {e:.1f} istruzioni per op tolta")
        out.append(f"   TETTO (per eccesso) = e x (M - R) / istruzioni(acceso) = {ceil:.2f} %   [senza togliere R: {e*M/ins['ON']*100:.2f} %]")
    txt = "\n".join(out) + "\n"
    if not files: (W5 / "s195-tetto-registri.out").write_text(txt)
    sys.stdout.write(txt)

if __name__ == "__main__":
    if len(sys.argv) == 3: main({"orm": {"on": sys.argv[1]}, "wp": {"on": sys.argv[2]}})  # dry run of the parser on existing census files
    else: main()
