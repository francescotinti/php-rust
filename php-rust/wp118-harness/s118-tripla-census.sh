#!/bin/bash
# s118-tripla-census.sh — criterio R3 gate 4: tripla census per-request sul
# SORGENTE del pin s117 (A′+L-A). Ricetta WP-72 INVARIATA: build memgc in
# target SEPARATO (mai il release di parità) + probe72-tripla.sh + analyze72.pl
# come giudici; qui solo build, identità e delega. Attesa: obj/req 0,000
# spread 0,000 (riferimento WP-72 post-fix).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
MEMT="/Volumes/Extreme Pro/Claude/phpr-mem-target"
H72="/Volumes/Extreme Pro/Claude/wp72-harness"
OUT="$REPO/wp118-harness/census-out"; mkdir -p "$OUT"

cd "$REPO" || exit 2
echo "head=$(git rev-parse --short=12 HEAD)" | tee "$OUT/identity.txt"
echo "parity_pin=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)" | tee -a "$OUT/identity.txt"

# build census (ricetta build-memgc72 INVARIATA: features mem-census+gc-census;
# il profilo A′ del Cargo.toml di root si applica da sé — SOURCE_DATE_EPOCH=0
# CARGO_INCREMENTAL=0 per coerenza di ricetta)
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$MEMT" \
  cargo build --release -p php-cli --features mem-census,php-runtime/gc-census \
  > "$OUT/build-memgc.log" 2>&1
brc=$?
echo "$brc" > "$OUT/build-memgc.rc"
[ "$brc" = 0 ] || { echo "BUILD memgc FALLITA rc=$brc"; exit 1; }
cp "$MEMT/release/phpr" "$MEMT/release/phpr-memgc118"
echo "census_bin=$(shasum -a 256 "$MEMT/release/phpr-memgc118" | cut -c1-16)  # build strumentata, NON pin" | tee -a "$OUT/identity.txt"

# giudici WP-72 invariati
"$H72/probe72-tripla.sh" "$MEMT/release/phpr-memgc118"
prc=$?
echo "$prc" > "$OUT/tripla.rc"
cp "$H72/tripla-out/probe72-tripla.verdict" "$OUT/" 2>/dev/null
cp "$H72/tripla-out/tripla72-table.txt" "$OUT/" 2>/dev/null
echo "tripla rc=$prc (verdict copiato in wp118-harness/census-out/)"
exit "$prc"
