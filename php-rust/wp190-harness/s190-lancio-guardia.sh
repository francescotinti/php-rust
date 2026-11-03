#!/bin/bash
# s190-lancio-guardia.sh — S-190 cura O(1) v2: FINESTRA NOTTURNA della sola GUARDIA a buffer pieno (criterio
# s190-criterio-o1v2.md p.7): gate di finestra EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4,
# processo singolo >50 % x2, dichiarazione uptime+top) -> lock TOKEN s190 -> pin s183 == stash per hash E bracci Z (riuso
# wp186) + B3 (wp190) presenti -> s186-guardia.sh TAG=b3 (B = phpr-B3) -> lock rimosso; esito in catena-out/catena-s190.done.
# COPIA DICHIARATA di ../wp186-harness/s186-lancio-catena.sh (manifest s190-lancio-guardia-copia.diff, generatore
# gen-lancio-guardia-s190.py) coi SOLI adattamenti elencati nel generatore. Testo S-186 conservato sotto:
# s186-lancio-catena.sh — S-186: CATENA NOTTURNA UNICA (delibera concilio S-186 firmata 2026-10-23; criterio
# s186-criterio-fetta0.md): gate di finestra EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4,
# processo singolo >50 % x2, dichiarazione uptime+top) -> lock TOKEN s186 -> pin s183 == stash per hash E bracci
# Z/B/B2/M/C presenti -> (1) GUARDIA a buffer pieno DUE volte (s186-guardia.sh: ret-buf-dq K0/4/16, Hoare E3, mcalls/calls
# ABAB tree vs s183, micro R=5, obj-churn; TAG=b con phpr-B = cura 22/10 com'è, TAG=b2 con phpr-B2 = + cura O(1)) -> (2) FETTA 0 (s186-fetta0.sh: xctrace O/P/M + C census) -> lock rimosso;
# esiti SOLO in catena-out/catena-s186.done. COPIA DICHIARATA di ../wp185-harness/s185-lancio-catena.sh (manifest
# s186-lancio-catena-copia.diff, generatore gen-lancio-catena-s186.py) coi SOLI adattamenti: harness wp186/catena-out,
# TOKEN s186, pin atteso = stash phpr-s183 (9a6ae11bf7e900ef) al posto di s182+s181, controllo bracci PRIMA del lock,
# SPD phpr-s186-orm, blocco finale (guardia + fetta 0 al posto di istruttoria + t27; niente E5), watchdog disco su
# catena-s186.done; fine() a 3 argomenti INVARIATA. NESSUNA igiene FS qui. Testo S-185 conservato sotto:
# s185-lancio-catena.sh — S-185: CATENA NOTTURNA UNICA (criterio s185-criterio-istruttoria-orm.md p.6): gate di finestra
# EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4, processo singolo >50 % x2, dichiarazione uptime+top)
# -> lock TOKEN s185 -> pin s182 E stash s181 per hash -> ISTRUTTORIA ORM 3 bracci (s185-orm-istruttoria.sh) -> se istr-out/e5.flag
# costanti E5 in env -> t27 dbal+ORM (s185-orm-coppia-t27.sh) -> lock rimosso; esiti SOLO in catena-out/catena-s185.done.
# COPIA DICHIARATA di wp184-harness/s184-lancio-orm-t26.sh (manifest s185-lancio-catena-copia.diff, generatore
# gen-lancio-catena.py) coi SOLI adattamenti: harness wp185/catena-out, TOKEN s185, watchdog disco su catena-s185.done, blocco
# finale (istruttoria + t27 al posto di archivio t25-raw + ORM t26; fine() a 3 argomenti). NESSUNA igiene FS qui.
# s184-lancio-orm-t26.sh — S-184 bis: t26 ORM-ONLY (dbal+ORM, s176-orm-coppia.sh emendato) @ pin s182, finestra PROPRIA dopo
# l'ORM t25 rc=8 NON GIUDICABILE (sentinella oracle fuori banda + ictx oracle1). COPIA DICHIARATA di s184-lancio-pair-t25.sh
# (manifest s184-lancio-orm-t26-copia.diff) coi SOLI adattamenti: log/esiti in orm-out (*-t26.*), lock TOKEN s184bis RIMOSSO
# dal lanciatore a ORM finito (finestra = il solo ORM), watchdog disco riferito a rimisura-t26.done (10 min dopo, max 3 h),
# e al posto del pair: archivio dei raw t25 di wp176-harness/orm-out in orm-out/t25-raw (verifica ls), pin == stash s182,
# s176-orm-coppia.sh con PIN_ATTESO/MAPPA_SP, copia verdetto+done in orm-out. Gate di finestra INVARIATI (quiete CI,
# anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4, processo singolo >50 % x2, dichiarazione uptime+top). NESSUNA igiene FS qui.
# s184-lancio-pair-t25.sh — COPPIA t25 @ pin s182 (criterio s184-criterio-coppia-t25.md p.3): COPIA DICHIARATA di
# ../wp182-harness/s182-lancio-pair-t24.sh (manifest s184-lancio-pair-copia.diff) coi SOLI adattamenti: nomi s184/t25/pair184,
# lock TOKEN s184, s184-pair.sh t25, PIÙ il gate sentinella «processo singolo >50 % CPU» ×2 (az.rev. S-183 #3, come
# s184-lancio-rt2.sh) dopo E2. Il resto INVARIATO (testo S-182 conservato sotto).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp190-harness"
H186="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"
LOG="$H/catena-out/lancio-catena.log"; mkdir -p "$H/catena-out"
DONE="$H/catena-out/catena-s190.done"; rm -f "$DONE"
# ADATTAMENTO S-186: i bracci della build serale (s186-leva-build.sh) devono esistere PRIMA di aprire la finestra
ZB="$H186/ab-out/s186-leva/phpr-Z"; B3B="$H/ab-out/s190-leva/phpr-B3"
[ -s "$ZB" ] || { echo "$(date '+%F %T') braccio Z ASSENTE ($ZB): niente finestra" >> "$LOG"; echo "rc=7 braccio Z assente" > "$DONE"; exit 7; }
[ -s "$B3B" ] || { echo "$(date '+%F %T') braccio B3 ASSENTE ($B3B): niente finestra" >> "$LOG"; echo "rc=7 braccio B3 assente" > "$DONE"; exit 7; }
echo "$(date '+%F %T') bracci Z/B3 presenti: Z $(shasum -a 256 "$ZB" | cut -c1-16) B3 $(shasum -a 256 "$B3B" | cut -c1-16)" >> "$LOG"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# EMENDA t24 (i): LOCK col TOKEN s183 — apre la finestra multi-gamba (la CI si mette in mutex sul lock); rimosso a chiusura di sessione
MLOCK=/private/tmp/phpr-measure.lock
echo "s190 guardia b3 (cura O(1) v2) @ pin s183 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s190)" >> "$LOG"
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
( WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s190.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 10800 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/catena-out/quiet-decl-catena.txt"
{ echo "== dichiarazione finestra quieta S-190 (guardia b3) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — guardia b3" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
[ "$PC" = "$PIN_ATTESO" ] || fine "pin canonico $PC != stash s183 $PIN_ATTESO" "rc=9 pin" 9
[ "$PIN_ATTESO" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN_ATTESO != 9a6ae11bf7e900ef" "rc=9 stash183" 9
echo "$(date '+%F %T') pin $PC == stash s183 — GUARDIA a buffer pieno TAG=b3 (PIN=pin, Z=$ZB, B=$B3B)" >> "$LOG"
# guardia della cura O(1) v2 (criterio s190 p.7): stesso giudice di S-186, braccio B = phpr-B3
PIN="$HOME/Claude/php-rust-output/release/phpr" Z="$ZB" B="$B3B" R=5 RE3=3 TAG=b3 /bin/bash "$H186/s186-guardia.sh"
rc1=$?
GR="$(cat "$H186/guardia-out-b3/guardia.done" 2>/dev/null)"
fine "guardia b3 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b3.out; lock rimosso" "rc=$rc1 guardia_b3=[$GR]" "$rc1"
