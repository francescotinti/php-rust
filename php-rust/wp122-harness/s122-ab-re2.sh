#!/bin/bash
# s122-ab-re2.sh smoke|full — A/B L-RE2 (criterio s122-criterio-re2.md p.5-8):
# BERSAGLIO re (promozione: D_med >= max(4,00; BL_re 5,00; 2×spread_A;
# 2×quanto)); guardie a solo-regressione = −max(2×spread; banda-v2; BL_cat;
# 2×quanto). floor med3 per-binario; famiglia 1,3×min; rc in FILE.
# Smoke R=2 su re+str: early-stop SOLO se concorde <=-1,00 su TUTTE le coppie
# E |D_med| > banda cat (re 10,00; str 5,00).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s120-re1"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s122-re2"
MODE="${1:?smoke|full}"
RBASE=5
if [ "$MODE" = smoke ]; then CATS="re str"; RBASE=2; EXTRA_MAX=0; else CATS="arith prop calls str arr re"; EXTRA_MAX=6; fi
OUT="$H/re2-out"; mkdir -p "$OUT"
VERD="$H/s122-re2-verdetto.out"
TSV="$OUT/re2-$MODE-runs.tsv"; : > "$TSV"
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
# smoke di GUARDIA (criterio p.8): early-stop SOLO se (a) D <= -1.00 su TUTTE
# le coppie della categoria E (b) |D_med| > banda (re: max(BL 5,00; 2×quanto
# 10,00)=10,00; str: max(BL 5,00; zavorra 2,50; 2×quanto 5,00)=5,00).
from collections import defaultdict
from statistics import median
BANDA = {"re": 10.00, "str": 5.00}
d = defaultdict(list)
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    d[c].append(round((ta-fa)/n*1e9-(tb-fb)/n*1e9, 2))
bad = False
for c, v in d.items():
    conc = all(x <= -1.00 for x in v)
    dmed = round(median(v), 2)
    if conc and abs(dmed) > BANDA[c]: bad = True
    if conc and abs(dmed) <= BANDA[c]:
        print(f"# {c}: morso concorde {v} ma D_med={dmed:+.2f} DENTRO banda {BANDA[c]:.2f} — REGISTRATO, giudica il full", file=sys.stderr)
print(1 if bad else 0)
PY
)
  echo "$SRC" > "$OUT/smoke-rc"
  echo "smoke_rc=$SRC (0=si procede al full; 1=early-stop fuori banda)"
  awk -F'\t' '{d=(($4-$6)/$2-($5-$7)/$2)*1e9; printf "%+.2f ", d}' "$TSV" | {
    read -r line; echo "smoke R=2 guardia (rc=$SRC, criterio p.8): D per coppia = $line" >> "$VERD"
  }
  exit "$SRC"
fi

if [ -n "$INVALID" ]; then
  echo "MISURA_INVALIDA:$INVALID — rerun, NESSUN verdetto"; echo 3 > "$OUT/full-rc"
  echo "full: MISURA_INVALIDA:$INVALID (rc=3)" >> "$VERD"; exit 3
fi

python3 - "$TSV" > "$OUT/full-verdetto.txt" <<'PY'
import sys
from statistics import median
# criterio p.6-7: bersaglio re; guardie = -max(2*spread, banda-v2, BL, 2*quanto)
BV2 = {"arith": 0.80, "prop": 3.33, "calls": 0.50, "str": 7.50, "arr": 6.67}
BL  = {"arith": 1.00, "prop": 1.00, "calls": 0.00, "str": 5.00, "arr": 3.33, "re": 5.00}
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
    fam = [(i, a, b) for i, a, b in rows[c] if a <= tA and b <= tB]
    for i, a, b in rows[c]:
        if not (a <= tA and b <= tB):
            print(f"{c}: ESCLUSA coppia{i} (A={a:.2f} B={b:.2f})")
    As = [a for _, a, _ in fam]; Ds = [a-b for _, a, b in fam]
    dmed = round(median(Ds), 2); pos = sum(1 for d in Ds if round(d, 2) > 0)
    sa = round(max(As)-min(As), 2); quanto = round(0.01/NN[c]*1e9, 2)
    if c == "re":
        soglia = round(max(4.00, BL[c], 2*sa, 2*quanto), 2)
        ok_bersaglio = dmed >= soglia
        print(f"re: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A={sa:.2f} "
              f"quanto={quanto:.2f} soglia_promozione=+{soglia:.2f} -> {'PROMOZIONE' if ok_bersaglio else 'SOTTO SOGLIA'}")
    else:
        soglia = round(-max(2*sa, BV2[c], BL[c], 2*quanto), 2)
        sf = dmed < soglia
        ok_guardie = ok_guardie and not sf
        print(f"{c}: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A={sa:.2f} "
              f"quanto={quanto:.2f} soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'} (tie: = -> tiene)")
if not ok_guardie:
    print("VERDETTO_MICRO=GUARDIA SFONDATA: REVERT (leva a UN componente, criterio p.9)"); print("RCV=1")
elif ok_bersaglio:
    print("VERDETTO_MICRO=BERSAGLIO re OLTRE SOGLIA + GUARDIE TENGONO (si procede ai gate p.9)"); print("RCV=0")
else:
    print("VERDETTO_MICRO=SOTTO SOGLIA (guardie tengono): NON promossa — keep-partial-wins"); print("RCV=4")
PY
sed '/^RCV=/d' "$OUT/full-verdetto.txt"
FR=$(awk -F= '/^RCV=/{print $2}' "$OUT/full-verdetto.txt")
echo "$FR" > "$OUT/full-rc"
{
  echo ""
  echo "VERDETTO A MACCHINA (s122-ab-re2.sh full, rc=$FR da re2-out/full-rc):"
  sed '/^RCV=/d' "$OUT/full-verdetto.txt"
} >> "$VERD"
exit "$FR"
