#!/bin/bash
# s163-lancio-pair.sh — catena p.1: attende la quiete CI (cargo/rustc/
# phpt-runner: run 38383a776a49 in volo all'apertura; il lock S-163 tiene
# ferma la coda), poi DICHIARA la finestra quieta (uptime + top nel log,
# rev. S-161) e lancia s163-pair.sh t13. Adattamento DICHIARATO di
# s162-lancio-pair.sh: RIMOSSA l'attesa rim.done (nessuna rimisura p.0 a
# S-163, criterio p.6); il resto INVARIATO.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp163-harness"
LOG="$H/pair-out/lancio-t13.log"; mkdir -p "$H/pair-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# dichiarazione finestra QUIETA (rev. S-161 / criterio p.6): uptime + top
Q="$H/pair-out/quiet-decl-t13.txt"
{ echo "== dichiarazione finestra quieta t13 $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-t13.txt — lancio pair t13" >> "$LOG"
exec /bin/bash "$H/s163-pair.sh" t13
