#!/bin/bash
# attesa quiete continua 6x30s (predicato riaggancio S-164/166) poi R=5 mckr5
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd|mdworker/ {s+=$2} END {print (s==""?0:s)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  [ "$calm" -lt 6 ] && sleep 30
done
cd "/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp166-harness"
./s166-ab-mck.sh "/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s166-mck-B" 092dcff4 mckr5 5 1fd8757d +20.0
echo $? > ab-out/mckr5.done
