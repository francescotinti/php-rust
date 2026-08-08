#!/bin/bash
# s115-heldout-la.sh — criterio s115-criterio-la.md p.7 (eredita S-114 p.5).
# R=5 su pin (stash phpr-s112) E candidato (phpr-s114-la) sui tre giudici
# held-out; net = user CPU mediano - pavimento per-binario. REGRESSIONE se
# net_candidato > baseline S-112 + max(2*spread_candidato; 0,12 s).
# Verdetto A MACCHINA; rc 0 = nessuna regressione, 1 = regressione.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
HD="$H/../wp111-harness/heldout"
EMPTY="$H/../wp97-harness/micro/empty.php"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-la"
R=5
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
echo "A(pin)=$(shasum -a 256 "$A"|cut -c1-8) B(cand)=$(shasum -a 256 "$B"|cut -c1-8) R=$R"
RC=0
declare -a BASE=("poly 9.59" "err 2.95" "wploop 6.76")
for pair in "${BASE[@]}"; do
  set -- $pair; c=$1; base=$2
  fa=(); fb=(); ta=(); tb=()
  for i in $(seq 1 "$R"); do fa+=("$(user_cpu "$A" "$EMPTY")"); fb+=("$(user_cpu "$B" "$EMPTY")"); done
  for i in $(seq 1 "$R"); do ta+=("$(user_cpu "$A" "$HD/$c.php")"); tb+=("$(user_cpu "$B" "$HD/$c.php")"); done
  out=$(python3 - "$c" "$base" "${fa[@]}" "${fb[@]}" "${ta[@]}" "${tb[@]}" <<'PY'
import sys
from statistics import median
c, base = sys.argv[1], float(sys.argv[2])
v = list(map(float, sys.argv[3:])); R = len(v)//4
fa, fb, ta, tb = v[:R], v[R:2*R], v[2*R:3*R], v[3*R:]
na = median(ta)-median(fa); nb = median(tb)-median(fb)
sb = max(tb)-min(tb)
lim = base + max(2*sb, 0.12)
verd = "REGRESSIONE" if nb > lim else "ok"
print(f"{c}: net_pin={na:.2f}s net_cand={nb:.2f}s spread_cand={sb:.2f}s baseline={base:.2f}s limite={lim:.2f}s -> {verd}")
print(f"RC={1 if nb > lim else 0}")
PY
)
  echo "$out" | sed '/^RC=/d'
  r=$(printf '%s\n' "$out" | awk -F= '/^RC=/{print $2}')
  [ "$r" = 1 ] && RC=1
done
echo "heldout_rc=$RC"
exit "$RC"
