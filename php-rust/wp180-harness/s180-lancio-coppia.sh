#!/bin/bash
# s180-lancio-coppia.sh — catena post-ri-pin: attende promo-out/lancio-promo.done con rc=0 E rcb=0; legge gli hash
# degli stash phpr-s180 / php-server-s180 (nati SOLO da pin-phpr.sh/pin-server.sh dentro la catena) e li esporta come
# PIN_ATTESO/SRV_ATTESO; poi lancia (daemonizzati) s180-lancio-pair.sh (→ s180-pair.sh t21) e s180-lancio-orm.sh
# (→ s176-orm-coppia.sh dopo pair180-t21.done rc=0). Una catena non a rc=0 NON fa partire la coppia.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp180-harness"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DZ="/Volumes/Extreme Pro/Claude/wp52-harness/daemonize.pl"
LOG="$H/pair-out/lancio-coppia.log"; mkdir -p "$H/pair-out" "$H/orm-out"
PD="$H/promo-out/lancio-promo.done"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
l "attesa $PD"
while [ ! -e "$PD" ]; do sleep 120; done
if ! grep -q '^rc=0 rcb=0' "$PD"; then l "catena NON rc=0/rcb=0 ($(cat "$PD")) — coppia NON lanciata"; exit 5; fi
[ -s "$STASH/phpr-s180" ] && [ -s "$STASH/php-server-s180" ] || { l "stash s180 assenti — coppia NON lanciata"; exit 5; }
export PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s180" | cut -c1-16)
export SRV_ATTESO=$(shasum -a 256 "$STASH/php-server-s180" | cut -c1-16)
l "catena rc=0 — pin s180 phpr=$PIN_ATTESO server=$SRV_ATTESO — lancio pair t21 + attesa ORM"
sleep 60
perl "$DZ" "$H/pair-out/lancio-t21.log" /bin/bash "$H/s180-lancio-pair.sh"
perl "$DZ" "$H/orm-out/lancio-orm.log" /bin/bash "$H/s180-lancio-orm.sh"
l "lanciati (daemonize): s180-lancio-pair.sh, s180-lancio-orm.sh"
exit 0
