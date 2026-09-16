#!/bin/bash
# s176-ab-flag.sh — A/B della leva «flag gc-idle» (criterio s176-criterio-flag.md p.2-4): COPIA DICHIARATA di
# ../wp174-harness/s175-ab-mock.sh (manifest s176-ab-flag-copia.diff) coi SOLI adattamenti: (1) bracci A = pin s175
# (5de14d68), Z = ab-out/phpr-C di S-174/175 (b6c4b587, gemello a contenuto ⇒ SL in-run), B = LEVA flag gc-idle
# (candidata), C = MS (dcd73bf4, tetto S-175: mutante SCORRETTO, solo contrasto B−C); (2) GIUDICE BERSAGLIO = prop-dq a
# nomina (attesa D_B ∈ [2;5]), GUARDIA = arith-dq a SOLA regressione (direzione attesa [1;2,5]); (3) esiti p.4:
# rc=0 CANDIDATA (prop nominato da ENTRAMBI gli stimatori, arith senza regressione) · rc=3 SOLO DIREZIONE
# (max(1,rumore,SL) ≤ D_B prop < soglia) · rc=4 KILL (D_B prop < max(1,rumore,SL)) · rc=8 SL sopra 4; (4) lock per
# TOKEN s176; verdetto s176-<TAG>-verdetto.out; PREV_* = A del mock S-175 (20,08 / 36,13). Tutto il resto INVARIATO
# (misura: R=5, rotazione dei 4 bracci per coppia, floors med3 per binario, ns/iter=(med raw−floor)/N con N LETTO
# dal driver, mediane per colonna e appaiate, rumore drop-1, parità dei 4 bracci contro l'ORACLE prima di misurare).
# Uso: s176-ab-flag.sh <APATH> <AEXP8> <ZPATH> <ZEXP8> <BPATH> <BEXP8> <CPATH> <CEXP8> <TAG> [R] [PREV_DQ_A] [PREV_PROP_A]
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${1:?APATH}"; AEXP="${2:?AEXP8}"; ZZ="${3:?ZPATH}"; ZEXP="${4:?ZEXP8}"; BB="${5:?BPATH}"; BEXP="${6:?BEXP8}"; CC="${7:?CPATH}"; CEXP="${8:?CEXP8}"; TAG="${9:?TAG}"; R="${10:-5}"; PREVDQ="${11:-20.08}"; PREVPD="${12:-36.13}"
O=/opt/homebrew/opt/php/bin/php
GD="$H/../wp164-harness/arith-dq.php"
PD="$H/../wp172-harness/prop-dq.php"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s176-$TAG-verdetto.out"; RC="$OUT/$TAG.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
for f in "$PD" "$GD" "$A" "$ZZ" "$BB" "$CC" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ], emenda S-170 p.4)" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
grep -qw s176 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s176 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-$TAG.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; ZM="$(shasum -a 256 "$ZZ" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"; CM="$(shasum -a 256 "$CC" | cut -c1-8)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != atteso $AEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$ZM" = "$ZEXP" ] || { echo "Z misurato $ZM != atteso $ZEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$CM" = "$CEXP" ] || { echo "C misurato $CM != atteso $CEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
# N del giudice e della guardia EMESSI dal sorgente (KS-GR-105-2; az.rev. S-170 #4): mai cablati
NGD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$GD")
NPD=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$PD")
[ -n "$NPD" ] && [ -n "$NGD" ] || { echo "N non leggibile dal driver (dq='$NGD' prop='$NPD')" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s176 LEVA $TAG — A=$AM MISURATO ($A, pin s175) Z=$ZM MISURATO ($ZZ, gemello a contenuto del pin = SL in-run) B=$BM MISURATO ($BB, LEVA flag gc-idle: candidata) C=$CM MISURATO ($CC, MS = tetto S-175, mutante SCORRETTO: solo contrasto) — R=$R, N: arith-dq=$NGD prop-dq=$NPD, PREV A: $PREVDQ / $PREVPD, $(date '+%F %T') =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
# PARITÀ vs ATTESO: l'atteso è l'output dell'ORACLE sullo stesso driver (generato qui, una volta per driver, `[ -s ]`).
for D in "$GD" "$PD"; do
  n="$(basename "$D" .php)"; EXP="$OUT/expected-$n.out"
  [ -s "$EXP" ] || "$O" "$D" > "$EXP" 2>&1
  [ -s "$EXP" ] || { echo "atteso VUOTO per $n — STOP"; echo 2 > "$RC"; exit 2; }
  for arm in A Z B C; do
    case "$arm" in A) bin="$A";; Z) bin="$ZZ";; B) bin="$BB";; C) bin="$CC";; esac
    "$bin" "$D" > "$OUT/$TAG-$arm-$n.out" 2>&1
    cmp -s "$OUT/$TAG-$arm-$n.out" "$EXP" || { echo "output $arm ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/$TAG-$arm-$n.out" | tr '\n' ' ')) — STOP"; echo 2 > "$RC"; exit 2; }
  done
  echo "parità $n: A==Z==B==C==atteso oracle ($(tr '\n' ' ' < "$EXP"))"
