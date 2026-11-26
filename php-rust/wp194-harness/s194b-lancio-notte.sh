#!/bin/bash
# s194b-lancio-notte.sh — NOTTE b di S-194 (criterio wp194-harness/s194-criterio-notte-b.md): CATENA UNICA, un solo lock TOKEN s194,
# gate di finestra INVARIATI, poi in ORDINE: (1) guardia b7 = s186-guardia.sh INVARIATO con A = stash phpr-s183, Z gemello S-186,
# B = binario CANONICO (pin s190) · (2) coppia dbal+ORM t31 @ pin s190 (costanti E5 INVARIATE) · (3) coppia WP full+media t29
# @ pin s190 per ULTIMA · poi s194b-notte-confronto.py (regola esiti -> conferma contro la notte del 22/11).
# COPIA DICHIARATA di s194-lancio-notte.sh (manifest s194b-lancio-notte-copia.diff, generatore gen-notte-s194b.py) coi SOLI
# adattamenti: cartella catena-out-b e file catena-s194b.done, etichette b7/t31/t29 e pin s190, controlli di hash (stash
# s183/s186/s190, canoniche phpr s190 e server s191, gemello Z), blocco finale coi tre passi della notte b. Testo precedente sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness"
LOG="$H/catena-out-b/lancio-catena.log"; mkdir -p "$H/catena-out-b"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# EMENDA t24 (i): LOCK col TOKEN s183 — apre la finestra multi-gamba (la CI si mette in mutex sul lock); rimosso a chiusura di sessione
MLOCK=/private/tmp/phpr-measure.lock
echo "s194 notte-b b7+t31+t29 @ pin s190 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s194)" >> "$LOG"
# predicato anti-flare PRE-registrato (criterio p.6b): quiete CONTINUA 6x30s
echo "$(date '+%F %T') CI quieta — attesa quiete continua anti-flare (6x30s <5%)" >> "$LOG"
calm=0
while [ "$calm" -lt 6 ]; do
  c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {a=$2} /photolibraryd/ {b=$2} END {a+=0; b+=0; print (a>b?a:b)}')
  if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi
  echo "$(date '+%F %T') mediaanalysisd/photolibraryd(max)=$c calm=$calm" >> "$LOG"
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
( WD="$H/catena-out-b/watchdog-catena.txt"; PD="$H/catena-out-b/catena-s194b.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 25200 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/catena-out-b/quiet-decl-catena.txt"
{ echo "== dichiarazione finestra quieta NOTTE b S-194 (b7, t31, t29) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — (1) guardia b7, (2) t31, (3) t29" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DONE="$H/catena-out-b/catena-s194b.done"; rm -f "$DONE"
fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN186=$(shasum -a 256 "$STASH/phpr-s186" | cut -c1-16); PIN190=$(shasum -a 256 "$STASH/phpr-s190" | cut -c1-16)
CANON="$HOME/Claude/php-rust-output/release/phpr"
PC=$(shasum -a 256 "$CANON" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN186" = "9977cc4b1cf19991" ] || fine "stash s186 $PIN186 != 9977cc4b1cf19991" "rc=9 stash186" 9
[ "$PIN190" = "97e000aa00fe802e" ] || fine "stash s190 $PIN190 != 97e000aa00fe802e" "rc=9 stash190" 9
[ "$PC" = "$PIN190" ] || fine "pin canonico $PC != stash s190 $PIN190" "rc=9 pin" 9
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b7.out" ] && fine "verdetto b7 ESISTE già: spostarlo prima" "rc=7 verdetto-b7" 7
echo "$(date '+%F %T') pin canonico $PC == stash s190 · stash s183 $PIN183 · server $SRVC (s191, invariato) · Z ok — (1) GUARDIA b7 (A=stash s183, Z, B=CANONICO s190, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b7 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b7/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b7 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b7.out — (2) ORM t31 (E5 INVARIATE: ORA_REF2 4,870 SENT [4,81;4,93]; REF2 [34,23;34,31])" >> "$LOG"
SPD=/private/tmp/phpr-s194b-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN190" MAPPA_SP="$SPD" /bin/bash "$H/s194b-orm-coppia-t31.sh"
rc2=$?; TR="$(cat "$H/orm-out-b/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t31 fine rc=$rc2 ($TR) — verdetto s194b-orm-coppia-verdetto-t31.out — (3) WP t29 full+media @ pin s190 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN190" SRV_ATTESO="$SRVC" /bin/bash "$H/s194b-pair.sh" t29
rc3=$?; PR="$(cat "$H/pair-out/pair194-t29.done" 2>/dev/null)"
python3 "$H/s194b-notte-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2" "$rc3"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte b fine: b7 rc=$rc1 · t31 rc=$rc2 · t29 rc=$rc3; confronto in s194b-notte-confronto.out; lock rimosso" "rc=$RC guardia_b7=[$GR] t31=[$TR] t29=[$PR]" "$RC"
