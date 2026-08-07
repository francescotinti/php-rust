#!/bin/bash
# s108-admission.sh — criterio lotto-2 punto 5: PRIMA dello smoke.
# (a) dump PHPR_DUMP_OPS dei sei giudici sul CANDIDATO: emissione attesa
#     (conteggio op del corpo per giudice + presenza per NOME delle op nuove);
# (b) modo OFF invariato AL BYTE: dump off candidato == dump off stash s107b;
# (c) taglia run_loop + bl-count a verbale (dichiarazione, non soglia).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
B="$HOME/Claude/php-rust-output/release/phpr"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s107b"
OUT="$H/census-out"; mkdir -p "$OUT"
CATS="arith prop calls str arr re"

echo "candidato=$(shasum -a 256 "$B" | cut -c1-16)  stash_A=$(shasum -a 256 "$A" | cut -c1-16)"
echo "== (a) dump ON del candidato =="
for c in $CATS; do
  PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/post2-$c.dump" > /dev/null
  echo "-- $c: op nuove nel dump --"
  grep -c "PropGetSlotRecv\|BinaryTCPropSetPop\|BinarySCSCDst\|LoadVarPushConst" "$OUT/post2-$c.dump" | sed 's/^/   siti_nuovi=/'
done
echo "== (b) OFF al byte: candidato vs stash =="
OFFRC=0
for c in $CATS; do
  PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/off-cand-$c.dump" > /dev/null
  PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$A" "$M/$c.php" 2> "$OUT/off-stash-$c.dump" > /dev/null
  if cmp -s "$OUT/off-cand-$c.dump" "$OUT/off-stash-$c.dump"; then
    echo "   $c: OFF IDENTICO al byte"
  else
    echo "   $c: OFF DIVERGE (VIOLAZIONE ADMISSION)"; OFFRC=1
  fi
done
echo "== (c) run_loop: taglia e bl-count (candidato vs stash) =="
for side in B A; do
  bin=$([ "$side" = B ] && echo "$B" || echo "$A")
  otool -tv "$bin" 2>/dev/null | awk '
    /^_.*run_loop.*:$/ {inloop=1; n=0; bl=0; next}
    inloop && /^_/ {print "   '"$side"' run_loop_instr=" n " bl=" bl; inloop=0}
    inloop {n++; if ($2=="bl") bl++}
  ' | head -2
done
echo "admission_off_rc=$OFFRC"
