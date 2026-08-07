#!/bin/bash
# s109-admission.sh v2 — criterio lotto-3 punto 6: PRIMA dello smoke.
# EMENDAMENTO DICHIARATO (v1 mis-specificata, s109-admission-v1.out): il dump
# PHPR_DUMP_OPS copre TUTTI i corpi del funnel (prelude compreso) e F1/F2
# fondono legittimamente NEL PRELUDE di ogni giudice — il controllo
# «cinque IDENTICI» si RI-COLLOCA sul solo {main} (il corpo del giudice),
# come il conto op/iter del census. str: {main} contiene ConcatNConst e
# NESSUN [PushConst,Unary(Neg)] residuo (il Neg nudo -$x resta legale).
# (b) OFF al byte INVARIATO (dump interi). (c) taglia run_loop da nm
# (differenza indirizzi) + bl-count otool sul simbolo esatto.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
B="$HOME/Claude/php-rust-output/release/phpr"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s108"
OUT="$H/census-out"; mkdir -p "$OUT"
CATS="arith prop calls str arr re"
RC=0

main_only() { awk '/^-- \{main\} /{f=1;next} /^-- /{f=0} f' "$1"; }

echo "candidato=$(shasum -a 256 "$B" | cut -c1-16)  stash_A=$(shasum -a 256 "$A" | cut -c1-16)"
echo "== (a) dump ON, sezione {main}: str = F1+F2, gli altri cinque IDENTICI al pin =="
for c in $CATS; do
  PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/post3-$c.dump" > /dev/null
  PHPR_DUMP_OPS=1 "$A" "$M/$c.php" 2> "$OUT/pin-on-$c.dump" > /dev/null
  main_only "$OUT/post3-$c.dump" > "$OUT/post3-$c.main"
  main_only "$OUT/pin-on-$c.dump" > "$OUT/pin-on-$c.main"
  if [ "$c" = str ]; then
    n_ccc=$(grep -c "ConcatNConst" "$OUT/post3-$c.main")
    n_neg=$(grep -c "Unary(Neg)" "$OUT/post3-$c.main")
    echo "   str{main}: ConcatNConst=$n_ccc (atteso >0), Unary(Neg)=$n_neg (atteso 0)"
    [ "$n_ccc" -gt 0 ] && [ "$n_neg" -eq 0 ] || { echo "   str: ADMISSION VIOLATA"; RC=1; }
  else
    if cmp -s "$OUT/post3-$c.main" "$OUT/pin-on-$c.main"; then
      echo "   $c: {main} ON IDENTICO al pin"
    else
      echo "   $c: {main} ON DIVERGE dal pin (VIOLAZIONE: lotto-3 tocca solo str)"; RC=1
    fi
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
echo "== (c) run_loop: taglia (nm) e bl-count (otool, simbolo esatto) =="
for side in B A; do
  bin=$([ "$side" = B ] && echo "$B" || echo "$A")
  nm -n "$bin" 2>/dev/null | grep -A1 "8run_loop17h" | head -2 | python3 -c '
import sys
lines = sys.stdin.read().split("\n")
a = int(lines[0].split()[0], 16); b = int(lines[1].split()[0], 16)
print(f"   run_loop_size={b - a} B")'
  sym=$(nm -n "$bin" 2>/dev/null | grep "8run_loop17h" | awk '{print $3}')
  otool -tv "$bin" 2>/dev/null | awk -v s="$sym:" '
    $0 == s {f=1; bl=0; next}
    f && /^__/ {print "   bl=" bl; exit}
    f && $2 == "bl" {bl++}
  '
done
echo "admission_rc=$RC"
exit "$RC"
