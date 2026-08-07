#!/bin/bash
# s111-ab-run.sh — A/B della leva hot-cluster (criterio s111-criterio.md,
# committato PRIMA della misura). A = pin 92909544 (stash phpr-s109),
# B = build leva. ABAB interleaved a livello di run, R per lato, user CPU
# netto del pavimento PER-binario, mediane e spread pubblicati, N emesso
# dal sorgente. Smoke: lanciare con R=2 e guardare il segno (early-stop a
# segno opposto sui bersagli); misura piena R=5.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
MICRO="$H/../wp97-harness/micro"
A="${A:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s109}"
B="${B:-$HOME/Claude/php-rust-output/release/phpr}"
R="${R:-5}"
CATS="arith prop arr calls"

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }
minmax() { printf '%s\n' "$@" | sort -n | awk 'NR==1{lo=$1} {hi=$1} END{printf "%s %s", lo, hi}'; }

echo "s111-ab: leva hot-cluster vs pin — ABAB, R=$R per lato"
echo "grade=VERDICT"
echo "formato=ascii-nudo"
echo "A_pin=$(shasum -a 256 "$A" | cut -c1-16)"
echo "B_leva=$(shasum -a 256 "$B" | cut -c1-16)"
echo "criterio=wp111-harness/s111-criterio.md (soglia per bersaglio: max(4 ns/iter; rumore ABAB; banda-layout 0,67 ns/iter); tetto x1,48 su arith)"

# pavimenti per-binario, interleaved
fa=(); fb=()
for i in $(seq 1 "$R"); do
  fa+=("$(user_cpu "$A" "$MICRO/empty.php")")
  fb+=("$(user_cpu "$B" "$MICRO/empty.php")")
done
FA=$(median "${fa[@]}"); FB=$(median "${fb[@]}")
echo "pavimento_A_s=$FA"
echo "pavimento_B_s=$FB"
echo

for c in $CATS; do
  ta=(); tb=()
  for i in $(seq 1 "$R"); do
    ta+=("$(user_cpu "$A" "$MICRO/$c.php")")   # A poi B: ABAB per run
    tb+=("$(user_cpu "$B" "$MICRO/$c.php")")
  done
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$MICRO/$c.php")
  echo "${c}_N_iter=${N:-nd}"
  MA=$(median "${ta[@]}"); MB=$(median "${tb[@]}")
  read -r LA HA <<<"$(minmax "${ta[@]}")"
  read -r LB HB <<<"$(minmax "${tb[@]}")"
  python3 - "$c" "$MA" "$MB" "$FA" "$FB" "$LA" "$HA" "$LB" "$HB" "${N:-0}" <<'PY'
import sys
c = sys.argv[1]
ma, mb, fa, fb, la, ha, lb, hb = map(float, sys.argv[2:10])
n = int(sys.argv[10])
na, nb = ma - fa, mb - fb
d = nb - na
print(f"{c}_A_netto_s={na:.2f}  {c}_B_netto_s={nb:.2f}")
print(f"{c}_delta_s={d:+.2f}  [B-A: negativo = leva piu' veloce]")
if n:
    print(f"{c}_delta_ns_per_iter={d*1e9/n:+.2f}  (soglia pavimento 4 ns/iter = {4*n/1e9:.2f} s)")
print(f"{c}_spread_A_s={ha-la:.2f}  {c}_spread_B_s={hb-lb:.2f}")
print(f"{c}_rapporto_B_su_A={nb/na:.3f}" if na > 0 else f"{c}_rapporto_B_su_A=nd")
PY
  echo
done
