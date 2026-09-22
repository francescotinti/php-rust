#!/bin/bash
# s181-lancio-cr1b.sh — RERUN (finestra cr1 a GUARDIA: un campione con GoogleUpdater --wake-all alle 14:11:48, Data 12G) — lanciatore dell'A/B della leva L-CR1 (criterio s181-criterio-cr1b.md p.3/p.7): COPIA DICHIARATA di
# ../wp177-harness/s177-lancio-cm1b.sh (manifest s181-lancio-cr1b-copia.diff) coi SOLI adattamenti: attende
# ab-out/s181-leva.done con `rc=0` (contenuto: bracci a parità E mutante che morde), bracci A=pin s180 Z=ab-out/s181-leva/
# phpr-Z B=ab-out/s181-leva/phpr-B (hash letti dai binari) passati come SYMLINK NEUTRI arm-A/arm-Z/arm-B (il gate s129 cerca
# «phpr» negli argv), token s181, nomi cr1b, PREV same-binary prop-dq 34.20, tentativi di quiescenza 60, copione
# s181-ab-cr1b.sh a 3 bracci. rc (ab-out/lancio-cr1b.rc): 0 = A/B eseguito in finestra pulita (rc del criterio in
# ab-out/cr1b.rc) · 6 = finestra sporca · 7 = bracci/build assenti o build rc≠0 · 8 = quiescenza mai PASS · 9 = pre-condizioni.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp181-harness"; O="$H/ab-out"; mkdir -p "$O"
LOG="$O/lancio-cr1b.log"; LRC="$O/lancio-cr1b.rc"; LDONE="$O/lancio-cr1b.done"; WDL="$O/watchdog-cr1b.log"; ALARM="$O/watchdog-cr1b.alarm"
DECL="$O/finestra-cr1b.txt"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN8=884399fc
Z="$O/s181-leva/phpr-Z"
B="$O/s181-leva/phpr-B"
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
BD="$O/s181-leva.done"
l "attesa $BD con rc=0"
while [ ! -e "$BD" ]; do sleep 60; done
grep -q '^rc=0' "$BD" || fine 7 "build/parità dei bracci rc≠0: $(cat "$BD")"
for f in "$PIN" "$Z" "$B"; do [ -s "$f" ] || fine 7 "braccio assente: $f"; done
Z8=$(shasum -a 256 "$Z" | cut -c1-8); B8=$(shasum -a 256 "$B" | cut -c1-8)
[ "$(shasum -a 256 "$PIN" | cut -c1-8)" = "$PIN8" ] || fine 9 "pin ≠ $PIN8"
[ "$B8" != "$PIN8" ] || fine 9 "B == pin"
grep -qw s181 /private/tmp/phpr-measure.lock 2>/dev/null || fine 9 "lock senza token s181"
# symlink NEUTRI (nessun token cercato da s129 negli argv del copione A/B)
ln -sfn "$PIN" "$O/arm-A"; ln -sfn "$Z" "$O/arm-Z"; ln -sfn "$B" "$O/arm-B"
# quiete build/CI + pre-condizioni
sleep 30
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
D0=$(data_g); [ "$D0" -ge 10 ] || fine 9 "Data ${D0}G <10G"
U0=$(updater); [ -z "$U0" ] || fine 9 "updater vivo: $U0"
# EMENDA E2 (S-177): calma CPU TOTALE (somma %cpu di tutti i processi) < 150 % per 4 campioni consecutivi a 30 s
# (tetto 180 campioni = 90 min), PRIMA del gate s129.
calm=0; tries=0
while [ "$calm" -lt 4 ]; do
  c=$(ps -Ao %cpu | awk 'NR>1{s+=$1} END{printf "%d", s}')
  if [ "$c" -lt 150 ]; then calm=$((calm+1)); else calm=0; fi
  l "calma CPU tot=${c}% calm=$calm"; tries=$((tries+1))
  [ "$tries" -ge 180 ] && fine 8 "calma CPU mai raggiunta in 90 min"
  [ "$calm" -lt 4 ] && sleep 30
done
Q=0
for i in $(seq 1 60); do
  /bin/bash "$SRC/wp129-harness/s129-quiescenza.sh" "$O/quiesce-cr1b.rc" > "$O/quiesce-cr1b-$i.log" 2>&1
  if [ "$(cat "$O/quiesce-cr1b.rc" 2>/dev/null)" = 0 ]; then Q=$i; break; fi
  l "quiescenza tentativo $i FAIL: $(tail -1 "$O/quiesce-cr1b-$i.log")"; sleep 30
done
[ "$Q" -gt 0 ] || fine 8 "quiescenza mai PASS in 60 tentativi"
{ echo "== finestra A/B L-CR1: quiescenza PASS al tentativo $Q, Data ${D0}G, bracci A=$PIN8 Z=$Z8 B=$B8 =="; sentinelle INIZIO; } > "$DECL"
: > "$WDL"
( while :; do d=$(data_g); s=$(updater | head -1)
    echo "$(date '+%T') Data=${d}G${s:+ UPDATER:$s}" >> "$WDL"
    if [ "$d" -lt 10 ] || [ -n "$s" ]; then echo "ALLARME" >> "$WDL"; touch "$ALARM"; fi
    sleep 30; done ) &
WD=$!
l "finestra aperta (quiescenza t$Q, Data ${D0}G, B=$B8) — s181-ab-cr1b.sh cr1b R=5"
/bin/bash "$H/s181-ab-cr1.sh" "$O/arm-A" "$PIN8" "$O/arm-Z" "$Z8" "$O/arm-B" "$B8" cr1b 5 34.20
ARC=$?
kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
{ sentinelle FINE; echo "watchdog: $(wc -l < "$WDL" | tr -d ' ') campioni, Data min=$(awk -F'Data=' '{split($2,a,"G"); print a[1]}' "$WDL" | sort -n | head -1)G, allarmi=$(grep -c ALLARME "$WDL")"; } >> "$DECL"
cat "$DECL" >> "$H/s181-cr1b-verdetto.out" 2>/dev/null
l "s181-ab-cr1b.sh rc=$ARC"
[ -e "$ALARM" ] && fine 6 "finestra SPORCA (watchdog): verdetto a GUARDIA"
fine 0 "A/B eseguito in finestra pulita; criterio rc=$ARC"
