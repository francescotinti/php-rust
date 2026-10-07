#!/bin/bash
# s184-lancio-rt2.sh — LEVA COMPOSTA L-RT1+L-RT2 (criterio s184-criterio-rt2.md p.6): COPIA DICHIARATA di ../wp182-harness/s183-lancio-rt1.sh
# (manifest s184-lancio-rt2-copia.diff) coi SOLI adattamenti: tag rt2, token s184, bracci ab-out/s184-leva, PREV prop-dq 31,27,
# s184-ab-rt2.sh; SENTINELLA come GATE (az.rev. S-183 #3): processo singolo > 50 % CPU (secondo campione di `top -l 2`) ⇒ attesa
# senza limite, loadavg(1) < 3 ×6, dichiarazione con i top-5 di `top -l 2`; Data < 10G ⇒ ATTESA (log ogni 60 s) invece di rc=9
# (S-184: Data a 9G in apertura, sotto decisione utente). Testo S-183/S-181 sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp184-harness"; O="$H/ab-out"; mkdir -p "$O"
LOG="$O/lancio-rt2.log"; LRC="$O/lancio-rt2.rc"; LDONE="$O/lancio-rt2.done"; WDL="$O/watchdog-rt2.log"; ALARM="$O/watchdog-rt2.alarm"
DECL="$O/finestra-rt2.txt"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN8=19a2faa8
AB="$O/s184-leva"
rm -f "$LRC" "$LDONE" "$ALARM"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
fine(){ l "fine rc=$1 ($2)"; echo "$1" > "$LRC"; echo "rc=$1" > "$LDONE"; exit "$1"; }
data_g(){ df -g /System/Volumes/Data | awk 'NR==2{print $4}'; }
updater(){ pgrep -fl -i "ShipIt|littlebird|keystone|GoogleUpdater" | head -3; }
sentinelle(){
  echo "-- sentinelle $1 $(date '+%F %T'): Data=$(data_g)G · $(sysctl -n vm.swapusage) · $(uptime | sed 's/.*load/load/')"
  echo "   updater: $(updater | tr '\n' ' ')"
  echo "   top-8 CPU:"; ps -Ao %cpu=,comm= -r | head -8 | awk '{printf "     %5.1f %s\n",$1,$2}'
  echo "   top -l 2 (secondo campione, top-5):"; /usr/bin/top -l 2 -n 5 -o cpu -stats cpu,command 2>/dev/null | tail -5 | awk '{printf "     %s\n",$0}'
}
BD="$O/s184-leva.done"
l "attesa $BD con rc=0"
while [ ! -e "$BD" ]; do sleep 60; done
grep -q '^rc=0' "$BD" || fine 7 "build/parità dei bracci rc≠0: $(cat "$BD")"
for a in Z B; do [ -s "$AB/phpr-$a" ] || fine 7 "braccio assente: $AB/phpr-$a"; done
[ "$(shasum -a 256 "$PIN" | cut -c1-8)" = "$PIN8" ] || fine 9 "pin ≠ $PIN8"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
Z8=$(h8 "$AB/phpr-Z"); B8=$(h8 "$AB/phpr-B"); [ "$B8" != "$PIN8" ] || fine 9 "B == pin"
grep -qw s184 /private/tmp/phpr-measure.lock 2>/dev/null || fine 9 "lock senza token s184"
sleep 30
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
D0=$(data_g); while [ "$D0" -lt 10 ]; do l "Data ${D0}G <10G — attesa (liberare spazio su Data)"; sleep 60; D0=$(data_g); done
uc=0
while [ "$uc" -lt 2 ]; do U0=$(updater | head -1); if [ -z "$U0" ]; then uc=$((uc+1)); else uc=0; l "updater/app utente vivo: $U0 — attesa"; fi; [ "$uc" -lt 2 ] && sleep 60; done
calm=0; tries=0
while [ "$calm" -lt 4 ]; do
  c=$(ps -Ao %cpu | awk 'NR>1{s+=$1} END{printf "%d", s}')
  if [ "$c" -lt 150 ]; then calm=$((calm+1)); else calm=0; fi
  l "calma CPU tot=${c}% calm=$calm"; tries=$((tries+1))
  [ "$calm" -lt 4 ] && sleep 30
