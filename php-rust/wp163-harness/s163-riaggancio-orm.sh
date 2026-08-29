#!/bin/bash
# s163-riaggancio-orm.sh — plumbing di RIAGGANCIO dopo rc=8 (quiescenza-fail
# dbal-leg1: flare mediaanalysisd oltre i 3 retry, rischio pre-dichiarato nel
# criterio p.1). Attende che mediaanalysisd stia sotto il 5% CPU su DUE
# campioni consecutivi distanziati (stesso predicato del gate), poi rilancia
# s163-lancio-orm.sh (che riverifica pair163-t13.done rc=0; i gate di
# quiescenza PER GAMBA restano gli arbitri — questo wrapper evita solo un
# lancio a vuoto). Nessun giudizio qui: solo attesa e exec.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp163-harness"
LOG="$H/orm-out/riaggancio.log"
echo "$(date '+%F %T') attesa rientro flare mediaanalysisd" >> "$LOG"
calm=0
while [ "$calm" -lt 2 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') campione mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 2 ] && sleep 120
done
echo "$(date '+%F %T') flare rientrato — rilancio s163-lancio-orm.sh" >> "$LOG"
exec /bin/bash "$H/s163-lancio-orm.sh"
