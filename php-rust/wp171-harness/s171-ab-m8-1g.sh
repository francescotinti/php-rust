#!/bin/bash
# s171-ab-m8-1g.sh <APATH> <AEXP8> <BPATH> <BEXP8> <TAG> [R] — az.rev. S-170 #4: m8 RIMISURATO a N=1G (tick 0,01 ns/iter,
# `time -p` a 0,01 s) con N LETTO DAL DRIVER; A = m0 (stash phpr-s168-m0 36d73812 == m0 S-170 al byte) vs B = m8 (stash phpr-s170-m8).
# GIUDICE = arith-e2-1g (wp171-harness/arith-e2-1g.php, derivato dichiarato di arith-e2 con N=1e9); GUARDIA = arith-dq (N dal driver)
# bilaterale a SOLA-REGRESSIONE. COPIA DICHIARATA di wp170-harness/s170-ab-mock.sh (manifest s171-ab-m8-1g-copia-v3.diff + copia-gate v3
# PER TOKEN) coi SOLI adattamenti: driver del giudice, N dal driver (due N), lock per TOKEN s171. Parità di A e B contro l'ATTESO oracle.
# R=5 interleaved per coppia (ABAB sul giudice); ns/iter=(med raw−floor)/N; rumore drop-1. rc autoritativo = ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; BB="${3:?BPATH}"; BEXP="${4:?BEXP8}"; TAG="${5:?TAG}"; R="${6:-5}"
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
E2="$H/arith-e2-1g.php"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s171-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
for f in "$DQ" "$E2" "$A" "$BB" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ])" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
grep -qw s171 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s171 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
NDQ=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$DQ")
NE2=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$E2")
[ -n "$NDQ" ] && [ -n "$NE2" ] || { echo "N non leggibile dal driver (dq='$NDQ' e2='$NE2')" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s171 az.rev.#4 $TAG — A=$AM MISURATO ($A) B=$BM MISURATO ($BB); GIUDICE arith-e2-1g N=$NE2 (loop nudo, N dal driver) + GUARDIA arith-dq N=$NDQ bilaterale; R=$R ABAB alternato =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
for D in "$DQ" "$E2"; do
  n="$(basename "$D" .php)"; EXP="$OUT/expected-$n.out"
  [ -s "$EXP" ] || "$O" "$D" > "$EXP" 2>&1
  [ -s "$EXP" ] || { echo "atteso VUOTO per $n — STOP"; echo 2 > "$RC"; exit 2; }
  "$A" "$D" > "$OUT/$TAG-A-$n.out" 2>&1; "$BB" "$D" > "$OUT/$TAG-B-$n.out" 2>&1
  cmp -s "$OUT/$TAG-A-$n.out" "$EXP" || { echo "output A ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-A-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  cmp -s "$OUT/$TAG-B-$n.out" "$EXP" || { echo "output B ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-B-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  echo "parità $n: A==B==atteso oracle ($(tr '\n' ' ' < "$EXP"))"
done
FA=$(floor3 "$A"); FB=$(floor3 "$BB"); FO=$(floor3 "$O")
echo "floors: A=$FA B=$FB oracle=$FO"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
for i in $(seq 1 "$R"); do
  if [ $((i % 2)) -eq 1 ]; then EA=$(ucpu "$A" "$E2"); EB=$(ucpu "$BB" "$E2"); ord=AB; else EB=$(ucpu "$BB" "$E2"); EA=$(ucpu "$A" "$E2"); ord=BA; fi
  TA=$(ucpu "$A" "$DQ"); TB=$(ucpu "$BB" "$DQ")
  EOR=$(ucpu "$O" "$E2"); TOR=$(ucpu "$O" "$DQ")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$TA" "$TB" "$EA" "$EB" "$TOR" "$EOR" >> "$TSV"
  echo "  coppia$i [$ord su E2-1g]: e2A=$EA e2B=$EB dqA=$TA dqB=$TB e2O=$EOR dqO=$TOR"
done
python3 - "$TSV" "$FA" "$FB" "$FO" "$NDQ" "$NE2" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fa, fb, fo = map(float, sys.argv[2:5]); ndq = float(sys.argv[5]); ne2 = float(sys.argv[6])
def col(i, f, n): return sorted((float(r[i])-f)/n*1e9 for r in rows)
def med(v):
    k = len(v); return v[k//2] if k % 2 else (v[k//2-1]+v[k//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
dqA, dqB, e2A, e2B, dqO, e2O = col(0,fa,ndq), col(1,fb,ndq), col(2,fa,ne2), col(3,fb,ne2), col(4,fo,ndq), col(5,fo,ne2)
D = med(e2A)-med(e2B); noise = max(dr1(e2A), dr1(e2B)); thr = max(4.0, noise, 0.94); thrd = max(noise, 0.94)
print(f"GIUDICE arith-e2-1g (loop nudo, 2 op/iter, tick 0,01): A={med(e2A):.3f} B={med(e2B):.3f} ns/iter D=A−B={D:+.3f} soglia={thr:.2f} (rumore drop-1 A'={dr1(e2A):.3f} B'={dr1(e2B):.3f}) -> {'NOMINATO' if D >= thr else 'NON nominato (vale 0)'}; S-170 a N=250M: D=+4,08 a filo (2 tick) -> {'CONFERMATO oltre il tick' if D >= 4.0 else 'NON confermato'}")
print(f"DECOMPOSIZIONE: soglia_dec={thrd:.2f} -> {'DIREZIONE FIRMATA' if abs(D) >= thrd else 'sotto il rumore'} ({'B più veloce' if D > 0 else 'B più lento'}; R={len(rows)})")
print(f"per-op (2 op/iter): A={med(e2A)/2:.3f} B={med(e2B)/2:.3f} oracle={med(e2O)/2:.3f} ns/op")
Ddq = med(dqB)-med(dqA); ndq_ = max(dr1(dqA), dr1(dqB)); g = max(4.0, ndq_)
print(f"GUARDIA arith-dq (sola-regressione): A={med(dqA):.2f} B={med(dqB):.2f} oracle={med(dqO):.2f} ns/iter; dqB−dqA={Ddq:+.2f} vs {g:.2f} -> {'ok' if Ddq <= g else 'REGRESSIONE (rc=5, D sospeso)'}{'; miglioramento su dq REGISTRATO, non contato' if Ddq < -thrd else ''}")
rc = 0
if Ddq > g: rc = 5
elif D < thr: rc = 8 if noise > 4.0 else 4
print(f"ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
