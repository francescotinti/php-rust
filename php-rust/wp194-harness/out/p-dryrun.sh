#!/bin/bash
# p-dryrun.sh — S-194 promotion, criterion p.3: the fixture gates of s194-promozione.sh (25 inherited + 5 new) run DRY with the
# candidate arm as BIN. The gate block is cut from the generated chain itself, so the dry run and the chain cannot drift.
# Outcome: out/p-dryrun.out (one line per gate), out/p-dryrun.rc (number of gates that bite).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; W="$SRC/wp194-harness"
H="$SRC/wp172-harness"; H4="$SRC/wp174-harness"; H8="$SRC/wp181-harness"; H9="$SRC/wp184-harness"; H3="$SRC/wp182-harness"
H10="$SRC/wp185-harness"; H11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"; H13="$SRC/wp193-harness"; H14="$SRC/wp194-harness"
BIN="$HOME/Claude/phpr-target/s192-armD/release/phpr"; REF="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s185"
OUT="$W/out/dryrun"; rm -rf "$OUT"; mkdir -p "$OUT"; V="$W/out/p-dryrun.out"; : > "$V"; BAD=0
note(){ echo "$1" >> "$V"; }
stop(){ echo "MORDE: $1" >> "$V"; BAD=$((BAD + 1)); return 0; }
cd "$SRC" || exit 2
echo "candidato $(shasum -a 256 "$BIN" | cut -c1-16) · REF $(shasum -a 256 "$REF" | cut -c1-16) · $(date '+%F %T')" >> "$V"
a=$(grep -n '^ORACLE=/opt/homebrew' "$W/s194-promozione.sh" | cut -d: -f1); b=$(grep -n '^# E2 (emenda S-177' "$W/s194-promozione.sh" | cut -d: -f1)
[ -n "$a" ] && [ -n "$b" ] || { echo "blocco dei gate non trovato" >> "$V"; echo 99 > "$W/out/p-dryrun.rc"; exit 99; }
sed -n "${a},$((b - 1))p" "$W/s194-promozione.sh" > "$OUT/gates.sh"
. "$OUT/gates.sh"
echo "gate che mordono: $BAD su $(grep -c -E '^(bilat|invar|expdiff) ' "$OUT/gates.sh")" >> "$V"; echo "$BAD" > "$W/out/p-dryrun.rc"
