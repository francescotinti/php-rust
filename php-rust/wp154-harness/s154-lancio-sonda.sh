#!/bin/bash
# s154-lancio-sonda.sh — catena p.2: attende orm-out/rimisura.done; SOLO se
# rc=0 lancia s154-sonda.sh (build fuori finestra di misura: le finestre
# tempo di p.1 sono chiuse quando l'ORM ha finito).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp154-harness"
LOG="$H/sonda-out/lancio-sonda.log"; mkdir -p "$H/sonda-out"
OD="$H/orm-out/rimisura.done"
echo "$(date '+%F %T') attesa $OD" >> "$LOG"
while [ ! -e "$OD" ]; do sleep 180; done
if ! grep -q '^rc=0' "$OD"; then
  echo "$(date '+%F %T') ORM NON rc=0 ($(cat "$OD")) — sonda NON lanciata" >> "$LOG"
  exit 5
fi
sleep 30
echo "$(date '+%F %T') ORM rc=0 — lancio sonda" >> "$LOG"
exec /bin/bash "$H/s154-sonda.sh"
