#!/bin/bash
# s140-profilo.sh — profilo SUITE ORM monobinario phpr @ pin s138 (criterio
# s140-criterio-profilo.md; base DICHIARATA: s126-orm-profile.sh, adattamenti:
# 2 repliche, aggregazione per FAMIGLIE, gate parità per NOME, sentinelle
# rustc per-finestra — az.rev. S-139 #2/#3). INDIZIO: mai cifra di tempo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp140-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
SP="${PROF_SP:?PROF_SP (workdir APFS) richiesto}"
OUT="$H/profilo-out"; mkdir -p "$OUT"
VERD="$H/s140-profilo-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/profilo.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "fa17dabd9eaa4bcb" ] || { echo "rc=9 pin!=s138" > "$OUT/profilo.done"; exit 9; }
# lock della SESSIONE: verify-only (criterio p.7)
[ -e /private/tmp/phpr-measure.lock ] || { echo "rc=6 measure-lock ASSENTE" > "$OUT/profilo.done"; exit 6; }

busy(){ pgrep -x rustc > /dev/null 2>&1 || pgrep -x cargo > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/profilo.done"; exit 8; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/profilo.done"; exit 7; }
  p "rep$rep START"
  "$PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
  PID=$!
  ( sleep 300; kill -9 "$PID" 2>/dev/null ) & WDPID=$!
  sleep 4
  for w in 1 2; do
    echo "rep$rep w$w pre=$(busy)" >> "$OUT/sentinelle.txt"
    sample "$PID" 18 -file "$OUT/sample-r$rep-w$w.txt" > "$OUT/sample-r$rep-w$w.log" 2>&1
    echo "rep$rep w$w post=$(busy)" >> "$OUT/sentinelle.txt"
  done
  wait "$PID"; RUNRC=$?
  kill "$WDPID" 2>/dev/null; wait "$WDPID" 2>/dev/null
  p "rep$rep rc=$RUNRC"
  echo "rep$rep run_rc=$RUNRC" >> "$OUT/runrc.txt"
  tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/rep$rep.failnames"
done

RC=0
for rep in 1 2; do
  diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/rep$rep.failnames" > /dev/null || { p "rep$rep parita' DIVERGE"; RC=1; }
done

python3 - "$OUT" > "$OUT/famiglie.txt" <<'PY'
import re, sys, os
out = sys.argv[1]
# lista CHIUSA dal criterio p.2 — primo match vince
FAM = [
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
def classify(sym):
    for name, fn in FAM:
        if fn(sym): return name
    return "other"
per_rep = {}
for rep in (1, 2):
    tot = 0; fam = {}
    for w in (1, 2):
        f = f"{out}/sample-r{rep}-w{w}.txt"
        if not os.path.exists(f): continue
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
    per_rep[rep] = (tot, fam)
for rep in (1, 2):
    tot, fam = per_rep[rep]
    if not tot: print(f"rep{rep}: NESSUN campione"); continue
    rank = sorted(fam.items(), key=lambda kv: -kv[1])
    print(f"rep{rep} tot_campioni={tot}")
    for k, v in rank:
        print(f"  rep{rep} {k}: {v} ({100*v/tot:.1f}%)")
    CH = sum(fam.get(k, 0) for k in ("churn_zval", "gc", "alloc", "map"))
    DP = fam.get("prop_dim", 0)
    VI = fam.get("vm_inline", 0)
    print(f"  rep{rep} MACRO: CHURN={100*CH/tot:.1f}% DIMPROP={100*DP/tot:.1f}% (vm_inline={100*VI/tot:.1f}% FUORI, memops={100*fam.get('memops',0)/tot:.1f}% FUORI)")
    print(f"  rep{rep} rank_top3: " + " > ".join(k for k, _ in rank[:3]))
PY

{ echo "== s140 profilo SUITE ORM phpr (INDIZIO per famiglie — criterio s140-criterio-profilo.md; MAI cifra di tempo) =="
  echo "grade=INDIZIO  # sample perturba"
  echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  cat "$OUT/runrc.txt"
  echo "parita' per NOME vs baseline16: rc=$RC (0=entrambe ==)"
  echo "--- sentinelle rustc/cargo per finestra (criterio p.6) ---"
  cat "$OUT/sentinelle.txt"
  echo "--- famiglie (top-of-stack cumulata, lista chiusa p.2) ---"
  cat "$OUT/famiglie.txt"
  for rep in 1 2; do
    echo "summary rep$rep: $(tr -d '\0' < "$OUT/run$rep.txt" | grep -E '^(Tests:|OK)' | tail -1)"
  done
} > "$VERD" 2>&1
echo "rc=$RC $(date +%T)" > "$OUT/profilo.done"
p "DONE rc=$RC"
