#!/bin/bash
# s125-layout-band.sh — criterio s125-criterio-banda.md p.3-4: SOLA misura.
# K=5 binari (P0 pin s124, P0b copia byte-id, P1..P3 probe s125), ordine
# PERMUTATO (latino ciclico 5x5), R=5, timer ucpu.py, N scalati wp123.
# Uscite: BANDA_V2, PAV_PIN, SOGLIA_LAYOUT per categoria + posizioni.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
H123="$REPO/wp123-harness"
SC="$H123/scaled"
S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
OUT="$H/banda-out"; mkdir -p "$OUT"
VERD="$H/s125-banda-verdetto.out"
TSV="$OUT/layout-v2-runs.tsv"; : > "$TSV"
BANDE="$OUT/layout-bande-v2.txt"
PIN_EXP16="c5ba2573a23adf69"

[ "$(cat "$OUT/build.rc" 2>/dev/null)" = 0 ] || { echo "build.rc != 0 — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
. "$OUT/layout-hashes.env" || { echo "layout-hashes.env ASSENTE — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }

declare -a B EXP
B=("$S/phpr-s124" "$S/phpr-s125-p0b" "$S/phpr-s125-lay1" "$S/phpr-s125-lay2" "$S/phpr-s125-lay3")
EXP=("$PIN_EXP16" "$PIN_EXP16" "$P1_HASH" "$P2_HASH" "$P3_HASH")

cp "${B[0]}" "${B[1]}" || { echo "copia P0b FALLITA — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
chmod +x "${B[1]}"
for k in 0 1 2 3 4; do
  hh=$(shasum -a 256 "${B[$k]}" | cut -c1-16)
  [ "$hh" = "${EXP[$k]}" ] || { echo "B$k hash $hh != ${EXP[$k]} — STOP" | tee -a "$VERD"; echo 1 > "$OUT/band.rc"; exit 1; }
done
echo "== s125 banda-LAYOUT v2 POST-PATCH: P0/P0b=${PIN_EXP16:0:8} P1=${P1_HASH:0:8} P2=${P2_HASH:0:8} P3=${P3_HASH:0:8} R=5 latino-ciclico timer=getrusage-µs ==" >> "$VERD"

u() { python3 "$H123/ucpu.py" "$1" "$2"; }
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
