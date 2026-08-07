#!/bin/bash
# s108-abx.sh <categoria_x> <R> — azione-2 revisore S-107: A/B interleaved ABAB
# fra pin S-106 (A, stash phpr-s106) e pin S-107b (B, release corrente) sui
# giudici a N MAGGIORATO (micro-x/), misura in ns/inner-iter. Il denominatore
# si EMETTE dal sorgente (KS-GR-105-2): OUTER = primo $r<K (default 1),
# INNER = ULTIMO $i<K nel sorgente; N = OUTER*INNER. Per arr_x il setup
# (100k insert) resta FUORI dal denominatore: bias +0,4% dichiarato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/micro-x"
E="$H/../wp97-harness/micro/empty.php"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s106"
B="$HOME/Claude/php-rust-output/release/phpr"
C="${1:?categoria}"; R="${2:-5}"
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
OUTER=$(awk 'match($0, /\$r<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
INNER=$(awk 'match($0, /\$i<[0-9]+/) {n=substr($0, RSTART+3, RLENGTH-3)} END{print n}' "$M/$C.php")
N=$(( ${OUTER:-1} * INNER ))
echo "cat=$C outer=${OUTER:-1} inner=$INNER N_inner_iter=$N A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) R=$R"
FA=$(user_cpu "$A" "$E"); FB=$(user_cpu "$B" "$E")
echo "floor_A=$FA floor_B=$FB"
for i in $(seq 1 "$R"); do
  TA=$(user_cpu "$A" "$M/$C.php"); TB=$(user_cpu "$B" "$M/$C.php")
  python3 - "$i" "$TA" "$TB" "$FA" "$FB" "$N" <<'PY'
import sys
i, ta, tb, fa, fb, n = int(sys.argv[1]), *map(float, sys.argv[2:])
na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
print(f"run{i}: A={na:.2f} ns/inner-iter  B={nb:.2f} ns/inner-iter  D={na-nb:+.2f} (B piu' veloce se +)")
PY
done
