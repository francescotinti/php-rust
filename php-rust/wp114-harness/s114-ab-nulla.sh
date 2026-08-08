#!/bin/bash
# s114-ab-nulla.sh [R] — A/B LEVA-NULLA layout (criterio s114-criterio-nulla.md).
# A = stash phpr-s112 (pin f71abd2a), B = phpr-s114-nulla (zavorra inerte,
# CONSERVATO: l'A/B non dipende dal churn di release). R=5 ABAB interleaved
# sulle SEI categorie, user CPU al netto del pavimento PER-binario, N dal
# sorgente. Mediane, spread, conteggio segni e BANDA emessi A MACCHINA qui
# (az.2 rev. S-113): nessuna derivazione a mano nel verdetto.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-nulla"
R="${1:-5}"
CATS="arith prop calls str arr re"
OUT="$H/ab-out"; mkdir -p "$OUT"
TSV="$OUT/nulla-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) R=$R"
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
print(f"  run{i}: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f}")
PY
  done
done
echo "== RIEPILOGO A MACCHINA (banda-layout pre-registrata: banda(cat)=|D mediano|; globale=max) =="
python3 - "$TSV" <<'PY'
import sys
from statistics import median
rows = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
    rows.setdefault(c, []).append((na, nb, na-nb))
banda_glob = 0.0
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    As = [r[0] for r in rows[c]]; Bs = [r[1] for r in rows[c]]; Ds = [r[2] for r in rows[c]]
    dmed = median(Ds)
    pos = sum(1 for d in Ds if d > 0); neg = sum(1 for d in Ds if d < 0)
    spread_a = max(As) - min(As); spread_b = max(Bs) - min(Bs)
    banda = abs(dmed); banda_glob = max(banda_glob, banda)
    print(f"{c}: D_mediano={dmed:+.2f} segni +{pos}/-{neg} spread_A={spread_a:.2f} "
          f"spread_B={spread_b:.2f} banda={banda:.2f}")
print(f"BANDA_GLOBALE={banda_glob:.2f} ns/iter (max su 6 categorie, N=1 perturbazione)")
PY
