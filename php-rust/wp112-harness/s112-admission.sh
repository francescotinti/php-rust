#!/bin/bash
# s112-admission.sh — criterio H-A2 punto 7 (PRIMA dello smoke).
# La leva NON tocca il pass: (a) dump ON {main} ×6 IDENTICI al pin;
# (b) OFF al byte (dump interi) ×6; (c) taglia run_loop (nm) + bl-count
# (otool, simbolo esatto) su entrambi i lati, a verbale.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
B="$HOME/Claude/php-rust-output/release/phpr"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s109"
OUT="$H/admission-out"; mkdir -p "$OUT"
CATS="arith prop calls str arr re"
RC=0

main_only() { awk '/^-- \{main\} /{f=1;next} /^-- /{f=0} f' "$1"; }

echo "candidato=$(shasum -a 256 "$B" | cut -c1-16)  stash_A=$(shasum -a 256 "$A" | cut -c1-16)"
echo "== (a) dump ON, sezione {main}: SEI IDENTICI al pin (la leva non tocca il pass) =="
for c in $CATS; do
  PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/cand-on-$c.dump" > /dev/null
  PHPR_DUMP_OPS=1 "$A" "$M/$c.php" 2> "$OUT/pin-on-$c.dump" > /dev/null
  main_only "$OUT/cand-on-$c.dump" > "$OUT/cand-on-$c.main"
  main_only "$OUT/pin-on-$c.dump" > "$OUT/pin-on-$c.main"
  if cmp -s "$OUT/cand-on-$c.main" "$OUT/pin-on-$c.main"; then
    echo "   $c: {main} ON IDENTICO al pin"
  else
    echo "   $c: {main} ON DIVERGE (VIOLAZIONE: H-A2 non deve toccare l'emissione)"; RC=1
  fi
done
echo "== (b) OFF al byte (dump interi): candidato vs stash =="
for c in $CATS; do
  PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/off-cand-$c.dump" > /dev/null
  PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$A" "$M/$c.php" 2> "$OUT/off-stash-$c.dump" > /dev/null
  if cmp -s "$OUT/off-cand-$c.dump" "$OUT/off-stash-$c.dump"; then
    echo "   $c: OFF IDENTICO al byte"
  else
    echo "   $c: OFF DIVERGE (VIOLAZIONE ADMISSION)"; RC=1
  fi
done
echo "== (c) run_loop: taglia (nm) e bl-count (otool, simbolo esatto; metodo S-109) =="
for side in B A; do
  bin=$([ "$side" = B ] && echo "$B" || echo "$A")
  echo "   lato $side:"
  nm -n "$bin" 2>/dev/null | grep -A1 "8run_loop17h" | head -2 | python3 -c '
import sys
lines = sys.stdin.read().split("\n")
a = int(lines[0].split()[0], 16); b = int(lines[1].split()[0], 16)
print(f"   run_loop_size={b - a} B")'
  sym=$(nm -n "$bin" 2>/dev/null | grep "8run_loop17h" | awk '{print $3}')
  otool -tv "$bin" 2>/dev/null | awk -v s="$sym:" '
    $0 == s {f=1; bl=0; br=0; next}
    f && /^__/ {print "   bl=" bl " br=" br; exit}
    f && $2 == "bl" {bl++}
    f && $2 == "br" {br++}
  '
done
echo "admission_rc=$RC"
exit "$RC"
