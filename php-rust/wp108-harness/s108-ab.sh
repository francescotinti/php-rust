#!/bin/bash
# s108-ab.sh <categoria> <R> — A/B interleaved ABAB fra pin S-107b (A, stash)
# e candidato lotto-2 S-108 (B, binario CONSERVATO phpr-s108-ab-candidate:
# lezione azione-5, l'A/B non dipende dal churn di release/phpr).
# user CPU al netto del pavimento PER-binario; N emesso dal sorgente.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s107b"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s108-ab-candidate"
C="${1:?categoria}"; R="${2:-5}"
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
echo "cat=$C N=$N A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) R=$R"
FA=$(user_cpu "$A" "$M/empty.php"); FB=$(user_cpu "$B" "$M/empty.php")
echo "floor_A=$FA floor_B=$FB"
for i in $(seq 1 "$R"); do
  TA=$(user_cpu "$A" "$M/$C.php"); TB=$(user_cpu "$B" "$M/$C.php")
  python3 - "$i" "$TA" "$TB" "$FA" "$FB" "$N" <<'PY'
import sys
i, ta, tb, fa, fb, n = int(sys.argv[1]), *map(float, sys.argv[2:])
na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
print(f"run{i}: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f} (B piu' veloce se +)")
PY
done
