#!/bin/bash
# s126-p23-chain.sh — catena SEQUENZIALE p.2→p.3: mappa2 POI aboff notturno
# (run pesanti sequenziali e detached). Lanciata via daemonize.pl.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H"; p(){ echo "$(date +%H:%M:%S) $1" >> "$H/p23-progress.txt"; }
: > "$H/p23-progress.txt"
rm -f "$H/p23-chain.done"
p "mappa2 START"
MAPPA_SP="${MAPPA_SP:?}" "$H/s126-mappa2-run.sh"; p "mappa2 rc=$? ($(cat "$H/mappa2-out/mappa2.done" 2>/dev/null))"
p "aboff START"
"$H/s126-aboff.sh"; p "aboff rc=$? ($(cat "$H/aboff-out/aboff.done" 2>/dev/null))"
echo "done $(date +%T)" > "$H/p23-chain.done"
p "CHAIN DONE"
