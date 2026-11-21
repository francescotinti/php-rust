#!/usr/bin/env python3
"""S-194 primitive anatomy: retired instructions per iteration, phpr pin vs PHP oracle.
Differential over two iteration counts so startup and compile cancel. Generated with AI assistance (Claude Fable 5.2)."""
import os, re, statistics, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
FX = os.path.join(HERE, "primitive")
ENG = {"php": "/opt/homebrew/opt/php/bin/php",
       "phpr": os.path.expanduser("~/Claude/php-rust-output/release/phpr")}
N1 = 1_000_000
REPS = 3
PRE = r'''class C { const K = 7; public $x = 0; public $y = 1; public static $sp = 3;
  function m($a) { return $a + 1; } static function sm($a) { return $a + 1; } function inc() { $this->x++; } }
class D { function __construct(public int $a) {} }
function f0() {} function f2($a, $b) { return $a + $b; } function ft(int $a, int $b): int { return $a + $b; }
'''
SETUP = r'''$s = 0; $x = 0; $t = ''; $f = 0.0; $o = new C; $u = "hello"; $w = str_repeat("abcdef", 5);
  $a = range(0, 1023); $h = ['alpha' => 1, 'beta' => 2, 'gamma' => 3]; $k = []; for ($j = 0; $j < 1024; $j++) { $k["k$j"] = $j; }
  $c = function ($a) { return $a + 1; }; $r = range(0, 99); $subj = "the quick brown fox jumps over the lazy dog 12345";
'''
# name, family, body, scale divisor
F = [
 ("noop", "loop", "", 1),
 ("assign_const", "loop", "$x = 1;", 1),
 ("assign_var", "loop", "$x = $i;", 1),
 ("add", "arith", "$s = $s + $i;", 1),
 ("add_assign", "arith", "$s += $i;", 1),
 ("arith_mix", "arith", "$s += $i*3 - ($i>>2);", 1),
 ("branch", "arith", "if ($i & 1) { $s++; }", 1),
 ("float_add", "arith", "$f = $f + 1.5;", 1),
 ("call_user0", "calls", "f0();", 1),
 ("call_user2", "calls", "$s = f2($s, 1);", 1),
 ("call_typed", "calls", "$s = ft($s, 1);", 1),
 ("call_method", "calls", "$s = $o->m($s);", 1),
 ("call_static", "calls", "$s = C::sm($s);", 1),
 ("call_closure", "calls", "$s = $c($s);", 1),
 ("call_this_inc", "calls", "$o->inc();", 1),
 ("builtin_strlen", "builtin", "$s += strlen($u);", 1),
 ("builtin_abs", "builtin", "$s += abs($i);", 1),
 ("builtin_count", "builtin", "$s += count($h);", 1),
 ("prop_read", "prop", "$s += $o->y;", 1),
 ("prop_write", "prop", "$o->x = $i;", 1),
 ("prop_rw", "prop", "$o->x = $o->y + 1; $s += $o->x;", 1),
 ("static_prop", "prop", "$s += C::$sp;", 1),
 ("class_const", "prop", "$s += C::K;", 1),
 ("instanceof", "prop", "if ($o instanceof C) { $s++; }", 1),
 ("new_obj", "object", "$q = new C;", 1),
 ("new_ctor", "object", "$q = new D($i);", 1),
 ("arr_int_read", "array", "$s += $a[$i & 1023];", 1),
 ("arr_int_write", "array", "$a[$i & 1023] = $i;", 1),
 ("arr_isset", "array", "if (isset($a[$i & 1023])) { $s++; }", 1),
 ("arr_str_read", "array", "$s += $h['beta'];", 1),
 ("arr_str_write", "array", "$h['beta'] = $i;", 1),
 ("arr_dynkey", "array", '$s += $k["k" . ($i & 1023)];', 1),
 ("arr_literal", "array", "$t = [$i, $i + 1];", 1),
 ("foreach100", "array", "foreach ($r as $v) { $s += $v; }", 50),
 ("str_concat", "string", '$t = $u . "abc";', 1),
 ("str_interp", "string", '$t = "k$i";', 1),
 ("str_append", "string", '$t .= "x";', 1),
 ("str_substr", "string", "$t = substr($w, -30);", 1),
 ("str_official", "string", '$t = substr($t . "abc", -30);', 1),
 ("str_eq", "string", 'if ($u === "hello") { $s++; }', 1),
 ("int_to_str", "string", "$t = (string)$i;", 1),
 ("preg_match", "regex", r"if (preg_match('/(\w+)\s+(\d+)/', $subj, $m)) { $s++; }", 10),
]
# global-scope variants: the shape of the official micro benchmarks
G = [
 ("g_noop", "global", "", 1),
 ("g_add_assign", "global", "$s += $i;", 1),
 ("g_call_user2", "global", "$s = f2($s, 1);", 1),
 ("g_prop_rw", "global", "$o->x = $o->y + 1; $s += $o->x;", 1),
]
# literal loop bound (the shape of the official micro benchmarks): one file per N
L = [
 ("lit_noop", "literal", "", 1),
 ("lit_assign_const", "literal", "$x = 1;", 1),
 ("lit_add_assign", "literal", "$s += $i;", 1),
 ("lit_add", "literal", "$s = $s + $i;", 1),
]
TAIL = ' echo $s, "|", strlen((string)$t), "|", $x, "\\n";'

