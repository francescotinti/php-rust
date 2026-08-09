#!/bin/bash
# s123-chain.sh — criterio s123-criterio-metro.md p.9: catena METRO detached,
# SEQUENZIALE, rc SEMPRE da file. gen-scaled -> banda v2 -> A/B ST1 -> giudice
# ST1 -> smoke RE2 -> [full RE2 se NON-DISTINGUIBILE o lato +] -> chain.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
OUT="$H/metro-out"; mkdir -p "$OUT"
rm -f "$OUT/chain.done" "$OUT/chain.rc"

step_rc() { [ -s "$1" ] && cat "$1" || echo 99; }

bash "$H/s123-gen-scaled.sh"
[ "$(step_rc "$OUT/gen.rc")" = 0 ] || { echo "gen-scaled FALLITO"; echo 1 > "$OUT/chain.rc"; echo done > "$OUT/chain.done"; exit 1; }

bash "$H/s123-layout-band-v2.sh"
[ "$(step_rc "$OUT/band.rc")" = 0 ] || { echo "banda v2 FALLITA rc=$(step_rc "$OUT/band.rc")"; echo 1 > "$OUT/chain.rc"; echo done > "$OUT/chain.done"; exit 1; }

bash "$H/s123-ab-v2.sh" "$S/phpr-s121-st1" 2e1eda8d st1 5 6 arith prop calls str arr re
bash "$H/s123-giudice-v2.sh" st1 str

bash "$H/s123-ab-v2.sh" "$S/phpr-s122-re2" 4eda5d6b re2smoke 6 0 re str
bash "$H/s123-giudice-v2.sh" re2smoke re
RS=$(step_rc "$OUT/giudice-re2smoke.rc")
if [ "$RS" = 0 ] || [ "$RS" = 2 ]; then
  echo "smoke RE2 rc=$RS (non archivia) -> FULL RE2 (criterio p.7)"
  bash "$H/s123-ab-v2.sh" "$S/phpr-s122-re2" 4eda5d6b re2full 5 6 arith prop calls str arr re
  bash "$H/s123-giudice-v2.sh" re2full re
fi

echo 0 > "$OUT/chain.rc"
echo done > "$OUT/chain.done"
