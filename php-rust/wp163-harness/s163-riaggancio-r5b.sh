#!/bin/bash
# s163-riaggancio-r5b.sh — riaggancio ROBUSTO del R=5 L-AU1: il flare
# mediaanalysisd e' INTERMITTENTE (0% alle valli, 15-37% ai picchi; il
# wrapper precedente campionava troppo rado). Predicato rafforzato: quiete
# CONTINUA di ~3 min (6 campioni consecutivi <5% ogni 30s), exec IMMEDIATO;
# se il gate del run rifiuta comunque (rc=1), TAG nuovo e nuovo giro
# (az.rev. S-131 #5) fino a 4 tentativi. Qualunque rc!=1 = esito del run,
# si esce con quello. I gate del run restano gli arbitri.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp163-harness"
LOG="$H/ab-out/riaggancio-r5b.log"
for TAG in au1r5c au1r5d au1r5e au1r5f; do
  echo "$(date '+%F %T') [$TAG] attesa quiete continua (6x30s <5%)" >> "$LOG"
  calm=0
  while [ "$calm" -lt 6 ]; do
    c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
    if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
    echo "$(date '+%F %T') [$TAG] mediaanalysisd=$c calm=$calm" >> "$LOG"
    [ "$calm" -lt 6 ] && sleep 30
  done
  echo "$(date '+%F %T') [$TAG] quiete continua — exec A/B R=5" >> "$LOG"
  /bin/bash "$H/s163-ab-au1.sh" "/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s163-au1-B" fea4a2d0 "$TAG" 5 20c63af4 45.5
  rc=$?
  echo "$(date '+%F %T') [$TAG] rc=$rc" >> "$LOG"
  [ "$rc" -ne 1 ] && exit "$rc"
done
echo "$(date '+%F %T') 4 tentativi rifiutati dal gate — istruttoria" >> "$LOG"
exit 1
