#!/bin/bash
# s142-census-rd1.sh — census CONTEGGI L-RD1 (rd1_arrays/rd1_elems/rd1_tombs) su
# SUITE ORM, monobinario census (sorgenti pin s142 + contatori cfg-gated post-pin
# verificati hash-A′ + feature zval-census). Criterio: s142-criterio-census-rd1.md
# (pre-registrato). MAI cifra di tempo da qui.
# Base DICHIARATA: wp141-harness/s141-census.sh (adattamenti: chiavi rd1_*,
# quota python dal criterio p.5-6, path wp142).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp142-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s142-census-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/census.done"

# criterio p.4: smoke — il probe deve emettere i contatori NUOVI rd1_*
printf '<?php $a=[1,"x",2.5]; unset($a); $h=["k"=>1]; unset($h["k"]); unset($h); echo "ok\n";' > "$SP/smoke142.php"
rm -f "$OUT/smoke-census.txt"
PHPR_ZVAL_CENSUS="$OUT/smoke-census.txt" "$CENSUS_PHPR" "$SP/smoke142.php" > "$SP/smoke142.out" 2>&1
grep -q "rd1_arrays" "$OUT/smoke-census.txt" || { echo "rc=8 probe MUTO sui contatori rd1_*" > "$OUT/census.done"; exit 8; }
p "smoke PASS (dump rd1_* vivo)"

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
KEYS = ["rd1_arrays","rd1_elems","rd1_tombs"]
T_SUITE_NS = 42.5e9   # criterio p.5: T_suite phpr S-140
PREZZO_ELEM = (0.5, 1.0)   # ns/elem = lato-basso modello S-141 (~0,8), cross-giudice DICHIARATO
PREZZO_ARR  = (2.0, 5.0)   # ns/array = costo fisso call-glue risparmiata
per_rep = {}
for f in sorted(glob.glob(f"{out}/census-r*.txt")):
    rep = re.search(r"census-(r\d)", f).group(1)
    tot = dict.fromkeys(KEYS, 0)
    for line in open(f, errors="replace"):
        if not line.startswith("zvalcensus_s142 "): continue
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
    lo = 100*(t["rd1_elems"]*PREZZO_ELEM[0] + t["rd1_arrays"]*PREZZO_ARR[0])/T_SUITE_NS
    hi = 100*(t["rd1_elems"]*PREZZO_ELEM[1] + t["rd1_arrays"]*PREZZO_ARR[1])/T_SUITE_NS
    print(f"quota_L-RD1 (INDIZIO, criterio p.5): {lo:.2f}%-{hi:.2f}% della suite ORM")
    if hi < 0.3:
        print("LETTURA p.6: quota <0,3% -> nel dossier p.3: guadagno ORM sotto risoluzione della coppia suite")
    else:
        print("LETTURA p.6: quota >=0,3% -> citata come canale residuo nel dossier")
    print("NESSUN GATE: la leva e' gia' promossa dalla catena; la quota si SCRIVE nel verbale di spedizione")
PY

{ echo "== s142 census L-RD1 su SUITE ORM (criterio s142-criterio-census-rd1.md — CONTEGGI, mai tempo) =="
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
