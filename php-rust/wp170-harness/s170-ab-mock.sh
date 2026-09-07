#!/bin/bash
# s170-ab-mock.sh <APATH> <AEXP8> <BPATH> <BEXP8> <TAG> [R] — MOCK «handler banale magro» (S-170, criterio s170-criterio.md p.2):
# GIUDICE = arith-e2 (loop nudo: CmpJmpSC + IncDecSlotJmp, N=250M); GUARDIA = arith-dq bilaterale a SOLA-REGRESSIONE.
# A = braccio di riferimento (m0 ricostruito in S-170, hash MISURATO) vs B = mock (m8/m9); hash MISURATI.
# COPIA DICHIARATA di wp169-harness/s169-ab-mock.sh (manifest s170-ab-mock-copia-v3.diff
# + copia-gate v3 PER TOKEN) coi SOLI adattamenti del criterio: giudice E2 ↔ guardia dq
# scambiati; lock per TOKEN (`grep -qw s170`); `[ -s ]` sui driver e sugli attesi;
# parità di A e B contro l'output ATTESO dell'ORACLE (non solo A==B; az.rev. S-169 #3).
# R=5 interleaved per coppia (ABAB sul giudice E2); ns/iter=(med raw−floor)/N; rumore drop-1.
# rc autoritativo = ab-out/<TAG>.rc; verdetto s170-<TAG>-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; BB="${3:?BPATH}"; BEXP="${4:?BEXP8}"; TAG="${5:?TAG}"; R="${6:-5}"
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
E2="$H/../wp168-harness/arith-e2.php"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s170-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
for f in "$DQ" "$E2" "$A" "$BB" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ], emenda S-170 p.4)" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
grep -qw s170 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s170 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s170 MOCK $TAG — A=$AM MISURATO ($A) B=$BM MISURATO ($BB); GIUDICE arith-e2 N=250M (loop nudo) + GUARDIA arith-dq bilaterale; R=$R ABAB alternato; criterio s170-criterio.md =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
# PARITÀ vs ATTESO: l'atteso è l'output dell'ORACLE sullo stesso driver (generato qui, una volta per driver, `[ -s ]`).
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
  TOR=$(ucpu "$O" "$DQ"); EOR=$(ucpu "$O" "$E2")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$TA" "$TB" "$EA" "$EB" "$TOR" "$EOR" >> "$TSV"
  echo "  coppia$i [$ord su E2]: e2A=$EA e2B=$EB dqA=$TA dqB=$TB e2O=$EOR dqO=$TOR"
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
D = med(e2A)-med(e2B); noise = max(dr1(e2A), dr1(e2B)); thr = max(4.0, noise, 0.94); thrd = max(noise, 0.94)
print(f"GIUDICE arith-e2 (loop nudo, 2 op/iter): A={med(e2A):.2f} B={med(e2B):.2f} ns/iter D=A−B={D:+.2f} soglia={thr:.2f} (rumore drop-1 A'={dr1(e2A):.2f} B'={dr1(e2B):.2f}) -> {'NOMINATO' if D >= thr else 'NON nominato (vale 0)'}")
print(f"DECOMPOSIZIONE (soglia = max(rumore, SL 0,94) SENZA pavimento 4): soglia_dec={thrd:.2f} -> {'DIREZIONE FIRMATA' if abs(D) >= thrd else 'sotto il rumore'} ({'B più veloce' if D > 0 else 'B più lento'}; R={len(rows)})")
print(f"per-op (2 op/iter): A={med(e2A)/2:.2f} B={med(e2B)/2:.2f} oracle={med(e2O)/2:.2f} ns/op; residuo B a dispatch 1,75/op (limite inferiore S-169) = corpo magro ≈ {med(e2B)/2-1.75:.2f} ns/op")
Ddq = med(dqB)-med(dqA); ndq = max(dr1(dqA), dr1(dqB)); g = max(4.0, ndq)
print(f"GUARDIA arith-dq (sola-regressione): A={med(dqA):.2f} B={med(dqB):.2f} oracle={med(dqO):.2f} ns/iter; dqB−dqA={Ddq:+.2f} vs {g:.2f} -> {'ok' if Ddq <= g else 'REGRESSIONE (rc=5, D sospeso)'}{'; miglioramento su dq REGISTRATO (stessi 2 op nel loop), non contato' if Ddq < -thrd else ''}")
print(f"riferimento finestra: gap-loop A={med(e2A)-med(e2O):.2f} B={med(e2B)-med(e2O):.2f}; dq gap totale A={med(dqA)-med(dqO):.2f} B={med(dqB)-med(dqO):.2f}")
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
