#!/bin/bash
# s163-riaggancio-r5.sh — riaggancio del R=5 L-AU1 dopo rc=1 (quiescenza
# rifiutata: flare mediaanalysisd 34/37%). Attesa rientro col predicato del
# gate (due campioni <5%), poi exec dell'A/B con TAG NUOVO au1r5b
# (az.rev. S-131 #5: tentativo nuovo = TAG nuovo). I gate del run restano
# gli arbitri; qui solo attesa e exec.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp163-harness"
LOG="$H/ab-out/riaggancio-r5.log"
echo "$(date '+%F %T') attesa rientro flare mediaanalysisd" >> "$LOG"
calm=0
while [ "$calm" -lt 2 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') campione mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 2 ] && sleep 120
done
echo "$(date '+%F %T') flare rientrato — exec s163-ab-au1.sh au1r5b R=5" >> "$LOG"
exec /bin/bash "$H/s163-ab-au1.sh" "/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s163-au1-B" fea4a2d0 au1r5b 5 20c63af4 45.5
