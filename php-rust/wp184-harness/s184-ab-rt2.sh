#!/bin/bash
# s184-ab-rt2.sh — A/B della LEVA COMPOSTA L-RT1 (nel tree) + L-RT2 «Ret fuso» (criterio s184-criterio-rt2.md p.3-5): COPIA DICHIARATA di
# ../wp182-harness/s183-ab-rt1.sh (manifest s184-ab-rt2-copia.diff) coi SOLI adattamenti: token s184, PREV prop-dq 31,27 (S-183),
# D = cifra COMPOSTA pin s181 → tree, attesa [3,5;6,5], promozione se D ≥ max(4, rumore, |A−Z|) con ENTRAMBI gli stimatori, rc=3 con
# SOTTOCASO (RT2 DIREZIONE se D ≥ 2,83+max(rumore,SL), altrimenti RT2 NULLA ⇒ revert), verdetto s184-<TAG>. Testo S-183/S-181 sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; ZZ="${3:?ZPATH}"; ZEXP="${4:?ZEXP8}"; BB="${5:?BPATH}"; BEXP="${6:?BEXP8}"; TAG="${7:?TAG}"; R="${8:-5}"; PREVPD="${9:-31.27}"
O=/opt/homebrew/opt/php/bin/php
CD="$H/calls-dq.php"
GD="$H/../wp164-harness/arith-dq.php"
PD="$H/../wp172-harness/prop-dq.php"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s184-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
for f in "$CD" "$PD" "$GD" "$A" "$ZZ" "$BB" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ], emenda S-170 p.4)" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
grep -qw s184 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s184 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; ZM="$(shasum -a 256 "$ZZ" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$ZM" = "$ZEXP" ] || { echo "Z misurato $ZM != atteso $ZEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
# N del giudice e delle guardie EMESSI dal sorgente (KS-GR-105-2; az.rev. S-170 #4): mai cablati
NCD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$CD")
NGD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$GD")
NPD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$PD")
[ -n "$NCD" ] && [ -n "$NPD" ] && [ -n "$NGD" ] || { echo "N non leggibile dal driver (calls='$NCD' dq='$NGD' prop='$NPD')" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s184 LEVA COMPOSTA L-RT1+L-RT2 $TAG — A=$AM MISURATO ($A, pin s181) Z=$ZM MISURATO ($ZZ, gemello del pin ricostruito$( [ "$ZM" = "$AM" ] && echo ' — BYTE-ID col pin: |A−Z| = solo rumore, SL in-run NON stimabile' )) B=$BM MISURATO ($BB, tree = L-RT1 + L-RT2) — R=$R, N: calls-dq=$NCD arith-dq=$NGD prop-dq=$NPD, PREV prop-dq A (stesso binario, S-183: 31,27): $PREVPD, $(date '+%F %T') =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
# PARITÀ vs ATTESO: l'atteso è l'output dell'ORACLE sullo stesso driver (generato qui, una volta per driver, `[ -s ]`).
for D in "$CD" "$GD" "$PD"; do
  n="$(basename "$D" .php)"; EXP="$OUT/expected-$n.out"
  [ -s "$EXP" ] || "$O" "$D" > "$EXP" 2>&1
  [ -s "$EXP" ] || { echo "atteso VUOTO per $n — STOP"; echo 2 > "$RC"; exit 2; }
  for arm in A Z B; do
    case "$arm" in A) bin="$A";; Z) bin="$ZZ";; B) bin="$BB";; esac
    "$bin" "$D" > "$OUT/$TAG-$arm-$n.out" 2>&1
    cmp -s "$OUT/$TAG-$arm-$n.out" "$EXP" || { echo "output $arm ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-$arm-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  done
  echo "parità $n: A==Z==B==atteso oracle ($(tr '\n' ' ' < "$EXP"))"
done
FA=$(floor3 "$A"); FZ=$(floor3 "$ZZ"); FB=$(floor3 "$BB"); FO=$(floor3 "$O")
echo "floors: A=$FA Z=$FZ B=$FB oracle=$FO"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
ORDS="AZB BAZ ZBA ABZ BZA ZAB"
for i in $(seq 1 "$R"); do
  ord=$(echo "$ORDS" | awk -v k="$i" '{print $(((k-1)%6)+1)}')
  CA=; CZ=; CB=; PA=; PZ=; PB=; GA=; GZ=; GB=
  for x in $(echo "$ord" | sed 's/./& /g'); do
    case "$x" in
      A) CA=$(ucpu "$A" "$CD"); PA=$(ucpu "$A" "$PD"); GA=$(ucpu "$A" "$GD");;
      Z) CZ=$(ucpu "$ZZ" "$CD"); PZ=$(ucpu "$ZZ" "$PD"); GZ=$(ucpu "$ZZ" "$GD");;
      B) CB=$(ucpu "$BB" "$CD"); PB=$(ucpu "$BB" "$PD"); GB=$(ucpu "$BB" "$GD");;
    esac
  done
  COR=$(ucpu "$O" "$CD"); POR=$(ucpu "$O" "$PD"); GOR=$(ucpu "$O" "$GD")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$CA" "$CZ" "$CB" "$PA" "$PZ" "$PB" "$GA" "$GZ" "$GB" "$COR" "$POR" "$GOR" >> "$TSV"
  echo "  coppia$i [$ord]: callsA=$CA callsZ=$CZ callsB=$CB propA=$PA propZ=$PZ propB=$PB dqA=$GA dqZ=$GZ dqB=$GB | callsO=$COR propO=$POR dqO=$GOR"
