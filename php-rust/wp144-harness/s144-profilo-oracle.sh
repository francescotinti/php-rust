#!/bin/bash
# s144-profilo-oracle.sh — profilo campionario SUITE ORM sul PHP ORACLE
# (criterio s144-criterio-profilo-oracle.md; base DICHIARATA: s140-profilo.sh,
# adattamenti: binario oracle, famiglie Zend p.2, UNA finestra per replica
# p.4, memory_limit=-1 §3.14). INDIZIO: mai cifra di tempo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp144-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
SP="${PROF_SP:?PROF_SP (workdir APFS) richiesto}"
OUT="$H/profilo-oracle-out"; mkdir -p "$OUT"
VERD="$H/s144-profilo-oracle-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/profilo.done" "$OUT/sentinelle.txt" "$OUT/runrc.txt"
[ -e /private/tmp/phpr-measure.lock ] || { echo "rc=6 measure-lock ASSENTE" > "$OUT/profilo.done"; exit 6; }

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/profilo.done"; exit 8; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/profilo.done"; exit 7; }
  p "rep$rep START"
  "$ORACLE" -d memory_limit=-1 vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
  PID=$!
  ( sleep 300; kill -9 "$PID" 2>/dev/null ) & WDPID=$!
  sleep 0.5
  echo "rep$rep w1 pre=$(busy)" >> "$OUT/sentinelle.txt"
  sample "$PID" 60 -file "$OUT/sample-r$rep-w1.txt" > "$OUT/sample-r$rep-w1.log" 2>&1
  echo "rep$rep w1 post=$(busy)" >> "$OUT/sentinelle.txt"
  wait "$PID"; RUNRC=$?
  kill "$WDPID" 2>/dev/null; wait "$WDPID" 2>/dev/null
  p "rep$rep rc=$RUNRC"
  echo "rep$rep run_rc=$RUNRC" >> "$OUT/runrc.txt"
  tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/rep$rep.failnames"
done

/usr/bin/python3 - "$OUT" > "$OUT/famiglie.txt" <<'PY'
import re, sys, os
out = sys.argv[1]
# lista CHIUSA dal criterio p.2 — primo match vince (mappa Zend)
FAM = [
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
def classify(sym):
    for name, rx in FAM:
        if rx.search(sym): return name
    return "other"
for rep in (1, 2):
    tot = 0; fam = {}
    f = f"{out}/sample-r{rep}-w1.txt"
    if os.path.exists(f):
        in_tos = False
        for line in open(f, errors="replace"):
            if "Sort by top of stack" in line: in_tos = True; continue
            if in_tos and line.startswith("Binary Images"): break
            if in_tos:
                m = re.match(r"\s+(.+?)\s+\(in [^)]+\)\s+(\d+)\s*$", line)
                if m:
                    sym = m.group(1); n = int(m.group(2))
                    tot += n
                    k = classify(sym)
                    fam[k] = fam.get(k, 0) + n
    if not tot:
        print(f"rep{rep}: NESSUN campione"); continue
    if tot < 100:
        print(f"rep{rep}: SOLO {tot} campioni — replica NON valida (criterio p.6.iii)")
    rank = sorted(fam.items(), key=lambda kv: -kv[1])
    print(f"rep{rep} tot_campioni={tot}")
    for k, v in rank:
        print(f"  rep{rep} {k}: {v} ({100*v/tot:.1f}%)")
    B = fam.get("churn_zval", 0) + fam.get("memops", 0)
    print(f"  rep{rep} BERSAGLI-B (churn_zval+memops): {100*B/tot:.1f}%")
    print(f"  rep{rep} rank_top3: " + " > ".join(k for k, _ in rank[:3]))
PY

{ echo "== s144 profilo SUITE ORM ORACLE (INDIZIO per famiglie — criterio s144-criterio-profilo-oracle.md; MAI cifra di tempo) =="
  echo "grade=INDIZIO  # sample perturba; asimmetria di inlining DICHIARATA (criterio p.3)"
  echo "oracle=$("$ORACLE" -v 2>/dev/null | head -1)"
  cat "$OUT/runrc.txt"
  echo "fail-names oracle: prima fotografia (osservativo, criterio p.5): rep1=$(wc -l < "$OUT/rep1.failnames" | tr -d ' ') rep2=$(wc -l < "$OUT/rep2.failnames" | tr -d ' ') nomi"
  echo "--- sentinelle per finestra (criterio p.5) ---"
  cat "$OUT/sentinelle.txt"
  echo "--- famiglie oracle (top-of-stack cumulata, lista chiusa p.2) ---"
  cat "$OUT/famiglie.txt"
  echo "--- riferimento phpr (s140-profilo-verdetto.out, stessa lente) ---"
  sed -n '/famiglie/,$p' "$REPO/wp140-harness/s140-profilo-verdetto.out" | grep -E "rep[12] (tot|[a-z_]+:)|MACRO" | head -30
  for rep in 1 2; do
    echo "summary rep$rep: $(tr -d '\0' < "$OUT/run$rep.txt" | grep -E '^(Tests:|OK)' | tail -1)"
  done
} > "$VERD" 2>&1
echo "rc=0 $(date +%T)" > "$OUT/profilo.done"
p "DONE"
