#!/bin/bash
# s166-riaggancio-orm.sh — plumbing di RIAGGANCIO dopo cifra NON GIUDICANTE (sentinella E2: oracle net 4,98/5,20 fuori [4,83;4,94], ictx storm bilaterale
# orm-leg1 ×3; flare verosimilmente INNESCATO dalla bonifica census S-164 in
# poi cifra a S-167 se sporca di nuovo). Adattamento
# DICHIARATO di s164-riaggancio-orm.sh: predicato RAFFORZATO alla quiete
# CONTINUA 6 campioni ×30s <5% (stesso predicato del lancio pair t15,
# az.rev. S-163 #5) e sorveglia ANCHE mdworker; poi exec s166-lancio-orm.sh
# (riverifica pair166-t15.done rc=0; i gate di quiescenza PER GAMBA restano
# gli arbitri). Nessun giudizio qui: solo attesa e exec.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp166-harness"
LOG="$H/orm-out/riaggancio.log"
echo "$(date '+%F %T') attesa quiete continua (6x30s <5% mediaanalysisd+mdworker)" >> "$LOG"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd|mdworker/ {s+=$2} END {print (s==""?0:s)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') indexer_cpu=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
echo "$(date '+%F %T') quiete continua — rilancio s166-lancio-orm.sh" >> "$LOG"
exec /bin/bash "$H/s166-lancio-orm.sh"
