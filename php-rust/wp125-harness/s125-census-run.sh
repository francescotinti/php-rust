#!/bin/bash
# s125-census-run.sh — criterio s125-criterio-census.md p.3: SOLO CONTEGGI,
# R=2, empty+6 micro originali, gambe A (fusa) e B (unfused). Nessun oracle.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s125-census-verdetto.out"
[ "$(cat "$OUT/build.rc" 2>/dev/null)" = 0 ] || { echo "build.rc != 0 — STOP" | tee -a "$VERD"; echo 1 > "$OUT/run.rc"; exit 1; }
for side in A B; do
  BIN="/Volumes/Extreme Pro/Claude/phpr-census-$([ "$side" = A ] && echo target || echo zv-target)/release/phpr"
  for c in empty arith prop calls str arr re; do
    for r in 1 2; do
      f="$OUT/$side-$c-r$r.census"; rm -f "$f"
      PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$BIN" "$MICRO/$c.php" > /dev/null 2>"$OUT/$side-$c-r$r.err"
      grep -q "^gacensus " "$f" || { echo "gacensus ASSENTE in $side-$c-r$r — STOP" | tee -a "$VERD"; echo 1 > "$OUT/run.rc"; exit 1; }
    done
  done
done
echo 0 > "$OUT/run.rc"
