#!/bin/bash
# s124-chain-ab.sh — catena DETACHED post-admission (criterio p.4-6):
# build ricetta A′ → stash candidato via scripts/pin-phpr.sh (collaudo
# nell'atto) → A/B alternato R=5 su 6 categorie → giudice v3 bersaglio str.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp124-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
TAG="${1:-str1}"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s124-$TAG"
OUT="$H/metro-out"; mkdir -p "$OUT"
rm -f "$OUT/ab-chain.done"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/ab-progress.txt"; }

p "chain ab start"
cd "$REPO" || exit 2
git diff --quiet -- crates/ || { p "PRE: crates/ sporco — STOP"; echo 1 > "$OUT/ab-chain.done"; exit 1; }

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-cand.log" 2>&1
rc=$?
p "build rc=$rc"
[ "$rc" = 0 ] || { echo 1 > "$OUT/ab-chain.done"; exit 1; }
CAND16=$(shasum -a 256 "$BIN" | cut -c1-16)
p "candidato $CAND16"

"$REPO/scripts/pin-phpr.sh" "s124-$TAG" > "$OUT/stash-cand.log" 2>&1
rc=$?
p "stash rc=$rc ($(tail -1 "$OUT/stash-cand.log"))"
[ "$rc" = 0 ] || { echo 1 > "$OUT/ab-chain.done"; exit 1; }
CAND8=$(printf '%s' "$CAND16" | cut -c1-8)

bash "$H/s124-ab.sh" "$STASH" "$CAND8" "$TAG" 5 3 arith prop calls str arr re > "$OUT/ab-run.log" 2>&1
rc=$?
p "ab rc=$rc"
[ "$rc" = 0 ] || { echo "$rc" > "$OUT/ab-chain.done"; exit "$rc"; }

bash "$H/s124-giudice-v3.sh" "$TAG" str > "$OUT/giudice-run.log" 2>&1
rc=$?
p "giudice rc=$rc"
echo "$rc" > "$OUT/ab-chain.done"
p "chain ab end rc=$rc"
