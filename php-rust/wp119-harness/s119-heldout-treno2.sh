#!/bin/bash
# s119-heldout-treno2.sh — held-out a N=3 (az. rev. S-118 §5, criterio treno-2
# p.3): baseline e banda da TRE serie del PIN nella STESSA sera (R=5 per
# serie), poi il candidato (R=5); limite = base + max(2*spread_cand, 0.12,
# banda_N3). poly = segno atteso (esercita MethodCall/iter); err/wploop
# guardie. rc in treno2-out/heldout.rc; esiti appesi al verdetto.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
HD="$H/../wp111-harness/heldout"
EMPTY="$H/../wp97-harness/micro/empty.php"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s118"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s119-treno2"
OUT="$H/treno2-out"; mkdir -p "$OUT"
VERD="$H/s119-treno2-verdetto.out"
R=5
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
net_series() { # <bin> <case> -> net (mediana R su caso - mediana R su empty)
  local f=() t=() i
  for i in $(seq 1 "$R"); do f+=("$(user_cpu "$1" "$EMPTY")"); done
  for i in $(seq 1 "$R"); do t+=("$(user_cpu "$1" "$HD/$2.php")"); done
  python3 - "${f[@]}" "${t[@]}" <<'PY'
import sys
from statistics import median
v=list(map(float,sys.argv[1:])); R=len(v)//2
print(f"{median(v[R:])-median(v[:R]):.2f} {max(v[R:])-min(v[R:]):.2f}")
PY
}
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) R=$R serie_pin=3"
RC=0
: > "$OUT/ho-esiti.txt"
for c in poly err wploop; do
  read -r n1 s1 <<< "$(net_series "$A" "$c")"
  read -r n2 s2 <<< "$(net_series "$A" "$c")"
  read -r n3 s3 <<< "$(net_series "$A" "$c")"
  read -r nb sb <<< "$(net_series "$B" "$c")"
  printf '%s pin_nets=%s %s %s (spread_in_serie %s %s %s) cand_net=%s cand_spread=%s\n' \
    "$c" "$n1" "$n2" "$n3" "$s1" "$s2" "$s3" "$nb" "$sb" >> "$OUT/ho-raw.txt"
  out=$(python3 - "$c" "$n1" "$n2" "$n3" "$nb" "$sb" <<'PY'
import sys
from statistics import median
c=sys.argv[1]; n1,n2,n3,nb,sb=map(float,sys.argv[2:])
base=round(median([n1,n2,n3]),2); banda=round(max(n1,n2,n3)-min(n1,n2,n3),2)
lim=round(base+max(2*sb,0.12,banda),2)
verd="REGRESSIONE" if nb>lim else "ok"
print(f"{c}: pin_N3={n1:.2f}/{n2:.2f}/{n3:.2f} base={base:.2f}s banda_N3={banda:.2f}s "
      f"cand={nb:.2f}s spread_cand={sb:.2f}s limite={lim:.2f}s -> {verd}"
      + (f" [direzione: {'≤ come atteso' if nb<=base else 'sopra base, dentro limite'}]" if c=="poly" and verd=="ok" else ""))
print(f"RC={1 if nb>lim else 0}")
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
  echo "HELD-OUT N=3 (s119-heldout-treno2.sh, rc=$RC da treno2-out/heldout.rc; baseline+banda da 3 serie del pin stessa sera):"
  cat "$OUT/ho-esiti.txt"
} >> "$VERD"
exit "$RC"
