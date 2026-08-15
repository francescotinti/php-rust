#!/bin/bash
# s144-census-tranche2.sh — census tranche-2 (rczval/vecargs → quota_obj_max)
# su SUITE ORM, monobinario census. Criterio: s144-criterio-tranche2.md
# (regola p.5 PRE-REGISTRATA). MAI cifra di tempo da qui.
# Base DICHIARATA: wp143-harness/s143-census-ch.sh (adattamenti: smoke sui
# contatori s144.* con esito ESATTO ≥1 per chiave; parser ESTERNO
# s144-census-parse.py collaudato dal golden-test; path wp144).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp144-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s144-census-tranche2.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/census.done" "$OUT/sentinelle.txt" "$OUT/runrc.txt"

# criterio p.6.i: smoke — ogni chiave s144.* attesa DEVE valere ≥1 nel dump
# (foreach by-ref su oggetto ⇒ rczval_prop; &$a[0] ⇒ rczval; variadica via
# call_user_func ⇒ vecargs). Esito ESATTO per chiave, mai «file non vuoto».
cat > "$SP/smoke144.php" <<'PHP'
<?php
class C { public $x=1; public $y=2; }
$o = new C;
foreach ($o as &$v) { $v = $v + 1; }
unset($v);
$a = [1,2];
$r = &$a[0];
function g(...$xs) { return count($xs); }
call_user_func('g', 1, 2);
echo "ok\n";
PHP
rm -f "$OUT/smoke-mem.txt"
PHPR_MEM_CENSUS="$OUT/smoke-mem.txt" "$CENSUS_PHPR" "$SP/smoke144.php" > "$SP/smoke144.out" 2>&1
SMOKE_FAIL=""
for k in s144.rczval_n s144.rczval_prop_n s144.vecargs_n; do
  v=$(sed -n 's/.* tag=exit .*'"$k"'=\([0-9][0-9]*\).*/\1/p' "$OUT/smoke-mem.txt" | head -1)
  [ -n "${v:-}" ] && [ "$v" -ge 1 ] || SMOKE_FAIL="$SMOKE_FAIL $k=${v:-ASSENTE}"
done
if [ -n "$SMOKE_FAIL" ]; then
  echo "rc=8 probe MUTO/zero su:$SMOKE_FAIL" > "$OUT/census.done"; exit 8
fi
p "smoke PASS (s144.* vivi:$(sed -n 's/.*tag=exit \(.*\)$/\1/p' "$OUT/smoke-mem.txt" | head -1 | tr ' ' '\n' | grep 's144' | tr '\n' ' '))"

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" > "$OUT/census.done"; exit 7; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/census.done"; exit 7; }
  RAW="$OUT/census-mem-r$rep-$(date +%Y%m%d).txt"; rm -f "$RAW"
  RAWZ="$OUT/census-zval-r$rep-$(date +%Y%m%d).txt"; rm -f "$RAWZ"
  echo "rep$rep pre=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep START"
  PHPR_MEM_CENSUS="$RAW" PHPR_ZVAL_CENSUS="$RAWZ" "$CENSUS_PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
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

/usr/bin/python3 "$H/s144-census-parse.py" "$OUT" > "$OUT/quota.txt"

{ echo "== s144 census TRANCHE-2 (rczval/vecargs => quota_obj_max) su SUITE ORM (criterio s144-criterio-tranche2.md — CONTEGGI, mai tempo) =="
  echo "grade=CENSUS  # monobinario census, quote = eventi/galloc_n stessa run (denominatore dal sorgente)"
  echo "census_phpr=$(shasum -a 256 "$CENSUS_PHPR" | cut -c1-16)  # probe build, MAI parita'"
  echo "parser=v3 s144-census-parse.py GOLDEN-TESTED (case1+case2 PASS, tag ESATTO)"
  echo "CONVENZIONE CONTEGGI (az.rev. S-143 #3): realloc-eventi FUORI dal denominatore per costruzione (A-LE-104-1), stampati come s144.grealloc_n; banda dyn_entries su quota_obj: +0/-0,69pp"
  cat "$OUT/runrc.txt"
  echo "parita' per NOME vs baseline16: rc=$RC (0=entrambe ==)"
  echo "--- sentinelle (non-gate, criterio p.1) ---"
  cat "$OUT/sentinelle.txt"
  echo "--- contatori e quote (parser v3) ---"
  cat "$OUT/quota.txt"
  for rep in 1 2; do
    echo "summary rep$rep: $(tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  done
} > "$VERD" 2>&1
echo "rc=$RC $(date +%T)" > "$OUT/census.done"
p "DONE rc=$RC"
