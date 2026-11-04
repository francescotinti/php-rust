#!/bin/bash
# s190-lancio-promo.sh — lanciatore DETACHED della catena di promozione COMPOSTA s190 → pin s184 (s190-promozione.sh).
# COPIA DICHIARATA di ../wp185-harness/s185-lancio-promo.sh (manifest s190-lancio-promo-copia.diff, generatore
# gen-promozione-s190.py): SOLI path/tag (wp190, PROMO_SP promo-s190-sp, lock TOKEN s190-promo). Come in S-185: attesa
# quiete CI (nessun cargo/rustc/phpt-runner, coda vuota; doppia verifica a 60 s), lock scritto dal lanciatore PRIMA della
# catena e rimosso a fine corsa. Via daemonize.pl; nessun token «phpr»/«php-server» nell'argv (falso positivo di
# s129-quiescenza): PROMO_SP è esportata QUI e il suo valore non contiene il token. Sentinelle (Data, swap, CPU totale,
# top-5) a inizio e fine nel log. Esiti: promo-out/lancio-promo.rc (rc della catena, letto da promo-out/rcb) + lancio-promo.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp190-harness"; O="$H/promo-out"; mkdir -p "$O"
LOG="$O/lancio-promo.log"; LRC="$O/lancio-promo.rc"; LDONE="$O/lancio-promo.done"
export PROMO_SP="/private/tmp/promo-s190-sp"
rm -f "$LRC" "$LDONE"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
sentinelle(){
  { echo "-- sentinelle $1 $(date '+%F %T'): Data=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')G · $(sysctl -n vm.swapusage | awk '{print "swap", $6, "/", $3}') · CPU tot=$(ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}')%"
    echo "   top-5 CPU:"; ps -Ao %cpu=,comm= -r | head -5 | awk '{printf "     %5s %s\n", $1, $2}'
  } >> "$LOG"
}
sentinelle inizio
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
ci_busy(){ pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner || [ -n "$(ls "$CIQ" 2>/dev/null)" ]; }
l "attesa quiete CI (cargo/rustc/phpt-runner assenti, coda vuota)"
while ci_busy; do sleep 60; done; sleep 60
while ci_busy; do sleep 60; done
MLOCK=/private/tmp/phpr-measure.lock
echo "s190 promozione pin s184 $(date '+%F %T') pid=$$" > "$MLOCK"
l "CI quieta — lock scritto: $MLOCK (TOKEN s190-promo)"
l "rustc: $(rustc --version) · HEAD $(git -C "$SRC" rev-parse --short HEAD) · lock: $(cat /private/tmp/phpr-measure.lock 2>/dev/null || echo ASSENTE)"
"$H/s190-promozione.sh" >> "$LOG" 2>&1
rc=$?
rm -f "$MLOCK"; l "lock rimosso"
RCB=$(cat "$O/rcb" 2>/dev/null || echo "assente")
sentinelle fine
l "fine: rc processo=$rc · rcb=$RCB"
echo "$rc" > "$LRC"; echo "rc=$rc rcb=$RCB" > "$LDONE"
exit "$rc"