done
FA=$(floor3 "$A"); FZ=$(floor3 "$ZZ"); FB=$(floor3 "$BB"); FC=$(floor3 "$CC"); FO=$(floor3 "$O")
echo "floors: A=$FA Z=$FZ B=$FB C=$FC oracle=$FO"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
ORDS="AZBC CBZA BCAZ ZACB CZAB BAZC"
for i in $(seq 1 "$R"); do
  ord=$(echo "$ORDS" | awk -v k="$i" '{print $(((k-1)%6)+1)}')
  EA=; EZ=; EB=; EC=; TA=; TZ=; TB=; TC=
  for x in $(echo "$ord" | sed 's/./& /g'); do
    case "$x" in
      A) EA=$(ucpu "$A" "$GD"); TA=$(ucpu "$A" "$PD");;
      Z) EZ=$(ucpu "$ZZ" "$GD"); TZ=$(ucpu "$ZZ" "$PD");;
      B) EB=$(ucpu "$BB" "$GD"); TB=$(ucpu "$BB" "$PD");;
      C) EC=$(ucpu "$CC" "$GD"); TC=$(ucpu "$CC" "$PD");;
    esac
  done
  EOR=$(ucpu "$O" "$GD"); TOR=$(ucpu "$O" "$PD")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$EA" "$EZ" "$EB" "$EC" "$TA" "$TZ" "$TB" "$TC" "$EOR" "$TOR" >> "$TSV"
  echo "  coppia$i [$ord]: dqA=$EA dqZ=$EZ dqB=$EB dqC=$EC propA=$TA propZ=$TZ propB=$TB propC=$TC | dqO=$EOR propO=$TOR"
