#!/bin/bash
# s123-classifica-run.sh — criterio s123-criterio-classifica.md p.4-5: SOLO
# CONTEGGI. R=2 su empty+6 micro ORIGINALI; gamba phpr (gacensus) + gamba
# oracle (count-interpose wp119). Poi report v1-vs-v2 con N omogenei.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp123-harness"
H119="$REPO/wp119-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/classifica-out"; mkdir -p "$OUT"
CENSUS_BIN="/Volumes/Extreme Pro/Claude/phpr-census-target/release/phpr"
ORACLE="/opt/homebrew/opt/php/bin/php"
DYLIB="$H119/count-interpose.dylib"
VERD="$H/s123-classifica-verdetto.out"

[ "$(cat "$OUT/build.rc" 2>/dev/null)" = 0 ] || { echo "build.rc != 0 — STOP" | tee -a "$VERD"; echo 1 > "$OUT/run.rc"; exit 1; }
[ -f "$DYLIB" ] || { echo "dylib interpose ASSENTE — STOP" | tee -a "$VERD"; echo 1 > "$OUT/run.rc"; exit 1; }

echo "oracle=$("$ORACLE" -v | head -1)" > "$OUT/identity-run.txt"
for c in empty arith prop calls str arr re; do
  for r in 1 2; do
    f="$OUT/phpr-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$CENSUS_BIN" "$MICRO/$c.php" > /dev/null 2>"$OUT/phpr-$c-r$r.err"
    g="$OUT/zend-$c-r$r.count"; rm -f "$g"
    CLITE_COUNT_OUT="$g" DYLD_INSERT_LIBRARIES="$DYLIB" USE_ZEND_ALLOC=0 \
      "$ORACLE" "$MICRO/$c.php" > /dev/null 2>"$OUT/zend-$c-r$r.err"
  done
done
echo 0 > "$OUT/run.rc"
