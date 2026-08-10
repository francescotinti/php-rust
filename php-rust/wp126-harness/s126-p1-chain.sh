#!/bin/bash
# s126-p1-chain.sh — catena SEQUENZIALE p.1: micro istruttoria (bilaterale,
# macchina quieta) POI profilo sample (unilaterale). Lanciata via daemonize.pl.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H/orm-out"; mkdir -p "$OUT"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/chain-progress.txt"; }
: > "$OUT/chain-progress.txt"
rm -f "$OUT/p1-chain.done"
p "micro START"
"$H/s126-orm-micro.sh"; p "micro rc=$? ($(cat "$OUT/micro.done" 2>/dev/null))"
p "profile START"
PROF_SP="${PROF_SP:?PROF_SP richiesto}" "$H/s126-orm-profile.sh"; p "profile rc=$? ($(cat "$OUT/profile.done" 2>/dev/null))"
echo "done $(date +%T)" > "$OUT/p1-chain.done"
p "CHAIN DONE"
