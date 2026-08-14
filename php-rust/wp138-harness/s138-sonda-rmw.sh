#!/bin/bash
# s138-sonda-rmw.sh <D_AB> — sonda attribuzione eccedenza fuori-modello RMW
# (criterio s138-criterio-sonda-rmw.md, commit PRIMA del run). Probe arm-only
# monobinario (patch s138-tp-rmw.patch dal diff del worktree @ HEAD leva):
# arm_fast (seg0) vs arm_full (seg0 + PHPR_TP_FULL=1) dallo STESSO binario.
# Identità: |(arm_full − arm_fast) − D_AB| ≤ 13,3 + ε 10 (p.4).
# rc autoritativo = sonda-rmw-out/sonda.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp138-harness"
J="$H/m-dimrmw.php"
FIX="$H/fixtures-rmw.php"
BIN="/Volumes/Extreme Pro/Claude/phpr-s137-target-s136/release/phpr"
CAND="$HOME/Claude/php-rust-output/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
OUT="$H/sonda-rmw-out"; mkdir -p "$OUT"
VERD="$H/s138-sonda-rmw-verdetto.out"
RC="$OUT/sonda.rc"
DAB="${1:?D_AB da R5}"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$BIN" ] || { echo "manca il probe" >&2; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$CAND" | cut -c1-8)" = "f06d7355" ] || { echo "cand mismatch" >&2; echo 9 > "$RC"; exit 9; }
"$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce.log" 2>&1 || { echo 8 > "$RC"; echo "quiescenza-fail" >&2; exit 8; }

{
echo "== s138 sonda attribuzione RMW (criterio s138-criterio-sonda-rmw.md; D_AB=$DAB) =="
echo "probe_bin=$(shasum -a 256 "$BIN" | cut -c1-16) (build emendata, MAI pinnabile)"
echo "quiescenza: rc=$(cat "$OUT/quiesce.rc")"
FAIL=0
NMC=$(nm "$BIN" | tr -d '\0' | grep -cE "field_rmw_fast|field_assign_fast")
echo "gate_nm_inline: simboli fast=$NMC (atteso 0)"
[ "$NMC" = 0 ] || { echo "gate_nm_inline: FALLITO"; FAIL=1; }
"$BIN" "$FIX" > "$OUT/fix-probe.txt" 2>&1
"$CAND" "$FIX" > "$OUT/fix-cand.txt" 2>&1
if diff -q "$OUT/fix-probe.txt" "$OUT/fix-cand.txt" > /dev/null; then
  echo "parita_fixtures_rmw: OK (probe==cand)"
else
  echo "parita_fixtures_rmw: ROTTA"; FAIL=1
fi
EXP=$("$ORACLE" "$J")
puser=(); cuser=()
for r in 1 2 3; do
  GOT=$( { /usr/bin/time -p "$BIN" "$J" > "$OUT/in-p$r.txt"; } 2> "$OUT/in-p$r.time"; cat "$OUT/in-p$r.txt")
  [ "$GOT" = "$EXP" ] || { echo "inerzia probe r$r: PARITA' ROTTA"; FAIL=1; }
  puser+=("$(awk '/^user/{print $2}' "$OUT/in-p$r.time")")
  GOT=$( { /usr/bin/time -p "$CAND" "$J" > "$OUT/in-c$r.txt"; } 2> "$OUT/in-c$r.time"; cat "$OUT/in-c$r.txt")
  [ "$GOT" = "$EXP" ] || { echo "inerzia cand r$r: PARITA' ROTTA"; FAIL=1; }
  cuser+=("$(awk '/^user/{print $2}' "$OUT/in-c$r.time")")
done
python3 - "${puser[@]}" "${cuser[@]}" <<'PY'
import sys
v = list(map(float, sys.argv[1:7]))
p, c = sorted(v[0:3]), sorted(v[3:6])
drop1 = max(p[1]-p[0], c[1]-c[0])
soglia = max(0.012, drop1)
delta = p[1] - c[1]
print(f"inerzia: probe_med3={p[1]:.2f} cand_med3={c[1]:.2f} delta={delta:+.3f}s soglia={soglia:.3f}s")
print("inerzia: OK" if abs(delta) <= soglia else "inerzia: FALLITA — dichiarare (criterio p.5)")
PY
declare -a M
for MODE in fast full cal; do
  vals=()
  for r in 1 2 3; do
    case "$MODE" in
      fast) GOT=$(PHPR_TP=0 PHPR_TP_OUT="$OUT/$MODE-r$r.tp" "$BIN" "$J");;
      full) GOT=$(PHPR_TP=0 PHPR_TP_FULL=1 PHPR_TP_OUT="$OUT/$MODE-r$r.tp" "$BIN" "$J");;
      cal)  GOT=$(PHPR_TP=9 PHPR_TP_OUT="$OUT/$MODE-r$r.tp" "$BIN" "$J");;
    esac
    [ "$GOT" = "$EXP" ] || { echo "$MODE r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
    vals+=("$(awk '{for(i=1;i<=NF;i++){if($i ~ /^ns=/){ns=substr($i,4)}; if($i ~ /^spans=/){sp=substr($i,7)}}; if(sp>0) printf "%.3f", ns/sp}' "$OUT/$MODE-r$r.tp")")
  done
  MV=$(printf '%s\n' "${vals[@]}" | sort -n | awk 'NR==2')
  echo "${MODE}_ns_span_med3=$MV (r: ${vals[*]})"
  eval "S_$MODE=$MV"
done
python3 - "$S_fast" "$S_full" "$S_cal" "$DAB" <<'PY'
import sys
sf, su, sc, dab = map(float, sys.argv[1:5])
arm_fast, arm_full = sf - sc, su - sc
diffa = arm_full - arm_fast
BANDA, EPS = 13.3, 10.0
print(f"overhead_coppia_ns={sc:.1f} (seg9, sottratto)")
print(f"arm_fast={arm_fast:.1f}  arm_full={arm_full:.1f}  (arm_full-arm_fast)={diffa:.1f}")
print(f"identita': {diffa:.1f} vs D_AB {dab:.1f} -> scarto {diffa-dab:+.1f} (banda {BANDA}+eps {EPS}; eps=prelude_skip+fill nel pieno del candidato, atteso verso l'alto)")
if abs(diffa - dab) <= BANDA + EPS:
    print("ESITO SONDA: ECCEDENZA ATTRIBUITA per NOME — l'eccedenza fuori-modello vive nei canali dichiarati")
    print("             (read-walk field_value + secondo preludio + write-walk field_set_op, contrasto omogeneo monobinario)")
else:
    print("ESITO SONDA: NON CHIUSA — promozione FERMA (criterio p.4)")
PY
if [ "$FAIL" = 0 ]; then echo "SONDA ACQUISITA (gate ok)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