done
# GATE sentinella (az.rev. S-183 #3): nessun processo singolo > 50 % CPU al SECONDO campione di `top -l 2`, ×2 consecutivi
hog(){ /usr/bin/top -l 2 -n 1 -o cpu -stats cpu,command 2>/dev/null | tail -1 | awk '$1+0 > 50 {print}'; }
hc=0
while [ "$hc" -lt 2 ]; do hg=$(hog); if [ -z "$hg" ]; then hc=$((hc+1)); else hc=0; l "processo singolo >50 % CPU: $hg — attesa"; fi; [ "$hc" -lt 2 ] && sleep 30; done
# loadavg(1) < 3 ×6 consecutivi
lc=0
while [ "$lc" -lt 6 ]; do la=$(sysctl -n vm.loadavg | awk '{print $2}'); if awk -v x="$la" 'BEGIN{exit !(x+0 < 3)}'; then lc=$((lc+1)); else lc=0; l "loadavg(1) $la ≥ 3 — attesa"; fi; [ "$lc" -lt 6 ] && sleep 30; done
Q=0
i=0
while :; do
  i=$((i+1)); /bin/bash "$SRC/wp129-harness/s129-quiescenza.sh" "$O/quiesce-rt2.rc" > "$O/quiesce-rt2-$i.log" 2>&1
  if [ "$(cat "$O/quiesce-rt2.rc" 2>/dev/null)" = 0 ]; then Q=$i; break; fi
  l "quiescenza tentativo $i FAIL: $(tail -1 "$O/quiesce-rt2-$i.log")"; sleep 60
done
{ echo "== finestra A/B COMPOSTA L-RT1+L-RT2: quiescenza PASS al tentativo $Q, Data ${D0}G, bracci A=$PIN8 Z=$Z8 B=$B8 =="; sentinelle INIZIO; } > "$DECL"
: > "$WDL"
( prev=0; while :; do d=$(data_g); s=$(updater | head -1)
    echo "$(date '+%T') Data=${d}G${s:+ UPDATER:$s}" >> "$WDL"
    if [ -n "$s" ]; then cur=1; else cur=0; fi
    if [ "$d" -lt 10 ] || { [ "$cur" = 1 ] && [ "$prev" = 1 ]; }; then echo "ALLARME" >> "$WDL"; touch "$ALARM"; fi
    prev=$cur; sleep 30; done ) &
WD=$!
ln -sfn "$PIN" "$O/arm-A"; ln -sfn "$AB/phpr-Z" "$O/arm-Z"; ln -sfn "$AB/phpr-B" "$O/arm-B"
l "finestra aperta (quiescenza t$Q, Data ${D0}G, B=$B8) — s184-ab-rt2.sh rt2 R=5"
/bin/bash "$H/s184-ab-rt2.sh" "$O/arm-A" "$PIN8" "$O/arm-Z" "$Z8" "$O/arm-B" "$B8" rt2 5 31.27
R1=$?
kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
{ sentinelle FINE; echo "watchdog: $(wc -l < "$WDL" | tr -d ' ') campioni, Data min=$(sed -n 's/.*Data=\([0-9]*\)G.*/\1/p' "$WDL" | sort -n | head -1)G, campioni con updater=$(grep -c UPDATER "$WDL"), allarmi=$(grep -c ALLARME "$WDL")"; } >> "$DECL"
cat "$DECL" >> "$H/s184-rt2-verdetto.out" 2>/dev/null
l "s184-ab-rt2.sh rc=$R1"
[ -e "$ALARM" ] && fine 6 "finestra SPORCA (watchdog): verdetti a GUARDIA"
fine 0 "A/B eseguito in finestra pulita; criterio rc=$R1"
