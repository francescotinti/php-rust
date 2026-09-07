#!/bin/bash
# s170-ab-dq.sh <APATH> <AEXP8> <BPATH> <BEXP8> <TAG> [R] — bracci del RESIDUO di BinarySCSCDst (S-170, criterio-b p.2):
# GIUDICE = arith-dq (N=250M); GUARDIA = arith-e2 bilaterale a SOLA-REGRESSIONE (i bracci non toccano il controllo-loop).
# A = m0 ricostruito in S-170 (hash MISURATO) vs B = mock (m10..m13); hash MISURATI.
# COPIA DICHIARATA di s170-ab-mock.sh (manifest s170-ab-dq-copia-v3.diff + copia-gate v3 PER TOKEN)
# col SOLO adattamento del criterio-b: giudice dq ↔ guardia e2 scambiati (ABAB sul giudice dq).
# Lock per TOKEN (`grep -qw s170`); `[ -s ]` sui driver e sugli attesi ([ -e ] sul pavimento, VUOTO per costruzione);
# parità di A e B contro l'output ATTESO dell'ORACLE. R=5; ns/iter=(med raw−floor)/N; rumore drop-1.
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
echo "== s170 RESIDUO $TAG — A=$AM MISURATO ($A) B=$BM MISURATO ($BB); GIUDICE arith-dq N=250M + GUARDIA arith-e2 bilaterale; R=$R ABAB alternato; criterio s170-criterio-b.md =="
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
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$DQ"); TB=$(ucpu "$BB" "$DQ"); ord=AB; else TB=$(ucpu "$BB" "$DQ"); TA=$(ucpu "$A" "$DQ"); ord=BA; fi
  EA=$(ucpu "$A" "$E2"); EB=$(ucpu "$BB" "$E2")
  TOR=$(ucpu "$O" "$DQ"); EOR=$(ucpu "$O" "$E2")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$TA" "$TB" "$EA" "$EB" "$TOR" "$EOR" >> "$TSV"
  echo "  coppia$i [$ord su dq]: dqA=$TA dqB=$TB e2A=$EA e2B=$EB dqO=$TOR e2O=$EOR"
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
print(f"DECOMPOSIZIONE (soglia = max(rumore, SL 0,94) SENZA pavimento 4): soglia_dec={thrd:.2f} -> {'DIREZIONE FIRMATA' if abs(D) >= thrd else 'sotto il rumore'} ({'B più veloce' if D > 0 else 'B più lento'}; R={len(rows)})")
stA, stB, stO = med(dqA)-med(e2A), med(dqB)-med(e2B), med(dqO)-med(e2O)
print(f"statement (dq−e2): A={stA:.2f} B={stB:.2f} oracle={stO:.2f}; residuo B oltre Sweep 2,9 + dispatch 1,75 (S-169) = corpo BinarySCSCDst B ≈ {stB-2.9-1.75:.2f} (A ≈ {stA-2.9-1.75:.2f})")
De2 = med(e2B)-med(e2A); ne2 = max(dr1(e2A), dr1(e2B)); g = max(4.0, ne2)
print(f"GUARDIA arith-e2 (sola-regressione): A={med(e2A):.2f} B={med(e2B):.2f} oracle={med(e2O):.2f} ns/iter; e2B−e2A={De2:+.2f} vs {g:.2f} -> {'ok' if De2 <= g else 'REGRESSIONE (rc=5, D sospeso)'}{'; variazione firmata su e2 = LAYOUT, dichiarata' if abs(De2) >= thrd else ''}")
print(f"riferimento finestra: dq gap totale A={med(dqA)-med(dqO):.2f} B={med(dqB)-med(dqO):.2f}")
rc = 0
if De2 > g: rc = 5
elif D < thr: rc = 8 if noise > 4.0 else 4
print(f"ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
