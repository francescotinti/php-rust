#!/bin/bash
# s172-ab-leva.sh <APATH> <AEXP8> <BPATH> <BEXP8> <CPATH> <CEXP8> <TAG> [R] — leva L-SL2 «forma sigillata Long» fetta 2 = PROP (S-172, criterio p.3-4):
# GIUDICE BERSAGLIO = prop-dq (N LETTO DAL DRIVER); GUARDIA = arith-dq a SOLA REGRESSIONE (i bracci non toccano BinarySCSCDst);
# bracci A=pin s171, B=P1 (bigramma fuso sigillato), C=P1+P2 (BinarySTDst sigillato) — B e C dallo STESSO albero, commit
# consecutivi: contrasto C−B = DIREZIONE+BANDA dichiarata (contributo di P2), mai magnitudine composta.
# COPIA DICHIARATA di wp171-harness/s171-ab-leva.sh (manifest s172-ab-leva-copia-v3.diff + copia-gate per TOKEN s172)
# coi SOLI adattamenti del criterio: giudice prop-dq al posto di arith-dq, arith-dq da giudice a guardia, bande/kill p.4,
# promozione = braccio con D maggiore purché nominato e senza regressione della guardia. Lock per TOKEN (`grep -qw s172`);
# `[ -s ]` sui driver e sugli attesi ([ -e ] sul pavimento, VUOTO per costruzione); parità di A, B e C contro l'ORACLE.
# R=5; ns/iter=(med raw−floor)/N; rumore drop-1. rc autoritativo = ab-out/<TAG>.rc; verdetto s172-<TAG>-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; BB="${3:?BPATH}"; BEXP="${4:?BEXP8}"; CC="${5:?CPATH}"; CEXP="${6:?CEXP8}"; TAG="${7:?TAG}"; R="${8:-5}"
O=/opt/homebrew/opt/php/bin/php
PD="$H/prop-dq.php"
GD="$H/../wp164-harness/arith-dq.php"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s172-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
for f in "$PD" "$GD" "$A" "$BB" "$CC" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ], emenda S-170 p.4)" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
grep -qw s172 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s172 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"; CM="$(shasum -a 256 "$CC" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$CM" = "$CEXP" ] || { echo "C misurato $CM != atteso $CEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
# N del giudice e della guardia EMESSI dal sorgente (KS-GR-105-2; az.rev. S-170 #4): mai cablati
NPD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$PD")
NGD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$GD")
[ -n "$NPD" ] && [ -n "$NGD" ] || { echo "N non leggibile dal driver (prop='$NPD' dq='$NGD')" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s172 LEVA $TAG — A=$AM MISURATO ($A) B=$BM MISURATO ($BB, P1) C=$CM MISURATO ($CC, P1+P2); GIUDICE prop-dq N=$NPD (bersaglio) E GUARDIA arith-dq N=$NGD (sola regressione; N dal driver); R=$R ABAB alternato su giudice e guardia; criterio s172-criterio.md =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
# PARITÀ vs ATTESO: l'atteso è l'output dell'ORACLE sullo stesso driver (generato qui, una volta per driver, `[ -s ]`).
for D in "$PD" "$GD"; do
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
    TA=$(ucpu "$A" "$PD"); TB=$(ucpu "$BB" "$PD"); EA=$(ucpu "$A" "$GD"); EB=$(ucpu "$BB" "$GD"); ord=AB
  else
    TB=$(ucpu "$BB" "$PD"); TA=$(ucpu "$A" "$PD"); EB=$(ucpu "$BB" "$GD"); EA=$(ucpu "$A" "$GD"); ord=BA
  fi
  TC=$(ucpu "$CC" "$PD"); EC=$(ucpu "$CC" "$GD")
  TOR=$(ucpu "$O" "$PD"); EOR=$(ucpu "$O" "$GD")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$TA" "$TB" "$EA" "$EB" "$TC" "$EC" "$TOR" "$EOR" >> "$TSV"
  echo "  coppia$i [$ord]: propA=$TA propB=$TB dqA=$EA dqB=$EB | propC=$TC dqC=$EC | propO=$TOR dqO=$EOR"
done
python3 - "$TSV" "$FA" "$FB" "$FC" "$FO" "$NPD" "$NGD" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fa, fb, fc, fo = map(float, sys.argv[2:6]); npd = float(sys.argv[6]); ngd = float(sys.argv[7])
def col(i, f, n): return sorted((float(r[i])-f)/n*1e9 for r in rows)
def med(v):
    k = len(v); return v[k//2] if k % 2 else (v[k//2-1]+v[k//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
pA, pB, gA, gB = col(0,fa,npd), col(1,fb,npd), col(2,fa,ngd), col(3,fb,ngd)
pC, gC, pO, gO = col(4,fc,npd), col(5,fc,ngd), col(6,fo,npd), col(7,fo,ngd)
def stat(name, A, X, lab):
    D = med(A)-med(X); noise = max(dr1(A), dr1(X)); thr = max(4.0, noise, 0.94); thrd = max(noise, 0.94)
    nom = D >= thr; reg = D < -max(4.0, noise)
    print(f"{name} {lab}: A={med(A):.2f} {lab}={med(X):.2f} ns/iter D=A−{lab}={D:+.2f} soglia={thr:.2f} (rumore drop-1 A'={dr1(A):.2f} {lab}'={dr1(X):.2f}) -> {'NOMINATO' if nom else 'NON nominato (vale 0)'}; direzione {'FIRMATA' if abs(D) >= thrd else 'sotto il rumore'} ({lab+' più veloce' if D > 0 else lab+' più lento'}); regressione (D < −max(4,rumore)): {'SÌ' if reg else 'no'}")
    return D, noise, nom, reg
print(f"GIUDICE prop-dq (bersaglio): oracle={med(pO):.2f} ns/iter; A/oracle={med(pA)/med(pO):.2f}× B/oracle={med(pB)/med(pO):.2f}× C/oracle={med(pC)/med(pO):.2f}×")
DB, nB, nomB, _ = stat("GIUDICE prop-dq", pA, pB, "B")
DC, nC, nomC, _ = stat("GIUDICE prop-dq", pA, pC, "C")
cb = med(pB)-med(pC)
print(f"CONTRASTO C−B (prop, DIREZIONE+BANDA = contributo P2): B−C={cb:+.2f} ns/iter -> {'P2 più veloce di P1 oltre il rumore' if cb >= max(nC, 0.94) else ('P2 più LENTO di P1 oltre il rumore: P2 cade a verdetto' if cb <= -max(nC, 0.94) else 'sotto il rumore')}")
print(f"ATTESE p.4: D_B ∈ [8;20]: {'CENTRATA' if 8 <= DB <= 20 else 'FUORI'} ({DB:+.2f}); D_C ∈ [12;30]: {'CENTRATA' if 12 <= DC <= 30 else 'FUORI'} ({DC:+.2f}); KILL-1 (D_C < 6): {'SCATTA — la forma sigillata NON paga su prop: peso in guardie/borrow, niente generalizzazione per analogia' if DC < 6 else 'NON scatta'}")
print(f"GUARDIA arith-dq (sola regressione): oracle={med(gO):.2f}")
_, _, _, regGB = stat("GUARDIA arith-dq", gA, gB, "B")
_, _, _, regGC = stat("GUARDIA arith-dq", gA, gC, "C")
cands = [(DC, 'C', nomC, nC, regGC), (DB, 'B', nomB, nB, regGB)]
ok = [c for c in cands if not c[4]]
rc = 0
if not ok:
    print("PROMOZIONE: NON ammessa — la guardia arith-dq REGREDISCE su B e su C (rc=5, leva SOSPESA)"); rc = 5
else:
    D, lab, nom, noise, _ = max(ok, key=lambda c: c[0])
    if nom:
        print(f"PROMOZIONE (p.4: braccio con D maggiore, nominato, guardia senza regressione): AMMESSA — braccio {lab} (D={D:+.2f})")
    else:
        rc = 8 if noise > 4.0 else 4
        print(f"PROMOZIONE: NON ammessa — miglior braccio {lab} D={D:+.2f} NON nominato (rc={rc})")
print(f"ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
