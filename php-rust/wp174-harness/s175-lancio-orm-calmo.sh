#!/bin/bash
# s175-lancio-orm-calmo.sh — rilancio ORM (tentativi 1-2 rc=8: quiescenza per gamba fallita 3/3 su flare di
# mediaanalysisd di 2-3 min > budget 3×30 s del canone). Pre-attesa anti-flare 6×30 s sotto il 5% (COPIA del
# blocco di s175-lancio-pair.sh), poi exec s175-lancio-orm.sh (invariato: pair174-t20.done rc=0 → s175-orm-coppia.sh).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp174-harness"
LOG="$H/orm-out/lancio-orm-calmo.log"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
echo "$(date '+%F %T') calma raggiunta — exec lancio-orm (tentativo 3)" >> "$LOG"
exec /bin/bash "$H/s175-lancio-orm.sh"
