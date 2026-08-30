#!/bin/bash
# s164-riaggancio-orm.sh — plumbing di RIAGGANCIO dopo rc=8 (quiescenza-fail
# orm-leg1 ×3; flare verosimilmente INNESCATO dalla bonifica census S-164 in
# finestra: 32 file mossi + commit ⇒ Spotlight — a verbale). Adattamento
# DICHIARATO di s163-riaggancio-orm.sh: predicato RAFFORZATO alla quiete
# CONTINUA 6 campioni ×30s <5% (stesso predicato del lancio pair t14,
# az.rev. S-163 #5) e sorveglia ANCHE mdworker; poi exec s164-lancio-orm.sh
# (riverifica pair164-t14.done rc=0; i gate di quiescenza PER GAMBA restano
# gli arbitri). Nessun giudizio qui: solo attesa e exec.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp164-harness"
LOG="$H/orm-out/riaggancio.log"
echo "$(date '+%F %T') attesa quiete continua (6x30s <5% mediaanalysisd+mdworker)" >> "$LOG"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd|mdworker/ {s+=$2} END {print (s==""?0:s)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') indexer_cpu=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
echo "$(date '+%F %T') quiete continua — rilancio s164-lancio-orm.sh" >> "$LOG"
exec /bin/bash "$H/s164-lancio-orm.sh"
