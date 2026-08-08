#!/bin/bash
# s117-heldout-la.sh — criterio s117-criterio-la.md p.4 (modifica (h)):
# baseline e banda dalla pipeline A′ (N=2), spread=max(pin,cand) ENTRAMBI
# pubblicati, tie: = limite -> ok. rc in la-out/heldout.rc; esiti appesi.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
HD="$H/../wp111-harness/heldout"
EMPTY="$H/../wp97-harness/micro/empty.php"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117-aprime"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117-la"
OUT="$H/la-out"; mkdir -p "$OUT"
VERD="$H/s117-la-verdetto.out"
R=5
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) R=$R"
RC=0
: > "$OUT/ho-esiti.txt"
for tripla in "poly 9.44 0.01" "err 2.99 0.06" "wploop 6.38 0.08"; do
  c=$(echo "$tripla" | awk '{print $1}'); base=$(echo "$tripla" | awk '{print $2}'); banda=$(echo "$tripla" | awk '{print $3}')
  fa=(); fb=(); ta=(); tb=()
  for i in $(seq 1 "$R"); do fa+=("$(user_cpu "$A" "$EMPTY")"); fb+=("$(user_cpu "$B" "$EMPTY")"); done
  for i in $(seq 1 "$R"); do ta+=("$(user_cpu "$A" "$HD/$c.php")"); tb+=("$(user_cpu "$B" "$HD/$c.php")"); done
  printf '%s raw A: floors=%s times=%s\n' "$c" "${fa[*]}" "${ta[*]}" >> "$OUT/ho-raw.txt"
  printf '%s raw B: floors=%s times=%s\n' "$c" "${fb[*]}" "${tb[*]}" >> "$OUT/ho-raw.txt"
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
incert = " [dentro l'incertezza layout]" if abs(nb-lim) <= banda else ""
print(f"{c}: net_A={na:.2f}s net_cand={nb:.2f}s spread_pin={sp:.2f}s spread_cand={sc:.2f}s "
      f"spread=max={spread:.2f}s baseline_Aprime={base:.2f}s banda_ho={banda:.2f}s limite={lim:.2f}s -> {verd}{incert}")
print(f"RC={1 if nb > lim else 0}")
PY
)
  printf '%s\n' "$out" | sed '/^RC=/d'
  printf '%s\n' "$out" | sed '/^RC=/d' >> "$OUT/ho-esiti.txt"
  r=$(printf '%s\n' "$out" | awk -F= '/^RC=/{print $2}')
  [ "$r" = 1 ] && RC=1
done
echo "$RC" > "$OUT/heldout.rc"
{
  echo ""
  echo "HELD-OUT (s117-heldout-la.sh, rc=$RC da la-out/heldout.rc; baseline+banda A′ N=2; spread=max(pin,cand) pubblicati):"
  cat "$OUT/ho-esiti.txt"
} >> "$VERD"
exit "$RC"
