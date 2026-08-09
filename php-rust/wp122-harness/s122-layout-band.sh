#!/bin/bash
# s122-layout-band.sh — criterio s122-criterio-layout.md p.3-4: SOLA misura.
# 4 layout (P0=pin + P1..P3 probe) interleaved ×R=5 sulle 6 categorie; floor
# med3 per-binario; famiglia 1,3×min per binario; BANDA_LAYOUT(cat) = max−min
# delle 4 mediane di famiglia. Righe macchina in layout-out/layout-bande.txt.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp122-harness"
M="$REPO/wp97-harness/micro"
S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
OUT="$H/layout-out"; mkdir -p "$OUT"
VERD="$H/s122-layout-verdetto.out"
TSV="$OUT/layout-runs.tsv"; : > "$TSV"
BANDE="$OUT/layout-bande.txt"
PIN_EXP="885d2c646ac7ff4c"
B0="$S/phpr-s120-re1"; B1="$S/phpr-s122-lay1"; B2="$S/phpr-s122-lay2"; B3="$S/phpr-s122-lay3"

HP0=$(shasum -a 256 "$B0" | cut -c1-16)
if [ "$HP0" != "$PIN_EXP" ]; then
  echo "P0=$HP0 != pin $PIN_EXP — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1
fi
for b in 1 2 3; do
  eval 'BB=$B'"$b"
  [ -x "$BB" ] || { echo "P$b assente — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
done
{
  echo "== s122 banda-LAYOUT: P0=$(shasum -a 256 "$B0"|cut -c1-8) P1=$(shasum -a 256 "$B1"|cut -c1-8) P2=$(shasum -a 256 "$B2"|cut -c1-8) P3=$(shasum -a 256 "$B3"|cut -c1-8) R=5 interleaved =="
} >> "$VERD"

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() {
  local f1 f2 f3; f1=$(user_cpu "$1" "$M/empty.php"); f2=$(user_cpu "$1" "$M/empty.php"); f3=$(user_cpu "$1" "$M/empty.php")
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}

declare -a FL
for b in 0 1 2 3; do
  eval 'BB=$B'"$b"
  FL[$b]=$(floor3 "$BB")
  echo "floor_P$b(med3)=${FL[$b]}"
done

for C in arith prop calls str arr re; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  [ "$C" = arr ] && N=6000000
  echo "cat=$C N=$N"
  for r in 1 2 3 4 5; do
    for b in 0 1 2 3; do
      eval 'BB=$B'"$b"
      T=$(user_cpu "$BB" "$M/$C.php")
      printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$b" "$r" "$N" "$T" "${FL[$b]}" >> "$TSV"
    done
  done
done

python3 - "$TSV" "$BANDE" > "$OUT/band-report.txt" <<'PY'
import sys
from statistics import median
rows = {}
for line in open(sys.argv[1]):
    c, b, r, n, t, f = line.split('\t')
    rows.setdefault(c, {}).setdefault(int(b), []).append((float(t)-float(f))/float(n)*1e9)
bande = []
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    meds = {}
    for b, ns in sorted(rows[c].items()):
        tfam = 1.3*min(ns)
        fam = [x for x in ns if x <= tfam]
        meds[b] = median(fam)
        print(f"{c} P{b}: net/iter = {' '.join(f'{x:.2f}' for x in ns)} | fam={len(fam)}/5 med_fam={meds[b]:.2f}")
    vals = list(meds.values())
    banda = round(max(vals)-min(vals), 2)
    bande.append((c, banda))
    print(f"{c}: mediane P0..P3 = {' '.join(f'{v:.2f}' for v in vals)}  BANDA_LAYOUT = {banda:.2f} ns/iter")
with open(sys.argv[2], "w") as fh:
    for c, banda in bande:
        fh.write(f"BANDA_LAYOUT {c} {banda:.2f}\n")
PY
prc=$?
cat "$OUT/band-report.txt" | tee -a "$VERD"
echo "$prc" > "$OUT/band.rc"
