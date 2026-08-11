#!/bin/bash
# s129-micro-gate.sh — gate micro R=5 sul pin s127b (obiettivo #2: osservare
# calls 4,9(*), REGOLE §4). Riusa run-micro.sh; confronto DICHIARATO coi
# rapporti s127b (arith 5,3 · prop 5,6 · calls 4,9 · str 4,2 · arr 3,2 · re 2,6).
# rc autoritativo = SOLO micro-out/micro.rc scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp129-harness"
OUT="$H/micro-out"; mkdir -p "$OUT"
VERD="$H/s129-micro-gate-verdetto.out"
PIN="$HOME/Claude/php-rust-output/release/phpr"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "ccb63dcaf565cffc" ] || { echo "pin!=s127b" > "$VERD"; echo 9 > "$OUT/micro.rc"; exit 9; }
PHPR="$PIN" R=5 "$REPO/wp97-harness/micro/run-micro.sh" > "$OUT/micro-raw.out" 2>&1
rrc=$?
{
echo "== s129 gate micro R=5 @ pin s127b (osservazione calls 4,9(*)) =="
echo "grade=VERDICT  # rc autoritativo = micro-out/micro.rc; riferimento = s127b-micro-pin-verdetto.out"
grep -E '^(pavimento|[a-z]+_N_iter|[a-z]+_(oracle|phpr)_netto_s|rapporto_|[a-z]+_spread)' "$OUT/micro-raw.out"
echo "-- confronto dichiarato vs s127b (run-to-run, non banda): arith 5,3 prop 5,6 calls 4,9 str 4,2 arr 3,2 re 2,6 --"
} > "$VERD" 2>&1
echo "$rrc" > "$OUT/micro.rc"
echo "done rc=$rrc $(date +%T)" > "$OUT/micro.done"
exit "$rrc"