done
python3 - "$TSV" "$FA" "$FZ" "$FB" "$FO" "$NCD" "$NPD" "$NGD" "$PREVPD" "$R" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fa, fz, fb, fo = map(float, sys.argv[2:6]); ncd = float(sys.argv[6]); npd = float(sys.argv[7]); ngd = float(sys.argv[8]); prevpd = float(sys.argv[9]); R = int(sys.argv[10])
def col(i, f, n): return [(float(r[i])-f)/n*1e9 for r in rows]
def med(v):
    v = sorted(v); k = len(v); return v[k//2] if k % 2 else (v[k//2-1]+v[k//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
def pmed(X, Y): return med([x-y for x, y in zip(X, Y)])   # mediana delle differenze APPAIATE (az.rev. S-173 (b))
def signs(X, Y): return sum(1 for x, y in zip(X, Y) if x - y > 0)
cA, cZ, cB, cO = col(0,fa,ncd), col(1,fz,ncd), col(2,fb,ncd), col(9,fo,ncd)
pA, pZ, pB, pO = col(3,fa,npd), col(4,fz,npd), col(5,fb,npd), col(10,fo,npd)
gA, gZ, gB, gO = col(6,fa,ngd), col(7,fz,ngd), col(8,fb,ngd), col(11,fo,ngd)
rc = 0
# az.rev. S-173 (a): SL in-run = |A−Z| per giudice (se Z è byte-id col pin: SOLO rumore, dichiarato in testa); banda tra run = |A oggi − A ultima misura| (STESSO binario)
def layout(name, A, Z, prev):
    sl = abs(med(A)-med(Z)); slp = abs(pmed(A, Z))
    band = "n/d (nessun PREV same-toolchain: la A di oggi diventa PREV)" if prev is None else f"{abs(med(A)-prev):.2f}"
    print(f"LAYOUT {name}: A={med(A):.2f} Z={med(Z):.2f} ⇒ |A−Z| (colonna) {sl:.2f} / (appaiata) {slp:.2f} ns/iter; banda tra run same-binary |A−PREV| = {band} (rumore drop-1 A'={dr1(A):.2f} Z'={dr1(Z):.2f})")
    return max(sl, slp)
SLc = layout("calls-dq", cA, cZ, None)
SLp = layout("prop-dq", pA, pZ, prevpd)
SLg = layout("arith-dq", gA, gZ, None)
# EMENDA E1 (S-177): banda same-binary |A−PREV| > 4 su prop-dq ⇒ finestra contaminata: nessun verdetto (rc=8)
BANDp = abs(med(pA)-prevpd)
if BANDp > 4.0:
    print(f"BANDA same-binary FUORI: |A−PREV| prop-dq {BANDp:.2f} > 4 ⇒ finestra CONTAMINATA (pesi esterni): nessun verdetto (rc=8, emenda E1 S-177)"); rc = 8
if SLc > 4.0 or SLp > 4.0 or SLg > 4.0:
    print(f"|A−Z| SOPRA il pavimento 4 (calls {SLc:.2f}, prop {SLp:.2f}, arith {SLg:.2f}): banda-layout/rumore > soglia — nessun verdetto, si estende R (rc=8)"); rc = 8
def stat(name, A, X, lab, alab, sl):
    Dc = med(A)-med(X); Dp = pmed(A, X); noise = max(dr1(A), dr1(X)); thr = max(4.0, noise, sl); thrd = max(noise, sl)
    nomc, nomp = Dc >= thr, Dp >= thr; nom = nomc and nomp
    reg = min(Dc, Dp) < -max(4.0, noise)
    sg = signs(A, X)
    if nom: cifra = f"CIFRA = [{min(Dc,Dp):+.2f};{max(Dc,Dp):+.2f}] (entrambi gli stimatori nominano)"
    elif nomc or nomp: cifra = "nomina UN solo stimatore ⇒ solo DIREZIONE (cifra negata)"
    else: cifra = "NON nominato (vale 0)"
    dirs = 'FIRMATA' if min(abs(Dc), abs(Dp)) >= thrd else 'sotto il rumore/SL'
    print(f"{name} {lab}: {alab}={med(A):.2f} {lab}={med(X):.2f} ns/iter D={alab}−{lab} colonna={Dc:+.2f} appaiata={Dp:+.2f} soglia={thr:.2f} (rumore drop-1 {alab}'={dr1(A):.2f} {lab}'={dr1(X):.2f}, SL={sl:.2f}) segni {sg}/{R} -> {cifra}; direzione {dirs} ({lab} {'più veloce' if Dc > 0 else 'più lento'}); regressione (D < −max(4,rumore)): {'SÌ' if reg else 'no'}")
    return Dc, Dp, noise, nom, reg, sg
print(f"GIUDICE calls-dq (BERSAGLIO, 1 Call+CheckArity+BinarySS+Ret per iter): oracle={med(cO):.2f} ns/iter; A/oracle={med(cA)/med(cO):.2f}× Z/oracle={med(cZ)/med(cO):.2f}× B/oracle={med(cB)/med(cO):.2f}×")
DCc, DCp, nC, nomC, regC, sgC = stat("GIUDICE calls-dq COMPOSTA L-RT1+L-RT2 (D = A−B, pin s181 → tree)", cA, cB, "B", "A", SLc)
DC = min(DCc, DCp)
print(f"ATTESA (criterio-rt2 p.4): D composta ∈ [3,5;6,5]: {'CENTRATA' if 3.5 <= DC <= 6.5 else 'FUORI'} ({DCc:+.2f}/{DCp:+.2f}); quota propria di L-RT2 = D − [2,50;2,83] di L-RT1 = INDICATIVA (regola 3: non cifra)")
print(f"GUARDIA prop-dq (sola regressione, nessun Call nel loop): oracle={med(pO):.2f} ns/iter; A/oracle={med(pA)/med(pO):.2f}× Z/oracle={med(pZ)/med(pO):.2f}× B/oracle={med(pB)/med(pO):.2f}×")
DPc, DPp, nP, nomP, regP, sgP = stat("GUARDIA prop-dq (attesa |D| < 1)", pA, pB, "B", "A", SLp)
print(f"GUARDIA prop-dq ESITO attesa |D|<1: {'RISPETTATA' if max(abs(DPc),abs(DPp)) < 1 else ('VIOLATA — segni pieni: effetto TRASVERSALE firmato' if sgP in (0,R) else 'VIOLATA — segni misti')} (D {DPc:+.2f}/{DPp:+.2f}, segni {sgP}/{R})")
print(f"GUARDIA arith-dq (sola regressione, nessun Call nel loop): oracle={med(gO):.2f} ns/iter; A/oracle={med(gA)/med(gO):.2f}× Z/oracle={med(gZ)/med(gO):.2f}× B/oracle={med(gB)/med(gO):.2f}×")
DGc, DGp, nG, nomG, regG, sgG = stat("GUARDIA arith-dq (attesa |D| < 1)", gA, gB, "B", "A", SLg)
print(f"GUARDIA arith-dq ESITO attesa |D|<1: {'RISPETTATA' if max(abs(DGc),abs(DGp)) < 1 else ('VIOLATA — segni pieni: effetto TRASVERSALE firmato' if sgG in (0,R) else 'VIOLATA — segni misti')} (D {DGc:+.2f}/{DGp:+.2f}, segni {sgG}/{R})")
print(f"PREV per la prossima misura (same-binary pin s181): calls-dq A={med(cA):.2f} arith-dq A={med(gA):.2f} prop-dq A={med(pA):.2f} (PREV: {prevpd:.2f})")
if rc == 8:
    print("ESITO rc=8"); sys.exit(8)
# ESITI (criterio-cr1 p.5)
if regP or regG:
    print(f"GUARDIA: regressione su {'prop-dq ' if regP else ''}{'arith-dq' if regG else ''} (prop {DPc:+.2f}/{DPp:+.2f}, arith {DGc:+.2f}/{DGp:+.2f}) ⇒ leva SOSPESA, nessuna promozione (rc=5)"); rc = 5
elif nomC:
    print(f"PROMOZIONE: D calls-dq NOMINATA da entrambi gli stimatori [{min(DCc,DCp):+.2f};{max(DCc,DCp):+.2f}] (soglia max(4,rumore,SL)), guardie senza regressione ⇒ promozione COMPOSTA del TREE (L-RT1+L-RT2) con copia dichiarata di s181-promozione.sh (tag s182, CARGO_TARGET_DIR esplicito) + coppia t25 + ORM (rc=0)")
elif DC >= max(nC, SLc) and sgC == R:
    rt1hi = 2.83
    if DC >= rt1hi + max(nC, SLc):
        print(f"SOLO DIREZIONE composta: D {DCc:+.2f}/{DCp:+.2f} ≥ max(rumore {nC:.2f}, SL {SLc:.2f}) con segni {sgC}/{R} ma sotto la soglia; SOTTOCASO RT2 DIREZIONE (D ≥ 2,83 + max(rumore,SL) = {rt1hi + max(nC, SLc):.2f}) ⇒ L-RT2 TENUTA nel tree con L-RT1 SOLO a CI batteria+corpus OK sul commit (criterio-rt2 p.5; rc=3)")
    else:
        print(f"SOLO DIREZIONE composta: D {DCc:+.2f}/{DCp:+.2f} ≥ max(rumore {nC:.2f}, SL {SLc:.2f}) con segni {sgC}/{R} ma sotto la soglia; SOTTOCASO RT2 NULLA (D < 2,83 + max(rumore,SL) = {rt1hi + max(nC, SLc):.2f}: la composta non supera L-RT1 sola) ⇒ REVERT del commit L-RT2, il tree torna a L-RT1 sola (criterio-rt2 p.5; rc=3)")
    rc = 3
else:
    print(f"KILL L-RT2: D composta {DCc:+.2f}/{DCp:+.2f} < max(rumore {nC:.2f}, SL {SLc:.2f}) o segni misti ({sgC}/{R}) ⇒ revert al byte del commit L-RT2 (L-RT1 resta sul suo A/B S-183), meccanismo a verbale (rc=4)"); rc = 4
print(f"ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
