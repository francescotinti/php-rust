#!/bin/bash
# s115-ab-la.sh smoke|full — A/B rimisura L-A (criterio s115-criterio-la.md).
# A = stash phpr-s112 (pin f71abd2a), B = phpr-s114-la (CONSERVATO 052ea417).
# Emendamenti S-115: (d) parita' stdout+stderr A vs B per categoria PRIMA del
# cronometro (gate, una divergenza ⇒ exit 2 = STOP leva); (a) famiglia = run
# entro 1,3x il MINIMO della propria serie, coppia esclusa per NOME se un lato
# e' fuori; integrazione automatica fino a +6 coppie per avere >=5 in famiglia;
# (b) <5 in famiglia dopo +6 ⇒ MISURA_INVALIDA per la categoria (exit 3 in
# full: rerun categoria a cura del chiamante, mai verdetto sulla leva).
# Verdetto soglie A MACCHINA su coppie in famiglia (punti 4 del criterio).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-la"
MODE="${1:?smoke|full}"
RBASE="${RBASE:-5}"
if [ "$MODE" = smoke ]; then CATS="prop"; RBASE=2; EXTRA_MAX=0; else CATS="arith prop calls str arr re"; EXTRA_MAX=6; fi
OUT="$H/ab-out"; mkdir -p "$OUT"
TSV="$OUT/la-$MODE-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) MODE=$MODE RBASE=$RBASE EXTRA_MAX=$EXTRA_MAX"

echo "== (d) parita' output per categoria (gate PRIMA del cronometro) =="
for C in $CATS; do
  "$A" "$M/$C.php" > "$OUT/par-$C-A.out" 2>&1
  "$B" "$M/$C.php" > "$OUT/par-$C-B.out" 2>&1
  diff -q "$OUT/par-$C-A.out" "$OUT/par-$C-B.out" > /dev/null
  d=$?
  if [ "$d" -eq 0 ]; then echo "   $C: output IDENTICO"; else echo "   $C: output DIVERGE — STOP LEVA (criterio p.5)"; exit 2; fi
done

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }

infam_count() { # quante coppie in famiglia per la categoria $1 (regola 1,3x min per braccio)
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
print(f"  coppia{i}: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f} (B piu' veloce se +)")
PY
}

INVALID=""
for C in $CATS; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  [ "$C" = arr ] && N=6000000
  FA=$(user_cpu "$A" "$M/empty.php"); FB=$(user_cpu "$B" "$M/empty.php")
  echo "cat=$C N=$N floor_A=$FA floor_B=$FB"
  for i in $(seq 1 "$RBASE"); do run_pair "$C" "$N" "$i" "$FA" "$FB"; done
  if [ "$MODE" = full ]; then
    x=0
    while [ "$(infam_count "$C")" -lt 5 ] && [ "$x" -lt "$EXTRA_MAX" ]; do
      x=$((x+1)); echo "  [$C] coppie in famiglia $(infam_count "$C")/5 — coppia extra $x/$EXTRA_MAX"
      run_pair "$C" "$N" "$((RBASE+x))" "$FA" "$FB"
    done
    if [ "$(infam_count "$C")" -lt 5 ]; then echo "  [$C] MISURA_INVALIDA (<5 coppie in famiglia dopo +$EXTRA_MAX)"; INVALID="$INVALID $C"; fi
  fi
done
[ "$MODE" = smoke ] && exit 0
if [ -n "$INVALID" ]; then echo "MISURA_INVALIDA nelle categorie:$INVALID — rerun (criterio p.3), NESSUN verdetto sulla leva"; exit 3; fi

echo "== VERDETTO SOGLIE A MACCHINA (criterio p.4; famiglie 1,3x min; banda S-114) =="
python3 - "$TSV" <<'PY'
import sys
from statistics import median
BANDA = {"arith": 0.40, "prop": 4.33, "calls": 5.50, "str": 5.00, "arr": 6.67, "re": 0.00}
rows = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
ok = True
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    na = [r[1] for r in rows[c]]; nb = [r[2] for r in rows[c]]
    tA, tB = 1.3*min(na), 1.3*min(nb)
    fam, escl = [], []
    for i, a, b in rows[c]:
        if a <= tA and b <= tB: fam.append((i, a, b))
        else: escl.append((i, a, b, "A" if a > tA else "B"))
    for i, a, b, lato in escl:
        print(f"{c}: ESCLUSA coppia{i} lato {lato} (A={a:.2f} B={b:.2f}; soglie fam A<={tA:.2f} B<={tB:.2f})")
    As = [a for _, a, _ in fam]; Ds = [a-b for _, a, b in fam]
    dmed = median(Ds); pos = sum(1 for d in Ds if d > 0)
    sa = max(As)-min(As)
    if c == "prop":
        soglia = max(4.0, sa, BANDA[c])
        p = dmed >= soglia and pos == len(Ds) and len(Ds) >= 5
        ok = ok and p
        print(f"prop: fam={len(Ds)} D_mediano={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={sa:.2f} "
              f"soglia_promozione={soglia:.2f} -> {'PASS' if p else 'FAIL'}")
    else:
        soglia = -max(2*sa, BANDA[c])
        sf = dmed < soglia
        ok = ok and not sf
        print(f"{c}: fam={len(Ds)} D_mediano={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={sa:.2f} "
              f"soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'}")
print(f"VERDETTO_MICRO={'PROMUOVIBILE (restano held-out+admission)' if ok else 'NON promuovibile'}")
PY
