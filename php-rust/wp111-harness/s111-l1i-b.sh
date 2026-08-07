#!/bin/bash
# s111-l1i-b.sh — contro-lettura delivery post-leva (criterio s111 §9):
# SOLO lato phpr = binario B (leva hot-cluster), giudici arith + arr
# (controllo), R=3, stesso template/metodo/export di s110-l1i-run.sh
# (le 4 guardie disco EMENDAMENTO S-110 incluse). Confronto: XML S-110
# del pin in wp110-harness/l1i-out/coll, parse identico.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp111-harness"
OUT="$H/l1i-b"; mkdir -p "$OUT"
SCRATCH_T="$OUT/tmp"; mkdir -p "$SCRATCH_T"; export TMPDIR="$SCRATCH_T"
DARWIN_T="$(getconf DARWIN_USER_TEMP_DIR 2>/dev/null || echo /tmp)"
guard_disk() {
  free_g=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
  if [ "$free_g" -lt 5 ]; then
    echo "rc=99 disco Data a ${free_g}G (<5G): FAIL-CLOSED" > "$OUT/coll.done"
    exit 99
  fi
}
L="$HOME/Claude/l1i-micro"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
R=3
rm -f "$OUT/coll.done"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
{
  echo "phpr_B=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "giudici_sha=$(shasum -a 256 "$L/arith.php" "$L/arr.php" | cut -c1-8 | tr '\n' ' ')"
  echo "template=CPU Counters (di serie, quote top-down)"
  echo "epoch=$(date +%s)"
} > "$OUT/identity.txt"
for j in arith arr; do
  for r in $(seq 1 $R); do
    T="$OUT/phpr-$j-r$r.trace"; rm -rf "$T"
    guard_disk
    step "record phpr $j r$r"
    xctrace record --template 'CPU Counters' --output "$T" --no-prompt \
      --time-limit 60s \
      --target-stdout /dev/null --launch -- "$PHPR" "$L/$j.php" \
      >> "$OUT/record.log" 2>&1
    rc=$?
    rm -f "$SCRATCH_T"/instruments*.ktrace "$DARWIN_T"/instruments*.ktrace 2>/dev/null
    step "record phpr $j r$r rc=$rc"
    if [ $rc -ne 0 ]; then echo "rc=$rc g=$j r=$r" > "$OUT/coll.done"; exit 1; fi
    xctrace export --input "$T" \
      --xpath '/trace-toc/run[@number="1"]/data/table[@schema="MetricTable"]' \
      > "$OUT/phpr-$j-r$r.xml" 2>>"$OUT/record.log"
    rc=$?
    step "export phpr $j r$r rc=$rc"
    if [ $rc -ne 0 ]; then echo "rc=$rc export g=$j r=$r" > "$OUT/coll.done"; exit 1; fi
    rm -rf "$T"
  done
done
echo "rc=0 $(date +%T)" > "$OUT/coll.done"