def gen():
    os.makedirs(FX, exist_ok=True)
    for name, _, body, _ in F:
        src = "<?php\n" + PRE + "function run($n) {\n  " + SETUP + "  for ($i = 0; $i < $n; $i++) { " + body + " }\n " + TAIL.replace("echo", "echo") + "\n}\nrun((int)$argv[1]);\n"
        open(os.path.join(FX, name + ".php"), "w").write(src)
    for name, _, body, _ in G:
        src = "<?php\n" + PRE + "$n = (int)$argv[1];\n" + SETUP + "for ($i = 0; $i < $n; $i++) { " + body + " }\n" + TAIL + "\n"
        open(os.path.join(FX, name + ".php"), "w").write(src)

def lit_path(name, n):
    body = [b for (nm, _, b, _) in L if nm == name][0]
    p = os.path.join(FX, f"{name}_{n}.php")
    open(p, "w").write("<?php\n" + PRE + "function run($n) {\n  " + SETUP + "  for ($i = 0; $i < " + str(n) + "; $i++) { " + body + " }\n " + TAIL + "\n}\nrun((int)$argv[1]);\n")
    return p

def once(eng, path, n):
    p = subprocess.run(["/usr/bin/time", "-l", ENG[eng], path, str(n)], capture_output=True)
    err = p.stderr.decode("latin-1")
    ins = int(re.search(r"(\d+)\s+instructions retired", err).group(1))
    cyc = int(re.search(r"(\d+)\s+cycles elapsed", err).group(1))
    return p.returncode, p.stdout, ins, cyc

def measure(eng, path, n):
    runs = [once(eng, path, n) for _ in range(REPS)]
    ins = [r[2] for r in runs]
    spread = (max(ins) - min(ins)) / statistics.median(ins) * 100
    return runs[0][0], runs[0][1], statistics.median(ins), statistics.median(r[3] for r in runs), spread

def main():
    gen()
    rows, notes = [], []
    only = set(sys.argv[2:])
    for name, fam, _, div in F + G + L:
        if only and name not in only: continue
        path = os.path.join(FX, name + ".php")
        n1 = N1 // div; n2 = 3 * n1
        paths = (lit_path(name, n1), lit_path(name, n2)) if fam == "literal" else (path, path)
        per = {}; outs = {}; ok = True; worst = 0.0
        for eng in ("php", "phpr"):
            rc1, o1, i1, c1, s1 = measure(eng, paths[0], n1)
            rc2, o2, i2, c2, s2 = measure(eng, paths[1], n2)
            if rc1 or rc2: ok = False; notes.append(f"{name}: rc {eng} {rc1}/{rc2}")
            outs[eng] = (o1, o2); worst = max(worst, s1, s2)
            per[eng] = ((i2 - i1) / (n2 - n1), (c2 - c1) / (n2 - n1))
        if outs["php"] != outs["phpr"]: ok = False; notes.append(f"{name}: stdout differs")
        rows.append((name, fam, per, ok, worst))
        print(name, "ok" if ok else "NULLA", flush=True)
    base = {r[0]: r[2] for r in rows}
    out = []
    out.append("== S-194 primitive: istruzioni ritirate per iterazione (differenziale N1/N2), pin phpr contro oracle. DIREZIONE ==")
    out.append(f"{'fixture':15s} {'fam':8s} {'php':>8s} {'phpr':>8s} {'x':>5s} | {'php net':>8s} {'phpr net':>8s} {'x net':>6s} {'eccesso':>8s} | {'cyc x':>5s} {'IPC php':>7s} {'IPC phpr':>8s} {'disp%':>5s}")
    for name, fam, per, ok, worst in rows:
        nb = "g_noop" if fam == "global" else ("lit_noop" if fam == "literal" else "noop")
        pi, pc = per["php"]; ri, rc = per["phpr"]
        npi = pi - base[nb]["php"][0]; nri = ri - base[nb]["phpr"][0]
        xn = f"{nri / npi:6.2f}" if npi > 2 else "     -"
        out.append(f"{name:15s} {fam:8s} {pi:8.1f} {ri:8.1f} {ri / pi:5.2f} | {npi:8.1f} {nri:8.1f} {xn} {nri - npi:8.1f} | {rc / pc:5.2f} {pi / pc:7.2f} {ri / rc:8.2f} {worst:5.2f}" + ("" if ok else "  NULLA"))
    out.append("note: " + ("; ".join(notes) if notes else "nessuna fixture nulla"))
    open(os.path.join(HERE, (sys.argv[1] if len(sys.argv) > 1 else "s194-primitive") + ".out"), "w").write("\n".join(out) + "\n")
    return 0

if __name__ == "__main__":
    sys.exit(main())