done
python3 - "$TSV" "$FA" "$FZ" "$FB" "$FC" "$FO" "$NGD" "$NPD" "$PREVDQ" "$PREVPD" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fa, fz, fb, fc, fo = map(float, sys.argv[2:7]); ngd = float(sys.argv[7]); npd = float(sys.argv[8]); prevdq = float(sys.argv[9]); prevpd = float(sys.argv[10])
def col(i, f, n): return [(float(r[i])-f)/n*1e9 for r in rows]
def med(v):
    v = sorted(v); k = len(v); return v[k//2] if k % 2 else (v[k//2-1]+v[k//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
def pmed(X, Y): return med([x-y for x, y in zip(X, Y)])   # mediana delle differenze APPAIATE (az.rev. S-173 (b))
gA, gZ, gB, gC, gO = col(0,fa,ngd), col(1,fz,ngd), col(2,fb,ngd), col(3,fc,ngd), col(8,fo,ngd)
pA, pZ, pB, pC, pO = col(4,fa,npd), col(5,fz,npd), col(6,fb,npd), col(7,fc,npd), col(9,fo,npd)
rc = 0
# az.rev. S-173 (a): SL in-run = |A−Z| (stessa sorgente, layout diverso) per giudice; banda tra run = |A oggi − A ultima misura|
def layout(name, A, Z, prev):
    sl = abs(med(A)-med(Z)); slp = abs(pmed(A, Z)); band = abs(med(A)-prev)
    print(f"LAYOUT {name}: A={med(A):.2f} Z={med(Z):.2f} ⇒ SL in-run (colonna) {sl:.2f} / (appaiata) {slp:.2f} ns/iter — confronto 0,94 (arith S-168): {'SOPRA' if max(sl, slp) > 0.94 else 'sotto'}; banda tra run |A−PREV {prev:.2f}| = {band:.2f} (rumore drop-1 A'={dr1(A):.2f} Z'={dr1(Z):.2f})")
    return max(sl, slp)
SLg = layout("arith-dq", gA, gZ, prevdq)
SLp = layout("prop-dq", pA, pZ, prevpd)
if SLg > 4.0 or SLp > 4.0:
    print(f"SL in-run SOPRA il pavimento 4 (arith {SLg:.2f}, prop {SLp:.2f}): banda-layout > soglia — nessun verdetto, si estende R (rc=8)"); rc = 8
def stat(name, A, X, lab, alab, sl):
    Dc = med(A)-med(X); Dp = pmed(A, X); noise = max(dr1(A), dr1(X)); thr = max(4.0, noise, sl); thrd = max(noise, sl)
    nomc, nomp = Dc >= thr, Dp >= thr; nom = nomc and nomp
    reg = min(Dc, Dp) < -max(4.0, noise)
    if nom: cifra = f"CIFRA = [{min(Dc,Dp):+.2f};{max(Dc,Dp):+.2f}] (entrambi gli stimatori nominano)"
    elif nomc or nomp: cifra = "nomina UN solo stimatore ⇒ solo DIREZIONE (cifra negata)"
    else: cifra = "NON nominato (vale 0)"
    dirs = 'FIRMATA' if min(abs(Dc), abs(Dp)) >= thrd else 'sotto il rumore/SL'
    print(f"{name} {lab}: {alab}={med(A):.2f} {lab}={med(X):.2f} ns/iter D={alab}−{lab} colonna={Dc:+.2f} appaiata={Dp:+.2f} soglia={thr:.2f} (rumore drop-1 {alab}'={dr1(A):.2f} {lab}'={dr1(X):.2f}, SL={sl:.2f}) -> {cifra}; direzione {dirs} ({lab} {'più veloce' if Dc > 0 else 'più lento'}); regressione (D < −max(4,rumore)): {'SÌ' if reg else 'no'}")
    return Dc, Dp, noise, nom, reg
print(f"GIUDICE prop-dq (BERSAGLIO, 2 Sweep/iter fusi): oracle={med(pO):.2f} ns/iter; A/oracle={med(pA)/med(pO):.2f}× Z/oracle={med(pZ)/med(pO):.2f}× B/oracle={med(pB)/med(pO):.2f}× C/oracle={med(pC)/med(pO):.2f}×")
DPBc, DPBp, nPB, nomPB, regPB = stat("GIUDICE prop-dq", pA, pB, "B", "A", SLp)
DPCc, DPCp, nPC, nomPC, _ = stat("TETTO prop-dq (MS, contrasto)", pA, pC, "C", "A", SLp)
DPBCc, DPBCp, nPBC, _, _ = stat("CONTRASTO prop-dq (B vs tetto MS: quota del predicato che il flag NON recupera)", pB, pC, "C", "B", SLp)
DPB, DPC = min(DPBc, DPBp), min(DPCc, DPCp)
print(f"ATTESE LEVA prop (criterio-flag p.3): D_B ∈ [2;5]: {'CENTRATA' if 2 <= DPB <= 5 else 'FUORI'} ({DPBc:+.2f}/{DPBp:+.2f}); quota del tetto: {DPB/DPC*100 if DPC > 0 else float('nan'):.0f}% (tetto {DPCc:+.2f}/{DPCp:+.2f})")
print(f"GUARDIA arith-dq (sola regressione, 1 Sweep/iter fuso): oracle={med(gO):.2f} ns/iter; A/oracle={med(gA)/med(gO):.2f}× Z/oracle={med(gZ)/med(gO):.2f}× B/oracle={med(gB)/med(gO):.2f}× C/oracle={med(gC)/med(gO):.2f}×")
DBc, DBp, nB, nomB, regB = stat("GUARDIA arith-dq", gA, gB, "B", "A", SLg)
DCc, DCp, nC, nomC, _ = stat("TETTO arith-dq (MS, contrasto)", gA, gC, "C", "A", SLg)
DB = min(DBc, DBp)
print(f"ATTESE LEVA arith (criterio-flag p.3, direzione): D_B ∈ [1;2,5]: {'CENTRATA' if 1 <= DB <= 2.5 else 'FUORI'} ({DBc:+.2f}/{DBp:+.2f}); regressione: {'SÌ' if regB else 'no'}")
if rc == 8:
    print("ESITO rc=8"); sys.exit(8)
# ESITI (criterio-flag p.4)
if DPB < max(1.0, nPB, SLp):
    print(f"KILL: D_B prop {DPBc:+.2f}/{DPBp:+.2f} < max(1, rumore {nPB:.2f}, SL {SLp:.2f}) ⇒ il costo sta nei load/cmp singoli, non nella somma: leva MORTA, revert al byte (rc=4)"); rc = 4
elif regB:
    print(f"GUARDIA: arith-dq REGREDISCE ({DBc:+.2f}/{DBp:+.2f}) ⇒ nessuna promozione (rc=5)"); rc = 5
elif nomPB:
    print(f"CANDIDATA: prop-dq NOMINATO da entrambi gli stimatori [{min(DPBc,DPBp):+.2f};{max(DPBc,DPBp):+.2f}] (soglia max(4,rumore,SL)), arith senza regressione ({DBc:+.2f}/{DBp:+.2f}) ⇒ promozione con copia dichiarata di s174-promozione.sh (tag s176) + coppia dovuta (rc=0)")
else:
    print(f"SOLO DIREZIONE: prop-dq {DPBc:+.2f}/{DPBp:+.2f} sopra max(1,rumore,SL) ma sotto la soglia ⇒ guadagno parziale TENUTO (feedback keep-partial-wins), promozione rimandata a rerun (rc=3)"); rc = 3
print(f"ESITO rc={rc}")
sys.exit(rc)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
