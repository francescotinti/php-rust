#!/usr/bin/env python3
# s145-riparse.py — az.rev. S-144 #1: riparse SIMMETRICO dei profili
# campionari con UN parser committato. Le famiglie restano le liste CANONICHE
# per lato (oracle = s144-profilo-oracle.sh p.2; phpr = s140-profilo.sh,
# copiate TESTUALMENTE: cambiarle romperebbe il confronto coi verdetti
# pubblicati); la NOVITÀ del parser è l'esclusione idle DENTRO il giudice,
# applicata a ENTRAMBI i lati (lista chiusa dell'emenda v2 S-144:
# __workq_kernreturn | __psynch* | kevent | __semwait). Collaudato da
# s145-riparse-golden-test.sh. INDIZIO: mai cifra di tempo.
# Uso: s145-riparse.py <label:mode:file> [...]   (mode = oracle | phpr)
import re
import sys

FAM_ORACLE = [
    ("vm_inline",  re.compile(r"execute_ex|ZEND_.*_SPEC|zend_execute")),
    ("churn_zval", re.compile(r"zval_ptr_dtor|rc_dtor|zend_assign_to_variable|zval_copy")),
    ("gc",         re.compile(r"zend_gc|gc_|collect_cycles")),
    ("alloc",      re.compile(r"_emalloc|_efree|zend_mm|malloc|\bfree\b")),
    ("map",        re.compile(r"zend_hash")),
    ("prop_dim",   re.compile(r"_property|zend_fetch_dimension|zend_std_|obj_prop")),
    ("calls",      re.compile(r"zend_call|execute_internal|zend_vm_stack|init_fcall|leave_helper")),
    ("memops",     re.compile(r"memcpy|memmove|memset|memcmp|platform_mem")),
    ("str",        re.compile(r"zend_string|smart_str|concat")),
    ("compile",    re.compile(r"zend_compile|zend_ast|zendparse|lex_scan|opcache")),
    ("refl",       re.compile(r"reflection", re.I)),
]
FAM_PHPR = [
    ("vm_inline",  lambda s: "run_loop" in s),
    ("churn_zval", lambda s: ("Zval" in s and ("clone" in s or "drop" in s.lower())) or "drop_in_place" in s),
    ("gc",         lambda s: "gc_" in s or "collect_cycles" in s or "sweep" in s or "demote" in s),
    ("alloc",      lambda s: "mi_malloc" in s or "mi_free" in s or "_mi_" in s or "malloc" in s or re.search(r"\bfree\b", s)),
    ("map",        lambda s: "PhpArray" in s or "hashbrown" in s or "RawTable" in s or "Hasher" in s or "sip" in s or "KeyIndex" in s),
    ("prop_dim",   lambda s: "prop_" in s or "field_" in s or "slot_of" in s or "resolve" in s),
    ("calls",      lambda s: "enter_callee" in s or "bind_params" in s or "recycle_frame" in s or "Frame" in s or "dispatch_instance_call" in s),
    ("memops",     lambda s: "memmove" in s or "memcmp" in s or "memcpy" in s or "memset" in s),
    ("str",        lambda s: "PhpStr" in s),
    ("compile",    lambda s: "compile" in s or "parse" in s or "lower" in s or "mago" in s),
    ("refl",       lambda s: "reflect" in s.lower()),
]
IDLE = re.compile(r"__workq_kernreturn|__psynch|kevent|__semwait")


def classify(sym, mode):
    if mode == "oracle":
        for name, rx in FAM_ORACLE:
            if rx.search(sym):
                return name
    else:
        for name, fn in FAM_PHPR:
            if fn(sym):
                return name
    return "other"


def parse(path, mode):
    tot = idle = 0
    fam = {}
    in_tos = False
    for line in open(path, errors="replace"):
        if "Sort by top of stack" in line:
            in_tos = True
            continue
        if in_tos and line.startswith("Binary Images"):
            break
        if in_tos:
            m = re.match(r"\s+(.+?)\s+\(in [^)]+\)\s+(\d+)\s*$", line)
            if m:
                sym, n = m.group(1), int(m.group(2))
                tot += n
                if IDLE.search(sym):
                    idle += n
                    continue
                k = classify(sym, mode)
                fam[k] = fam.get(k, 0) + n
    return tot, idle, fam


for arg in sys.argv[1:]:
    label, mode, path = arg.split(":", 2)
    assert mode in ("oracle", "phpr"), f"mode ignoto: {mode}"
    tot, idle, fam = parse(path, mode)
    att = tot - idle
    if att <= 0:
        print(f"{label}: NESSUN campione attivo (tot={tot} idle={idle})")
        continue
    print(f"{label} [{mode}]: tot={tot} idle={idle} attivi={att} (idle escluso NEL parser, lista chiusa v2)")
    for k, v in sorted(fam.items(), key=lambda kv: -kv[1]):
        print(f"  {label} {k}: {v} ({100*v/att:.2f}%)")
    ch, mo = fam.get("churn_zval", 0), fam.get("memops", 0)
    print(f"  {label} GIUDICE-UNICO: churn_zval={100*ch/att:.2f}% memops={100*mo/att:.2f}%")
