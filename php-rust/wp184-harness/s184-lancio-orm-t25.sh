#!/bin/bash
# s184-lancio-orm-t25.sh — COPPIA S-184 @ pin s182: attende pair184-t25.done; SOLO se rc=0
# lancia s176-orm-coppia.sh (MAPPA_SP dedicato APFS). Un pair fallito NON
# fa partire l'ORM: la sessione istruisce prima.
# COPIA DICHIARATA di ../wp182-harness/s182-lancio-orm-t24.sh (manifest s184-lancio-orm-copia.diff): soli nomi s184/t25/pair184 + archivio del verdetto ORM in wp184-harness/orm-out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$REPO/wp184-harness"
LOG="$H/orm-out/lancio-orm-t25.log"; mkdir -p "$H/orm-out"
PD="$H/pair-out/pair184-t25.done"
echo "$(date '+%F %T') attesa $PD" >> "$LOG"
while [ ! -e "$PD" ]; do sleep 120; done
if ! grep -q '^rc=0' "$PD"; then
  echo "$(date '+%F %T') pair t25 NON rc=0 ($(cat "$PD")) — ORM NON lanciato" >> "$LOG"
  exit 5
fi
sleep 60
SPD=/private/tmp/phpr-s184-orm; mkdir -p "$SPD"
echo "$(date '+%F %T') pair rc=0 — lancio ORM (MAPPA_SP=$SPD)" >> "$LOG"
PIN_ATTESO="${PIN_ATTESO:?pin s182 atteso (coppia S-184)}" MAPPA_SP="$SPD" /bin/bash "$REPO/wp176-harness/s176-orm-coppia.sh"
rc=$?
cp "$REPO/wp176-harness/s176-orm-coppia-verdetto.out" "$H/orm-out/s184-orm-coppia-verdetto-t25.out" 2>/dev/null
cp "$REPO/wp176-harness/orm-out/rimisura.done" "$H/orm-out/rimisura-t25.done" 2>/dev/null
echo "$(date '+%F %T') ORM fine rc=$rc ($(cat "$REPO/wp176-harness/orm-out/rimisura.done" 2>/dev/null)) — verdetto archiviato in wp184-harness/orm-out" >> "$LOG"
exit "$rc"
