#!/bin/bash
# s150-guardie-r5.sh — riparazione incidente 17 (az.rev.1 S-149): guardie
# SOLO-REGRESSIONE a R=5 come pre-registrato (criterio s149-criterio-bt1.md
# p.5: sei micro wp97 + m-dimread + m-dimrmw) + conferma leva m-backtrace R=5
# con pavimento oracle MISURATO (az.rev.4) + disasm bl-count run_loop (metodo
# S-109) nel .out di RECORD. A = stash pin s145 · B = candidato s150.
# Giudizio guardie (criterio s150-criterio-promo.md p.6): mediana user R=5;
# ROSSA se med_B > med_A + max(drop-1 di A, 0,01). rc = guardie-out/rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp150-harness"
A="${ARM_A:?stash pin s145}"; B="${ARM_B:?candidato}"
ORACLE=/opt/homebrew/opt/php/bin/php
EMPTY="$REPO/wp97-harness/micro/empty.php"
J="$REPO/wp149-harness/m-backtrace.php"
N_ITER=150000
OUT="$H/guardie-out"; mkdir -p "$OUT"
VERD="$H/s150-guardie-verdetto.out"
RC="$OUT/rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = TAG nuovo" >&2; exit 7; }

user_of(){ awk '/^user/{print $2}' "$1"; }
run_once(){ # $1=bin $2=script $3=tag → user; parita' stdout vs oracle
  local bin="$1" scr="$2" tag="$3"
  local exp; exp="$("$ORACLE" "$scr")"
  local got; got=$( { /usr/bin/time -p "$bin" "$scr" > "$OUT/$tag.out"; } 2> "$OUT/$tag.time"; cat "$OUT/$tag.out")
  [ "$got" = "$exp" ] || { echo "PARITA' ROTTA $tag" >> "$OUT/parita.log"; }
  user_of "$OUT/$tag.time"
}
med5(){ printf '%s\n' "$@" | sort -n | awk 'NR==3'; }

{
echo "== s150 guardie R=5 + conferma m-backtrace + disasm (criterio s150-criterio-promo.md p.6; riparazione incidente 17) =="
echo "armA=$(shasum -a 256 "$A" | cut -c1-16) (stash pin s145)  armB=$(shasum -a 256 "$B" | cut -c1-16) (candidato s150)"
date "+start=%F %T"
: > "$OUT/parita.log"

# pavimenti per-binario (med3 su empty) + pavimento ORACLE (az.rev.4)
for arm in A B; do
  bin=$([ $arm = A ] && echo "$A" || echo "$B")
  for r in 1 2 3; do run_once "$bin" "$EMPTY" "fl$arm$r" > "$OUT/fl$arm$r.u"; done
done
FLA=$(sort -n "$OUT"/flA[123].u | awk 'NR==2')
FLB=$(sort -n "$OUT"/flB[123].u | awk 'NR==2')
for r in 1 2 3; do
  { /usr/bin/time -p "$ORACLE" "$EMPTY" > /dev/null; } 2> "$OUT/flor$r.time"
  user_of "$OUT/flor$r.time" > "$OUT/flor$r.u"
done
FLOR=$(sort -n "$OUT"/flor[123].u | awk 'NR==2')
echo "pavimenti: A=$FLA B=$FLB oracle=$FLOR (med3, misurati)"

GRC=0
for g in ARITH PROP CALLS STR ARR RE DR RMW; do
  case $g in
    ARITH) S="$REPO/wp97-harness/micro/arith.php";;
    PROP)  S="$REPO/wp97-harness/micro/prop.php";;
    CALLS) S="$REPO/wp97-harness/micro/calls.php";;
    STR)   S="$REPO/wp97-harness/micro/str.php";;
    ARR)   S="$REPO/wp97-harness/micro/arr.php";;
    RE)    S="$REPO/wp97-harness/micro/re.php";;
    DR)    S="$REPO/wp145-harness/m-dimread.php";;
    RMW)   S="$REPO/wp138-harness/m-dimrmw.php";;
  esac
  gau=(); gbu=()
  for r in 1 2 3 4 5; do gau+=("$(run_once "$A" "$S" "g$g-A$r")"); gbu+=("$(run_once "$B" "$S" "g$g-B$r")"); done
  MA=$(med5 "${gau[@]}"); MB=$(med5 "${gbu[@]}")
  LINEA=$(python3 - "$MA" "$MB" "${gau[@]}" <<'PY'
import sys
ma, mb = float(sys.argv[1]), float(sys.argv[2])
a = sorted(map(float, sys.argv[3:8]))
drop1 = a[3] - a[0]                      # spread delle 4 migliori (drop-1)
soglia = max(drop1, 0.01)
print(f"drop1A={drop1:.2f} soglia={soglia:.2f} -> {'VERDE' if mb <= ma + soglia + 1e-9 else 'ROSSA'}")
PY
)
  echo "guardia $g A: ${gau[*]}  B: ${gbu[*]}  medA=$MA medB=$MB $LINEA"
  case "$LINEA" in *VERDE) : ;; *) GRC=1 ;; esac
