#!/bin/bash
# s195-dryrun.sh — S-195 promotion to pin s191, criterion s195-criterio-promozione.md p.7 (declared copy of wp194-harness/out/e-dryrun.sh, manifest
# s195-dryrun-copia.diff: names only): the fixture gates of s195-promozione.sh run DRY with the candidate arm as BIN. The gate block is cut from the
# generated chain itself, so the dry run and the chain cannot drift. Outcome: out/dryrun.out (one line per gate), out/dryrun.rc (gates that bite).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; W="$SRC/wp195-harness"
H="$SRC/wp172-harness"; H4="$SRC/wp174-harness"; H8="$SRC/wp181-harness"; H9="$SRC/wp184-harness"; H3="$SRC/wp182-harness"
H10="$SRC/wp185-harness"; H11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"; H13="$SRC/wp193-harness"; H14="$SRC/wp194-harness"; H15="$SRC/wp195-harness"
BIN="$W/out/phpr-cand"; REF="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s190"
OUT="$W/out/dryrun"; rm -rf "$OUT"; mkdir -p "$OUT"; V="$W/out/dryrun.out"; : > "$V"; BAD=0
note(){ echo "$1" >> "$V"; }
stop(){ echo "MORDE: $1" >> "$V"; BAD=$((BAD + 1)); return 0; }
cd "$SRC" || exit 2
echo "candidato $(shasum -a 256 "$BIN" | cut -c1-16) · REF $(shasum -a 256 "$REF" | cut -c1-16) · $(date '+%F %T')" >> "$V"
a=$(grep -n '^ORACLE=/opt/homebrew' "$W/s195-promozione.sh" | cut -d: -f1); b=$(grep -n '^# E2 (emenda S-177' "$W/s195-promozione.sh" | cut -d: -f1)
[ -n "$a" ] && [ -n "$b" ] || { echo "blocco dei gate non trovato" >> "$V"; echo 99 > "$W/out/dryrun.rc"; exit 99; }
sed -n "${a},$((b - 1))p" "$W/s195-promozione.sh" > "$OUT/gates.sh"
. "$OUT/gates.sh"
echo "gate che mordono: $BAD su $(grep -c -E '^(bilat|invar|expdiff) ' "$OUT/gates.sh")" >> "$V"; echo "$BAD" > "$W/out/dryrun.rc"
