#!/bin/bash
# s171-ab-leva.sh <APATH> <AEXP8> <BPATH> <BEXP8> <CPATH> <CEXP8> <TAG> [R] — leva L-SL1 «forma sigillata Long» (S-171, criterio p.2-3):
# DUE GIUDICI BERSAGLIO: arith-dq E arith-e2 (N LETTO DAL DRIVER, az.rev. S-170 #4); bracci A=pin s166, B=candidato,
# C=tetto m13 (binario di ALTRO albero: contrasto B−C = DIREZIONE+BANDA dichiarata, mai magnitudine).
# COPIA DICHIARATA di wp170-harness/s170-ab-dq.sh (manifest s171-ab-leva-copia-v3.diff + copia-gate v3 PER TOKEN)
# coi SOLI adattamenti del criterio: braccio C, N dal driver, e2 promosso da guardia a giudice (ABAB anche su e2),
# nessuna guardia (entrambi bersaglio), kill/bande p.3. Lock per TOKEN (`grep -qw s171`); `[ -s ]` sui driver e sugli
# attesi ([ -e ] sul pavimento, VUOTO per costruzione); parità di A, B e C contro l'output ATTESO dell'ORACLE.
# R=5; ns/iter=(med raw−floor)/N; rumore drop-1. rc autoritativo = ab-out/<TAG>.rc; verdetto s171-<TAG>-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; BB="${3:?BPATH}"; BEXP="${4:?BEXP8}"; CC="${5:?CPATH}"; CEXP="${6:?CEXP8}"; TAG="${7:?TAG}"; R="${8:-5}"
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
E2="$H/../wp168-harness/arith-e2.php"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s171-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
for f in "$DQ" "$E2" "$A" "$BB" "$CC" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ], emenda S-170 p.4)" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
grep -qw s171 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s171 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"; CM="$(shasum -a 256 "$CC" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$CM" = "$CEXP" ] || { echo "C misurato $CM != atteso $CEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
# N del giudice EMESSO dal sorgente (KS-GR-105-2; az.rev. S-170 #4): mai cablato
NDQ=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$DQ")
NE2=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$E2")
[ -n "$NDQ" ] && [ -n "$NE2" ] || { echo "N non leggibile dal driver (dq='$NDQ' e2='$NE2')" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s171 LEVA $TAG — A=$AM MISURATO ($A) B=$BM MISURATO ($BB) C=$CM MISURATO ($CC, tetto m13); GIUDICI arith-dq N=$NDQ E arith-e2 N=$NE2 (N dal driver); R=$R ABAB alternato su ogni giudice; criterio s171-criterio.md =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
# PARITÀ vs ATTESO: l'atteso è l'output dell'ORACLE sullo stesso driver (generato qui, una volta per driver, `[ -s ]`).
for D in "$DQ" "$E2"; do
  n="$(basename "$D" .php)"; EXP="$OUT/expected-$n.out"
  [ -s "$EXP" ] || "$O" "$D" > "$EXP" 2>&1
  [ -s "$EXP" ] || { echo "atteso VUOTO per $n — STOP"; echo 2 > "$RC"; exit 2; }
  "$A" "$D" > "$OUT/$TAG-A-$n.out" 2>&1; "$BB" "$D" > "$OUT/$TAG-B-$n.out" 2>&1; "$CC" "$D" > "$OUT/$TAG-C-$n.out" 2>&1
  cmp -s "$OUT/$TAG-A-$n.out" "$EXP" || { echo "output A ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-A-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  cmp -s "$OUT/$TAG-B-$n.out" "$EXP" || { echo "output B ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-B-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  cmp -s "$OUT/$TAG-C-$n.out" "$EXP" || { echo "output C ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-C-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  echo "parità $n: A==B==C==atteso oracle ($(tr '\n' ' ' < "$EXP"))"
done
FA=$(floor3 "$A"); FB=$(floor3 "$BB"); FC=$(floor3 "$CC"); FO=$(floor3 "$O")
echo "floors: A=$FA B=$FB C=$FC oracle=$FO"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
for i in $(seq 1 "$R"); do
  if [ $((i % 2)) -eq 1 ]; then
    TA=$(ucpu "$A" "$DQ"); TB=$(ucpu "$BB" "$DQ"); EA=$(ucpu "$A" "$E2"); EB=$(ucpu "$BB" "$E2"); ord=AB
  else
    TB=$(ucpu "$BB" "$DQ"); TA=$(ucpu "$A" "$DQ"); EB=$(ucpu "$BB" "$E2"); EA=$(ucpu "$A" "$E2"); ord=BA
  fi
  TC=$(ucpu "$CC" "$DQ"); EC=$(ucpu "$CC" "$E2")
  TOR=$(ucpu "$O" "$DQ"); EOR=$(ucpu "$O" "$E2")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$TA" "$TB" "$EA" "$EB" "$TC" "$EC" "$TOR" "$EOR" >> "$TSV"
  echo "  coppia$i [$ord]: dqA=$TA dqB=$TB e2A=$EA e2B=$EB | dqC=$TC e2C=$EC | dqO=$TOR e2O=$EOR"
done
python3 - "$TSV" "$FA" "$FB" "$FC" "$FO" "$NDQ" "$NE2" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fa, fb, fc, fo = map(float, sys.argv[2:6]); ndq = float(sys.argv[6]); ne2 = float(sys.argv[7])
def col(i, f, n): return sorted((float(r[i])-f)/n*1e9 for r in rows)
def med(v):
    k = len(v); return v[k//2] if k % 2 else (v[k//2-1]+v[k//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
dqA, dqB, e2A, e2B = col(0,fa,ndq), col(1,fb,ndq), col(2,fa,ne2), col(3,fb,ne2)
dqC, e2C, dqO, e2O = col(4,fc,ndq), col(5,fc,ne2), col(6,fo,ndq), col(7,fo,ne2)
def judge(name, A, B, C, O):
    D = med(A)-med(B); noise = max(dr1(A), dr1(B)); thr = max(4.0, noise, 0.94); thrd = max(noise, 0.94)
    print(f"GIUDICE {name}: A={med(A):.2f} B={med(B):.2f} ns/iter D=A−B={D:+.2f} soglia={thr:.2f} (rumore drop-1 A'={dr1(A):.2f} B'={dr1(B):.2f}) -> {'NOMINATO' if D >= thr else 'NON nominato (vale 0)'}; direzione {'FIRMATA' if abs(D) >= thrd else 'sotto il rumore'} ({'B più veloce' if D > 0 else 'B più lento'}); regressione (D < −max(4,rumore)): {'SÌ' if D < -max(4.0, noise) else 'no'}")
    print(f"  tetto C=m13 {name}: C={med(C):.2f}; contrasto B−C={med(B)-med(C):+.2f} (DIREZIONE+BANDA: C è binario di altro albero); oracle={med(O):.2f}; B/oracle={med(B)/med(O):.2f}× (A/oracle {med(A)/med(O):.2f}×, C/oracle {med(C)/med(O):.2f}×)")
    return D, noise, thr, thrd
Ddq, ndq_, thrdq, _ = judge("arith-dq", dqA, dqB, dqC, dqO)
De2, ne2_, thre2, thrde2 = judge("arith-e2", e2A, e2B, e2C, e2O)
bc = med(dqB)-med(dqC)
band = "≤5: tetto riprodotto entro il match per op" if bc <= 5 else ("(5;10]: il match per op pesa — fetta 2 = specializzazione, a verdetto" if bc <= 10 else ">10: la forma NON riproduce il mock — xctrace m0/B/m13 PRIMA di generalizzare")
print(f"BANDA B−C (dq) = {bc:+.2f} -> {band}")
print(f"KILL-1 (D_dq < 10): {'SCATTA — non si generalizza, la fetta torna a R4 con le cifre' if Ddq < 10 else 'NON scatta'}; attesa D_dq ∈ [17;26]: {'CENTRATA' if 17 <= Ddq <= 26 else 'FUORI'} ({Ddq:+.2f}); attesa D_e2 ≈ +4: {De2:+.2f}")
print(f"statement (dq−e2, lettura non giudicante): A={med(dqA)-med(e2A):.2f} B={med(dqB)-med(e2B):.2f} C={med(dqC)-med(e2C):.2f} oracle={med(dqO)-med(e2O):.2f}")
rc = 0
if Ddq < -max(4.0, ndq_) or De2 < -max(4.0, ne2_): rc = 5
elif De2 < -thrde2: rc = 6
elif Ddq < thrdq: rc = 8 if ndq_ > 4.0 else 4
print(f"PROMOZIONE (p.3: D_dq NOMINATA e D_e2 ≥ −rumore): {'AMMESSA' if rc == 0 else 'NON ammessa'}; ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
