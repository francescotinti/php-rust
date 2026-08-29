#!/bin/bash
# s162-lancio-rimisura.sh — attende la quiete CI (cargo/rustc/phpt-runner)
# poi riesegue s162-rimisura-al2.sh. Adattamento dichiarato del pattern
# s161-lancio-pair.sh (attesa + riassestamento singolo). Esiti da rim.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp162-harness"
LOG="$H/al2rim-out/lancio-rimisura.log"; mkdir -p "$H/al2rim-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
echo "$(date '+%F %T') quiete raggiunta — rilancio rimisura AL2" >> "$LOG"
exec /bin/bash "$H/s162-rimisura-al2.sh"
