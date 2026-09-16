#!/bin/bash
# s176-micro-record.sh — micro R=5 di RECORD al pin s175 in finestra PULITA (NEXT §S-176 p.1; sana l'incidente #1
# S-175: micro/conferma misurate con updater ShipIt attivo e Data 0,47G, igiene assente dal .out).
# Contratto PRE-registrato (REGOLE §3: misura di record senza prova d'igiene nel .out = incidente):
#  (1) pin per hash == 5de14d6856d760a8 (symlink neutro /private/tmp/s176-bins/A: la quiescenza s129 morde gli argv);
#  (2) Data ≥10G e nessun ShipIt/updater vivo PRIMA; quiescenza s129 (gate separato, rc da file) fino a PASS, max 30;
#  (3) watchdog disco per TUTTA la finestra (df ogni 30 s; allarme se Data <10G o ShipIt vivo ⇒ esito 6 = finestra
#      SPORCA, cifre a GUARDIA, non di record);
#  (4) sentinelle nel .out: Data, vm.swapusage, top-8 CPU, language-server, updater — a INIZIO e FINE finestra;
#  (5) run-micro.sh R=5 INVARIATO (copia d'uso, nessuna emenda); rapporti pubblicati SOLO se rc=0.
# rc (micro-out/s176-micro.rc): 0=di record · 6=finestra sporca (watchdog) · 7=quiescenza mai PASS · 8=pin diverso ·
# 9=pre-condizioni (Data/updater) · 10=run-micro fallito.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp176-harness"; O="$H/micro-out"; mkdir -p "$O"
OUT="$H/s176-micro-verdetto.out"; RC="$O/s176-micro.rc"; DONE="$O/s176-micro.done"; LOG="$O/lancio-micro.log"
WDL="$O/watchdog.log"; ALARM="$O/watchdog.alarm"
PIN_ATTESO=5de14d6856d760a8
BIN="/private/tmp/s176-bins/A"
rm -f "$RC" "$DONE" "$ALARM"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
fine(){ echo "ESITO rc=$1 ($2) fine $(date '+%F %T')" >> "$OUT"; echo "$1" > "$RC"; echo "rc=$1" > "$DONE"; exit "$1"; }
data_g(){ df -g /System/Volumes/Data | awk 'NR==2{print $4}'; }
updater(){ pgrep -fl -i "ShipIt|littlebird|keystone|GoogleUpdater" | head -3; }
sentinelle(){
  echo "-- sentinelle $1 $(date '+%F %T'): Data=$(data_g)G · $(sysctl -n vm.swapusage) · $(uptime | sed 's/.*load/load/')"
  echo "   updater: $(updater | tr '\n' ' ')"
  echo "   language-server: $(pgrep -fl -i 'rust-analyzer|serena|antigravity' | awk '{print $2}' | sort -u | tr '\n' ' ')"
  echo "   top-8 CPU:"; ps -Ao %cpu=,comm= -r | head -8 | awk '{printf "     %5.1f %s\n",$1,$2}'
}

: > "$OUT"
echo "== s176 MICRO di RECORD R=5 al pin s175 (contratto in testa a s176-micro-record.sh; run-micro.sh INVARIATO) $(date '+%F %T') ==" >> "$OUT"
# (1) pin
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
echo "pin: $BIN -> $(readlink "$BIN") hash=$HB atteso=$PIN_ATTESO" >> "$OUT"
[ "$HB" = "$PIN_ATTESO" ] || fine 8 "pin diverso dall'atteso"
grep -qw s176 /private/tmp/phpr-measure.lock 2>/dev/null || fine 9 "lock senza token s176"
# (2) pre-condizioni
D0=$(data_g); echo "pre: Data=${D0}G" >> "$OUT"
[ "$D0" -ge 10 ] || fine 9 "Data ${D0}G <10G"
U0=$(updater); [ -z "$U0" ] || fine 9 "updater vivo: $U0"
sentinelle INIZIO >> "$OUT"
Q=0
for i in $(seq 1 30); do
  /bin/bash "$SRC/wp129-harness/s129-quiescenza.sh" "$O/quiesce.rc" > "$O/quiesce-$i.log" 2>&1
  if [ "$(cat "$O/quiesce.rc" 2>/dev/null)" = 0 ]; then Q=$i; break; fi
  l "quiescenza tentativo $i FAIL: $(tail -1 "$O/quiesce-$i.log")"; sleep 30
done
[ "$Q" -gt 0 ] || fine 7 "quiescenza mai PASS in 30 tentativi"
echo "quiescenza s129: PASS al tentativo $Q ($(tail -1 "$O/quiesce-$Q.log"))" >> "$OUT"
# (3) watchdog disco per la finestra
: > "$WDL"
( while :; do d=$(data_g); s=$(updater | head -1)
    echo "$(date '+%T') Data=${d}G${s:+ UPDATER:$s}" >> "$WDL"
    if [ "$d" -lt 10 ] || [ -n "$s" ]; then echo "ALLARME" >> "$WDL"; touch "$ALARM"; fi
    sleep 30; done ) &
WD=$!
l "finestra aperta (quiescenza al tentativo $Q, Data ${D0}G) — run-micro R=5"
# (5) misura
PHPR="$BIN" R=5 /bin/bash "$SRC/wp97-harness/micro/run-micro.sh" > "$O/micro-pin-s175-R5.out" 2>&1
MRC=$?
kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
sentinelle FINE >> "$OUT"
echo "watchdog: $(wc -l < "$WDL" | tr -d ' ') campioni, Data min=$(awk -F'Data=' '{split($2,a,"G"); print a[1]}' "$WDL" | sort -n | head -1)G, allarmi=$(grep -c ALLARME "$WDL")" >> "$OUT"
[ "$MRC" = 0 ] || fine 10 "run-micro rc=$MRC"
echo "-- run-micro (micro-out/micro-pin-s175-R5.out):" >> "$OUT"
grep -vE '^( |$)' "$O/micro-pin-s175-R5.out" >> "$OUT"
[ -e "$ALARM" ] && fine 6 "finestra SPORCA (watchdog): cifre a GUARDIA, non di record"
fine 0 "micro DI RECORD: igiene provata nel .out"
