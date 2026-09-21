#!/bin/bash
# s180-lancio-orm.sh — catena p.1: attende pair180-t21.done; SOLO se rc=0
# lancia s176-orm-coppia.sh (MAPPA_SP dedicato APFS). Un pair fallito NON
# fa partire l'ORM: la sessione istruisce prima.
# COPIA DICHIARATA di s172-lancio-orm.sh (manifest s180-lancio-orm-copia.diff): soli nomi s180/t21/pair180.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$REPO/wp180-harness"
LOG="$H/orm-out/lancio-orm.log"; mkdir -p "$H/orm-out"
PD="$H/pair-out/pair180-t21.done"
echo "$(date '+%F %T') attesa $PD" >> "$LOG"
while [ ! -e "$PD" ]; do sleep 120; done
if ! grep -q '^rc=0' "$PD"; then
  echo "$(date '+%F %T') pair t21 NON rc=0 ($(cat "$PD")) — ORM NON lanciato" >> "$LOG"
  exit 5
fi
sleep 60
SPD=/private/tmp/phpr-s180-orm; mkdir -p "$SPD"
echo "$(date '+%F %T') pair rc=0 — lancio ORM (MAPPA_SP=$SPD)" >> "$LOG"
PIN_ATTESO="${PIN_ATTESO:?pin s180 atteso}" MAPPA_SP="$SPD" exec /bin/bash "$REPO/wp176-harness/s176-orm-coppia.sh"
