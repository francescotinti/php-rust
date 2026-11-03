#!/bin/bash
# s189-lancio.sh — FETTA 1 S-189 (guardia AX + tripwire W=1, criterio s189-criterio-guardia-ax.md p.7): COPIA
# DICHIARATA di ../wp188-harness/s188-lancio.sh (manifest s189-lancio-copia.diff, generatore gen-fetta1.py) coi SOLI
# adattamenti: harness wp189, nomi fetta1, lock TOKEN s189, watchdog su catena-fetta1.done, exec s189-catena-fetta1.sh.
# Il resto INVARIATO (gate di quiete CI, anti-flare, loadavg, E2 CPU, sentinella hog, watchdog Data, dichiarazione quieta).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp189-harness"
LOG="$H/pair-out/lancio-fetta1.log"; mkdir -p "$H/pair-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# EMENDA t24 (i): LOCK col TOKEN s183 — apre la finestra multi-gamba (la CI si mette in mutex sul lock); rimosso a chiusura di sessione
MLOCK=/private/tmp/phpr-measure.lock
echo "s189 FETTA 1 guardia AX + tripwire W=1 @ pin s189 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s189)" >> "$LOG"
# predicato anti-flare PRE-registrato (criterio p.6b): quiete CONTINUA 6x30s
echo "$(date '+%F %T') CI quieta — attesa quiete continua anti-flare (6x30s <5%)" >> "$LOG"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
# EMENDA t23 EREDITATA + EMENDA t24 (ii): PRE-gate di carico PRIMA del pair — loadavg1 <3 (era <5) per 6 campioni consecutivi a 30 s
# (finestra NOTTURNA = conseguenza del gate, non un orario; il gate s129 per gamba resta INVARIATO)
lc=0
while [ "$lc" -lt 6 ]; do
  L1=$(sysctl -n vm.loadavg | awk '{print $2}')
  if awk -v l="$L1" 'BEGIN{exit !(l+0 < 3.0)}'; then lc=$((lc+1)); else lc=0; fi
  echo "$(date '+%F %T') loadavg1=$L1 lc=$lc" >> "$LOG"
  [ "$lc" -lt 6 ] && sleep 30
done
# EMENDA t24 (iii): E2 calma CPU TOTALE <150 % per 4 campioni a 30 s (predicato COPIATO da s181-promozione.sh:169-176; qui ripetuto finché passa)
cpu_tot(){ ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}'; }
t=0
while :; do
  t=$((t+1)); ok=1; smp=""
  for k in 1 2 3 4; do c=$(cpu_tot); smp="$smp $c"; [ "$c" -lt 150 ] || ok=0; sleep 30; done
  if [ "$ok" = 1 ]; then echo "$(date '+%F %T') E2 calma CPU: PASS al tentativo $t (campioni:$smp %)" >> "$LOG"; break; fi
  echo "$(date '+%F %T') E2 calma CPU: tentativo $t FALLITO (campioni:$smp %)" >> "$LOG"
done
# GATE sentinella (az.rev. S-183 #3): nessun processo singolo > 50 % CPU al SECONDO campione di `top -l 2`, ×2 consecutivi
hog(){ /usr/bin/top -l 2 -n 1 -o cpu -stats cpu,command 2>/dev/null | tail -1 | awk '$1+0 > 50 {print}'; }
hc=0
while [ "$hc" -lt 2 ]; do hg=$(hog); if [ -z "$hg" ]; then hc=$((hc+1)); else hc=0; echo "$(date '+%F %T') processo singolo >50 % CPU: $hg — attesa" >> "$LOG"; fi; [ "$hc" -lt 2 ] && sleep 30; done
# EMENDA t24 (iv): watchdog disco Data (campione a 60 s in pair-out/watchdog-t24.txt; allarme <8G a verbale, NON abortisce: le misure sono a user-CPU);
# vive finché pair182-t24.done esiste da 4 h (copre l'ORM) o 14 h totali
( WD="$H/pair-out/watchdog-fetta1.txt"; PD="$H/pair-out/catena-fetta1.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 50400 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/pair-out/quiet-decl-fetta1.txt"
{ echo "== dichiarazione finestra quieta fetta1 $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-fetta1.txt — lancio FETTA 1 S-189" >> "$LOG"
exec /bin/bash "$H/s189-catena-fetta1.sh"
