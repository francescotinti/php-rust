#!/bin/bash
# s149-ab-bt1.sh — A/B leva BT1 debug_backtrace options/limit (criterio
# s149-criterio-bt1.md, COMMITTATO prima del run). Base DICHIARATA:
# wp145-harness/s145-ab-fr1.sh (adattamenti: giudice m-backtrace 150k iter;
# gate fixture fx-backtrace BYTE-ID su B PRIMA delle misure — su A e' la
# divergenza che la cura chiude, stampata informativa; guardie famiglia
# reale: dimrmw+dimread+arr+prop+calls+str; rapporto bilaterale oracle med3).
# A = pin s145 · B = build leva (target separato). R=5 ABAB, user CPU netto
# pavimento PER-binario, mediana; smoke R=2 early-stop. rc = ab-out/ab.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp149-harness"
A="${ARM_A:?}"; B="${ARM_B:?}"
ORACLE=/opt/homebrew/opt/php/bin/php
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
EMPTY="$REPO/wp97-harness/micro/empty.php"
J="$H/m-backtrace.php"
FX="$H/fx-backtrace.php"
N_ITER=150000
G_RMW="$REPO/wp138-harness/m-dimrmw.php"
G_DR="$REPO/wp145-harness/m-dimread.php"
G_ARR="$REPO/wp97-harness/micro/arr.php"
G_PROP="$REPO/wp97-harness/micro/prop.php"
G_CALLS="$REPO/wp97-harness/micro/calls.php"
G_STR="$REPO/wp97-harness/micro/str.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s149-ab-bt1-verdetto${TAG:-}.out"
RC="$OUT/ab.rc"
LOCK="/private/tmp/phpr-measure.lock"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = TAG nuovo" >&2; exit 7; }

user_of(){ awk '/^user/{print $2}' "$1"; }
run_once(){ # $1=bin $2=script $3=tag → stampa user; verifica parita' stdout
  local bin="$1" scr="$2" tag="$3"
  local exp; exp="$("$ORACLE" "$scr")"
  local got; got=$( { /usr/bin/time -p "$bin" "$scr" > "$OUT/$tag.out"; } 2> "$OUT/$tag.time"; cat "$OUT/$tag.out")
  [ "$got" = "$exp" ] || { echo "PARITA' ROTTA $tag" >> "$OUT/parita.log"; }
  user_of "$OUT/$tag.time"
}
busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

