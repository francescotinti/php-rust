#!/bin/bash
# s119-clite-run.sh — C-lite ESECUZIONE (criterio s119-clite-criterio.md).
# SOLO CONTEGGI: nessuna cifra di tempo esce da questa run (REGOLE §3).
# Gamba phpr: build census (zval-census+mem-census, target separato) — riga
# `gacensus` (galloc_n/zvclone_*) da PHPR_MEM_CENSUS. Gamba Zend: oracle brew
# con USE_ZEND_ALLOC=0 + dylib interposta (deroga 3, conteggio eventi).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
MICRO="$H/../wp97-harness/micro"
OUT="$H/clite-out"
CENSUS_BIN="/Volumes/Extreme Pro/Claude/phpr-census-target/release/phpr"
ORACLE="/opt/homebrew/opt/php/bin/php"
DYLIB="$H/count-interpose.dylib"
R=2
CATS="empty arith prop calls str arr re"
mkdir -p "$OUT"

{
  echo "census_bin=$(shasum -a 256 "$CENSUS_BIN" | cut -c1-16)  # build strumentata, NON il pin"
  echo "parity_pin=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
  echo "oracle=$("$ORACLE" -v | head -1)"
  echo "head=$(git -C "$H/.." rev-parse --short HEAD)"
  echo "epoch=$(date +%s)"
} > "$OUT/identity.txt"

step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"

for c in $CATS; do
  for r in $(seq 1 $R); do
    f="$OUT/phpr-$c-r$r.census"; rm -f "$f"
    step "phpr $c r$r"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$CENSUS_BIN" "$MICRO/$c.php" > /dev/null 2>"$OUT/phpr-$c-r$r.err"
    echo "rc=$?" >> "$OUT/phpr-$c-r$r.err"
  done
done

for c in $CATS; do
  for r in $(seq 1 $R); do
    f="$OUT/zend-$c-r$r.count"; rm -f "$f"
    step "zend $c r$r"
    CLITE_COUNT_OUT="$f" DYLD_INSERT_LIBRARIES="$DYLIB" USE_ZEND_ALLOC=0 \
      "$ORACLE" "$MICRO/$c.php" > /dev/null 2>"$OUT/zend-$c-r$r.err"
    echo "rc=$?" >> "$OUT/zend-$c-r$r.err"
  done
done

step "DONE"
echo done > "$OUT/run.done"
