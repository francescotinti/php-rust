#!/bin/bash
# s115-ab-nulla2.sh — A/B SECONDA leva-nulla (criterio s115-criterio-nulla2.md).
# A = stash phpr-s112 (pin f71abd2a), B = phpr-s115-nulla2 (candidato CONSERVATO).
# Protocollo emendato S-115: parita' output per categoria PRIMA del cronometro
# (gate), famiglie 1,3x min con esclusione per NOME e integrazione max +6,
# <5 coppie in famiglia ⇒ MISURA_INVALIDA (exit 3). Uscite = banda2(cat) =
# |D mediano in famiglia|; banda2 globale = max. Piu' held-out (p.4-bis):
# poly/err/wploop R=5 net per-binario, |Δ net| = primo campione banda held-out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
HD="$H/../wp111-harness/heldout"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s115-nulla2"
CATS="arith prop calls str arr re"
RBASE=5; EXTRA_MAX=6
OUT="$H/ab-out"; mkdir -p "$OUT"
TSV="$OUT/nulla2-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) RBASE=$RBASE EXTRA_MAX=$EXTRA_MAX"

echo "== parita' output per categoria (gate PRIMA del cronometro) =="
for C in $CATS; do
  "$A" "$M/$C.php" > "$OUT/n2par-$C-A.out" 2>&1
  "$B" "$M/$C.php" > "$OUT/n2par-$C-B.out" 2>&1
  diff -q "$OUT/n2par-$C-A.out" "$OUT/n2par-$C-B.out" > /dev/null
  d=$?
  if [ "$d" -eq 0 ]; then echo "   $C: output IDENTICO"; else echo "   $C: output DIVERGE — VIOLAZIONE (leva nulla!)"; exit 2; fi
done

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

run_pair() {
  TA=$(user_cpu "$A" "$M/$1.php"); TB=$(user_cpu "$B" "$M/$1.php")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$TA" "$TB" "$4" "$5" >> "$TSV"
  python3 - "$3" "$TA" "$TB" "$4" "$5" "$2" <<'PY'
import sys
i, ta, tb, fa, fb, n = int(sys.argv[1]), *map(float, sys.argv[2:])
na, nb = (ta-fa)/n*1e9, (tb-fb)/n*1e9
print(f"  coppia{i}: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f}")
PY
}

INVALID=""
for C in $CATS; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  [ "$C" = arr ] && N=6000000
  FA=$(user_cpu "$A" "$M/empty.php"); FB=$(user_cpu "$B" "$M/empty.php")
  echo "cat=$C N=$N floor_A=$FA floor_B=$FB"
  for i in $(seq 1 "$RBASE"); do run_pair "$C" "$N" "$i" "$FA" "$FB"; done
  x=0
  while [ "$(infam_count "$C")" -lt 5 ] && [ "$x" -lt "$EXTRA_MAX" ]; do
    x=$((x+1)); echo "  [$C] coppie in famiglia $(infam_count "$C")/5 — coppia extra $x/$EXTRA_MAX"
    run_pair "$C" "$N" "$((RBASE+x))" "$FA" "$FB"
  done
  if [ "$(infam_count "$C")" -lt 5 ]; then echo "  [$C] MISURA_INVALIDA"; INVALID="$INVALID $C"; fi
done
if [ -n "$INVALID" ]; then echo "MISURA_INVALIDA nelle categorie:$INVALID — rerun (criterio p.3)"; exit 3; fi

echo "== BANDA2 A MACCHINA (criterio p.4: banda2(cat)=|D mediano in famiglia|; confronto S-114) =="
python3 - "$TSV" <<'PY'
import sys
from statistics import median
S114 = {"arith": 0.40, "prop": 4.33, "calls": 5.50, "str": 5.00, "arr": 6.67, "re": 0.00}
rows = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
g = 0.0
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    na = [r[1] for r in rows[c]]; nb = [r[2] for r in rows[c]]
    tA, tB = 1.3*min(na), 1.3*min(nb)
    fam, escl = [], []
    for i, a, b in rows[c]:
        if a <= tA and b <= tB: fam.append((i, a, b))
        else: escl.append((i, a, b, "A" if a > tA else "B"))
    for i, a, b, lato in escl:
        print(f"{c}: ESCLUSA coppia{i} lato {lato} (A={a:.2f} B={b:.2f}; fam A<={tA:.2f} B<={tB:.2f})")
    Ds = [a-b for _, a, b in fam]; As = [a for _, a, _ in fam]
    dmed = median(Ds); pos = sum(1 for d in Ds if d > 0)
    banda2 = abs(dmed); g = max(g, banda2)
    print(f"{c}: fam={len(Ds)} D_mediano={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={max(As)-min(As):.2f} "
          f"banda2={banda2:.2f} (S114={S114[c]:.2f}, max={max(banda2, S114[c]):.2f})")
print(f"BANDA2_GLOBALE={g:.2f} (S114_globale=6.67; instabile_se>13.34={'SI' if g > 13.34 else 'no'})")
PY

echo "== HELD-OUT banda-layout (p.4-bis): R=5 net per-binario, |Dnet| = campione banda in s =="
EMPTY="$M/empty.php"
for c in poly err wploop; do
  fa=(); fb=(); ta=(); tb=()
  for i in $(seq 1 5); do fa+=("$(user_cpu "$A" "$EMPTY")"); fb+=("$(user_cpu "$B" "$EMPTY")"); done
  for i in $(seq 1 5); do ta+=("$(user_cpu "$A" "$HD/$c.php")"); tb+=("$(user_cpu "$B" "$HD/$c.php")"); done
  python3 - "$c" "${fa[@]}" "${fb[@]}" "${ta[@]}" "${tb[@]}" <<'PY'
import sys
from statistics import median
c = sys.argv[1]; v = list(map(float, sys.argv[2:])); R = len(v)//4
fa, fb, ta, tb = v[:R], v[R:2*R], v[2*R:3*R], v[3*R:]
na = median(ta)-median(fa); nb = median(tb)-median(fb)
print(f"{c}: net_A={na:.2f}s net_B={nb:.2f}s Dnet={na-nb:+.2f}s banda_heldout={abs(na-nb):.2f}s "
      f"spread_A={max(ta)-min(ta):.2f} spread_B={max(tb)-min(tb):.2f}")
PY
done
