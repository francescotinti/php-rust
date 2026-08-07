#!/bin/bash
# s110-l1i-run.sh — raccolta bilaterale delle quote top-down (criterio v2 9ff53cf).
# Per lato (oracle|phpr) x giudice (arith|prop|arr) x R=3: una registrazione
# CPU Counters (template di serie), export MetricTable, mediana per nome-metrica
# sulle finestre ~1ms del processo bersaglio. Giudici = copie interne
# byte-identiche (~/Claude/l1i-micro, sha 86e4d89f per arith). Esito in
# l1i-out/coll-verdetto.out; i trace restano fuori dal repo (.gitignore).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp110-harness"
OUT="$H/l1i-out/coll"; mkdir -p "$OUT"
# EMENDAMENTO APPARATO S-110 (il criterio v2 non cambia): lo scratch di xctrace
# (NSTemporaryDirectory) ha riempito il volume interno (15G di .ktrace, ENOSPC,
# rc=134). Quattro guardie: TMPDIR sul volume esterno; cap 60s per recording;
# fail-closed se Data <5G; pulizia scratch dopo OGNI record.
SCRATCH_T="$H/l1i-out/tmp"; mkdir -p "$SCRATCH_T"; export TMPDIR="$SCRATCH_T"
DARWIN_T="$(getconf DARWIN_USER_TEMP_DIR 2>/dev/null || echo /private/var/folders/n2/3ynvhhr1275c7p2j80kc1q5r0000gn/T)"
guard_disk() {
  free_g=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
  if [ "$free_g" -lt 5 ]; then
    echo "rc=99 disco Data a ${free_g}G (<5G): FAIL-CLOSED" > "$H/l1i-out/coll.done"
    exit 99
  fi
}
L="$HOME/Claude/l1i-micro"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
R=3
rm -f "$H/l1i-out/coll.done"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
{
  echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "oracle=$("$ORACLE" -v | head -1)"
  echo "giudici_sha=$(shasum -a 256 "$L/arith.php" "$L/prop.php" "$L/arr.php" | cut -c1-8 | tr '\n' ' ')"
  echo "template=CPU Counters (di serie, quote top-down)"
  echo "epoch=$(date +%s)"
} > "$OUT/identity.txt"
for side in oracle phpr; do
  BIN="$ORACLE"; [ "$side" = phpr ] && BIN="$PHPR"
  for j in arith prop arr; do
    for r in $(seq 1 $R); do
      T="$OUT/$side-$j-r$r.trace"; rm -rf "$T"
      guard_disk
      step "record $side $j r$r"
      xctrace record --template 'CPU Counters' --output "$T" --no-prompt \
        --time-limit 60s \
        --target-stdout /dev/null --launch -- "$BIN" "$L/$j.php" \
        >> "$OUT/record.log" 2>&1
      rc=$?
      rm -f "$SCRATCH_T"/instruments*.ktrace "$DARWIN_T"/instruments*.ktrace 2>/dev/null
      step "record $side $j r$r rc=$rc"
      if [ $rc -ne 0 ]; then echo "rc=$rc lato=$side g=$j r=$r" > "$H/l1i-out/coll.done"; exit 1; fi
      xctrace export --input "$T" \
        --xpath '/trace-toc/run[@number="1"]/data/table[@schema="MetricTable"]' \
        > "$OUT/$side-$j-r$r.xml" 2>>"$OUT/record.log"
      rc=$?
      step "export $side $j r$r rc=$rc"
      if [ $rc -ne 0 ]; then echo "rc=$rc export lato=$side g=$j r=$r" > "$H/l1i-out/coll.done"; exit 1; fi
      rm -rf "$T"
    done
  done
done
echo "rc=0 $(date +%T)" > "$H/l1i-out/coll.done"
