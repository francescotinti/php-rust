#!/bin/bash
# s114-ab-la.sh smoke|full — A/B leva L-A (criterio s114-criterio-la.md).
# A = stash phpr-s112 (pin f71abd2a), B = phpr-s114-la (candidato CONSERVATO).
# smoke: prop R=2 (early-stop a segno opposto è deciso dal chiamante sui numeri
# emessi qui). full: 6 categorie R=5 ABAB + verdetto soglie A MACCHINA
# (promozione punto 3, guardie punto 4 con banda MISURATA S-114).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-la"
MODE="${1:?smoke|full}"
if [ "$MODE" = smoke ]; then CATS="prop"; R=2; else CATS="arith prop calls str arr re"; R=5; fi
OUT="$H/ab-out"; mkdir -p "$OUT"
TSV="$OUT/la-$MODE-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) MODE=$MODE R=$R"
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
for C in $CATS; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  [ "$C" = arr ] && N=6000000
  FA=$(user_cpu "$A" "$M/empty.php"); FB=$(user_cpu "$B" "$M/empty.php")
  echo "cat=$C N=$N floor_A=$FA floor_B=$FB"
  for i in $(seq 1 "$R"); do
    TA=$(user_cpu "$A" "$M/$C.php"); TB=$(user_cpu "$B" "$M/$C.php")
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$N" "$i" "$TA" "$TB" "$FA" "$FB" >> "$TSV"
    python3 - "$i" "$TA" "$TB" "$FA" "$FB" "$N" <<'PY'
import sys
i, ta, tb, fa, fb, n = int(sys.argv[1]), *map(float, sys.argv[2:])
na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
print(f"  run{i}: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f} (B piu' veloce se +)")
PY
  done
done
[ "$MODE" = smoke ] && exit 0
echo "== VERDETTO SOGLIE A MACCHINA (criterio punti 3-4; banda S-114 da s114-nulla-verdetto.out) =="
python3 - "$TSV" <<'PY'
import sys
from statistics import median
BANDA = {"arith": 0.40, "prop": 4.33, "calls": 5.50, "str": 5.00, "arr": 6.67, "re": 0.00}
rows = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    rows.setdefault(c, []).append(((ta-fa)/n*1e9, (tb-fb)/n*1e9))
ok = True
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    As = [r[0] for r in rows[c]]; Bs = [r[1] for r in rows[c]]
    Ds = [a-b for a, b in rows[c]]
    dmed = median(Ds); pos = sum(1 for d in Ds if d > 0); neg = sum(1 for d in Ds if d < 0)
    sa = max(As)-min(As); sb = max(Bs)-min(Bs)
    if c == "prop":
        soglia = max(4.0, sa, BANDA[c])
        p = dmed >= soglia and pos == len(Ds)
        ok = ok and p
        print(f"prop: D_mediano={dmed:+.2f} segni +{pos}/-{neg} spread_A={sa:.2f} spread_B={sb:.2f} "
              f"soglia_promozione={soglia:.2f} -> {'PASS' if p else 'FAIL'}")
    else:
        soglia = -max(2*sa, BANDA[c])
        sf = dmed < soglia
        ok = ok and not sf
        print(f"{c}: D_mediano={dmed:+.2f} segni +{pos}/-{neg} spread_A={sa:.2f} spread_B={sb:.2f} "
              f"soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'}")
print(f"VERDETTO_MICRO={'PROMUOVIBILE (restano held-out+admission)' if ok else 'NON promuovibile'}")
PY
