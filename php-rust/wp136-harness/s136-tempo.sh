#!/bin/bash
# s136-tempo.sh — modello del tempo FieldAssign su m-dimwrite (criterio
# s136-criterio-tempo.md, commit PRIMA del run). COPIA DICHIARATA di
# wp135-harness/s135-tempo.sh: adattamenti = binario dal worktree s135
# (patch s136-tp-s135.patch) — MAI pinnabile —, bench m-dimwrite.php,
# segmenti 0-8 (+9 calibrazione) e derivate del criterio s136. Metodo
# INVARIATO: UN segmento per run (PHPR_TP), R=3, mediana ns/span; overhead
# = seg 9 sottratto; parita' stdout vs oracle a ogni run.
# rc autoritativo = tempo-out/tempo.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
J="$H/micro-bisez/m-dimwrite.php"
BIN="/Volumes/Extreme Pro/Claude/phpr-s136-target-s135/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
OUT="$H/tempo-out"; mkdir -p "$OUT"
VERD="$H/s136-tempo-verdetto.out"
RC="$OUT/tempo.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$BIN" ] || { echo "manca il binario tp" >&2; echo 1 > "$RC"; exit 1; }

{
echo "== s136 modello tempo FieldAssign su m-dimwrite (criterio s136-criterio-tempo.md) =="
echo "tp_bin=$(shasum -a 256 "$BIN" | cut -c1-16) (build emendata, MAI pinnabile)"
EXP=$("$ORACLE" "$J")
FAIL=0
for seg in 0 1 2 3 4 5 6 7 8 9; do
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
python3 - "$S0" "$S1" "$S2" "$S3" "$S4" "$S5" "$S6" "$S7" "$S8" "$S9" <<'PY'
import sys
s0, s1, s2, s3, s4, s5, s6, s7, s8, s9 = map(float, sys.argv[1:11])
t = lambda x: x - s9
print(f"overhead_coppia_ns={s9:.1f} (seg9, sottratto)")
print(f"arm_totale={t(s0):.1f}  pop+keys={t(s1):.1f}  prelude_skip={t(s2):.1f}  field_set={t(s3):.1f}")
print(f"  field_write={t(s4):.1f}  prop_step={t(s5):.1f}  resolve={t(s6):.1f}  guardia={t(s7):.1f}  leaf_index={t(s8):.1f}")
print(f"derivate: dispatch+push(0-1-2-3)={t(s0)-t(s1)-t(s2)-t(s3):.1f}  plumbing_set(3-4)={t(s3)-t(s4):.1f}  walk_driver(4-5)={t(s4)-t(s5):.1f}  prop_step_altro(5-6-7-8)={t(s5)-t(s6)-t(s7)-t(s8):.1f}")
somma = t(s1) + t(s2) + t(s3)
print(f"chiusura=(1+2+3)/0 = {somma:.1f}/{t(s0):.1f} = {somma/t(s0)*100 if t(s0)>0 else 0:.0f}%  (criterio: >=90% o modello INCOMPLETO)")
byp = t(s6) + t(s7)
print(f"canali_bypassati_leva: resolve+guardia={byp:.1f} (+ quote di prop_step_altro/plumbing da nominare nel criterio leva)")
PY
if [ "$FAIL" = 0 ]; then echo "TEMPO ACQUISITO (parita' ok)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
