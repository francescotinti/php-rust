#!/bin/bash
# s169-ab-mock.sh <APATH> <AEXP8> <BPATH> <BEXP8> <TAG> [R] — MOCK sottrattivi F0 (S-169: R parametrico, default 5; soglia DECOMPOSIZIONE = max(rumore, SL 0,94) stampata ACCANTO al pavimento 4 — az.rev.1 S-168)
# (criterio s169-criterio-mock.md): A = braccio di riferimento (pin s166 dallo
# stash, o m0 «braccio nullo» stessa ricetta/copia — EMENDA: m0 36d73812 != pin
# perché i path della copia entrano nel binario) vs B = mock; hash MISURATI;
# giudice arith-dq N=250M; braccio E2 «loop nudo» bilaterale (A, B, oracle).
# COPIA DICHIARATA di wp167-harness/s167-f0ab.sh (manifest s169-ab-mock-copia.diff
# + copia-gate v2) coi SOLI adattamenti del criterio: due binari (A stash, B
# mock, hash MISURATI), ordine ABAB alternato, colonne e2, verdetto per TAG.
# R=5 interleaved per coppia; ns/iter=(med raw−floor)/N; rumore drop-1.
# rc autoritativo = ab-out/<TAG>.rc; verdetto s169-<TAG>-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; BB="${3:?BPATH}"; BEXP="${4:?BEXP8}"; TAG="${5:?TAG}"; R="${6:-5}"
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
E2="$H/arith-e2.php"
EMPTY="$H/../wp164-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s169-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
grep -qi "s169\|s-168" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s169 MOCK $TAG — A=$AM MISURATO ($A) B=$BM MISURATO ($BB); giudice arith-dq N=250M + E2 loop nudo bilaterale; R=$R ABAB alternato; criterio s169-criterio.md =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
for D in "$DQ" "$E2"; do
  "$A" "$D" > "$OUT/$TAG-A.out" 2>&1; "$BB" "$D" > "$OUT/$TAG-B.out" 2>&1
  diff -q "$OUT/$TAG-A.out" "$OUT/$TAG-B.out" > /dev/null || { echo "output DIVERGE su $(basename "$D") — STOP"; echo 2 > "$RC"; exit 2; }
done
FA=$(floor3 "$A"); FB=$(floor3 "$BB"); FO=$(floor3 "$O")
echo "floors: A=$FA B=$FB oracle=$FO"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
for i in $(seq 1 "$R"); do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$DQ"); TB=$(ucpu "$BB" "$DQ"); ord=AB; else TB=$(ucpu "$BB" "$DQ"); TA=$(ucpu "$A" "$DQ"); ord=BA; fi
  EA=$(ucpu "$A" "$E2"); EB=$(ucpu "$BB" "$E2")
  TOR=$(ucpu "$O" "$DQ"); EOR=$(ucpu "$O" "$E2")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$TA" "$TB" "$EA" "$EB" "$TOR" "$EOR" >> "$TSV"
  echo "  coppia$i [$ord]: dqA=$TA dqB=$TB e2A=$EA e2B=$EB dqO=$TOR e2O=$EOR"
done
python3 - "$TSV" "$FA" "$FB" "$FO" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fa, fb, fo = map(float, sys.argv[2:5])
N = 250e6
def col(i, f): return sorted((float(r[i])-f)/N*1e9 for r in rows)
def med(v):
    n = len(v); return v[n//2] if n % 2 else (v[n//2-1]+v[n//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
dqA, dqB, e2A, e2B, dqO, e2O = col(0,fa), col(1,fb), col(2,fa), col(3,fb), col(4,fo), col(5,fo)
D = med(dqA)-med(dqB); noise = max(dr1(dqA), dr1(dqB)); thr = max(4.0, noise, 0.94); thrd = max(noise, 0.94)
print(f"GIUDICE arith-dq: A={med(dqA):.2f} B={med(dqB):.2f} ns/iter D=A−B={D:+.2f} soglia={thr:.2f} (rumore drop-1 A'={dr1(dqA):.2f} B'={dr1(dqB):.2f}) -> {'NOMINATO' if D >= thr else 'NON nominato (vale 0)'}")
print(f"DECOMPOSIZIONE (az.rev.1 S-168, soglia = max(rumore, SL 0,94) SENZA pavimento 4): soglia_dec={thrd:.2f} -> {'DIREZIONE FIRMATA' if abs(D) >= thrd else 'sotto il rumore'} ({'B più veloce' if D > 0 else 'B più lento'}; R={len(rows)})")
De2 = med(e2B)-med(e2A); ne2 = max(dr1(e2A), dr1(e2B)); g = max(4.0, ne2)
print(f"E2 loop nudo: A={med(e2A):.2f} B={med(e2B):.2f} oracle={med(e2O):.2f} ns/iter; guardia |e2B−e2A|={abs(De2):.2f} ≤ {g:.2f} -> {'ok' if abs(De2) <= g else 'MORDE (rc=5, D sospeso)'}")
stA, stO = med(dqA)-med(e2A), med(dqO)-med(e2O)
print(f"statement (dq−e2): A={stA:.2f} oracle={stO:.2f} gap-statement={stA-stO:.2f}; controllo-loop: A={med(e2A):.2f} oracle={med(e2O):.2f} gap-loop={med(e2A)-med(e2O):.2f}; riferimento finestra: dq oracle={med(dqO):.2f} gap totale={med(dqA)-med(dqO):.2f}")
rc = 0
if abs(De2) > g: rc = 5
elif D < thr: rc = 8 if noise > 4.0 else 4
print(f"ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
