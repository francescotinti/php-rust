#!/bin/bash
# s195-lancio-promo.sh — lanciatore DETACHED della catena di promozione s195 → pin s191 (s195-promozione.sh).
# COPIA DICHIARATA di wp194-harness/s194e-lancio-promo.sh (manifest s195-lancio-promo-copia.diff, generatore gen-promozione-s195.py): soli path/tag.
# COPIA DICHIARATA di s194d-lancio-promo.sh (manifest s194e-lancio-promo-copia.diff, generatore gen-promozione-s194e.py): soli path/tag.
# COPIA DICHIARATA di s194c-lancio-promo.sh (manifest s194d-lancio-promo-copia.diff, generatore gen-promozione-s194d.py): soli path/tag.
# COPIA DICHIARATA di s194b-lancio-promo.sh (manifest s194c-lancio-promo-copia.diff, generatore gen-promozione-s194c.py): soli path/tag.
# COPIA DICHIARATA di s194-lancio-promo.sh (manifest s194b-lancio-promo-copia.diff, generatore gen-promozione-s194b.py): soli path/tag.
# COPIA DICHIARATA di ../wp193-harness/s193-lancio-promo.sh (manifest s194-lancio-promo-copia.diff, generatore
# gen-promozione-s194.py): soli path/tag (wp194, PROMO_SP promo-s194-sp, lock TOKEN s194-promo). Come in S-193: Data ≥ 10G
# PRIMA del lock, attesa quiete CI, lock scritto dal lanciatore e rimosso a fine corsa, sentinelle a inizio e fine nel log.
# Esiti: promo-out/lancio-promo.rc (rc della catena) + lancio-promo.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp195-harness"; O="$H/promo-out"; mkdir -p "$O"
LOG="$O/lancio-promo.log"; LRC="$O/lancio-promo.rc"; LDONE="$O/lancio-promo.done"
export PROMO_SP="/private/tmp/promo-s195-sp"
rm -f "$LRC" "$LDONE"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
sentinelle(){
  { echo "-- sentinelle $1 $(date '+%F %T'): Data=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')G · $(sysctl -n vm.swapusage | awk '{print "swap", $6, "/", $3}') · CPU tot=$(ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}')%"
    echo "   top-5 CPU:"; ps -Ao %cpu=,comm= -r | head -5 | awk '{printf "     %5s %s\n", $1, $2}'
  } >> "$LOG"
}
sentinelle inizio
DG=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
if [ "${DG:-0}" -lt 7 ]; then l "STOP: Data ${DG}G < 7G (soglia 10G abbassata per delibera 28/11) al lancio (veto S-188) — nessuna catena"; echo 7 > "$LRC"; echo "rc=7 rcb=assente (Data ${DG}G < 7G (soglia 10G abbassata per delibera 28/11))" > "$LDONE"; exit 7; fi
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
ci_busy(){ pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner || [ -n "$(ls "$CIQ" 2>/dev/null)" ]; }
l "attesa quiete CI (cargo/rustc/phpt-runner assenti, coda vuota)"
while ci_busy; do sleep 60; done; sleep 60
while ci_busy; do sleep 60; done
MLOCK=/private/tmp/phpr-measure.lock
echo "s195 promozione pin s191 $(date '+%F %T') pid=$$" > "$MLOCK"
l "CI quieta — lock scritto: $MLOCK (TOKEN s195-promo)"
l "rustc: $(rustc --version) · HEAD $(git -C "$SRC" rev-parse --short HEAD) · lock: $(cat /private/tmp/phpr-measure.lock 2>/dev/null || echo ASSENTE)"
"$H/s195-promozione.sh" >> "$LOG" 2>&1
rc=$?
rm -f "$MLOCK"; l "lock rimosso"
RCB=$(cat "$O/rcb" 2>/dev/null || echo "assente")
sentinelle fine
l "fine: rc processo=$rc · rcb=$RCB"
echo "$rc" > "$LRC"; echo "rc=$rc rcb=$RCB" > "$LDONE"
exit "$rc"
