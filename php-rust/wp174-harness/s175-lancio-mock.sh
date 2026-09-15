#!/bin/bash
# s175-lancio-mock.sh — lanciatore del mock «flag gc-idle» (criterio s175-criterio-mock.md p.5): attende la fine
# della coppia ORM (orm-out/rimisura.done), poi quiete CI + anti-flare (copia del canone di s175-lancio-pair.sh),
# dichiara la finestra e lancia s175-ab-mock.sh coi 4 bracci (A=braccio C, Z=pin s175, B=MS, C=M3). rc da ab-out/mock.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp174-harness"
LOG="$H/ab-out/lancio-mock.log"; mkdir -p "$H/ab-out"
RD="$H/orm-out/rimisura.done"
echo "$(date '+%F %T') attesa $RD" >> "$LOG"
while [ ! -e "$RD" ]; do sleep 120; done
echo "$(date '+%F %T') coppia ORM finita ($(cat "$RD")) — attesa quiete CI" >> "$LOG"
sleep 60
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
Q="$H/ab-out/quiet-decl-mock.txt"
{ echo "== dichiarazione finestra quieta mock $(date '+%F %T') =="; uptime; top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12; } > "$Q" 2>&1
MS="$H/ab-out/s175-mutgc/phpr-MS"; M3="$H/ab-out/s175-mutgc/phpr-M3"
for f in "$MS" "$M3" "$H/ab-out/phpr-C"; do [ -s "$f" ] || { echo "$(date '+%F %T') braccio ASSENTE: $f — mock NON lanciato" >> "$LOG"; exit 7; }; done
MSH=$(shasum -a 256 "$MS" | cut -c1-8); M3H=$(shasum -a 256 "$M3" | cut -c1-8)
PIN="$HOME/Claude/php-rust-output/release/phpr"
echo "$(date '+%F %T') quiete raggiunta — lancio mock (B=MS $MSH, C=M3 $M3H)" >> "$LOG"
exec /bin/bash "$H/s175-ab-mock.sh" "$H/ab-out/phpr-C" b6c4b587 "$PIN" 5de14d68 "$MS" "$MSH" "$M3" "$M3H" mock 5 20.92 36.47
