#!/bin/bash
# s177-lancio-cm1b.sh — RERUN (cm1 contaminata) con EMENDA E2 (calma CPU totale) — lanciatore dell'A/B della leva COMPOSTA «flag gc-idle + L-CM1» (criterio s177-criterio-cm1.md
# p.7): COPIA DICHIARATA di ../wp176-harness/s176-lancio-flag.sh (manifest s177-lancio-cm1b-copia.diff) coi SOLI
# adattamenti: attende ab-out/s177-leva.done con `rc=0` (contenuto), bracci A=pin s175 Z=wp174 ab-out/phpr-C
# B=wp176 ab-out/s176-flag/phpr-B (6c7bbb55) C=ab-out/s177-leva/phpr-C (hash letto dal binario), token s177, nomi
# cm1b, PREV same-binary 20.28/36.20, tentativi di quiescenza 60 (la CI locale può ancora girare: gate s129 a ogni 30 s),
# copione s177-ab-cm1b.sh. rc (ab-out/lancio-cm1b.rc): 0 = A/B eseguito in finestra pulita (rc del criterio in
# ab-out/cm1b.rc) · 6 = finestra sporca · 7 = bracci/build assenti o build rc≠0 · 8 = quiescenza mai PASS · 9 = pre-condizioni.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp177-harness"; O="$H/ab-out"; mkdir -p "$O"
LOG="$O/lancio-cm1b.log"; LRC="$O/lancio-cm1b.rc"; LDONE="$O/lancio-cm1b.done"; WDL="$O/watchdog-cm1b.log"; ALARM="$O/watchdog-cm1b.alarm"
DECL="$O/finestra-cm1b.txt"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN8=5de14d68
Z="$SRC/wp174-harness/ab-out/phpr-C"; Z8=b6c4b587
B="$SRC/wp176-harness/ab-out/s176-flag/phpr-B"; B8=6c7bbb55
C="$O/s177-leva/phpr-C"
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
BD="$O/s177-leva.done"
l "attesa $BD con rc=0"
while [ ! -e "$BD" ]; do sleep 60; done
grep -q '^rc=0' "$BD" || fine 7 "build/parità del braccio C rc≠0: $(cat "$BD")"
[ -s "$C" ] || fine 7 "braccio C assente"
for f in "$PIN" "$Z" "$B"; do [ -s "$f" ] || fine 7 "braccio assente: $f"; done
C8=$(shasum -a 256 "$C" | cut -c1-8)
[ "$(shasum -a 256 "$PIN" | cut -c1-8)" = "$PIN8" ] || fine 9 "pin ≠ $PIN8"
[ "$(shasum -a 256 "$B" | cut -c1-8)" = "$B8" ] || fine 9 "B ≠ $B8"
grep -qw s177 /private/tmp/phpr-measure.lock 2>/dev/null || fine 9 "lock senza token s177"
# quiete build/CI + pre-condizioni
sleep 30
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
D0=$(data_g); [ "$D0" -ge 10 ] || fine 9 "Data ${D0}G <10G"
U0=$(updater); [ -z "$U0" ] || fine 9 "updater vivo: $U0"
# EMENDA E2 (S-177, finestra cm1 contaminata da app utente a 4×100 %): calma CPU TOTALE (somma %cpu di tutti i processi)
# < 150 % per 4 campioni consecutivi a 30 s (tetto 180 campioni = 90 min), PRIMA del gate s129.
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
  /bin/bash "$SRC/wp129-harness/s129-quiescenza.sh" "$O/quiesce-cm1b.rc" > "$O/quiesce-cm1b-$i.log" 2>&1
  if [ "$(cat "$O/quiesce-cm1b.rc" 2>/dev/null)" = 0 ]; then Q=$i; break; fi
  l "quiescenza tentativo $i FAIL: $(tail -1 "$O/quiesce-cm1b-$i.log")"; sleep 30
done
[ "$Q" -gt 0 ] || fine 8 "quiescenza mai PASS in 60 tentativi"
{ echo "== finestra A/B composta flag+L-CM1: quiescenza PASS al tentativo $Q, Data ${D0}G, bracci A=$PIN8 Z=$Z8 B=$B8 C=$C8 =="; sentinelle INIZIO; } > "$DECL"
: > "$WDL"
( while :; do d=$(data_g); s=$(updater | head -1)
    echo "$(date '+%T') Data=${d}G${s:+ UPDATER:$s}" >> "$WDL"
    if [ "$d" -lt 10 ] || [ -n "$s" ]; then echo "ALLARME" >> "$WDL"; touch "$ALARM"; fi
    sleep 30; done ) &
WD=$!
l "finestra aperta (quiescenza t$Q, Data ${D0}G, C=$C8) — s177-ab-cm1b.sh cm1b R=5"
/bin/bash "$H/s177-ab-cm1b.sh" "$PIN" "$PIN8" "$Z" "$Z8" "$B" "$B8" "$C" "$C8" cm1b 5 20.28 36.20
ARC=$?
kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
{ sentinelle FINE; echo "watchdog: $(wc -l < "$WDL" | tr -d ' ') campioni, Data min=$(awk -F'Data=' '{split($2,a,"G"); print a[1]}' "$WDL" | sort -n | head -1)G, allarmi=$(grep -c ALLARME "$WDL")"; } >> "$DECL"
cat "$DECL" >> "$H/s177-cm1b-verdetto.out" 2>/dev/null
l "s177-ab-cm1b.sh rc=$ARC"
[ -e "$ALARM" ] && fine 6 "finestra SPORCA (watchdog): verdetto a GUARDIA"
fine 0 "A/B eseguito in finestra pulita; criterio rc=$ARC"
