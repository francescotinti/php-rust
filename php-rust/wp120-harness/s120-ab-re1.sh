#!/bin/bash
# s120-ab-re1.sh smoke|full — A/B L-RE1 sotto A′ (s120-criterio-re1.md p.3-5/7):
# BERSAGLIO re (promozione: D_med >= max(banda micro N=2 re 10,00; 2×spread_A;
# 2×quanto)); le altre cinque categorie GUARDIE a solo-regressione (banda-v2);
# floor = mediana di 3 per binario; rc in FILE. Morso calls dentro 0,50 si
# REGISTRA (forma s119).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s119"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s120-re1"
MODE="${1:?smoke|full}"
RBASE="${RBASE:-5}"
if [ "$MODE" = smoke ]; then CATS="re prop"; RBASE=2; EXTRA_MAX=0; else CATS="arith prop calls str arr re"; EXTRA_MAX=6; fi
OUT="$H/re1-out"; mkdir -p "$OUT"
VERD="$H/s120-re1-verdetto.out"
TSV="$OUT/re1-$MODE-runs.tsv"; : > "$TSV"
echo "A=$(shasum -a 256 "$A"|cut -c1-8) B=$(shasum -a 256 "$B"|cut -c1-8) MODE=$MODE RBASE=$RBASE"

for C in $CATS; do
  "$A" "$M/$C.php" > "$OUT/abpar-$C-A.out" 2>&1
  "$B" "$M/$C.php" > "$OUT/abpar-$C-B.out" 2>&1
  if ! diff -q "$OUT/abpar-$C-A.out" "$OUT/abpar-$C-B.out" > /dev/null; then
    echo "$C: output DIVERGE — STOP LEVA"; echo 2 > "$OUT/$MODE-rc"
    echo "$MODE: parità output DIVERGE su $C — STOP LEVA (rc=2)" >> "$VERD"; exit 2
  fi
done

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() {
  local f1 f2 f3; f1=$(user_cpu "$1" "$M/empty.php"); f2=$(user_cpu "$1" "$M/empty.php"); f3=$(user_cpu "$1" "$M/empty.php")
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}

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
print(f"  coppia{i}: A={na:.2f} ns/iter  B={nb:.2f} ns/iter  D={na-nb:+.2f} (B più veloce se +)")
PY
}

INVALID=""
for C in $CATS; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  [ "$C" = arr ] && N=6000000
  FA=$(floor3 "$A"); FB=$(floor3 "$B")
  echo "cat=$C N=$N floor_A(med3)=$FA floor_B(med3)=$FB"
  for i in $(seq 1 "$RBASE"); do run_pair "$C" "$N" "$i" "$FA" "$FB"; done
  if [ "$MODE" = full ]; then
    x=0
    while [ "$(infam_count "$C")" -lt 5 ] && [ "$x" -lt "$EXTRA_MAX" ]; do
      x=$((x+1)); echo "  [$C] coppie in famiglia $(infam_count "$C")/5 — coppia extra $x/$EXTRA_MAX"
      run_pair "$C" "$N" "$((RBASE+x))" "$FA" "$FB"
    done
    if [ "$(infam_count "$C")" -lt 5 ]; then echo "  [$C] MISURA_INVALIDA"; INVALID="$INVALID $C"; fi
  fi
done

if [ "$MODE" = smoke ]; then
  SRC=$(python3 - "$TSV" <<'PY'
import sys
# smoke di GUARDIA (criterio p.7): early-stop SOLO su regressione marcata
# concorde (D <= -1.00 su tutte le coppie di una categoria); il resto si
# registra, la banda giudica nel full.
from collections import defaultdict
d=defaultdict(list)
for line in open(sys.argv[1]):
    c,n,i,ta,tb,fa,fb=line.split('\t')
    n,ta,tb,fa,fb=float(n),float(ta),float(tb),float(fa),float(fb)
    d[c].append(round((ta-fa)/n*1e9-(tb-fb)/n*1e9,2))
bad=any(all(x<=-1.00 for x in v) for v in d.values())
print(1 if bad else 0)
PY
)
  echo "$SRC" > "$OUT/smoke-rc"
  echo "smoke_rc=$SRC (0=nessuna regressione concorde <=-1.00, 1=early-stop)"
  awk -F'\t' '{d=(($4-$6)/$2-($5-$7)/$2)*1e9; printf "%+.2f ", d}' "$TSV" | {
    read -r line; echo "smoke R=2 guardia (rc=$SRC): D per coppia = $line" >> "$VERD"
  }
  exit "$SRC"
