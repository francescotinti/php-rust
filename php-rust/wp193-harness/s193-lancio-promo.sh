#!/bin/bash
# s193-lancio-promo.sh — lanciatore DETACHED della catena di promozione s193 → pin s185 (s193-promozione.sh).
# COPIA DICHIARATA di ../wp190-harness/s190-lancio-promo.sh (manifest s193-lancio-promo-copia.diff, generatore
# gen-promozione-s193.py): path/tag (wp193, PROMO_SP promo-s193-sp, lock TOKEN s193-promo) e UN gate nuovo: Data ≥ 10G
# PRIMA del lock (veto S-188 «Data <10G al lancio»; la build a freddo sulla canonica ne ricrea ~5). Come in S-190: attesa
# quiete CI, lock scritto dal lanciatore PRIMA della catena e rimosso a fine corsa, sentinelle a inizio e fine nel log.
# Esiti: promo-out/lancio-promo.rc (rc della catena) + lancio-promo.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp193-harness"; O="$H/promo-out"; mkdir -p "$O"
LOG="$O/lancio-promo.log"; LRC="$O/lancio-promo.rc"; LDONE="$O/lancio-promo.done"
export PROMO_SP="/private/tmp/promo-s193-sp"
rm -f "$LRC" "$LDONE"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
sentinelle(){
  { echo "-- sentinelle $1 $(date '+%F %T'): Data=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')G · $(sysctl -n vm.swapusage | awk '{print "swap", $6, "/", $3}') · CPU tot=$(ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}')%"
    echo "   top-5 CPU:"; ps -Ao %cpu=,comm= -r | head -5 | awk '{printf "     %5s %s\n", $1, $2}'
  } >> "$LOG"
}
sentinelle inizio
DG=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
if [ "${DG:-0}" -lt 10 ]; then l "STOP: Data ${DG}G < 10G al lancio (veto S-188) — nessuna catena"; echo 7 > "$LRC"; echo "rc=7 rcb=assente (Data ${DG}G < 10G)" > "$LDONE"; exit 7; fi
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
ci_busy(){ pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner || [ -n "$(ls "$CIQ" 2>/dev/null)" ]; }
l "attesa quiete CI (cargo/rustc/phpt-runner assenti, coda vuota)"
while ci_busy; do sleep 60; done; sleep 60
while ci_busy; do sleep 60; done
MLOCK=/private/tmp/phpr-measure.lock
echo "s193 promozione pin s185 $(date '+%F %T') pid=$$" > "$MLOCK"
l "CI quieta — lock scritto: $MLOCK (TOKEN s193-promo)"
l "rustc: $(rustc --version) · HEAD $(git -C "$SRC" rev-parse --short HEAD) · lock: $(cat /private/tmp/phpr-measure.lock 2>/dev/null || echo ASSENTE)"
"$H/s193-promozione.sh" >> "$LOG" 2>&1
rc=$?
rm -f "$MLOCK"; l "lock rimosso"
RCB=$(cat "$O/rcb" 2>/dev/null || echo "assente")
sentinelle fine
l "fine: rc processo=$rc · rcb=$RCB"
echo "$rc" > "$LRC"; echo "rc=$rc rcb=$RCB" > "$LDONE"
exit "$rc"
