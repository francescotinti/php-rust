#!/bin/bash
# s117-ab-uplift.sh — A/B interleaved ABAB pin↔A′ sui sei giudici (criterio
# s117-criterio-aprime.md §4): il numero serve SOLO a KS-A (uplift mediano),
# NON è magnitudine di leva. A = pin s112 (stash), B = candidato A′ conservato.
# ns/iter al netto dei pavimenti PER-binario; famiglie 1,3×min; N dal sorgente;
# parità output per categoria PRIMA del cronometro; rc/esiti in FILE dallo script.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117-aprime"
RBASE="${RBASE:-5}"; EXTRA_MAX=6
CATS="arith prop calls str arr re"
OUT="$H/aprime-out"; mkdir -p "$OUT"
VERD="$H/s117-aprime-verdetto.out"
TSV="$OUT/uplift-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) R=$RBASE"

for C in $CATS; do
  "$A" "$M/$C.php" > "$OUT/up-par-$C-A.out" 2>&1
  "$B" "$M/$C.php" > "$OUT/up-par-$C-B.out" 2>&1
  if ! diff -q "$OUT/up-par-$C-A.out" "$OUT/up-par-$C-B.out" > /dev/null; then
    echo "$C: output DIVERGE — STOP"; echo 2 > "$OUT/uplift-rc"
    echo "uplift: parità output DIVERGE su $C — STOP (rc=2)" >> "$VERD"; exit 2
  fi
done
echo "parità output: IDENTICA su tutte le categorie"

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }

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

run_pair() { # $1=cat $2=N $3=idx $4=FA $5=FB
  TA=$(user_cpu "$A" "$M/$1.php"); TB=$(user_cpu "$B" "$M/$1.php")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$TA" "$TB" "$4" "$5" >> "$TSV"
  python3 - "$3" "$TA" "$TB" "$4" "$5" "$2" <<'PY'
import sys
i, ta, tb, fa, fb, n = int(sys.argv[1]), *map(float, sys.argv[2:])
na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
print(f"  coppia{i}: pin={na:.2f} ns/iter  A'={nb:.2f} ns/iter  D={na-nb:+.2f} (A' più veloce se +)")
PY
}

INVALID=""
for C in $CATS; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  [ "$C" = arr ] && N=6000000
  FA=$(user_cpu "$A" "$M/empty.php"); FB=$(user_cpu "$B" "$M/empty.php")
  echo "cat=$C N=$N floor_pin=$FA floor_aprime=$FB"
  for i in $(seq 1 "$RBASE"); do run_pair "$C" "$N" "$i" "$FA" "$FB"; done
  x=0
  while [ "$(infam_count "$C")" -lt 5 ] && [ "$x" -lt "$EXTRA_MAX" ]; do
    x=$((x+1)); echo "  [$C] coppie in famiglia $(infam_count "$C")/5 — coppia extra $x/$EXTRA_MAX"
    run_pair "$C" "$N" "$((RBASE+x))" "$FA" "$FB"
  done
  if [ "$(infam_count "$C")" -lt 5 ]; then INVALID="$INVALID $C"; fi
done

if [ -n "$INVALID" ]; then
  echo "MISURA_INVALIDA:$INVALID"; echo 3 > "$OUT/uplift-rc"
  echo "uplift: MISURA_INVALIDA nelle categorie:$INVALID (rc=3)" >> "$VERD"; exit 3
fi

python3 - "$TSV" > "$OUT/uplift-verdetto.txt" <<'PY'
import sys
from statistics import median
rows = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
ups = []
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    na = [r[1] for r in rows[c]]; nb = [r[2] for r in rows[c]]
    tA, tB = 1.3*min(na), 1.3*min(nb)
    fam = [(a, b) for _, a, b in rows[c] if a <= tA and b <= tB]
    As = [a for a, _ in fam]; Ds = [a-b for a, b in fam]
    dmed = round(median(Ds), 2); medA = median(As)
    pos = sum(1 for d in Ds if round(d, 2) > 0)
    up = round(dmed/medA*100, 2)
    ups.append(up)
    print(f"{c}: fam={len(Ds)} pin_med={medA:.2f} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} uplift={up:+.2f}%")
print(f"UPLIFT_MEDIANO={round(median(ups),2):+.2f}% (giudice KS-A: scatta se <2% E banda_new max >5)")
PY
cat "$OUT/uplift-verdetto.txt"
echo 0 > "$OUT/uplift-rc"
{
  echo ""
  echo "A/B uplift pin↔A′ (s117-ab-uplift.sh R=$RBASE, rc=0 da aprime-out/uplift-rc):"
  cat "$OUT/uplift-verdetto.txt"
} >> "$VERD"
exit 0
