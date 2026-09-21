#!/bin/bash
# s180-lancio-promo.sh — lanciatore DETACHED della catena di ri-pin s180 (s180-promozione.sh).
# Via daemonize.pl; nessun token «phpr»/«php-server» nell'argv (falso positivo di s129-quiescenza,
# memoria quiescenza-pgrep-argv-false-positive): PROMO_SP è esportata QUI, non passata sulla riga di comando,
# e il suo valore non contiene il token. Sentinelle (Data, swap, CPU totale, top-5) a inizio e fine nel log.
# Esiti: promo-out/lancio-promo.rc (rc della catena, letto da promo-out/rcb) + lancio-promo.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp180-harness"; O="$H/promo-out"; mkdir -p "$O"
LOG="$O/lancio-promo.log"; LRC="$O/lancio-promo.rc"; LDONE="$O/lancio-promo.done"
export PROMO_SP="/private/tmp/promo-s180-sp"
rm -f "$LRC" "$LDONE"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
sentinelle(){
  { echo "-- sentinelle $1 $(date '+%F %T'): Data=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')G · $(sysctl -n vm.swapusage | awk '{print "swap", $6, "/", $3}') · CPU tot=$(ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}')%"
    echo "   top-5 CPU:"; ps -Ao %cpu=,comm= -r | head -5 | awk '{printf "     %5s %s\n", $1, $2}'
  } >> "$LOG"
}
sentinelle inizio
l "rustc: $(rustc --version) · HEAD $(git -C "$SRC" rev-parse --short HEAD) · lock: $(cat /private/tmp/phpr-measure.lock 2>/dev/null || echo ASSENTE)"
"$H/s180-promozione.sh" >> "$LOG" 2>&1
rc=$?
RCB=$(cat "$O/rcb" 2>/dev/null || echo "assente")
sentinelle fine
l "fine: rc processo=$rc · rcb=$RCB"
echo "$rc" > "$LRC"; echo "rc=$rc rcb=$RCB" > "$LDONE"
exit "$rc"
