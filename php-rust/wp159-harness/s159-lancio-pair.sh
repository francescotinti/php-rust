#!/bin/bash
# s159-lancio-pair.sh — attende la fine del job CI in volo (cargo/rustc/
# phpt-runner) poi lancia s159-pair.sh t9. Il lock misura è della SESSIONE
# (già creato); il runner CI a lock presente resta in quiet_wait tra i job.
# Adattamento dichiarato di s155-lancio-pair.sh (plumbing, esiti da .done).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp159-harness"
LOG="$H/pair-out/lancio-t9.log"; mkdir -p "$H/pair-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  # riassestamento: torna ad attendere una volta sola, poi procede comunque
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
echo "$(date '+%F %T') quiete raggiunta — lancio pair t9" >> "$LOG"
exec /bin/bash "$H/s159-pair.sh" t9
