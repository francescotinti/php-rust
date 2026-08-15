#!/bin/bash
# s141-census.sh — census CONTEGGI TakeSlot (would_take_safe*) su SUITE ORM,
# monobinario census (sorgenti pin s140 + feature zval-census). Criterio:
# s141-criterio-census.md (pre-registrato). MAI cifra di tempo da qui.
# Base DICHIARATA: wp140-harness/s140-profilo.sh (adattamenti: niente sample,
# env census per replica, quota python dal criterio p.5-6).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp141-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s141-census-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/census.done"

# criterio p.3: smoke — il probe non deve essere muto
printf '<?php $a="x"; $b=$a; echo strlen($b), "\n";' > "$SP/smoke141.php"
rm -f "$OUT/smoke-census.txt"
PHPR_ZVAL_CENSUS="$OUT/smoke-census.txt" "$CENSUS_PHPR" "$SP/smoke141.php" > "$SP/smoke141.out" 2>&1
grep -q "would_take_safe" "$OUT/smoke-census.txt" || { echo "rc=8 probe MUTO" > "$OUT/census.done"; exit 8; }
p "smoke PASS (dump vivo)"

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" > "$OUT/census.done"; exit 7; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/census.done"; exit 7; }
  RAW="$OUT/census-r$rep-$(date +%Y%m%d).txt"; rm -f "$RAW"
  echo "rep$rep pre=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep START"
  PHPR_ZVAL_CENSUS="$RAW" "$CENSUS_PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
  PID=$!
  ( sleep 1800; kill -9 "$PID" 2>/dev/null ) & WDPID=$!
  wait "$PID"; RUNRC=$?
  kill "$WDPID" 2>/dev/null; wait "$WDPID" 2>/dev/null
  echo "rep$rep post=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep rc=$RUNRC"
  echo "rep$rep run_rc=$RUNRC raw=$(basename "$RAW")" >> "$OUT/runrc.txt"
  tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/rep$rep.failnames"
done

RC=0
for rep in 1 2; do
  diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/rep$rep.failnames" > /dev/null || { p "rep$rep parita' DIVERGE"; RC=1; }
done

python3 - "$OUT" > "$OUT/quota.txt" <<'PY'
import glob, re, sys
out = sys.argv[1]
KEYS = ["slot_reads","slot_reads_rc","would_take","would_take_rc",
        "would_take_safe","would_take_safe_rc","would_take_safe_str",
        "would_take_safe_ref","sites_total","sites_movable","sites_safe"]
T_SUITE_NS = 42.5e9   # criterio p.5: T_suite phpr S-140
PREZZO = (1.1, 1.7)   # ns/evento clone+drop Rc (HC1, cross-giudice DICHIARATO)
per_rep = {}
for f in sorted(glob.glob(f"{out}/census-r*.txt")):
    rep = re.search(r"census-(r\d)", f).group(1)
    tot = dict.fromkeys(KEYS, 0)
    for line in open(f, errors="replace"):
        if not line.startswith("zvalcensus "): continue
        for k, v in re.findall(r"(\w+)=(\d+)", line):
            if k in tot: tot[k] += int(v)
    per_rep[rep] = tot
for rep, tot in per_rep.items():
    print(f"{rep}: " + " ".join(f"{k}={tot[k]}" for k in KEYS))
reps = list(per_rep.values())
if len(reps) == 2:
    for k in KEYS:
        a, b = reps[0][k], reps[1][k]
        m = max(a, b)
        if m and abs(a-b)/m > 0.01:
            print(f"DICHIARA scarto>1% su {k}: r1={a} r2={b}")
if reps:
    t = reps[0]
    for k in ("would_take_safe","would_take_safe_rc","would_take_safe_str","would_take_safe_ref"):
        lo = 100*t[k]*PREZZO[0]/T_SUITE_NS; hi = 100*t[k]*PREZZO[1]/T_SUITE_NS
        print(f"quota_{k} (INDIZIO): {lo:.2f}%-{hi:.2f}% della suite")
    wtr = t["would_take_rc"]
    p2 = 100*t["would_take_safe_rc"]/wtr if wtr else 0.0
    print(f"P2 storica: safe_rc/would_take_rc = {p2:.1f}% (soglia 60)")
    qlo = 100*t["would_take_safe_str"]*PREZZO[0]/T_SUITE_NS
    qhi = 100*t["would_take_safe_str"]*PREZZO[1]/T_SUITE_NS
    if qlo >= 2.0: dec = "DECISIONE p.6: quota_str >=2% -> ISTRUTTORIA OPCODE (F3/F4; divergenze S-96 RILETTE PRIMA)"
    elif qhi < 0.5: dec = "DECISIONE p.6: quota_str <0,5% -> TakeSlot DELUDE sul suo meccanismo -> filone alternativo (Repr-drop 11% / frame 9%)"
    else: dec = "DECISIONE p.6: zona 0,5-2% (o cavallo di soglia) -> confronto con Repr-drop 11% / frame 9%, si sceglie il canale maggiore"
    print(dec)
PY

{ echo "== s141 census TakeSlot su SUITE ORM (criterio s141-criterio-census.md — CONTEGGI, mai tempo) =="
  echo "grade=CENSUS  # monobinario census, quote = INDIZIO (prezzo cross-giudice dichiarato)"
  echo "census_phpr=$(shasum -a 256 "$CENSUS_PHPR" | cut -c1-16)  # probe build, MAI parita'"
  cat "$OUT/runrc.txt"
  echo "parita' per NOME vs baseline16: rc=$RC (0=entrambe ==)"
  echo "--- sentinelle (non-gate, criterio p.7) ---"
  cat "$OUT/sentinelle.txt"
  echo "--- contatori e quote ---"
  cat "$OUT/quota.txt"
  for rep in 1 2; do
    echo "summary rep$rep: $(tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  done
} > "$VERD" 2>&1
echo "rc=$RC $(date +%T)" > "$OUT/census.done"
p "DONE rc=$RC"