{
echo "== s149 A/B BT1 debug_backtrace options/limit (criterio s149-criterio-bt1.md; decisione in s149-decisione-bt1.md scritta PRIMA) =="
echo "armA=$(shasum -a 256 "$A" | cut -c1-16) (pin s145)  armB=$(shasum -a 256 "$B" | cut -c1-16) (leva BT1)"
date "+start=%F %T"
echo "s149-ab-bt1 pid=$$" > "$LOCK"; echo "lock: creato"
QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; echo "quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || { echo "quiescenza MAI PASS — STOP"; echo 8 > "$RC"; rm -f "$LOCK"; exit 8; }
: > "$OUT/parita.log"

# gate fixture (criterio p.2): B DEVE essere byte-id con l'oracle; A stampa
# la divergenza che la cura chiude (informativa, attesa DIVERSA).
"$ORACLE" "$FX" > "$OUT/fx-oracle.out" 2>&1
"$B" "$FX" > "$OUT/fx-B.out" 2>&1
"$A" "$FX" > "$OUT/fx-A.out" 2>&1
if cmp -s "$OUT/fx-oracle.out" "$OUT/fx-B.out"; then
  echo "fixture fx-backtrace: B==oracle BYTE-ID (gate PASS)"
else
  echo "fixture fx-backtrace: B DIVERGE dall'oracle — STOP (criterio p.2)"
  diff "$OUT/fx-oracle.out" "$OUT/fx-B.out" | head -20
  echo 9 > "$RC"; rm -f "$LOCK"; date "+end=%F %T"; exit 9
fi
if cmp -s "$OUT/fx-oracle.out" "$OUT/fx-A.out"; then
  echo "fixture su A: identica (INATTESO: la divergenza dichiarata non morde qui)"
else
  echo "fixture su A: DIVERGE ($(diff "$OUT/fx-oracle.out" "$OUT/fx-A.out" | grep -c '^<') righe oracle) — la divergenza che BT1 chiude, attesa"
fi

# pavimenti per-binario (med3 su empty)
for arm in A B; do
  bin=$([ $arm = A ] && echo "$A" || echo "$B")
  for r in 1 2 3; do run_once "$bin" "$EMPTY" "fl$arm$r" > "$OUT/fl$arm$r.u"; done
done
FLA=$(sort -n "$OUT"/flA[123].u 2>/dev/null | awk 'NR==2'); [ -n "$FLA" ] || FLA=$(cat "$OUT"/flA1.u)
FLB=$(sort -n "$OUT"/flB[123].u 2>/dev/null | awk 'NR==2'); [ -n "$FLB" ] || FLB=$(cat "$OUT"/flB1.u)
echo "pavimenti: A=$FLA B=$FLB"

# bilaterale: oracle med3 sul giudice (rapporto, non gate)
for r in 1 2 3; do
  { /usr/bin/time -p "$ORACLE" "$J" > /dev/null; } 2> "$OUT/or$r.time"
  user_of "$OUT/or$r.time" > "$OUT/or$r.u"
done
ORU=$(sort -n "$OUT"/or[123].u | awk 'NR==2')
echo "oracle med3 user=$ORU (bilaterale, rapporto nel giudizio)"

# smoke R=2 ABAB (early-stop a segno opposto: B piu' LENTO in entrambe)
sa=(); sb=()
for r in 1 2; do
  echo "smoke r$r pre=$(busy)"
  sa+=("$(run_once "$A" "$J" "sm-A$r")"); sb+=("$(run_once "$B" "$J" "sm-B$r")")
done
SMOKE=$(python3 - "$FLA" "$FLB" "$N_ITER" "${sa[@]}" "${sb[@]}" <<'PY'
import sys
fa, fb, n, a1, a2, b1, b2 = map(float, sys.argv[1:8])
da = [(x-fa)/n*1e9 for x in (a1, a2)]
db = [(x-fb)/n*1e9 for x in (b1, b2)]
d = [x-y for x, y in zip(da, db)]
print(f"smoke ns/iter A={da[0]:.0f}/{da[1]:.0f} B={db[0]:.0f}/{db[1]:.0f} D={d[0]:+.0f}/{d[1]:+.0f}")
print("SMOKE-STOP" if (d[0] < 0 and d[1] < 0) else "SMOKE-OK")
PY
)
echo "$SMOKE"
if echo "$SMOKE" | grep -q "SMOKE-STOP"; then
  echo "smoke: SEGNO OPPOSTO in entrambe — STOP (criterio p.7, leva a catalogo)"
  rm -f "$LOCK"; echo 3 > "$RC"; date "+end=%F %T"; exit 3
fi

# R=5 ABAB sul giudice
ua=(); ub=()
for r in 1 2 3 4 5; do
  ua+=("$(run_once "$A" "$J" "ab-A$r")")
  ub+=("$(run_once "$B" "$J" "ab-B$r")")
done
echo "grezzi A: ${ua[*]}  B: ${ub[*]}"

# guardie R=3 per braccio (SOLO-REGRESSIONE, banda drop-1 della serie A)
for g in RMW DR ARR PROP CALLS STR; do
  case $g in RMW) S="$G_RMW";; DR) S="$G_DR";; ARR) S="$G_ARR";; PROP) S="$G_PROP";; CALLS) S="$G_CALLS";; STR) S="$G_STR";; esac
  gau=(); gbu=()
  for r in 1 2 3; do gau+=("$(run_once "$A" "$S" "g$g-A$r")"); gbu+=("$(run_once "$B" "$S" "g$g-B$r")"); done
  echo "guardia $g A: ${gau[*]}  B: ${gbu[*]}"
done
echo "parita' stdout: $(wc -l < "$OUT/parita.log" | tr -d ' ') violazioni ($(sort -u "$OUT/parita.log" 2>/dev/null | tr '\n' ' '))"
rm -f "$LOCK"; echo "lock: rimosso"
date "+end=%F %T"
echo 0 > "$RC"
} > "$VERD" 2>&1
exit "$(cat "$RC")"
