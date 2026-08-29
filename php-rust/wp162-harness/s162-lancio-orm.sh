#!/bin/bash
# s162-lancio-orm.sh — catena p.1: attende pair162-t12.done; SOLO se rc=0
# lancia s162-orm-coppia.sh (MAPPA_SP dedicato APFS). Un pair fallito NON
# fa partire l'ORM: la sessione istruisce prima.
# Adattamento dichiarato di s161-lancio-orm.sh (plumbing, esiti da .done).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp162-harness"
LOG="$H/orm-out/lancio-orm.log"; mkdir -p "$H/orm-out"
PD="$H/pair-out/pair162-t12.done"
echo "$(date '+%F %T') attesa $PD" >> "$LOG"
while [ ! -e "$PD" ]; do sleep 120; done
if ! grep -q '^rc=0' "$PD"; then
  echo "$(date '+%F %T') pair t12 NON rc=0 ($(cat "$PD")) — ORM NON lanciato" >> "$LOG"
  exit 5
fi
sleep 60
SPD=/private/tmp/phpr-s162-orm; mkdir -p "$SPD"
echo "$(date '+%F %T') pair rc=0 — lancio ORM (MAPPA_SP=$SPD)" >> "$LOG"
MAPPA_SP="$SPD" exec /bin/bash "$H/s162-orm-coppia.sh"
