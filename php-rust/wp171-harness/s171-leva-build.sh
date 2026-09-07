#!/bin/bash
# s171-leva-build.sh — build del CANDIDATO B (leva L-SL1, criterio p.2) dall'ALBERO
# DI LAVORO del repo (commit dichiarato nel .out; albero crates DEVE essere pulito)
# con la ricetta del pin (SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build
# --release -p php-cli) in TARGET DEDICATO /private/tmp/s171-leva-tgt (mai
# php-rust-output: il pin canonico resta intatto fino alla promozione).
# Esiti: hash in ab-out/build-B.out, rc in ab-out/build-B.rc, .done a fine.
# Il target resta per il riuso (promozione = build canonica separata) e lo
# rimuove l'epilogo di sessione (binario tenuto nello stash via --braccio).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp171-harness"; OUT="$H/ab-out"; mkdir -p "$OUT"
TGT=/private/tmp/s171-leva-tgt
RC="$OUT/build-B.rc"; LOG="$OUT/build-B.log"; RES="$OUT/build-B.out"; DONE="$OUT/build-B.done"
rm -f "$RC" "$DONE"
cd "$REPO" || { echo 4 > "$RC"; touch "$DONE"; exit 4; }
DIRTY=$(git status --porcelain crates | wc -l | tr -d ' ')
echo "== build B $(date '+%F %T') — sorgente @ $(git rev-parse --short HEAD) (crates sporchi: $DIRTY) target $TGT" > "$RES"
[ "$DIRTY" = 0 ] || { echo "STOP: albero crates SPORCO — il candidato deve nascere da un commit" >> "$RES"; echo 3 > "$RC"; touch "$DONE"; exit 3; }
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-cli > "$LOG" 2>&1
brc=$?
if [ "$brc" -ne 0 ]; then echo "BUILD FALLITA rc=$brc (log $LOG)" >> "$RES"; echo "$brc" > "$RC"; touch "$DONE"; exit "$brc"; fi
[ -s "$TGT/release/phpr" ] || { echo "binario ASSENTE dopo build" >> "$RES"; echo 7 > "$RC"; touch "$DONE"; exit 7; }
echo "bin=$TGT/release/phpr hash=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16) fine $(date '+%F %T')" >> "$RES"
echo 0 > "$RC"; touch "$DONE"; exit 0
