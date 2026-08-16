#!/bin/bash
# s145-ab-fr1.sh — A/B leva FR1 dim-read fuso (criterio s145-criterio-fr1.md,
# COMMITTATO prima del run). A = worktree 407e93f (pre-leva, target dir
# dedicata) · B = HEAD 546a6f7 (leva). R=5 ABAB su m-dimread, user CPU al
# netto del pavimento PER-binario, mediana; smoke R=2 early-stop a segno
# opposto. GUARDIE (emenda DICHIARATA: il criterio p.5 nominava m-dimwrite
# che non esiste con quel nome — famiglia reale: m-dimrmw + m-diminc + arr +
# prop) SOLO-REGRESSIONE, banda = drop-1 della serie. rc = ab-out/ab.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp145-harness"
A="${ARM_A:?}"; B="${ARM_B:?}"
ORACLE=/opt/homebrew/opt/php/bin/php
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
EMPTY="$REPO/wp97-harness/micro/empty.php"
J="$H/m-dimread.php"
G_RMW="$REPO/wp138-harness/m-dimrmw.php"
G_INC="$REPO/wp138-harness/m-diminc.php"
G_ARR="$REPO/wp97-harness/micro/arr.php"
G_PROP="$REPO/wp97-harness/micro/prop.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s145-ab-fr1-verdetto${TAG:-}.out"
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
echo "== s145 A/B FR1 dim-read fuso (criterio s145-criterio-fr1.md) =="
echo "armA=$(shasum -a 256 "$A" | cut -c1-16) (407e93f pre-leva)  armB=$(shasum -a 256 "$B" | cut -c1-16) (546a6f7 leva)"
date "+start=%F %T"
echo "s145-ab pid=$$" > "$LOCK"; echo "lock: creato"
QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; echo "quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || { echo "quiescenza MAI PASS — STOP"; echo 8 > "$RC"; rm -f "$LOCK"; exit 8; }
: > "$OUT/parita.log"

# pavimenti per-binario (med3 su empty)
for arm in A B; do
  bin=$([ $arm = A ] && echo "$A" || echo "$B")
  for r in 1 2 3; do run_once "$bin" "$EMPTY" "fl$arm$r" > "$OUT/fl$arm$r.u"; done
done
FLA=$(sort -n "$OUT"/flA[123].u 2>/dev/null | awk 'NR==2'); [ -n "$FLA" ] || FLA=$(cat "$OUT"/flA1.u)
FLB=$(sort -n "$OUT"/flB[123].u 2>/dev/null | awk 'NR==2'); [ -n "$FLB" ] || FLB=$(cat "$OUT"/flB1.u)
echo "pavimenti: A=$FLA B=$FLB"

# smoke R=2 ABAB (early-stop a segno opposto)
sa=(); sb=()
for r in 1 2; do
  echo "smoke r$r pre=$(busy)"
  sa+=("$(run_once "$A" "$J" "sm-A$r")"); sb+=("$(run_once "$B" "$J" "sm-B$r")")
done
SMOKE=$(python3 - "$FLA" "$FLB" "${sa[@]}" "${sb[@]}" <<'PY'
import sys
fa, fb, a1, a2, b1, b2 = map(float, sys.argv[1:7])
da = [(x-fa)/3e6*1e9 for x in (a1, a2)]
db = [(x-fb)/3e6*1e9 for x in (b1, b2)]
d = [x-y for x, y in zip(da, db)]
print(f"smoke ns/iter A={da[0]:.1f}/{da[1]:.1f} B={db[0]:.1f}/{db[1]:.1f} D={d[0]:+.1f}/{d[1]:+.1f}")
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
for g in RMW INC ARR PROP; do
  case $g in RMW) S="$G_RMW";; INC) S="$G_INC";; ARR) S="$G_ARR";; PROP) S="$G_PROP";; esac
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
