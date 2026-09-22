#!/bin/bash
# s181-lancio-pair.sh — catena p.1: attende la quiete CI (cargo/rustc/
# phpt-runner: job CI in volo al lancio; il lock S-171 tiene
# ferma la coda), POI esige quiete CONTINUA anti-flare (criterio p.6,
# az.rev. S-163 #5: 6 campioni consecutivi x30s con mediaanalysisd <5%,
# predicato del riaggancio-r5b s163 ora PRE-registrato), DICHIARA la
# finestra quieta (uptime + top nel log) e lancia s181-pair.sh t23.
# COPIA DICHIARATA di s172-lancio-pair.sh (manifest s181-lancio-pair-copia.diff):
# soli nomi s181/t23; il resto INVARIATO (blocco anti-flare EREDITATO).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp181-harness"
LOG="$H/pair-out/lancio-t23.log"; mkdir -p "$H/pair-out"
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
# EMENDA t23 (S-181, dichiarata: t22 abortita rc=8 per loadavg1 7,87 ≥6 al gate di leg3-on, macchina in uso):
# PRE-gate di carico PRIMA del pair — loadavg1 <5 per 6 campioni consecutivi a 30 s (il gate s129 per gamba resta INVARIATO)
lc=0
while [ "$lc" -lt 6 ]; do
  L1=$(sysctl -n vm.loadavg | awk '{print $2}')
  if awk -v l="$L1" 'BEGIN{exit !(l+0 < 5.0)}'; then lc=$((lc+1)); else lc=0; fi
  echo "$(date '+%F %T') loadavg1=$L1 lc=$lc" >> "$LOG"
  [ "$lc" -lt 6 ] && sleep 30
done
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/pair-out/quiet-decl-t23.txt"
{ echo "== dichiarazione finestra quieta t23 $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-t23.txt — lancio pair t23" >> "$LOG"
exec /bin/bash "$H/s181-pair.sh" t23
