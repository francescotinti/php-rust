#!/bin/bash
# s166-lancio-pair.sh — catena p.1: attende la quiete CI (cargo/rustc/
# phpt-runner: job CI in volo al lancio; il lock S-166 tiene
# ferma la coda), POI esige quiete CONTINUA anti-flare (criterio p.6,
# az.rev. S-163 #5: 6 campioni consecutivi x30s con mediaanalysisd <5%,
# predicato del riaggancio-r5b s163 ora PRE-registrato), DICHIARA la
# finestra quieta (uptime + top nel log) e lancia s166-pair2.sh t16.
# Adattamento DICHIARATO di s164-lancio-pair.sh: AGGIUNTO il blocco
# anti-flare tra attesa CI e dichiarazione; il resto INVARIATO.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp166-harness"
LOG="$H/pair-out/lancio-t16.log"; mkdir -p "$H/pair-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# predicato anti-flare PRE-registrato (criterio p.6b): quiete CONTINUA 6x30s
echo "$(date '+%F %T') CI quieta — attesa quiete continua anti-flare (6x30s <5%)" >> "$LOG"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/pair-out/quiet-decl-t16.txt"
{ echo "== dichiarazione finestra quieta t16 $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-t16.txt — lancio pair t16" >> "$LOG"
exec /bin/bash "$H/s166-pair2.sh" t16
