#!/bin/bash
# s138-ab-mdw.sh — A/B pin s135 vs pin s136 su m-dimwrite (criterio
# s138-criterio-ab-mdw.md, commit PRIMA del run). ABAB interleaved R=5,
# smoke = prime 2 ripetizioni con early-stop a segno opposto. ZERO probe.
# rc autoritativo = ab-mdw-out/ab.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp138-harness"
J="$REPO/wp136-harness/micro-bisez/m-dimwrite.php"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s135"
B="$HOME/Claude/php-rust-output/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
OUT="$H/ab-mdw-out"; mkdir -p "$OUT"
VERD="$H/s138-ab-mdw-verdetto.out"
RC="$OUT/ab.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ "$(shasum -a 256 "$A" | cut -c1-16)" = "6518a1e14a266d52" ] || { echo 9 > "$RC"; echo "hash s135 mismatch" >&2; exit 9; }
[ "$(shasum -a 256 "$B" | cut -c1-16)" = "1e14793ec0d9650c" ] || { echo 9 > "$RC"; echo "hash s136 mismatch" >&2; exit 9; }
"$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce.log" 2>&1 || { echo 8 > "$RC"; echo "quiescenza-fail" >&2; exit 8; }

{
echo "== s138 A/B m-dimwrite: pin s135 (6518a1e1) vs pin s136 (1e14793e) — criterio s138-criterio-ab-mdw.md =="
echo "quiescenza: rc=$(cat "$OUT/quiesce.rc")"
FAIL=0
EXP=$("$ORACLE" "$J")
ua=(); ub=()
run_leg() { # $1=bin $2=tag $3=r -> user in $RUSER, parità verificata
  local got
  got=$( { /usr/bin/time -p "$1" "$J" > "$OUT/$2-r$3.txt"; } 2> "$OUT/$2-r$3.time"; cat "$OUT/$2-r$3.txt")
  [ "$got" = "$EXP" ] || { echo "$2 r$3: PARITA' STDOUT ROTTA"; FAIL=1; }
  RUSER=$(awk '/^user/{print $2}' "$OUT/$2-r$3.time")
}
for r in 1 2 3 4 5; do
  run_leg "$A" a "$r"; ua+=("$RUSER")
  run_leg "$B" b "$r"; ub+=("$RUSER")
  echo "r$r: s135=${ua[$((r-1))]}s s136=${ub[$((r-1))]}s"
  if [ "$r" = 2 ]; then
    SM=$(python3 -c "
a=[${ua[0]},${ua[1]}]; b=[${ub[0]},${ub[1]}]
print('OK' if min(a) > max(b) or (sum(a)/2 > sum(b)/2) else 'SEGNO-OPPOSTO')")
    echo "smoke(2): $SM"
    [ "$SM" = "SEGNO-OPPOSTO" ] && { echo "SMOKE EARLY-STOP: segno opposto — A/B FERMATO (criterio p.3)"; FAIL=1; break; }
  fi
done
if [ "$FAIL" = 0 ]; then
python3 - "${ua[@]}" "${ub[@]}" <<'PY'
import sys
v = list(map(float, sys.argv[1:11]))
a, b = sorted(v[0:5]), sorted(v[5:10])
med_a, med_b = a[2], b[2]
N = 3_000_000
d_ns = (med_a - med_b) / N * 1e9
drop1 = max((a[3]-a[1])/N*1e9, (b[3]-b[1])/N*1e9)  # spread dei 3 centrali (drop-1 simmetrico)
rumore = max(4.0, drop1)
UB, BANDA = 69.6, 13.3
print(f"med5: s135={med_a:.2f}s s136={med_b:.2f}s (raw a: {v[0:5]}, b: {v[5:10]})")
print(f"D_mdw={d_ns:.1f} ns/iter  rumore={rumore:.1f}  (denominatore 3e6 dal sorgente)")
sc = d_ns - UB
print(f"identita'-prezzo: D_mdw {d_ns:.1f} vs UB {UB} -> scarto {sc:+.1f} (banda {BANDA})")
if abs(sc) <= BANDA:
    print("ESITO: IDENTITA'-PREZZO CHIUSA sul giudice del modello — BLOCCO DIM-WRITE RIMOSSO;")
    print("       eccedenza objdatains +13,7 RIATTRIBUITA a cross-giudice (apertura per NOME, senza blocco)")
else:
    verso = "D>UB: eccedenza REALE anche sul giudice proprio" if sc > 0 else "D<UB: il modello SOVRASTIMA i canali bypassati"
    print(f"ESITO: NON CHIUSA ({verso}) — blocco dim-write PERSISTE")
arm_v2 = 51.9
print(f"coerenza-arm (informativa, cross-strumento, banda non-omogeneita' >=2 ns/seg):")
print(f"  arm_v2 {arm_v2} + D_mdw {d_ns:.1f} = {arm_v2+d_ns:.1f} vs arm_pre 118,2 -> scarto {arm_v2+d_ns-118.2:+.1f}")
PY
fi
if [ "$FAIL" = 0 ]; then echo "A/B ACQUISITO (parita' ok)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
