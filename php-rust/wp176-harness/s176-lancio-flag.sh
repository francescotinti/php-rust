#!/bin/bash
# s176-lancio-flag.sh — lanciatore dell'A/B della leva «flag gc-idle» (criterio s176-criterio-flag.md p.6): COPIA
# DICHIARATA di s176-micro-record.sh p.2-4 (pre-condizioni, quiescenza s129 a gate separato, watchdog disco, sentinelle
# nel .out) + ../wp174-harness/s175-lancio-mock.sh (attesa dei bracci, dichiarazione della finestra, exec del copione).
# Adattamenti: attende ab-out/s176-flag.done con `rc=0` (gata sul CONTENUTO, rilievo 2 S-175), bracci A=pin s175
# Z=ab-out/phpr-C B=ab-out/s176-flag/phpr-B C=ab-out/s175-mutgc/phpr-MS (hash attesi dichiarati qui sotto e
# ri-verificati dal copione), esce con rc=6 se il watchdog ha suonato (finestra SPORCA: verdetto a GUARDIA).
# rc (ab-out/lancio-flag.rc): 0 = A/B eseguito in finestra pulita (rc del criterio in ab-out/flag.rc) · 6 = finestra
# sporca · 7 = bracci/build assenti o build rc≠0 · 8 = quiescenza mai PASS · 9 = pre-condizioni.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp176-harness"; O="$H/ab-out"; mkdir -p "$O"
LOG="$O/lancio-flag.log"; LRC="$O/lancio-flag.rc"; LDONE="$O/lancio-flag.done"; WDL="$O/watchdog-flag.log"; ALARM="$O/watchdog-flag.alarm"
DECL="$O/finestra-flag.txt"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN8=5de14d68
Z="$SRC/wp174-harness/ab-out/phpr-C"; Z8=b6c4b587
B="$O/s176-flag/phpr-B"
C="$SRC/wp174-harness/ab-out/s175-mutgc/phpr-MS"; C8=dcd73bf4
rm -f "$LRC" "$LDONE" "$ALARM"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
fine(){ l "fine rc=$1 ($2)"; echo "$1" > "$LRC"; echo "rc=$1" > "$LDONE"; exit "$1"; }
data_g(){ df -g /System/Volumes/Data | awk 'NR==2{print $4}'; }
updater(){ pgrep -fl -i "ShipIt|littlebird|keystone|GoogleUpdater" | head -3; }
sentinelle(){
  echo "-- sentinelle $1 $(date '+%F %T'): Data=$(data_g)G · $(sysctl -n vm.swapusage) · $(uptime | sed 's/.*load/load/')"
  echo "   updater: $(updater | tr '\n' ' ')"
  echo "   top-8 CPU:"; ps -Ao %cpu=,comm= -r | head -8 | awk '{printf "     %5.1f %s\n",$1,$2}'
}
# bracci: attesa della build con rc=0 nel .done (contenuto, non esistenza)
BD="$O/s176-flag.done"
l "attesa $BD con rc=0"
while [ ! -e "$BD" ]; do sleep 60; done
grep -q '^rc=0' "$BD" || fine 7 "build/parità dei bracci rc≠0: $(cat "$BD")"
[ -s "$B" ] || fine 7 "braccio B assente"
for f in "$PIN" "$Z" "$C"; do [ -s "$f" ] || fine 7 "braccio assente: $f"; done
B8=$(shasum -a 256 "$B" | cut -c1-8)
[ "$(shasum -a 256 "$PIN" | cut -c1-8)" = "$PIN8" ] || fine 9 "pin ≠ $PIN8"
grep -qw s176 /private/tmp/phpr-measure.lock 2>/dev/null || fine 9 "lock senza token s176"
# quiete build/CI + pre-condizioni
sleep 30
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
D0=$(data_g); [ "$D0" -ge 10 ] || fine 9 "Data ${D0}G <10G"
U0=$(updater); [ -z "$U0" ] || fine 9 "updater vivo: $U0"
Q=0
for i in $(seq 1 30); do
  /bin/bash "$SRC/wp129-harness/s129-quiescenza.sh" "$O/quiesce-flag.rc" > "$O/quiesce-flag-$i.log" 2>&1
  if [ "$(cat "$O/quiesce-flag.rc" 2>/dev/null)" = 0 ]; then Q=$i; break; fi
  l "quiescenza tentativo $i FAIL: $(tail -1 "$O/quiesce-flag-$i.log")"; sleep 30
done
[ "$Q" -gt 0 ] || fine 8 "quiescenza mai PASS in 30 tentativi"
{ echo "== finestra A/B flag gc-idle: quiescenza PASS al tentativo $Q, Data ${D0}G, bracci A=$PIN8 Z=$Z8 B=$B8 C=$C8 =="; sentinelle INIZIO; } > "$DECL"
: > "$WDL"
( while :; do d=$(data_g); s=$(updater | head -1)
    echo "$(date '+%T') Data=${d}G${s:+ UPDATER:$s}" >> "$WDL"
    if [ "$d" -lt 10 ] || [ -n "$s" ]; then echo "ALLARME" >> "$WDL"; touch "$ALARM"; fi
    sleep 30; done ) &
WD=$!
l "finestra aperta (quiescenza t$Q, Data ${D0}G, B=$B8) — s176-ab-flag.sh flag R=5"
/bin/bash "$H/s176-ab-flag.sh" "$PIN" "$PIN8" "$Z" "$Z8" "$B" "$B8" "$C" "$C8" flag 5 20.08 36.13
ARC=$?
kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
{ sentinelle FINE; echo "watchdog: $(wc -l < "$WDL" | tr -d ' ') campioni, Data min=$(awk -F'Data=' '{split($2,a,"G"); print a[1]}' "$WDL" | sort -n | head -1)G, allarmi=$(grep -c ALLARME "$WDL")"; } >> "$DECL"
cat "$DECL" >> "$H/s176-flag-verdetto.out" 2>/dev/null
l "s176-ab-flag.sh rc=$ARC"
[ -e "$ALARM" ] && fine 6 "finestra SPORCA (watchdog): verdetto a GUARDIA"
fine 0 "A/B eseguito in finestra pulita; criterio rc=$ARC"
