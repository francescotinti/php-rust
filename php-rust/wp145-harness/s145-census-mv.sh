#!/bin/bash
# s145-census-mv.sh — sonda-B lato CONTEGGI (movimenti per classe + note
# container) su SUITE ORM, monobinario census. Modello s145-sonda-b-modello.md;
# base DICHIARATA: wp144-harness/s144-census-tranche2.sh (adattamenti: chiavi
# s145.*, smoke anche sul dump zval per gcnote_cont, output in sonda-out/
# accanto ai prezzi per il parser unico). MAI cifra di tempo da qui.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp145-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s145-census-mv.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
rm -f "$OUT/census.done" "$OUT/sentinelle.txt" "$OUT/runrc.txt"

# smoke: chiavi s145 a esito ESATTO >=1 (ref/rcother FUORI smoke, dichiarato
# nel modello: in uno script minimo possono restare 0).
rm -f "$OUT/smoke-mem.txt" "$OUT/smoke-zval.txt"
PHPR_MEM_CENSUS="$OUT/smoke-mem.txt" PHPR_ZVAL_CENSUS="$OUT/smoke-zval.txt" \
  "$CENSUS_PHPR" "$H/smoke145.php" > "$SP/smoke145.out" 2>&1
SMOKE_FAIL=""
for k in s145.clone_scalar_n s145.clone_str_n s145.clone_arr_n s145.clone_obj_n; do
  v=$(sed -n 's/.* tag=exit .*'"$k"'=\([0-9][0-9]*\).*/\1/p' "$OUT/smoke-mem.txt" | head -1)
  [ -n "${v:-}" ] && [ "$v" -ge 1 ] || SMOKE_FAIL="$SMOKE_FAIL $k=${v:-ASSENTE}"
done
v=$(sed -n 's/^zvalcensus_s145 gcnote_cont=\([0-9][0-9]*\)$/\1/p' "$OUT/smoke-zval.txt" | head -1)
[ -n "${v:-}" ] && [ "$v" -ge 1 ] || SMOKE_FAIL="$SMOKE_FAIL gcnote_cont=${v:-ASSENTE}"
if [ -n "$SMOKE_FAIL" ]; then
  echo "rc=8 probe MUTO/zero su:$SMOKE_FAIL" > "$OUT/census.done"; exit 8
fi
p "smoke PASS (s145.* vivi)"

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" > "$OUT/census.done"; exit 7; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/census.done"; exit 7; }
  RAW="$OUT/census-mem-r$rep.txt"; rm -f "$RAW"
  RAWZ="$OUT/census-zval-r$rep.txt"; rm -f "$RAWZ"
  echo "rep$rep pre=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep START"
  PHPR_MEM_CENSUS="$RAW" PHPR_ZVAL_CENSUS="$RAWZ" "$CENSUS_PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
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

{ echo "== s145 census MOVIMENTI (clone per classe + gcnote_cont) su SUITE ORM (modello s145-sonda-b-modello.md — CONTEGGI, mai tempo) =="
  echo "grade=CENSUS  # monobinario census; la partizione la fa s145-partition.py (golden-tested)"
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
