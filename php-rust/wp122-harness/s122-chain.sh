#!/bin/bash
# s122-chain.sh — criterio s122-criterio-layout.md p.9: catena SEQUENZIALE
# build P1..P3 → banda-LAYOUT → full A/B L-ST1 → giudice. rc SEMPRE da file;
# uno stadio fallito ferma la catena (fail-closed).
set -u
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp122-harness"
LOG="$H/layout-out"; mkdir -p "$LOG"

echo "== stadio 1: build probes =="
bash "$H/s122-layout-build.sh"
[ "$(cat "$LOG/build.rc" 2>/dev/null)" = 0 ] || { echo "CHAIN STOP: build.rc != 0"; exit 1; }

echo "== stadio 2: banda-LAYOUT =="
bash "$H/s122-layout-band.sh"
[ "$(cat "$LOG/band.rc" 2>/dev/null)" = 0 ] || { echo "CHAIN STOP: band.rc != 0"; exit 2; }

echo "== stadio 3: full A/B L-ST1 (misura invariata; verdetto vecchio = advisory) =="
bash "$H/s122-ab-st1.sh" full || true
FR=$(cat "$H/st1-out/full-rc" 2>/dev/null)
case "$FR" in
  2|3) echo "CHAIN STOP: full-rc=$FR (parità/misura)"; exit 3;;
esac

echo "== stadio 4: giudice S-122 =="
bash "$H/s122-st1-giudice.sh"
echo "CHAIN DONE: giudice.rc=$(cat "$H/st1-out/giudice.rc" 2>/dev/null)"
echo done > "$LOG/chain.done"
