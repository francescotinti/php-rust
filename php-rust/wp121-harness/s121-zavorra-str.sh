#!/bin/bash
# s121-zavorra-str.sh — zavorra N=3 sulla categoria str (debito NEXT_SESSION:
# «str/arr bande larghe 7,50/6,67: zavorre N≥3 prima di leve str/arr»).
# 3 blocchi INDIPENDENTI sul pin s120 (stash phpr-s120-re1 == pin al byte):
# ogni blocco = floor med3 su empty + R=5 su str.php; net = (T−F)/N·1e9;
# mediana di blocco sulla famiglia 1,3×min (forma s119-ab). banda-zavorra-str
# = max−min delle 3 mediane di blocco. SOLO misura: nessuna leva, nessun edit.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp121-harness"
M="$REPO/wp97-harness/micro"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s120-re1"
OUT="$H/zavorra-out"; mkdir -p "$OUT"
TSV="$OUT/zavorra-str.tsv"; : > "$TSV"
VERD="$OUT/zavorra-verdetto.txt"
PIN_EXP="885d2c646ac7ff4c"
HA=$(shasum -a 256 "$A" | cut -c1-16)
if [ "$HA" != "$PIN_EXP" ]; then
  echo "stash phpr-s120-re1=$HA != pin s120 $PIN_EXP — STOP" | tee "$VERD"
  echo 1 > "$OUT/zavorra.rc"; exit 1
fi
N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/str.php")

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() {
  local f1 f2 f3; f1=$(user_cpu "$A" "$M/empty.php"); f2=$(user_cpu "$A" "$M/empty.php"); f3=$(user_cpu "$A" "$M/empty.php")
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}

for b in 1 2 3; do
  F=$(floor3)
  echo "blocco $b floor(med3)=$F"
  for i in 1 2 3 4 5; do
    T=$(user_cpu "$A" "$M/str.php")
    printf '%s\t%s\t%s\t%s\t%s\n' "$b" "$i" "$N" "$T" "$F" >> "$TSV"
  done
done

python3 - "$TSV" > "$VERD" <<'PY'
import sys
from statistics import median
blocks = {}
for line in open(sys.argv[1]):
    b, i, n, t, f = line.split('\t')
    blocks.setdefault(int(b), []).append((float(t)-float(f))/float(n)*1e9)
meds = {}
for b, ns in sorted(blocks.items()):
    tfam = 1.3*min(ns)
    fam = [x for x in ns if x <= tfam]
    escl = len(ns)-len(fam)
    meds[b] = round(median(fam), 2)
    print(f"blocco{b}: net/iter = {' '.join(f'{x:.2f}' for x in ns)} | fam={len(fam)}/5"
          f"{f' (escluse {escl})' if escl else ''} med_fam={meds[b]:.2f}")
vals = list(meds.values())
banda = round(max(vals)-min(vals), 2)
print(f"mediane di blocco: {' '.join(f'{v:.2f}' for v in vals)}")
print(f"BANDA_ZAVORRA_STR(N=3) = {banda:.2f} ns/iter (banda-v2 str dichiarata: 7,50)")
PY
zrc=$?; echo "$zrc" > "$OUT/zavorra.rc"
cat "$VERD"
