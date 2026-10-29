#!/bin/bash
# s186-guardia.sh — GUARDIA A BUFFER PIENO della cura 22/10 (`gc_sweep_after_ret` O(|gc_buf|) per Ret; concilio S-186, PROMOZIONE s184
# ordine obbligato punto (2); criterio s186-criterio-guardia.md): COPIA DICHIARATA di ../wp185-harness/s185-ab-mc2.sh (manifest
# s186-guardia-copia.diff) coi SOLI adattamenti: ingressi per ENV (PIN/Z/B/R) al posto dei posizionali, A = pin s183 9a6ae11bf7e900ef
# verificato per hash (rc=9), Z = gemello del pin ricostruito, B = tree (pin + cura 22/10), lock SOLO verificato (rc=6 se assente),
# TAG (b = tree com'è, b2 = tree + cura O(1)) che nomina guardia-out-$TAG/ e il verdetto; dobj.txt (D_obj del pin, un float) per la fetta 0;
# GIUDICI: mcalls-dq + calls-dq (ereditati) + ret-buf-dq K∈{0,4,16} (NUOVI: K oggetti vivi in gc_buf a ogni Ret), E3 Hoare
# (array_map N=1e5/2e5, ratio ≤2,5), companion obj-churn (D_obj ns/oggetto), micro R=5 su pin e B; verdetto rc=0 PASS / 5 MORDE / 8 no-verdetto.
# prop-dq, PREV e attesa [4;9] della leva TOLTI (nessuna leva: è una guardia a SOLA REGRESSIONE). Variabili scalari via eval (bash 3.2:
# niente declare -A). Testo S-185 conservato nel manifest.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${PIN:-$HOME/Claude/php-rust-output/release/phpr}"; AEXP="9a6ae11bf7e900ef"
ZZ="${Z:-$H/ab-out/s186-leva/phpr-Z}"; BB="${B:-$H/ab-out/s186-leva/phpr-B}"; R="${R:-5}"; RE3="${RE3:-3}"
# TAG: la catena esegue la guardia DUE volte — b (B = tree con la cura 22/10 com'è) e b2 (B = phpr-B2, tree + cura O(1), ordine (3))
TAG="${TAG:-b}"
O=/opt/homebrew/opt/php/bin/php
CD="$H/../wp185-harness/mcalls-dq.php"
GD="$H/../wp185-harness/calls-dq.php"
K0="$H/ret-buf-dq-k0.php"; K4="$H/ret-buf-dq-k4.php"; K16="$H/ret-buf-dq-k16.php"
E3A="$H/e3-churn-1e5.php"; E3B="$H/e3-churn-2e5.php"
OBJ="$H/obj-churn.php"
MICRO="$H/../wp97-harness/micro/run-micro.sh"
EMPTY="$H/../wp160-harness/empty.php"
OUT="$H/guardia-out-$TAG"; mkdir -p "$OUT"
VERD="$H/s186-guardia-verdetto-$TAG.out"; RC="$OUT/guardia.rc"; DONE="$OUT/guardia.done"; rm -f "$DONE"
fin(){ echo "$1" > "$RC"; echo "rc=$1 $(date '+%F %T')" > "$DONE"; exit "$1"; }
[ -e "$VERD" ] && { echo "verdetto ESISTE ($VERD) — spostarlo prima di rilanciare" >&2; fin 7; }
for f in "$CD" "$GD" "$K0" "$K4" "$K16" "$E3A" "$E3B" "$OBJ" "$MICRO" "$A" "$ZZ" "$BB" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; fin 7; }; done
[ -e "$EMPTY" ] || { echo "driver del pavimento assente: $EMPTY (VUOTO per costruzione: [ -e ], emenda S-170 p.4)" | tee -a "$VERD"; fin 7; }
# lock della SESSIONE: si VERIFICA soltanto (il lanciatore lo scrive col TOKEN s186)
[ -e /private/tmp/phpr-measure.lock ] || { echo "measure-lock ASSENTE (finestra non aperta)" | tee -a "$VERD"; fin 6; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-guardia.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; fin 8; }
AM="$(shasum -a 256 "$A" | cut -c1-16)"; ZM="$(shasum -a 256 "$ZZ" | cut -c1-16)"; BM="$(shasum -a 256 "$BB" | cut -c1-16)"
[ "$AM" = "$AEXP" ] || { echo "A misurato $AM != pin s183 atteso $AEXP" | tee -a "$VERD"; fin 9; }
[ "$BM" != "$AM" ] || { echo "B == pin ($BM): la cura NON è nel binario B" | tee -a "$VERD"; fin 9; }
# N del giudice e delle guardie EMESSI dal sorgente (KS-GR-105-2; az.rev. S-170 #4): mai cablati
nsrc(){ awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$1"; }
ne3(){ awk 'match($0, /\$N = [0-9]+;/) {print substr($0, RSTART+5, RLENGTH-6); exit}' "$1"; }
NCD=$(nsrc "$CD"); NGD=$(nsrc "$GD"); NK0=$(nsrc "$K0"); NK4=$(nsrc "$K4"); NK16=$(nsrc "$K16"); NOBJ=$(nsrc "$OBJ"); NE3A=$(ne3 "$E3A"); NE3B=$(ne3 "$E3B")
for v in "$NCD" "$NGD" "$NK0" "$NK4" "$NK16" "$NOBJ" "$NE3A" "$NE3B"; do [ -n "$v" ] || { echo "N non leggibile da un driver (mcalls='$NCD' calls='$NGD' k0='$NK0' k4='$NK4' k16='$NK16' obj='$NOBJ' e3='$NE3A/$NE3B')" | tee -a "$VERD"; fin 7; }; done
[ "$NK0" = "$NK4" ] && [ "$NK0" = "$NK16" ] || { echo "N dei ret-buf-dq non uniforme ($NK0/$NK4/$NK16)" | tee -a "$VERD"; fin 7; }
# Ret per iterazione dei ret-buf-dq (pin + M metodi inline nello STESSO statement): EMESSO dal sorgente, il giudice normalizza a ns/Ret
rpi(){ awk 'match($0, /RETS_PER_ITER=[0-9]+/) {print substr($0, RSTART+14, RLENGTH-14); exit}' "$1"; }
RPI=$(rpi "$K0"); [ -n "$RPI" ] && [ "$RPI" = "$(rpi "$K4")" ] && [ "$RPI" = "$(rpi "$K16")" ] || { echo "RETS_PER_ITER non leggibile o non uniforme ('$RPI'/'$(rpi "$K4")'/'$(rpi "$K16")')" | tee -a "$VERD"; fin 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
binof(){ case "$1" in A) echo "$A";; Z) echo "$ZZ";; B) echo "$BB";; O) echo "$O";; esac; }
drvof(){ case "$1" in cd) echo "$CD";; gd) echo "$GD";; k0) echo "$K0";; k4) echo "$K4";; k16) echo "$K16";; esac; }
{
echo "== s186 GUARDIA a buffer pieno (cura 22/10) TAG=$TAG — A=$AM pin s183 ($A) Z=$ZM ($ZZ, gemello del pin ricostruito$( [ "$ZM" = "$AM" ] && echo ' — BYTE-ID col pin: |A−Z| = solo rumore, SL in-run NON stimabile' )) B=$BM ($BB, tree = pin + cura) oracle=$("$O" -r 'echo PHP_VERSION;') · R=$R RE3=$RE3 · $(date '+%F %T') =="
echo "N dal sorgente: mcalls-dq=$NCD calls-dq=$NGD ret-buf-dq K0/K4/K16=$NK0 iter × $RPI Ret/iter (giudice in ns/Ret) e3=$NE3A/$NE3B obj-churn=$NOBJ"
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
# PARITÀ vs ATTESO: l'atteso è l'output dell'ORACLE sullo stesso driver (generato qui, una volta per driver, `[ -s ]`).
for D in "$CD" "$GD" "$K0" "$K4" "$K16" "$E3A" "$E3B" "$OBJ"; do
  n="$(basename "$D" .php)"; EXP="$OUT/expected-$n.out"
  [ -s "$EXP" ] || "$O" "$D" > "$EXP" 2>&1
  [ -s "$EXP" ] || { echo "atteso VUOTO per $n — STOP"; fin 2; }
  for arm in A Z B; do
    bin=$(binof "$arm")
    perl -e 'alarm 900; exec @ARGV or die' -- "$bin" "$D" > "$OUT/parita-$arm-$n.out" 2>&1
    cmp -s "$OUT/parita-$arm-$n.out" "$EXP" || { echo "output $arm ≠ ATTESO oracle su $n ($(head -c 60 "$OUT/parita-$arm-$n.out" | tr '\n' ' ')) — STOP"; fin 2; }
  done
  echo "parità $n: A==Z==B==atteso oracle ($(tr '\n' ' ' < "$EXP"))"
done
FA=$(floor3 "$A"); FZ=$(floor3 "$ZZ"); FB=$(floor3 "$BB"); FO=$(floor3 "$O")
echo "floors (user, med3 di empty.php): A=$FA Z=$FZ B=$FB oracle=$FO"
# (b) ABAB R interleaved sui 5 giudici: colonne per giudice = A Z B O (20 colonne: cd gd k0 k4 k16 × A Z B O)
TSV="$OUT/guardia-runs.tsv"; : > "$TSV"
ORDS="AZB BAZ ZBA ABZ BZA ZAB"
for i in $(seq 1 "$R"); do
  ord=$(echo "$ORDS" | awk -v k="$i" '{print $(((k-1)%6)+1)}')
  for x in $(echo "$ord" | sed 's/./& /g') O; do
    bin=$(binof "$x")
    for j in cd gd k0 k4 k16; do v=$(ucpu "$bin" "$(drvof "$j")"); eval "T_${x}_${j}=\$v"; done
  done
  row=""; line=""
  for j in cd gd k0 k4 k16; do
    cell=""
    for x in A Z B O; do eval "v=\$T_${x}_${j}"; row="$row$v	"; cell="$cell/$v"; done
    line="$line $j=${cell#/}"
  done
  printf '%s\n' "${row%	}" >> "$TSV"
  echo "  coppia$i [$ord] (A/Z/B/O):$line"
done
# (c) E3 Hoare: A e B alternati, RE3 corse per N, user CPU grezzo (stesso pavimento dentro il rapporto: si dichiara)
E3TSV="$OUT/e3-runs.tsv"; : > "$E3TSV"
for i in $(seq 1 "$RE3"); do
  if [ $((i % 2)) = 1 ]; then ord="A B"; else ord="B A"; fi
  for x in $ord; do bin=$(binof "$x"); v=$(ucpu "$bin" "$E3A"); eval "E_${x}_1=\$v"; v=$(ucpu "$bin" "$E3B"); eval "E_${x}_2=\$v"; done
  printf '%s\t%s\t%s\t%s\n' "$E_A_1" "$E_A_2" "$E_B_1" "$E_B_2" >> "$E3TSV"
  echo "  e3 corsa$i [$ord]: A 1e5=$E_A_1 2e5=$E_A_2 · B 1e5=$E_B_1 2e5=$E_B_2"
done
# (d) companion obj-churn: O, A, B interleaved, R corse
OBJTSV="$OUT/obj-runs.tsv"; : > "$OBJTSV"
OORDS="OAB ABO BOA OBA AOB BAO"
for i in $(seq 1 "$R"); do
  ord=$(echo "$OORDS" | awk -v k="$i" '{print $(((k-1)%6)+1)}')
  for x in $(echo "$ord" | sed 's/./& /g'); do bin=$(binof "$x"); v=$(ucpu "$bin" "$OBJ"); eval "J_${x}=\$v"; done
  printf '%s\t%s\t%s\n' "$J_O" "$J_A" "$J_B" >> "$OBJTSV"
  echo "  obj-churn corsa$i [$ord]: O=$J_O A=$J_A B=$J_B"
done
# (e) micro R (run-micro.sh, spina dorsale): pin poi B — rapporto per categoria (user CPU netto pavimento, come da canone)
PHPR="$A" ORACLE="$O" R="$R" /bin/bash "$MICRO" > "$OUT/micro-pin.out" 2>&1 || echo "micro pin: rc $? (vedi guardia-out-$TAG/micro-pin.out)"
PHPR="$BB" ORACLE="$O" R="$R" /bin/bash "$MICRO" > "$OUT/micro-B.out" 2>&1 || echo "micro B: rc $? (vedi guardia-out-$TAG/micro-B.out)"
echo "micro pin: $(grep -oE 'rapporto_[a-z]+=[0-9.]+' "$OUT/micro-pin.out" | tr '\n' ' ')"
echo "micro B:   $(grep -oE 'rapporto_[a-z]+=[0-9.]+' "$OUT/micro-B.out" | tr '\n' ' ')"
# (f) VERDETTO pre-registrato (criterio s186-criterio-guardia.md)
python3 - "$TSV" "$E3TSV" "$OBJTSV" "$OUT/micro-pin.out" "$OUT/micro-B.out" "$FA" "$FZ" "$FB" "$FO" "$NCD" "$NGD" "$NK0" "$NOBJ" "$R" "$RPI" "$OUT/dobj.txt" <<'PY'
import sys, re
dobj_path = sys.argv[16]
rows = [l.split() for l in open(sys.argv[1]) if l.strip()]
e3 = [list(map(float, l.split())) for l in open(sys.argv[2]) if l.strip()]
ob = [list(map(float, l.split())) for l in open(sys.argv[3]) if l.strip()]
mpin = open(sys.argv[4]).read(); mB = open(sys.argv[5]).read()
fa, fz, fb, fo = map(float, sys.argv[6:10]); ncd, ngd, nk, nobj = map(float, sys.argv[10:14]); R = int(sys.argv[14]); rpi = float(sys.argv[15])
FL = {'A': fa, 'Z': fz, 'B': fb, 'O': fo}
# ret-buf-dq: ns per RET (N iterazioni × RETS_PER_ITER Ret), così la soglia 4 ns vale per Ret come nei giudici a 1 Ret/iter
J = ['mcalls-dq', 'calls-dq', 'ret-buf-dq-K0', 'ret-buf-dq-K4', 'ret-buf-dq-K16']; NN = [ncd, ngd, nk*rpi, nk*rpi, nk*rpi]
ARMS = ['A', 'Z', 'B', 'O']
def col(j, arm): i = j*4 + ARMS.index(arm); return [(float(r[i]) - FL[arm]) / NN[j] * 1e9 for r in rows]
def med(v):
    v = sorted(v); k = len(v); return v[k//2] if k % 2 else (v[k//2-1]+v[k//2])/2
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x: (abs(x-m), x))[:-1]; return max(w)-min(w) if len(w) > 1 else 0.0
def pmed(X, Y): return med([x-y for x, y in zip(X, Y)])
def signs(X, Y): return sum(1 for x, y in zip(X, Y) if x - y > 0)
rc = 0; morde = []
print(f"floors: A={fa} Z={fz} B={fb} O={fo}; N: mcalls={ncd:.0f} calls={ngd:.0f} ret-buf-dq={nk:.0f}×{rpi:.0f} Ret (ns/Ret) obj-churn={nobj:.0f}; corse valide R={len(rows)}")
D = {}
for j, name in enumerate(J):
    cA, cZ, cB, cO = col(j, 'A'), col(j, 'Z'), col(j, 'B'), col(j, 'O')
    sl = max(abs(med(cA)-med(cZ)), abs(pmed(cA, cZ)))
    print(f"LAYOUT {name}: A={med(cA):.2f} Z={med(cZ):.2f} ⇒ |A−Z| colonna {abs(med(cA)-med(cZ)):.2f} / appaiata {abs(pmed(cA,cZ)):.2f} ns/iter (rumore drop-1 A'={dr1(cA):.2f} Z'={dr1(cZ):.2f})")
    if sl > 4.0:
        print(f"|A−Z| {sl:.2f} > 4 su {name}: banda-layout/rumore sopra soglia — nessun verdetto (rc=8)"); rc = 8
    Dc = med(cB) - med(cA); Dp = pmed(cB, cA); noise = max(dr1(cA), dr1(cB)); thr = max(4.0, noise, sl); sg = signs(cB, cA)
    D[name] = (Dc, Dp, noise, sl, thr)
    print(f"GIUDICE {name}: oracle={med(cO):.2f} ns/iter · A={med(cA):.2f} ({med(cA)/med(cO):.2f}×) Z={med(cZ):.2f} B={med(cB):.2f} ({med(cB)/med(cO):.2f}×) · D=B−A colonna={Dc:+.2f} appaiata={Dp:+.2f} soglia={thr:.2f} (rumore drop-1 {noise:.2f}, SL {sl:.2f}) segni B>A {sg}/{R}")
# (i) K=0 e giudici ereditati: |D| < max(rumore, 4, SL)
for name in ('mcalls-dq', 'calls-dq', 'ret-buf-dq-K0'):
    Dc, Dp, noise, sl, thr = D[name]
    ok = max(abs(Dc), abs(Dp)) < thr
    print(f"ATTESA (i) {name} |D| < max(rumore, 4, SL)={thr:.2f}: {'PASS' if ok else 'VIOLATA'} (|D| colonna {abs(Dc):.2f} appaiata {abs(Dp):.2f})")
    if not ok: morde.append(f"(i) {name} D {Dc:+.2f}/{Dp:+.2f}")
# (ii) K piatto: D_B(K) − D_B(K0) < max(rumore, 4)
D0c, D0p, n0, _, _ = D['ret-buf-dq-K0']
for name in ('ret-buf-dq-K4', 'ret-buf-dq-K16'):
    Dc, Dp, noise, sl, thr = D[name]
    gc, gp = Dc - D0c, Dp - D0p; t = max(4.0, noise, n0)
    ok = max(abs(gc), abs(gp)) < t
    print(f"ATTESA (ii) {name} piatto: D_B(K)−D_B(K0) colonna {gc:+.2f} appaiata {gp:+.2f} vs soglia {t:.2f}: {'PASS' if ok else 'MORDE-K (lineare in K)'}")
    if not ok: morde.append(f"(ii) {name} Δ {gc:+.2f}/{gp:+.2f}")
# (iii) E3
a1 = med([r[0] for r in e3]); a2 = med([r[1] for r in e3]); b1 = med([r[2] for r in e3]); b2 = med([r[3] for r in e3])
ra = a2/a1 if a1 > 0 else float('nan'); rb = b2/b1 if b1 > 0 else float('nan')
print(f"E3 Hoare array_map(fn ⇒ new A, range(1,N)): A 1e5={a1:.2f}s 2e5={a2:.2f}s ratio={ra:.2f} · B 1e5={b1:.2f}s 2e5={b2:.2f}s ratio={rb:.2f} (user grezzo, pavimento dentro — dichiarato)")
ok = rb <= 2.5
print(f"ATTESA (iii) ratio_B ≤ 2,5: {'PASS' if ok else 'MORDE-E3 (quadratico)'} (ratio_A {ra:.2f} a confronto)")
if not ok: morde.append(f"(iii) E3 ratio_B {rb:.2f}")
# (iv) micro: rapporto_B ≤ rapporto_pin + 0,2 (tolleranza fissa: il rapporto è già una mediana netta su R corse; spread a verbale)
def micro(txt):
    return dict(re.findall(r'rapporto_([a-z]+)=([0-9.]+)', txt)), dict((c, (so, sp)) for c, so, sp in re.findall(r'([a-z]+)_spread_oracle_s=([0-9.]+)\s+[a-z]+_spread_phpr_s=([0-9.]+)', txt))
rp, spp = micro(mpin); rb_, spb = micro(mB)
for c in ('arith', 'prop', 'calls', 'str', 'arr', 're'):
    if c not in rp or c not in rb_:
        print(f"micro {c}: n/d (pin {rp.get(c)} B {rb_.get(c)})"); morde.append(f"(iv) micro {c} n/d"); continue
    p, b = float(rp[c]), float(rb_[c]); tol = 0.2
    ok = b <= p + tol
    print(f"micro {c}: pin {p:.1f}× B {b:.1f}× (tolleranza +{tol:.1f}; spread phpr s pin/B {spp.get(c, ('?','?'))[1]}/{spb.get(c, ('?','?'))[1]}): {'PASS' if ok else 'REGRESSIONE'}")
    if not ok: morde.append(f"(iv) micro {c} {p:.1f}→{b:.1f}")
# (v) obj-churn companion
oo = med([r[0] for r in ob]) - fo; oa = med([r[1] for r in ob]) - fa; obb = med([r[2] for r in ob]) - fb
dobj_pin = (oa-oo)/nobj*1e9
print(f"OBJ-CHURN (companion, nessun gate): oracle netto {oo:.3f}s pin {oa:.3f}s B {obb:.3f}s su {nobj:.0f} oggetti ⇒ D_obj(pin)={dobj_pin:.0f} ns/oggetto · D_obj(B)={(obb-oo)/nobj*1e9:.0f} ns/oggetto · B−pin={(obb-oa)/nobj*1e9:+.0f} ns/oggetto (tetto oggetti della fetta 0 = N_obj × D_obj)")
# dobj.txt: UN float = D_obj del pin A in ns/oggetto, letto dalla fetta 0 per il tetto oggetti
open(dobj_path, 'w').write(f"{dobj_pin:.1f}\n")
print("PREV per la prossima misura (same-binary pin s183): " + " ".join(f"{n} A={med(col(j,'A')):.2f}" for j, n in enumerate(J)) + " (ret-buf-dq: la A di oggi diventa PREV)")
if rc == 8:
    print("ESITO rc=8 (nessun verdetto: |A−Z| sopra soglia, si estende R)"); sys.exit(8)
if morde:
    print("GUARDIA MORDE: " + " · ".join(morde) + " ⇒ cura O(1) PRIMA della promozione s184 (ordine obbligato (3)), guardia da ri-eseguire (rc=5)"); sys.exit(5)
print("GUARDIA PASS: (i) K=0 e giudici ereditati sotto soglia, (ii) K piatto, (iii) E3 ≤ 2,5, (iv) micro senza regressione ⇒ pin s184 com'è; cura O(1) = leva per NOME dopo la fetta 0 (rc=0)")
sys.exit(0)
PY
prc=$?
echo "sentinella LS fine: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
echo "ESITO rc=$prc $(date '+%F %T')"
} 2>&1 | tee -a "$VERD"
prc=$(sed -n 's/^ESITO rc=\([0-9]*\).*/\1/p' "$VERD" | tail -1)
fin "${prc:-7}"
