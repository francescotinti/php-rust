#!/bin/bash
# s162-lancio-pair.sh — catena p.1: attende (a) rim.done della rimisura AL2
# (p.0, run sequenziali) e (b) la quiete CI (cargo/rustc/phpt-runner), poi
# DICHIARA la finestra quieta (uptime + top nel log, rev. S-161) e lancia
# s162-pair.sh t12. Adattamento dichiarato di s161-lancio-pair.sh.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp162-harness"
LOG="$H/pair-out/lancio-t12.log"; mkdir -p "$H/pair-out"
RIM="$H/al2rim-out/rim.done"
echo "$(date '+%F %T') attesa rimisura AL2 ($RIM)" >> "$LOG"
while [ ! -e "$RIM" ]; do sleep 60; done
echo "$(date '+%F %T') rimisura AL2 conclusa ($(cat "$RIM")) — attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# dichiarazione finestra QUIETA (rev. S-161 / NEXT_SESSION p.1): uptime + top
Q="$H/pair-out/quiet-decl-t12.txt"
{ echo "== dichiarazione finestra quieta t12 $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-t12.txt — lancio pair t12" >> "$LOG"
exec /bin/bash "$H/s162-pair.sh" t12