fi

if [ -n "$INVALID" ]; then
  echo "MISURA_INVALIDA:$INVALID — rerun, NESSUN verdetto"; echo 3 > "$OUT/full-rc"
  echo "full: MISURA_INVALIDA:$INVALID (rc=3)" >> "$VERD"; exit 3
fi

echo "== VERDETTO A MACCHINA (bersaglio re: soglia +max(10,00; 2×spread_A; 2×quanto); guardie banda-v2; tie 2 dec) =="
python3 - "$TSV" > "$OUT/full-verdetto.txt" <<'PY'
import sys
from statistics import median
BANDA = {"arith": 0.80, "prop": 3.33, "calls": 0.50, "str": 7.50, "arr": 6.67}
BANDA_RE_MICRO_N2 = 10.00  # criterio p.4: sostituisce banda-v2 re 0,00
rows = {}; NN = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    NN[c] = n
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
ok_guardie = True; ok_bersaglio = False
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    na = [r[1] for r in rows[c]]; nb = [r[2] for r in rows[c]]
    tA, tB = 1.3*min(na), 1.3*min(nb)
    fam, escl = [], []
    for i, a, b in rows[c]:
        if a <= tA and b <= tB: fam.append((i, a, b))
        else: escl.append((i, a, b, "A" if a > tA else "B"))
    for i, a, b, lato in escl:
        print(f"{c}: ESCLUSA coppia{i} lato {lato} (A={a:.2f} B={b:.2f}; fam A<={tA:.2f} B<={tB:.2f})")
    As = [a for _, a, _ in fam]; Ds = [a-b for _, a, b in fam]
    dmed = round(median(Ds), 2); pos = sum(1 for d in Ds if round(d, 2) > 0)
    sa = round(max(As)-min(As), 2)
    quanto = round(0.01/NN[c]*1e9, 2)
    if c == "re":
        soglia = round(max(BANDA_RE_MICRO_N2, 2*sa, 2*quanto), 2)
        ok_bersaglio = dmed >= soglia
        print(f"re: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={sa:.2f} "
              f"quanto={quanto:.2f} soglia_promozione=+{soglia:.2f} -> {'PROMOZIONE' if ok_bersaglio else 'SOTTO SOGLIA'}")
    else:
        soglia = round(-max(2*sa, BANDA[c], 2*quanto), 2)
        sf = dmed < soglia
        ok_guardie = ok_guardie and not sf
        reg = " [REGISTRATA: morso dentro banda]" if (c == "calls" and -0.50 <= dmed < 0) else ""
        print(f"{c}: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A_dep={sa:.2f} "
              f"quanto={quanto:.2f} soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'} (tie: = -> tiene){reg}")
if not ok_guardie:
    print("VERDETTO_MICRO=GUARDIA SFONDATA: bisezione pre-registrata (stacca d, poi c)")
    print("RCV=1")
elif ok_bersaglio:
    print("VERDETTO_MICRO=BERSAGLIO re OLTRE SOGLIA + GUARDIE TENGONO (si procede ai gate di promozione)")
    print("RCV=0")
else:
    print("VERDETTO_MICRO=SOTTO SOGLIA (guardie tengono): leva NON promossa sul tempo — vale feedback-keep-partial-wins")
    print("RCV=4")
PY
sed '/^RCV=/d' "$OUT/full-verdetto.txt"
FR=$(awk -F= '/^RCV=/{print $2}' "$OUT/full-verdetto.txt")
echo "$FR" > "$OUT/full-rc"
{
  echo ""
  echo "VERDETTO A MACCHINA (s120-ab-re1.sh full, rc=$FR da re1-out/full-rc):"
  sed '/^RCV=/d' "$OUT/full-verdetto.txt"
} >> "$VERD"
exit "$FR"
