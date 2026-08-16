#!/bin/bash
# s149-census-orm.sh — census tranche-4 per-NOME-builtin su SUITE ORM
# (criterio s149-criterio-tr4.md). Base DICHIARATA: wp148-harness/
# s148-census-orm.sh (adattamenti: smoke s149 nomi str_repeat/sprintf +
# identita' s149sum EREDITANDO lo smoke s148; output in census-out/, verdetto
# s149). Manifest s149-census-copia.diff (copia-gate). MAI cifra di tempo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp149-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s149-tr4-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
rm -f "$OUT/census.done" "$OUT/sentinelle.txt" "$OUT/runrc.txt"

# smoke (criterio p.5): esito ESATTO — s148 EREDITATO (frame/hostcall/arrgrow
# >=1 + identita' s148sum) E s149: nomi str_repeat/sprintf n>=1, identita'
# hostcall_n==sum_name_n+unnamed_n, unnamed==0, overflow==0.
rm -f "$OUT/smoke-mem.txt"
PHPR_MEM_CENSUS="$OUT/smoke-mem.txt" \
  "$CENSUS_PHPR" "$H/smoke149.php" > "$SP/smoke149.out" 2>&1
SMOKE_FAIL=""
for t in frame hostcall arrgrow; do
  v=$(sed -n 's/^s148tag pid=[0-9]* tag=[0-9] name='"$t"' n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
  [ -n "${v:-}" ] && [ "$v" -ge 1 ] || SMOKE_FAIL="$SMOKE_FAIL s148tag.$t=${v:-ASSENTE}"
done
ga=$(sed -n 's/^s148sum pid=[0-9]* galloc_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
sm=$(sed -n 's/^s148sum pid=[0-9]* galloc_n=[0-9]* sum_n=\([0-9][0-9]*\)$/\1/p' "$OUT/smoke-mem.txt" | head -1)
if [ -z "${ga:-}" ] || [ -z "${sm:-}" ] || [ "$ga" != "$sm" ]; then
  SMOKE_FAIL="$SMOKE_FAIL identita_s148=galloc_n:${ga:-ASSENTE}!=sum_n:${sm:-ASSENTE}"
fi
for nm in str_repeat sprintf; do
  v=$(sed -n 's/^s149name pid=[0-9]* name='"$nm"' n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
  [ -n "${v:-}" ] && [ "$v" -ge 1 ] || SMOKE_FAIL="$SMOKE_FAIL s149name.$nm=${v:-ASSENTE}"
done
hn=$(sed -n 's/^s149sum pid=[0-9]* hostcall_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
sn=$(sed -n 's/^s149sum pid=[0-9]* hostcall_n=[0-9]* sum_name_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
un=$(sed -n 's/^s149sum pid=[0-9]* .* unnamed_n=\([0-9][0-9]*\) .*/\1/p' "$OUT/smoke-mem.txt" | head -1)
ov=$(sed -n 's/^s149sum pid=[0-9]* .* overflow=\([0-9][0-9]*\)$/\1/p' "$OUT/smoke-mem.txt" | head -1)
if [ -z "${hn:-}" ] || [ -z "${sn:-}" ] || [ -z "${un:-}" ] || [ -z "${ov:-}" ] \
   || [ "$hn" != "$((sn + un))" ] || [ "$un" != 0 ] || [ "$ov" != 0 ]; then
  SMOKE_FAIL="$SMOKE_FAIL identita_s149=hostcall:${hn:-ASSENTE},sum:${sn:-ASSENTE},unnamed:${un:-ASSENTE},overflow:${ov:-ASSENTE}"
fi
if [ -n "$SMOKE_FAIL" ]; then
  echo "rc=8 probe MUTO/zero su:$SMOKE_FAIL" > "$OUT/census.done"; exit 8
fi
p "smoke PASS (s148 ereditato + s149 nomi vivi + identita' ESATTA + unnamed/overflow 0)"

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" > "$OUT/census.done"; exit 7; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/census.done"; exit 7; }
  RAW="$OUT/census-mem-r$rep.txt"; rm -f "$RAW"
  echo "rep$rep pre=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep START"
  PHPR_MEM_CENSUS="$RAW" \
    "$CENSUS_PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
  PID=$!
  ( sleep 1800; kill -9 "$PID" 2>/dev/null ) & WDPID=$!
  wait "$PID"; RUNRC=$?
  kill "$WDPID" 2>/dev/null; wait "$WDPID" 2>/dev/null
  echo "rep$rep post=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep rc=$RUNRC"
  echo "rep$rep run_rc=$RUNRC raw=$(basename "$RAW")" >> "$OUT/runrc.txt"
  tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/rep$rep.failnames"
done

RC=0
for rep in 1 2; do
  diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/rep$rep.failnames" > /dev/null || { p "rep$rep parita' DIVERGE"; RC=1; }
done

{ echo "== s149 census tranche-4 per-NOME-builtin (hostcall.other 165,6M) su SUITE ORM (criterio s149-criterio-tr4.md — CONTEGGI, mai tempo) =="
  echo "grade=CENSUS  # monobinario census; l'aggregazione la fa s149-parse.py (golden-tested)"
  echo "census_phpr=$(shasum -a 256 "$CENSUS_PHPR" | cut -c1-16)  # probe build, MAI parita'"
  cat "$OUT/runrc.txt"
  echo "parita' per NOME vs baseline16: rc=$RC (0=entrambe ==)"
  echo "--- sentinelle (non-gate, S-143 p.1) ---"
  cat "$OUT/sentinelle.txt"
  for rep in 1 2; do
    echo "summary rep$rep: $(tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  done
} > "$VERD" 2>&1
echo "rc=$RC $(date +%T)" > "$OUT/census.done"
p "DONE rc=$RC"
