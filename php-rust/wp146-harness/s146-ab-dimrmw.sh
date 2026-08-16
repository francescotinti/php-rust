#!/bin/bash
# s146-ab-dimrmw.sh — az.rev. S-145 #2: rimisura m-dimrmw a densita' 10x
# (criterio s146-criterio-dimrmw.md COMMITTATO prima del run).
# A = stash phpr-s142 (pre-leva) · B = stash phpr-s145 (leva FR1).
# Smoke R=2 riscaldamento MAI giudicato, poi R=5 ABAB; giudice e BANDE
# DENTRO lo script (az.rev. S-145 #3): verdetto nel .out, rc dal giudizio.
# Il lock misura di finestra NON si tocca qui (veto trap-EXIT-altrui).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp146-harness"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
A="$STASH/phpr-s142"; B="$STASH/phpr-s145"
A_ATTESO="bba8a7346d727e0e"; B_ATTESO="a89faf32c62142f9"
ORACLE=/opt/homebrew/opt/php/bin/php
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
EMPTY="$REPO/wp97-harness/micro/empty.php"
J="$H/m-dimrmw10.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s146-ab-dimrmw-verdetto${TAG:-}.out"
RCF="$OUT/dimrmw.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = TAG nuovo" >&2; exit 7; }

HA="$(shasum -a 256 "$A" | cut -c1-16)"; HB="$(shasum -a 256 "$B" | cut -c1-16)"
if [ "$HA" != "$A_ATTESO" ] || [ "$HB" != "$B_ATTESO" ]; then
  echo "ABORT pin: A=$HA (atteso $A_ATTESO) B=$HB (atteso $B_ATTESO)" >&2
  echo 9 > "$RCF"; exit 9
fi

user_of(){ awk '/^user/{print $2}' "$1"; }
run_once(){ # $1=bin $2=script $3=tag → stampa user; parita' stdout vs oracle
  local bin="$1" scr="$2" tag="$3"
  local exp; exp="$("$ORACLE" "$scr")"
  local got; got=$( { /usr/bin/time -p "$bin" "$scr" > "$OUT/$tag.out"; } 2> "$OUT/$tag.time"; cat "$OUT/$tag.out")
  [ "$got" = "$exp" ] || { echo "PARITA' ROTTA $tag" >> "$OUT/dimrmw-parita.log"; }
  user_of "$OUT/$tag.time"
}
busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

{
echo "== s146 rimisura m-dimrmw 10x (criterio s146-criterio-dimrmw.md; grade GUARDIA — nessuna cifra in PERF_MAP) =="
echo "armA=$HA (stash phpr-s142 pre-leva)  armB=$HB (stash phpr-s145 leva FR1)"
echo "lock-finestra: $( [ -e /private/tmp/phpr-measure.lock ] && echo PRESENTE || echo ASSENTE )"
date "+start=%F %T"
QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/dimrmw-quiesce.rc" > "$OUT/dimrmw-quiesce-t$t.log" 2>&1; then QOK=0; echo "quiescenza: PASS al tentativo $t"; break; fi
  /bin/sleep 60
done
[ "$QOK" = 0 ] || { echo "quiescenza MAI PASS — STOP"; echo 8 > "$RCF"; date "+end=%F %T"; exit 8; }
: > "$OUT/dimrmw-parita.log"

# pavimenti per-binario (med3 su empty)
for arm in A B; do
  bin=$([ $arm = A ] && echo "$A" || echo "$B")
  for r in 1 2 3; do run_once "$bin" "$EMPTY" "dr-fl$arm$r" > "$OUT/dr-fl$arm$r.u"; done
done
FLA=$(sort -n "$OUT"/dr-flA[123].u | awk 'NR==2'); FLB=$(sort -n "$OUT"/dr-flB[123].u | awk 'NR==2')
echo "pavimenti: A=$FLA B=$FLB"

# smoke R=2 ABAB — RISCALDAMENTO dichiarato, MAI giudicato (criterio p.4)
for r in 1 2; do
  echo "smoke r$r pre=$(busy) A=$(run_once "$A" "$J" "dr-sm-A$r") B=$(run_once "$B" "$J" "dr-sm-B$r") post=$(busy)"
done

# R=5 ABAB giudicati
ua=(); ub=()
for r in 1 2 3 4 5; do
  echo "r$r pre=$(busy)"
  ua+=("$(run_once "$A" "$J" "dr-A$r")")
  ub+=("$(run_once "$B" "$J" "dr-B$r")")
done
echo "grezzi A: ${ua[*]}  B: ${ub[*]}"
PV=$(wc -l < "$OUT/dimrmw-parita.log" | tr -d ' ')
echo "parita' stdout: $PV violazioni"
if [ "$PV" != 0 ]; then echo "VERDETTO: PARITA' ROTTA — run invalida"; echo 5 > "$RCF"; date "+end=%F %T"; exit 5; fi

# giudice DENTRO lo script (az.rev. S-145 #3): bande calcolate, rc dal giudizio
python3 - "$FLA" "$FLB" "${ua[@]}" "${ub[@]}" <<'PY'
import sys
fa, fb = float(sys.argv[1]), float(sys.argv[2])
A = [(float(x)-fa)/3e7*1e9 for x in sys.argv[3:8]]
B = [(float(x)-fb)/3e7*1e9 for x in sys.argv[8:13]]
D = [b-a for a, b in zip(A, B)]                     # >0 = B (s145) piu' lento = regressione
def drop1(s):
    t = sorted(s); c = t[1:] if (t[-1]-t[1]) < (t[-2]-t[0]) else t[:-1]
    return c[-1]-c[0]
rumore = max(drop1(A), drop1(B))
soglia = max(1.0, rumore, 0.67)                     # criterio p.5: 3x risoluzione, rumore, banda-layout
Dm = sorted(D)[2]
segni = sum(1 for d in D if d > 0)
print(f"ns/iter A={['%.2f'%x for x in A]} B={['%.2f'%x for x in B]}")
print(f"D=B-A per replica: {['%+.2f'%d for d in D]}  Dmed={Dm:+.2f}  segni_pos={segni}/5")
print(f"rumore_drop1=max(A,B)={rumore:.2f}  soglia=max(1.0,{rumore:.2f},0.67)={soglia:.2f}")
if Dm >= soglia and segni >= 4:
    print("VERDETTO (p.6): REGRESSIONE CONFERMATA => leva FR1 in ISTRUTTORIA (dimread resta - keep-partial-wins)")
elif Dm <= -soglia and segni <= 1:
    print("VERDETTO (p.6): SEGNO OPPOSTO stabile => nessuna regressione, guardia chiusa")
else:
    print("VERDETTO (p.6): NON CONFERMATA (sotto soglia) => tick S-145 chiuso a verbale")
PY
date "+end=%F %T"
echo 0 > "$RCF"
} > "$VERD" 2>&1
exit "$(cat "$RCF")"
