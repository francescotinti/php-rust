#!/bin/bash
# s193-coda-guardia.sh — CODA della notte di S-193 (emenda p.8 di s193-criterio-notte.md): ripete la SOLA guardia di costo,
# tag b5b, DOPO la fine della catena (catena-s193.done presente, lock assente), con lock proprio e gli STESSI gate di finestra.
# COPIA DICHIARATA di s193-lancio-notte.sh (manifest s193-coda-guardia-copia.diff, generatore gen-coda-guardia-s193.py)
# coi SOLI adattamenti: attesa della fine catena, log/done/watchdog/dichiarazione propri, tag b5b, tolti i passi 2-4.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp193-harness"
LOG="$H/catena-out/coda-guardia.log"; mkdir -p "$H/catena-out"
echo "$(date '+%F %T') attesa fine catena (catena-s193.done presente e lock assente)" >> "$LOG"
while [ ! -e "$H/catena-out/catena-s193.done" ] || [ -e /private/tmp/phpr-measure.lock ]; do sleep 60; done
[ -e "$H/../wp186-harness/s186-guardia-verdetto-b5b.out" ] && { echo "$(date '+%F %T') verdetto b5b ESISTE: esco" >> "$LOG"; exit 7; }
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# EMENDA t24 (i): LOCK col TOKEN s183 — apre la finestra multi-gamba (la CI si mette in mutex sul lock); rimosso a chiusura di sessione
MLOCK=/private/tmp/phpr-measure.lock
echo "s193 coda guardia b5b @ pin s185 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s193)" >> "$LOG"
# predicato anti-flare PRE-registrato (criterio p.6b): quiete CONTINUA 6x30s
echo "$(date '+%F %T') CI quieta — attesa quiete continua anti-flare (6x30s <5%)" >> "$LOG"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') mediaanalysisd=$c calm=$calm" >> "$LOG"
  [ "$calm" -lt 6 ] && sleep 30
done
# EMENDA t23 EREDITATA + EMENDA t24 (ii): PRE-gate di carico PRIMA del pair — loadavg1 <3 (era <5) per 6 campioni consecutivi a 30 s
# (finestra NOTTURNA = conseguenza del gate, non un orario; il gate s129 per gamba resta INVARIATO)
lc=0
while [ "$lc" -lt 6 ]; do
  L1=$(sysctl -n vm.loadavg | awk '{print $2}')
  if awk -v l="$L1" 'BEGIN{exit !(l+0 < 3.0)}'; then lc=$((lc+1)); else lc=0; fi
  echo "$(date '+%F %T') loadavg1=$L1 lc=$lc" >> "$LOG"
  [ "$lc" -lt 6 ] && sleep 30
done
# EMENDA t24 (iii): E2 calma CPU TOTALE <150 % per 4 campioni a 30 s (predicato COPIATO da s181-promozione.sh:169-176; qui ripetuto finché passa)
cpu_tot(){ ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}'; }
t=0
while :; do
  t=$((t+1)); ok=1; smp=""
  for k in 1 2 3 4; do c=$(cpu_tot); smp="$smp $c"; [ "$c" -lt 150 ] || ok=0; sleep 30; done
  if [ "$ok" = 1 ]; then echo "$(date '+%F %T') E2 calma CPU: PASS al tentativo $t (campioni:$smp %)" >> "$LOG"; break; fi
  echo "$(date '+%F %T') E2 calma CPU: tentativo $t FALLITO (campioni:$smp %)" >> "$LOG"
done
# GATE sentinella (az.rev. S-183 #3): nessun processo singolo > 50 % CPU al SECONDO campione di `top -l 2`, ×2 consecutivi
hog(){ /usr/bin/top -l 2 -n 1 -o cpu -stats cpu,command 2>/dev/null | tail -1 | awk '$1+0 > 50 {print}'; }
hc=0
while [ "$hc" -lt 2 ]; do hg=$(hog); if [ -z "$hg" ]; then hc=$((hc+1)); else hc=0; echo "$(date '+%F %T') processo singolo >50 % CPU: $hg — attesa" >> "$LOG"; fi; [ "$hc" -lt 2 ] && sleep 30; done
# EMENDA t24 (iv): watchdog disco Data (campione a 60 s in pair-out/watchdog-t24.txt; allarme <8G a verbale, NON abortisce: le misure sono a user-CPU);
# vive finché pair182-t24.done esiste da 4 h (copre l'ORM) o 14 h totali
( WD="$H/catena-out/watchdog-coda.txt"; PD="$H/catena-out/coda-guardia.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 25200 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/catena-out/quiet-decl-coda.txt"
{ echo "== dichiarazione finestra quieta NOTTE S-193 (coda: guardia b5b) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-coda.txt — guardia b5b" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DONE="$H/catena-out/coda-guardia.done"; rm -f "$DONE"
fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN185=$(shasum -a 256 "$STASH/phpr-s185" | cut -c1-16)
CANON="$HOME/Claude/php-rust-output/release/phpr"
PC=$(shasum -a 256 "$CANON" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16); SRV190=$(shasum -a 256 "$STASH/php-server-s190" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN185" = "d9d6429301f60f4e" ] || fine "stash s185 $PIN185 != d9d6429301f60f4e" "rc=9 stash185" 9
[ "$PC" = "$PIN185" ] || fine "pin canonico $PC != stash s185 $PIN185" "rc=9 pin" 9
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
[ "$SRV190" = "721b7802e2d12058" ] || fine "stash server s190 $SRV190 != 721b7802e2d12058" "rc=9 stash190" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b5b.out" ] && fine "verdetto b5b ESISTE già: spostarlo prima" "rc=7 verdetto-b5b" 7
echo "$(date '+%F %T') pin canonico $PC == stash s185 · stash s183 $PIN183 · server $SRVC / stash s190 $SRV190 · Z ok — GUARDIA b5b (A=stash s183, Z, B=CANONICO s185, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b5b /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b5b/guardia.done" 2>/dev/null)"
fine "coda fine: guardia b5b rc=$rc1 ($GR); lock rimosso" "rc=$rc1 guardia_b5b=[$GR]" "$rc1"
