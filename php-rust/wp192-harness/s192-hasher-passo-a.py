#!/usr/bin/env python3
"""s192-hasher-passo-a.py — S-192 ter, step (a) of the hasher lever (criterion s192-criterio-hasher.md p.2).

Mechanical edit, run ONCE on the tree: in lower/ and compile/ the `use std::collections::{HashMap, HashSet};` import
becomes the FxHash alias, and every UNQUALIFIED `HashMap::new()` / `HashSet::new()` / `::with_capacity(n)` in those two
directories follows (`::default()`, `::with_capacity_and_hasher(n, Default::default())`). Paths written out in full as
`std::collections::…` are left alone on purpose (module boundaries). Prints what it changed.
Generato con assistenza AI (Claude Fable 5.2).
"""
import pathlib, re

R = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/crates/php-runtime/src")
OLD = "use std::collections::{HashMap, HashSet};"
NEW = "use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet}; // S-192: FxHash for engine maps (RULEBOOK, internal hashing)"
total = 0
for d in ("lower", "compile"):
    for f in sorted((R / d).glob("*.rs")):
        t = f.read_text()
        n_imp = t.count(OLD)
        u = t.replace(OLD, NEW)
        u, n_new = re.subn(r"(?<![:\w])(HashMap|HashSet)::new\(\)", r"\1::default()", u)
        u, n_cap = re.subn(r"(?<![:\w])(HashMap|HashSet)::with_capacity\(([^()]*(?:\([^()]*\))?[^()]*)\)",
                           r"\1::with_capacity_and_hasher(\2, Default::default())", u)
        if u != t:
            f.write_text(u)
            total += 1
            print(f"{d}/{f.name}: import {n_imp} · new→default {n_new} · with_capacity {n_cap}")
print("files changed:", total)
