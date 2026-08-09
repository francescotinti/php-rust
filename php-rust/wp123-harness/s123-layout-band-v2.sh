#!/bin/bash
# s123-layout-band-v2.sh — criterio s123-criterio-metro.md p.3-4: SOLA misura.
# K=5 binari (P0 pin, P0b copia byte-id, P1..P3 probe s122), ordine PERMUTATO
# per round (quadrato latino ciclico 5x5), R=5, timer ucpu.py (getrusage µs),
# N scalati. Uscite: BANDA_V2, PAV_PIN, SOGLIA_LAYOUT per categoria + posizioni.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
SC="$H/scaled"
S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
OUT="$H/metro-out"; mkdir -p "$OUT"
VERD="$H/s123-metro-verdetto.out"
TSV="$OUT/layout-v2-runs.tsv"; : > "$TSV"
BANDE="$OUT/layout-bande-v2.txt"
PIN_EXP16="885d2c646ac7ff4c"

declare -a B EXP
B=("$S/phpr-s120-re1" "$S/phpr-s123-p0b" "$S/phpr-s122-lay1" "$S/phpr-s122-lay2" "$S/phpr-s122-lay3")
EXP=(885d2c64 885d2c64 29565241 dc98d1b1 f3bc390e)

cp "${B[0]}" "${B[1]}" || { echo "copia P0b FALLITA — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
chmod +x "${B[1]}"
[ "$(shasum -a 256 "${B[1]}" | cut -c1-16)" = "$PIN_EXP16" ] || {
  echo "P0b hash != pin — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
for k in 0 1 2 3 4; do
  hh=$(shasum -a 256 "${B[$k]}" | cut -c1-8)
  [ "$hh" = "${EXP[$k]}" ] || { echo "B$k hash $hh != ${EXP[$k]} — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
done
echo "== s123 banda-LAYOUT v2: P0/P0b=885d2c64 P1=29565241 P2=dc98d1b1 P3=f3bc390e R=5 latino-ciclico timer=getrusage-µs ==" >> "$VERD"

u() { python3 "$H/ucpu.py" "$1" "$2"; }
floor3() {
  local f1 f2 f3
  f1=$(u "$1" "$SC/empty.php"); f2=$(u "$1" "$SC/empty.php"); f3=$(u "$1" "$SC/empty.php")
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
n_of() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; esac; }

declare -a FL
for k in 0 1 2 3 4; do FL[$k]=$(floor3 "${B[$k]}"); echo "floor_B$k(med3)=${FL[$k]}"; done

for C in arith prop calls str arr re; do
  N=$(n_of "$C")
  echo "cat=$C N=$N"
  for r in 1 2 3 4 5; do
    for j in 0 1 2 3 4; do
      k=$(( (r-1+j) % 5 ))
      T=$(u "${B[$k]}" "$SC/$C.php")
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$k" "$r" "$j" "$N" "$T" "${FL[$k]}" >> "$TSV"
    done
  done
done

python3 - "$TSV" "$BANDE" > "$OUT/band-report.txt" <<'PY'
import sys
from statistics import median
BINNAME = {0: "P0", 1: "P0b", 2: "P1", 3: "P2", 4: "P3"}
rows, posrows = {}, {}
for line in open(sys.argv[1]):
    c, b, r, j, n, t, f = line.split('\t')
    net = (float(t)-float(f))/float(n)*1e9
    rows.setdefault(c, {}).setdefault(int(b), []).append(net)
    posrows.setdefault(c, {}).setdefault(int(j), []).append(net)
out = []
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    meds = {}
    for b, ns in sorted(rows[c].items()):
        tfam = 1.3*min(ns)
        fam = [x for x in ns if x <= tfam]
        meds[b] = median(fam)
        print(f"{c} {BINNAME[b]}: net/iter = {' '.join(f'{x:.2f}' for x in ns)} | fam={len(fam)}/5 med_fam={meds[b]:.2f}")
    banda = round(max(meds[b] for b in (0, 2, 3, 4)) - min(meds[b] for b in (0, 2, 3, 4)), 2)
    pav = round(abs(meds[0]-meds[1]), 2)
    soglia = round(max(banda, pav), 2)
    pos = {j: round(median(v), 2) for j, v in sorted(posrows[c].items())}
    print(f"{c}: mediane P0 P0b P1 P2 P3 = {' '.join(f'{meds[b]:.2f}' for b in (0,1,2,3,4))}")
    print(f"{c}: BANDA_V2={banda:.2f} PAV_PIN={pav:.2f} SOGLIA_LAYOUT={soglia:.2f} ns/iter | med per POSIZIONE 0..4 = {' '.join(f'{pos[j]:.2f}' for j in range(5))}")
    out.append((c, banda, pav, soglia))
with open(sys.argv[2], "w") as fh:
    for c, banda, pav, soglia in out:
        fh.write(f"BANDA_V2 {c} {banda:.2f}\nPAV_PIN {c} {pav:.2f}\nSOGLIA_LAYOUT {c} {soglia:.2f}\n")
PY
prc=$?
cat "$OUT/band-report.txt" | tee -a "$VERD"
echo "$prc" > "$OUT/band.rc"
exit "$prc"
