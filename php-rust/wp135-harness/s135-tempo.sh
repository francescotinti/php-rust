#!/bin/bash
# s135-tempo.sh — modello del tempo AssignPath su m3_int0 (criterio
# s135-criterio-tempo.md, commit PRIMA del run). Binario EMENDATO dal worktree
# s134 (patch s135-tp-s134.patch) — MAI pinnabile. UN segmento per run
# (PHPR_TP), R=3 per segmento, mediana ns/span; overhead = seg 9 (span vuoto),
# sottratto e dichiarato. Unilaterale phpr (indizio, REGOLE §4); parita'
# stdout vs oracle su ogni run. rc autoritativo = tempo-out/tempo.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
J="$H/micro-bisez/m3_int0.php"
BIN="/Volumes/Extreme Pro/Claude/phpr-s135-target-s134/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
OUT="$H/tempo-out"; mkdir -p "$OUT"
VERD="$H/s135-tempo-verdetto.out"
RC="$OUT/tempo.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$BIN" ] || { echo "manca il binario tp" >&2; echo 1 > "$RC"; exit 1; }

{
echo "== s135 modello tempo AssignPath su m3_int0 (criterio s135-criterio-tempo.md) =="
echo "tp_bin=$(shasum -a 256 "$BIN" | cut -c1-16) (build emendata, MAI pinnabile)"
EXP=$("$ORACLE" "$J")
FAIL=0
for seg in 0 1 2 3 4 5 6 9; do
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
python3 - "$S0" "$S1" "$S2" "$S3" "$S4" "$S5" "$S6" "$S9" <<'PY'
import sys
s0, s1, s2, s3, s4, s5, s6, s9 = map(float, sys.argv[1:9])
t = lambda x: x - s9
print(f"overhead_coppia_ns={s9:.1f} (seg9, sottratto)")
print(f"arm_totale={t(s0):.1f}  pop+keysvec={t(s1):.1f}  probe_aa={t(s2):.1f}  path_op={t(s3):.1f}")
print(f"  apply_last={t(s4):.1f}  set_displaced={t(s5):.1f}  gc_note={t(s6):.1f}")
print(f"derivate: walk(path_op-apply_last-gc)={t(s3)-t(s4)-t(s6):.1f}  coerce+ensure(apply_last-set)={t(s4)-t(s5):.1f}")
somma = t(s1) + t(s2) + t(s3)
print(f"chiusura=(pop+probe+path_op)/arm = {somma:.1f}/{t(s0):.1f} = {somma/t(s0)*100 if t(s0)>0 else 0:.0f}%  (criterio: >=90% o modello INCOMPLETO)")
print(f"dominante_35pct: " + " ".join(f"{n}={v/t(s0)*100:.0f}%" for n, v in (("pop", t(s1)), ("probe", t(s2)), ("path_op", t(s3))) if t(s0) > 0))
PY
if [ "$FAIL" = 0 ]; then echo "TEMPO ACQUISITO (parita' ok)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
