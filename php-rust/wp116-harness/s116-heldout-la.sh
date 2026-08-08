#!/bin/bash
# s116-heldout-la.sh — criterio s116-criterio-la.md p.3+p.4+p.5+p.8.
# R=5 pin (phpr-s112) E candidato (phpr-s114-la) su poly/err/wploop; net =
# user CPU mediano - pavimento per-binario. REGRESSIONE se net_cand (2 dec)
# STRETTAMENTE > limite = baseline S-112 + max(2*spread; 0,12; banda_heldout),
# con spread = max(spread_pin, spread_cand) ed ENTRAMBI PUBBLICATI.
# banda_heldout (N=1, s115-nulla2): poly 0,20 · err 0,01 · wploop 0,06.
# rc scritto in heldout-out/rc; esiti appesi al verbale dallo script.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
HD="$H/../wp111-harness/heldout"
EMPTY="$H/../wp97-harness/micro/empty.php"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-la"
OUT="$H/heldout-out"; mkdir -p "$OUT"
VERD="$H/s116-la-verdetto.out"
R=5
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
echo "A(pin)=$(shasum -a 256 "$A"|cut -c1-8) B(cand)=$(shasum -a 256 "$B"|cut -c1-8) R=$R"
RC=0
: > "$OUT/esiti.txt"
declare -a BASE=("poly 9.59 0.20" "err 2.95 0.01" "wploop 6.76 0.06")
for tripla in "${BASE[@]}"; do
  set -- $tripla; c=$1; base=$2; banda=$3
  fa=(); fb=(); ta=(); tb=()
  for i in $(seq 1 "$R"); do fa+=("$(user_cpu "$A" "$EMPTY")"); fb+=("$(user_cpu "$B" "$EMPTY")"); done
  for i in $(seq 1 "$R"); do ta+=("$(user_cpu "$A" "$HD/$c.php")"); tb+=("$(user_cpu "$B" "$HD/$c.php")"); done
  printf '%s raw pin: floors=%s times=%s\n' "$c" "${fa[*]}" "${ta[*]}" >> "$OUT/raw.txt"
  printf '%s raw cand: floors=%s times=%s\n' "$c" "${fb[*]}" "${tb[*]}" >> "$OUT/raw.txt"
  out=$(python3 - "$c" "$base" "$banda" "${fa[@]}" "${fb[@]}" "${ta[@]}" "${tb[@]}" <<'PY'
import sys
from statistics import median
c, base, banda = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
v = list(map(float, sys.argv[4:])); R = len(v)//4
fa, fb, ta, tb = v[:R], v[R:2*R], v[2*R:3*R], v[3*R:]
na = round(median(ta)-median(fa), 2); nb = round(median(tb)-median(fb), 2)
sp = round(max(ta)-min(ta), 2); sc = round(max(tb)-min(tb), 2)
spread = max(sp, sc)
lim = round(base + max(2*spread, 0.12, banda), 2)
verd = "REGRESSIONE" if nb > lim else "ok"
incert = " [dentro l'incertezza layout: |net-limite|<=banda]" if abs(nb-lim) <= banda else ""
print(f"{c}: net_pin={na:.2f}s net_cand={nb:.2f}s spread_pin={sp:.2f}s spread_cand={sc:.2f}s "
      f"spread=max={spread:.2f}s baseline={base:.2f}s banda_heldout={banda:.2f}s limite={lim:.2f}s -> {verd}{incert}")
print(f"RC={1 if nb > lim else 0}")
PY
)
  printf '%s\n' "$out" | sed '/^RC=/d'
  printf '%s\n' "$out" | sed '/^RC=/d' >> "$OUT/esiti.txt"
  r=$(printf '%s\n' "$out" | awk -F= '/^RC=/{print $2}')
  [ "$r" = 1 ] && RC=1
done
echo "$RC" > "$OUT/rc"
echo "heldout_rc=$RC (scritto in heldout-out/rc)"
{
  echo ""
  echo "HELD-OUT (s116-heldout-la.sh, rc=$RC da heldout-out/rc; spread=max(pin,cand), entrambi pubblicati; tie: = limite -> ok):"
  cat "$OUT/esiti.txt"
} >> "$VERD"
exit "$RC"
