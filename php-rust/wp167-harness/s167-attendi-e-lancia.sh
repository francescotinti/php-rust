#!/bin/bash
# s167-attendi-e-lancia.sh <script> <done_file> — attende quiete continua
# 6x30s (mediaanalysisd+mdworker <5%, predicato S-164/166) poi exec dello
# script; az.3 revisione S-166: il wrapper vive NEL repo con gli atti.
set -u
S="${1:?script}"; D="${2:?done}"
# attesa CI-quiet (cargo/rustc/phpt-runner) PRIMA del predicato indexer
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 30
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd|mdworker/ {s+=$2} END {print (s==""?0:s)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  [ "$calm" -lt 6 ] && sleep 30
done
"$S"; echo $? > "$D"
