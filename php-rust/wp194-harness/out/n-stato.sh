#!/bin/bash
# n-stato.sh — S-194 night: state probe (file-based by rule).
W="$(cd "$(dirname "$0")/.." && pwd)"
tail -4 "/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; ls "/Volumes/Extreme Pro/Claude/phpr-ci/queue" 2>/dev/null | head -3
pgrep -fl "n-coda|lancio-notte|s186-guardia|orm-coppia|s194-pair" | cut -c1-120; pgrep -x cargo | head -2
ls -la "$W/catena-out" | tail -5; tail -5 "$W/catena-out/lancio-catena.log" 2>/dev/null | cut -c1-200; date
