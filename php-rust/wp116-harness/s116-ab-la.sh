#!/bin/bash
# s116-ab-la.sh smoke|full — A/B promozione L-A (criterio s116-criterio-la.md).
# A = stash phpr-s112 (pin f71abd2a), B = phpr-s114-la (CONSERVATO 052ea417).
# INVARIATO da s115-ab-la.sh salvo (a) banda micro max N=2 (re 10,00);
# (c) tie pre-registrati su valori ARROTONDATI a 2 decimali (prop: >= soglia
# = PASS; guardie: sfondano solo se STRETTAMENTE < soglia); (d) rc scritto in
# FILE (ab-out/smoke-rc | ab-out/full-rc), esiti appesi al verbale dallo
# script a esito acquisito. Smoke R=2 con early-stop a segno opposto (rc=1).
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
VERD="$H/s116-la-verdetto.out"
TSV="$OUT/la-$MODE-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) MODE=$MODE RBASE=$RBASE EXTRA_MAX=$EXTRA_MAX"

echo "== (d/p.5 S-115) parita' output per categoria (gate PRIMA del cronometro) =="
for C in $CATS; do
  "$A" "$M/$C.php" > "$OUT/par-$C-A.out" 2>&1
  "$B" "$M/$C.php" > "$OUT/par-$C-B.out" 2>&1
  diff -q "$OUT/par-$C-A.out" "$OUT/par-$C-B.out" > /dev/null
  d=$?
  if [ "$d" -eq 0 ]; then echo "   $C: output IDENTICO"; else
    echo "   $C: output DIVERGE — STOP LEVA"; echo 2 > "$OUT/$MODE-rc"
    echo "$MODE: parita' output DIVERGE su $C — STOP LEVA (rc=2)" >> "$VERD"; exit 2
  fi
done

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }

infam_count() { # coppie in famiglia per la categoria $1 (regola 1,3x min per braccio)
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

if [ "$MODE" = smoke ]; then
  SRC=$(python3 - "$TSV" <<'PY'
import sys
neg=0
for line in open(sys.argv[1]):
    c,n,i,ta,tb,fa,fb=line.split('\t')
    n,ta,tb,fa,fb=float(n),float(ta),float(tb),float(fa),float(fb)
    d=round((ta-fa)/n*1e9-(tb-fb)/n*1e9,2)
    if d<=0: neg+=1
print(1 if neg else 0)
PY
)
  echo "$SRC" > "$OUT/smoke-rc"
  echo "smoke_rc=$SRC (0=segni concordi +, 1=segno opposto: early-stop)"
  awk -F'\t' '{d=(($4-$6)/$2-($5-$7)/$2)*1e9; printf "%+.2f ", d}' "$TSV" | {
    read -r line; echo "smoke R=2 (rc=$SRC): D per coppia = $line" >> "$VERD"
  }
  exit "$SRC"
fi

if [ -n "$INVALID" ]; then
  echo "MISURA_INVALIDA nelle categorie:$INVALID — rerun (criterio p.6/S-115 p.3), NESSUN verdetto sulla leva"
  echo 3 > "$OUT/full-rc"
  echo "full: MISURA_INVALIDA nelle categorie:$INVALID — rerun, nessun verdetto sulla leva (rc=3)" >> "$VERD"
  exit 3
fi

echo "== VERDETTO SOGLIE A MACCHINA (criterio p.2+p.4; famiglie 1,3x min; banda max N=2; tie su 2 decimali) =="
python3 - "$TSV" > "$OUT/full-verdetto.txt" <<'PY'
import sys
from statistics import median
BANDA = {"arith": 0.40, "prop": 4.33, "calls": 5.50, "str": 5.00, "arr": 6.67, "re": 10.00}
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
    dmed = round(median(Ds), 2); pos = sum(1 for d in Ds if round(d, 2) > 0)
    sa = round(max(As)-min(As), 2)
    if c == "prop":
        soglia = round(max(4.0, sa, BANDA[c]), 2)
        p = dmed >= soglia and pos == len(Ds) and len(Ds) >= 5
        ok = ok and p
        print(f"prop: fam={len(Ds)} D_mediano={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={sa:.2f} "
              f"soglia_promozione={soglia:.2f} -> {'PASS' if p else 'FAIL'} (tie: >= soglia = PASS)")
    else:
        soglia = round(-max(2*sa, BANDA[c]), 2)
        sf = dmed < soglia
        ok = ok and not sf
        print(f"{c}: fam={len(Ds)} D_mediano={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={sa:.2f} "
              f"soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'} (tie: = soglia -> tiene)")
print(f"VERDETTO_MICRO={'PROMUOVIBILE (restano admission+held-out)' if ok else 'NON promuovibile'}")
print(f"RCV={0 if ok else 1}")
PY
sed '/^RCV=/d' "$OUT/full-verdetto.txt"
FR=$(awk -F= '/^RCV=/{print $2}' "$OUT/full-verdetto.txt")
echo "$FR" > "$OUT/full-rc"
echo "full_rc=$FR (scritto in ab-out/full-rc)"
{
  echo ""
  echo "VERDETTO SOGLIE A MACCHINA (s116-ab-la.sh full, rc=$FR da ab-out/full-rc):"
  sed '/^RCV=/d' "$OUT/full-verdetto.txt"
} >> "$VERD"
exit "$FR"
