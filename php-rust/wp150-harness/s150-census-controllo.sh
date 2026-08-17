#!/bin/bash
# s150-census-controllo.sh — run di CONTROLLO census workdir-lungo (criterio
# s150-criterio-census.md, az.rev.2 S-149). Base DICHIARATA: wp149-harness/
# s149-census-orm.sh (adattamenti: 1 SOLA replica; guardia ${#SP}≥100 char;
# guardia hash probe == f3a111ac92cac3ef; estrazione diretta s149sum dal raw
# — niente ri-aggregazione per NOME; verdetto s150). MAI cifra di tempo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp150-harness"
H149="$REPO/wp149-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s150-census-controllo-verdetto.out"
rm -f "$OUT/census.done"

HP=$(shasum -a 256 "$CENSUS_PHPR" | cut -c1-16)
[ "$HP" = "f3a111ac92cac3ef" ] || { echo "rc=6 probe $HP != f3a111ac92cac3ef (criterio p.2)" > "$OUT/census.done"; exit 6; }
WLEN=${#SP}
WORK="$SP/orm-work"; WWLEN=${#WORK}
[ "$WWLEN" -ge 100 ] || { echo "rc=6 workdir ${WWLEN} char < 100 (criterio p.2)" > "$OUT/census.done"; exit 6; }

# smoke probe EREDITATO (esito ESATTO, come s149)
rm -f "$OUT/smoke-mem.txt"
PHPR_MEM_CENSUS="$OUT/smoke-mem.txt" \
  "$CENSUS_PHPR" "$H149/smoke149.php" > "$SP/smoke150.out" 2>&1
SMOKE_FAIL=""
for t in frame hostcall arrgrow; do
  v=$(sed -n 's/^s148tag pid=[0-9]* tag=[0-9] name='"$t"' n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
  [ -n "${v:-}" ] && [ "$v" -ge 1 ] || SMOKE_FAIL="$SMOKE_FAIL s148tag.$t=${v:-ASSENTE}"
done
hn=$(sed -n 's/^s149sum pid=[0-9]* hostcall_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
sn=$(sed -n 's/^s149sum pid=[0-9]* hostcall_n=[0-9]* sum_name_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
un=$(sed -n 's/^s149sum pid=[0-9]* .* unnamed_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
ov=$(sed -n 's/^s149sum pid=[0-9]* .* overflow=\([0-9][0-9]*\)$/\1/p' "$OUT/smoke-mem.txt" | head -1)
if [ -z "${hn:-}" ] || [ -z "${sn:-}" ] || [ -z "${un:-}" ] || [ -z "${ov:-}" ] \
   || [ "$hn" != "$((sn + un))" ] || [ "$un" != 0 ] || [ "$ov" != 0 ]; then
  SMOKE_FAIL="$SMOKE_FAIL identita_s149=hostcall:${hn:-ASSENTE},sum:${sn:-ASSENTE},unnamed:${un:-ASSENTE},overflow:${ov:-ASSENTE}"
fi
[ -z "$SMOKE_FAIL" ] || { echo "rc=8 probe MUTO/zero su:$SMOKE_FAIL" > "$OUT/census.done"; exit 8; }

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

rm -rf "$WORK"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" > "$OUT/census.done"; exit 7; }
cd "$WORK" || { echo "rc=7 cd" > "$OUT/census.done"; exit 7; }
RAW="$OUT/census-mem-ctrl.txt"; rm -f "$RAW"
PRE=$(busy)
PHPR_MEM_CENSUS="$RAW" \
  "$CENSUS_PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run-ctrl.txt" 2>&1 &
PID=$!
( sleep 1800; kill -9 "$PID" 2>/dev/null ) & WDPID=$!
wait "$PID"; RUNRC=$?
kill "$WDPID" 2>/dev/null; wait "$WDPID" 2>/dev/null
POST=$(busy)
tr -d '\0' < "$OUT/run-ctrl.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/ctrl.failnames"
PAR=0; diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/ctrl.failnames" > /dev/null || PAR=1

# aggregazione s149sum su TUTTI i pid del raw (somma; identita' obbligatoria)
read -r HN SN UN OV <<EOF
$(awk '/^s149sum pid=/{ for(i=1;i<=NF;i++){ split($i,kv,"="); if(kv[1]=="hostcall_n")h+=kv[2]; else if(kv[1]=="sum_name_n")s+=kv[2]; else if(kv[1]=="unnamed_n")u+=kv[2]; else if(kv[1]=="overflow")o+=kv[2] } } END{print h, s, u, o}' "$RAW")
EOF
{ echo "== s150 census CONTROLLO workdir-lungo (criterio s150-criterio-census.md — CONTEGGI, mai tempo) =="
  echo "grade=CENSUS  # monobinario census, 1 replica dichiarata"
  echo "census_phpr=$HP  # probe CONSERVATO s149, MAI parita'"
  echo "workdir_len=${WWLEN} char ($WORK)"
  echo "run_rc=$RUNRC raw=$(basename "$RAW")"
  echo "sentinelle: pre=$PRE post=$POST (non-gate)"
  echo "summary: $(tr -d '\0' < "$OUT/run-ctrl.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  echo "parita' fail-set vs baseline16 (sentinella): rc=$PAR"
  echo "identita': hostcall_n=$HN sum_name_n=$SN unnamed_n=$UN overflow=$OV $([ "$HN" = "$((SN + UN))" ] && [ "$UN" = 0 ] && [ "$OV" = 0 ] && echo OK || echo VIOLATA-VOID)"
  python3 - "$HN" <<'PY'
import sys
hn = int(sys.argv[1]); s148 = 325416908; s149 = 335837200
d148 = (hn - s148) / s148 * 100; d149 = (hn - s149) / s149 * 100
print(f"confronto: vs s148(long-path)={d148:+.3f}%  vs s149(short-path)={d149:+.3f}%")
if abs(d148) <= 1.0:
    print("GIUDIZIO (p.3): spiegazione REGGE — lo scarto +3,2% era il path del workdir")
elif abs(d149) <= 1.0:
    print("GIUDIZIO (p.3): spiegazione CADE — il conteggio segue il run corto: scarto da RE-ISTRUIRE")
else:
    print("GIUDIZIO (p.3): FUORI da entrambe le bande — scarto da RE-ISTRUIRE")
PY
} > "$VERD" 2>&1
echo "rc=0 $(date +%T)" > "$OUT/census.done"
