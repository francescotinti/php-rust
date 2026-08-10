#!/bin/bash
# s125-ab.sh <BPATH> <BEXP8> <TAG> <R> <EXTRA_MAX> <cat...> — criterio
# s125-criterio-cbargs.md p.5: A/B ordine ALTERNATO per coppia (dispari A->B,
# pari B->A), timer ucpu.py µs, file SCALATI wp123, floor med3 per binario,
# parità output. A = pin s124. TSV: cat N i tA tB fA fB ord. rc su FILE.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
H123="$H/../wp123-harness"
SC="$H123/scaled"
UCPU="$H123/ucpu.py"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s124"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; RBASE="${4:?R}"; EXTRA_MAX="${5:?EXTRA}"
shift 5; CATS="$*"
OUT="$H/metro-out"; mkdir -p "$OUT"
VERD="$H/s125-$TAG-verdetto.out"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
RC="$OUT/$TAG-ab.rc"

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "c5ba2573" ] || { echo "A != pin s124 — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
echo "== s125-ab $TAG: A=c5ba2573 B=$BEXP R=$RBASE extra<=$EXTRA_MAX cats=[$CATS] ordine ALTERNATO ==" >> "$VERD"

for C in $CATS; do
  "$A" "$SC/$C.php" > "$OUT/abpar-$TAG-$C-A.out" 2>&1
  "$BB" "$SC/$C.php" > "$OUT/abpar-$TAG-$C-B.out" 2>&1
  if ! diff -q "$OUT/abpar-$TAG-$C-A.out" "$OUT/abpar-$TAG-$C-B.out" > /dev/null; then
    echo "$C: output DIVERGE — STOP LEVA" | tee -a "$VERD"; echo 2 > "$RC"; exit 2
  fi
done

u() { python3 "$UCPU" "$1" "$2"; }
floor3() {
  local f1 f2 f3
  f1=$(u "$1" "$SC/empty.php"); f2=$(u "$1" "$SC/empty.php"); f3=$(u "$1" "$SC/empty.php")
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
n_of() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; esac; }

infam_count() {
python3 - "$TSV" "$1" <<'PY'
import sys
rows=[l.split('\t') for l in open(sys.argv[1]) if l.startswith(sys.argv[2]+'\t')]
if not rows: print(0); sys.exit()
na=[(float(t[3])-float(t[5]))/float(t[1])*1e9 for t in rows]
nb=[(float(t[4])-float(t[6]))/float(t[1])*1e9 for t in rows]
ta,tb=1.3*min(na),1.3*min(nb)
print(sum(1 for a,b in zip(na,nb) if a<=ta and b<=tb))
PY
}

run_pair() { # C N i FA FB
  local C=$1 N=$2 i=$3 FA=$4 FB=$5 TA TB ord
  if [ $((i % 2)) -eq 1 ]; then
    TA=$(u "$A" "$SC/$C.php"); TB=$(u "$BB" "$SC/$C.php"); ord=AB
  else
    TB=$(u "$BB" "$SC/$C.php"); TA=$(u "$A" "$SC/$C.php"); ord=BA
  fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$N" "$i" "$TA" "$TB" "$FA" "$FB" "$ord" >> "$TSV"
  python3 - "$i" "$TA" "$TB" "$FA" "$FB" "$N" "$ord" <<'PY'
import sys
i, ta, tb, fa, fb, n = int(sys.argv[1]), *map(float, sys.argv[2:7])
na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
print(f"  coppia{i} [{sys.argv[7]}]: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f} (B più veloce se +)")
PY
}

INVALID=""
for C in $CATS; do
  N=$(n_of "$C")
  FA=$(floor3 "$A"); FB=$(floor3 "$BB")
  echo "cat=$C N=$N floor_A(med3)=$FA floor_B(med3)=$FB"
  for i in $(seq 1 "$RBASE"); do run_pair "$C" "$N" "$i" "$FA" "$FB"; done
  x=0
  # MINFAM (az. rev. S-125 #5): in modalità smoke (R<5) la famiglia richiesta
  # è R; il giudice finale resta a 5 (default).
  MF="${MINFAM:-5}"
  while [ "$(infam_count "$C")" -lt "$MF" ] && [ "$x" -lt "$EXTRA_MAX" ]; do
    x=$((x+1)); echo "  [$C] coppie in famiglia $(infam_count "$C")/$MF — coppia extra $x/$EXTRA_MAX"
    run_pair "$C" "$N" "$((RBASE+x))" "$FA" "$FB"
  done
  if [ "$(infam_count "$C")" -lt "$MF" ]; then echo "  [$C] MISURA_INVALIDA"; INVALID="$INVALID $C"; fi
done

if [ -n "$INVALID" ]; then
  echo "MISURA_INVALIDA:$INVALID — NESSUN verdetto" | tee -a "$VERD"; echo 3 > "$RC"; exit 3
fi
echo 0 > "$RC"
