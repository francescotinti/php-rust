#!/bin/bash
# s138-sonda-v2.sh — identità arm-only eccedenza FD1 (criterio
# s138-criterio-sonda-v2.md, commit f6fcbe5 PRIMA del run). Probe v2 ARM-ONLY
# (patch s138-tp-armonly.patch dal diff del worktree): seg0 = arm intero,
# seg9 = calibrazione; call-site di field_assign_fast INTATTO.
# Gate nell'ordine: nm (inline), parità fixtures, inerzia (PHPR_TP unset vs
# pin, R=3 user), poi tempo seg {0,9} R=3 mediana, overhead seg9 sottratto.
# Identità: |arm − 34,9| ≤ 13,3 → CHIUSA, altrimenti NON CHIUSA.
# rc autoritativo = sonda-v2-out/sonda.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp138-harness"
J="$REPO/wp136-harness/micro-bisez/m-dimwrite.php"
FIX="$REPO/wp136-harness/fixtures-fd1.php"
BIN="/Volumes/Extreme Pro/Claude/phpr-s137-target-s136/release/phpr"
PINBIN="$HOME/Claude/php-rust-output/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
OUT="$H/sonda-v2-out"; mkdir -p "$OUT"
VERD="$H/s138-sonda-v2-verdetto.out"
RC="$OUT/sonda.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$BIN" ] || { echo "manca il binario v2" >&2; echo 1 > "$RC"; exit 1; }
PIN_ATTESO="1e14793ec0d9650c"
[ "$(shasum -a 256 "$PINBIN" | cut -c1-16)" = "$PIN_ATTESO" ] || { echo "pin mismatch" >&2; echo 9 > "$RC"; exit 9; }
"$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce.log" 2>&1 || { echo 8 > "$RC"; echo "quiescenza-fail" >&2; exit 8; }

{
echo "== s138 sonda v2 arm-only (criterio s138-criterio-sonda-v2.md) =="
echo "v2_bin=$(shasum -a 256 "$BIN" | cut -c1-16) (build emendata, MAI pinnabile)"
echo "quiescenza: rc=$(cat "$OUT/quiesce.rc")"
FAIL=0
# gate (a): inline mantenuto nel probe v2 (nm, dettaglio bl-count fuori script)
NMC=$(nm "$BIN" | tr -d '\0' | grep -c "field_assign_fast")
echo "gate_nm_inline: field_assign_fast simboli=$NMC (atteso 0)"
[ "$NMC" = 0 ] || { echo "gate_nm_inline: FALLITO — sonda NON interpretabile"; FAIL=1; }
# gate (b): parità fixtures-fd1 probe==pin
"$BIN" "$FIX" > "$OUT/fix-probe.txt" 2>&1
"$PINBIN" "$FIX" > "$OUT/fix-pin.txt" 2>&1
if diff -q "$OUT/fix-probe.txt" "$OUT/fix-pin.txt" > /dev/null; then
  echo "parita_fixtures_fd1: OK (probe==pin)"
else
  echo "parita_fixtures_fd1: ROTTA"; FAIL=1
fi
EXP=$("$ORACLE" "$J")
# gate (d): inerzia — probe v2 SENZA PHPR_TP vs pin, R=3 user (criterio p.3d)
puser=(); nuser=()
for r in 1 2 3; do
  GOT=$( { /usr/bin/time -p "$BIN" "$J" > "$OUT/inerzia-probe-$r.txt"; } 2> "$OUT/inerzia-probe-$r.time"; cat "$OUT/inerzia-probe-$r.txt")
  [ "$GOT" = "$EXP" ] || { echo "inerzia probe r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
  puser+=("$(awk '/^user/{print $2}' "$OUT/inerzia-probe-$r.time")")
  GOT=$( { /usr/bin/time -p "$PINBIN" "$J" > "$OUT/inerzia-pin-$r.txt"; } 2> "$OUT/inerzia-pin-$r.time"; cat "$OUT/inerzia-pin-$r.txt")
  [ "$GOT" = "$EXP" ] || { echo "inerzia pin r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
  nuser+=("$(awk '/^user/{print $2}' "$OUT/inerzia-pin-$r.time")")
done
python3 - "${puser[@]}" "${nuser[@]}" <<'PY'
import sys
v = list(map(float, sys.argv[1:7]))
p, n = sorted(v[0:3]), sorted(v[3:6])
drop1 = max(p[1]-p[0], n[1]-n[0])  # drop-1 simmetrico: spread dei 2 migliori
soglia = max(0.012, drop1)         # 4 ns/iter x 3e6
delta = p[1] - n[1]
print(f"inerzia: probe_user_med3={p[1]:.2f} pin_user_med3={n[1]:.2f} "
      f"delta={delta:+.3f}s soglia={soglia:.3f}s (probe r: {v[0:3]}, pin r: {v[3:6]})")
print("inerzia: OK" if abs(delta) <= soglia else "inerzia: FALLITA — dichiarare (criterio p.3d)")
PY
# tempo: seg 0 (arm) e 9 (calibrazione), R=3, mediana ns/span
for seg in 0 9; do
  vals=()
  for r in 1 2 3; do
    GOT=$(PHPR_TP=$seg PHPR_TP_OUT="$OUT/seg$seg-r$r.tp" "$BIN" "$J")
    [ "$GOT" = "$EXP" ] || { echo "seg$seg r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
    vals+=("$(awk '{for(i=1;i<=NF;i++){if($i ~ /^ns=/){ns=substr($i,4)}; if($i ~ /^spans=/){sp=substr($i,7)}}; if(sp>0) printf "%.3f", ns/sp}' "$OUT/seg$seg-r$r.tp")")
  done
  M=$(printf '%s\n' "${vals[@]}" | sort -n | awk 'NR==2')
  echo "seg${seg}_ns_span_med3=$M (r: ${vals[*]})"
  eval "S$seg=$M"
done
python3 - "$S0" "$S9" <<'PY'
import sys
s0, s9 = map(float, sys.argv[1:3])
arm = s0 - s9
ATTESO, BANDA = 34.9, 13.3
print(f"overhead_coppia_ns={s9:.1f} (seg9, sottratto)")
print(f"arm_v2={arm:.1f} vs atteso {ATTESO} (118,2-83,3) banda ±{BANDA}")
print(f"scarto={arm-ATTESO:+.1f}")
if abs(arm - ATTESO) <= BANDA:
    print("ESITO SONDA V2: IDENTITA' CHIUSA senza ripartizione (criterio p.1) — "
          "blocco dim-write RIMOSSO; ipotesi densita'-timer-inattivi CORROBORATA")
else:
    print("ESITO SONDA V2: NON CHIUSA — blocco dim-write PERSISTE (=> p.4 NEXT_SESSION)")
PY
if [ "$FAIL" = 0 ]; then echo "SONDA V2 ACQUISITA (gate ok)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
