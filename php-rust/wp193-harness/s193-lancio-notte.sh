#!/bin/bash
# s193-lancio-notte.sh — NOTTE di S-193 (criterio wp193-harness/s193-criterio-notte.md): CATENA UNICA, un solo lock TOKEN s193,
# gate di finestra EREDITATI, poi in ORDINE: (1) guardia b5 = s186-guardia.sh INVARIATO con A = stash phpr-s183, Z gemello S-186,
# B = binario CANONICO (pin s185): costo K0 TOTALE di s185 rispetto a s183 (b4 su s184 mordeva +4,62 ns/Ret) · (2) coppia
# dbal+ORM t29 @ pin s185 (costanti E5 INVARIATE: Delta_norm legge s185 vs riferimento s182) · (3) coppia server t4: s191
# canonica vs s190 stash (guardia AX + tripwire W=1) · (4) coppia WP full+media t27 @ pin s185 per ULTIMA.
# COPIA DICHIARATA di ../wp191-harness/s191-lancio-notte.sh (manifest s193-lancio-notte-copia.diff, generatore gen-notte-s193.py)
# coi SOLI adattamenti: harness wp193, TOKEN s193, controlli di hash (stash s183/s185, canoniche phpr s185 e server s191, stash
# server s190, gemello Z), blocco finale coi quattro passi di S-193 (tag b5/t29/t4/t27). Testo S-191/S-185/S-184 sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp193-harness"
LOG="$H/catena-out/lancio-catena.log"; mkdir -p "$H/catena-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# EMENDA t24 (i): LOCK col TOKEN s183 — apre la finestra multi-gamba (la CI si mette in mutex sul lock); rimosso a chiusura di sessione
MLOCK=/private/tmp/phpr-measure.lock
echo "s193 notte b5+t29+server-t4+t27 @ pin s185 $(date '+%F %T') pid=$$" > "$MLOCK"
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
( WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s193.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 25200 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/catena-out/quiet-decl-catena.txt"
{ echo "== dichiarazione finestra quieta NOTTE S-193 (b5, t29, server t4, t27) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — (1) guardia b5, (2) t29, (3) server t4, (4) t27" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DONE="$H/catena-out/catena-s193.done"; rm -f "$DONE"
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
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b5.out" ] && fine "verdetto b5 ESISTE già: spostarlo prima" "rc=7 verdetto-b5" 7
echo "$(date '+%F %T') pin canonico $PC == stash s185 · stash s183 $PIN183 · server $SRVC / stash s190 $SRV190 · Z ok — (1) GUARDIA b5 (A=stash s183, Z, B=CANONICO s185, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b5 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b5/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b5 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b5.out — (2) ORM t29 (E5 INVARIATE: ORA_REF2 4,870 SENT [4,81;4,93]; REF2 [34,23;34,31] = riferimento phpr s182: Delta_norm legge s185 vs s182)" >> "$LOG"
SPD=/private/tmp/phpr-s193-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN185" MAPPA_SP="$SPD" /bin/bash "$H/s193-orm-coppia-t29.sh"
rc2=$?; TR="$(cat "$H/orm-out/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t29 fine rc=$rc2 ($TR) — verdetto s193-orm-coppia-verdetto-t29.out — (3) coppia server t4: s191 canonica vs s190 stash" >> "$LOG"
T=t4 PIN_ATTESO="$SRVC" PIN_OLD="$SRV190" SRVOLD="$STASH/php-server-s190" /bin/bash "$H/s193-catena-fetta1.sh"
rc3=$?; SR="$(cat "$H/pair-out/catena-fetta1.done" 2>/dev/null)"
echo "$(date '+%F %T') server t4 fine rc=$rc3 ($SR) — (4) WP t27 full+media @ pin s185 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN185" SRV_ATTESO="$SRVC" /bin/bash "$H/s193-pair.sh" t27
rc4=$?; PR="$(cat "$H/pair-out/pair193-t27.done" 2>/dev/null)"
RC=0; for r in "$rc1" "$rc2" "$rc3" "$rc4"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte fine: b5 rc=$rc1 · t29 rc=$rc2 · server t4 rc=$rc3 · t27 rc=$rc4; lock rimosso" "rc=$RC guardia_b5=[$GR] t29=[$TR] server_t4=[$SR] t27=[$PR]" "$RC"
