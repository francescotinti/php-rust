#!/bin/bash
# s103-stack-census.sh — S-103 punto 3 (drop-census, prefisso H-C2): census pila operandi (A-BA-103-1)
# + gamba alloc a mem-census diretto (A-LE-103-1) sul giudice prop.
# grade=CENSUS (conteggi esatti, non-timing).
#
# Build census ISOLATA (target-dir dedicato, mai il release di parita'):
#   features zval-census (contatori specie + stackcensus) + mem-census
#   (CountingMi global_allocator: byte E conteggi alloc/free).
# Run: empty / prop_small 100K / prop 30M — linearita' 300:1 e coda
# costante come in S-101; l'assert conteggi<->dump si fa nel .out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp103-harness"
MICRO="$REPO/wp97-harness/micro"
TGT="/Volumes/Extreme Pro/Claude/phpr-census-target"
OUT="$H/stack-census-out"
mkdir -p "$OUT"

cd "$REPO" || exit 2
echo "== build census (target isolato $TGT)"
cargo build --release -p php-cli --features zval-census,mem-census \
  --target-dir "$TGT" 2>&1 | tail -2 || exit 3
BIN="$TGT/release/phpr"
echo "census_bin=$(shasum -a 256 "$BIN" | cut -c1-16)"

run1() { # $1=label $2=script
  local raw="$OUT/$1.census"
  rm -f "$raw"
  PHPR_ZVAL_CENSUS="$raw" MIMALLOC_PURGE_DELAY=0 "$BIN" "$MICRO/$2" > "$OUT/$1.stdout" 2>&1
  echo "== $1 rc=$? (stdout: $(head -c 60 "$OUT/$1.stdout" | tr '\n' ' '))"
}
run1 empty empty.php
run1 prop_small prop_small.php
run1 prop prop.php
echo "== raw in $OUT/*.census — analisi nel .out di sessione"