done

# conferma leva: m-backtrace R=5 ABAB, ns/iter netto pavimento per-binario
ua=(); ub=()
for r in 1 2 3 4 5; do
  ua+=("$(run_once "$A" "$J" "mb-A$r")")
  ub+=("$(run_once "$B" "$J" "mb-B$r")")
done
echo "m-backtrace grezzi A: ${ua[*]}  B: ${ub[*]}"
for r in 1 2 3; do
  { /usr/bin/time -p "$ORACLE" "$J" > /dev/null; } 2> "$OUT/or$r.time"
  user_of "$OUT/or$r.time" > "$OUT/or$r.u"
done
ORU=$(sort -n "$OUT"/or[123].u | awk 'NR==2')
python3 - "$FLA" "$FLB" "$FLOR" "$ORU" "$N_ITER" "${ua[@]}" "${ub[@]}" <<'PY'
import sys
fa, fb, flor, oru, n = map(float, sys.argv[1:6])
ar = list(map(float, sys.argv[6:11])); br = list(map(float, sys.argv[11:16]))
segni = sum(1 for x, y in zip(ar, br) if x > y)   # in ORDINE di run (ABAB)
a = sorted(ar); b = sorted(br)
na = (a[2]-fa)/n*1e9; nb = (b[2]-fb)/n*1e9
drop1 = (a[3]-a[0])/n*1e9
soglia = max(4.0, drop1)
d = na - nb
orn = (oru - flor)/n*1e9
print(f"m-backtrace: A={na:.0f} ns/iter  B={nb:.0f} ns/iter  D={d:+.0f}  segni={segni}/5  soglia={soglia:.0f} -> {'CONFERMA' if d > soglia else 'SOTTO SOGLIA'}")
print(f"bilaterale NETTO (az.rev.4): oracle={orn:.0f} ns/iter (pavimento {flor} MISURATO) -> A/or={na/orn:.2f}x  B/or={nb/orn:.2f}x")
PY

# disasm bl-count run_loop (metodo S-109) — RECORD, atteso INVARIATO
for side in A B; do
  bin=$([ "$side" = A ] && echo "$A" || echo "$B")
  sym=$(nm -n "$bin" 2>/dev/null | grep "8run_loop17h" | awk '{print $3}')
  SZ=$(nm -n "$bin" 2>/dev/null | grep -A1 "8run_loop17h" | head -2 | python3 -c '
import sys
lines = sys.stdin.read().split("\n")
a = int(lines[0].split()[0], 16); b = int(lines[1].split()[0], 16)
print(b - a)')
  BL=$(otool -tv "$bin" 2>/dev/null | awk -v s="$sym:" '
    $0 == s {f=1; bl=0; next}
    f && /^__/ {print bl; exit}
    f && $2 == "bl" {bl++}
  ')
  echo "disasm $side: run_loop size=$SZ B bl=$BL"
  eval "BL_$side=$BL"
done
if [ "${BL_A:-x}" = "${BL_B:-y}" ]; then
  echo "disasm: bl-count INVARIATO ($BL_A) — come atteso (leva fuori dal dispatcher)"
else
  echo "disasm: bl-count DIFFORME (A=$BL_A B=$BL_B) — DICHIARATO a verbale (registro, non gate; criterio p.6)"
fi

echo "parita' stdout: $(wc -l < "$OUT/parita.log" | tr -d ' ') violazioni ($(sort -u "$OUT/parita.log" 2>/dev/null | tr '\n' ' '))"
echo "attese: SOLO mb-A* (la divergenza che BT1 chiude; A=pin s145)"
date "+end=%F %T"
echo "$GRC" > "$RC"
echo "GUARDIE rc=$GRC (0 = 8/8 VERDI)"
} > "$VERD" 2>&1
exit "$(cat "$RC")"
