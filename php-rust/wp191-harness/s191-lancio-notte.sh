#!/bin/bash
# s191-lancio-notte.sh — NOTTE 04→05/11 (criterio wp191-harness/s191-criterio-notte.md): CATENA UNICA, un solo lock TOKEN s191,
# gate di finestra EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4, processo singolo >50 % x2, dichiarazione
# uptime+top), poi in ORDINE: (1) guardia b4 = s186-guardia.sh con A = stash phpr-s183, Z gemello S-186, B = stash phpr-s184
# (scioglie la riserva K0 della conferma diurna) · (2) coppia dbal+ORM t28 @ pin s184 (s191-orm-coppia-t28.sh, costanti E5 fisse:
# ORA_REF2/SENT = oracle invariato, REF2 = riferimento phpr s182 così Delta_norm legge s184 vs s182) · (3) coppia server t3:
# s190 canonica vs s189 stash (s191-catena-fetta1.sh: guardia AX + tripwire W=1) · (4) coppia WP full+media t26 @ pin s184
# (s191-pair.sh t26) per ULTIMA: le gambe media risvegliano mediaanalysisd e stallerebbero i gate dei passi successivi.
# COPIA DICHIARATA di ../wp185-harness/s185-lancio-catena.sh (manifest s191-lancio-notte-copia.diff, generatore gen-notte-s191.py)
# coi SOLI adattamenti: harness wp191, TOKEN s191, watchdog disco 7 h (era 3 h: quattro passi), controlli di hash su stash
# s183/s184/s189 + canoniche phpr s184 e server s190 + gemello Z, il blocco finale a 4 passi (al posto di istruttoria + t27),
# fine() con i quattro rc (rc complessivo = massimo). Nessuna igiene FS qui. Testo S-185/S-184 conservato sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp191-harness"
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
echo "s191 notte b4+t28+server-t3+t26 @ pin s184 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s191)" >> "$LOG"
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
( WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s191.done"; t0=$(date +%s); tdone=0
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
{ echo "== dichiarazione finestra quieta NOTTE S-191 (b4, t28, server t3, t26) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — (1) guardia b4, (2) t28, (3) server t3, (4) t26" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DONE="$H/catena-out/catena-s191.done"; rm -f "$DONE"
fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN184=$(shasum -a 256 "$STASH/phpr-s184" | cut -c1-16)
PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16); SRV189=$(shasum -a 256 "$STASH/php-server-s189" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN184" = "60b10d857eb0343a" ] || fine "stash s184 $PIN184 != 60b10d857eb0343a" "rc=9 stash184" 9
[ "$PC" = "$PIN184" ] || fine "pin canonico $PC != stash s184 $PIN184" "rc=9 pin" 9
[ "$SRVC" = "721b7802e2d12058" ] || fine "server canonico $SRVC != s190 721b7802e2d12058" "rc=9 server" 9
[ "$SRV189" = "80eea401b71c2be7" ] || fine "stash server s189 $SRV189 != 80eea401b71c2be7" "rc=9 stash189" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b4.out" ] && fine "verdetto b4 ESISTE già: spostarlo prima" "rc=7 verdetto-b4" 7
echo "$(date '+%F %T') pin $PC == stash s184 · stash s183 $PIN183 · server $SRVC / stash s189 $SRV189 · Z ok — (1) GUARDIA b4 (A=stash s183, Z, B=stash s184, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$STASH/phpr-s184" R=5 RE3=3 TAG=b4 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b4/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b4 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b4.out — (2) ORM t28 (E5: ORA_REF2 4,870 SENT [4,81;4,93] = oracle invariato; REF2 [34,23;34,31] = riferimento phpr s182: Delta_norm legge s184 vs s182)" >> "$LOG"
SPD=/private/tmp/phpr-s191-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN184" MAPPA_SP="$SPD" /bin/bash "$H/s191-orm-coppia-t28.sh"
rc2=$?; TR="$(cat "$H/orm-out/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t28 fine rc=$rc2 ($TR) — verdetto s191-orm-coppia-verdetto-t28.out — (3) coppia server t3: s190 canonica vs s189 stash" >> "$LOG"
T=t3 PIN_ATTESO="$SRVC" PIN_OLD="$SRV189" SRVOLD="$STASH/php-server-s189" /bin/bash "$H/s191-catena-fetta1.sh"
rc3=$?; SR="$(cat "$H/pair-out/catena-fetta1.done" 2>/dev/null)"
echo "$(date '+%F %T') server t3 fine rc=$rc3 ($SR) — (4) WP t26 full+media @ pin s184 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN184" SRV_ATTESO="$SRVC" /bin/bash "$H/s191-pair.sh" t26
rc4=$?; PR="$(cat "$H/pair-out/pair191-t26.done" 2>/dev/null)"
RC=0; for r in "$rc1" "$rc2" "$rc3" "$rc4"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte fine: b4 rc=$rc1 · t28 rc=$rc2 · server t3 rc=$rc3 · t26 rc=$rc4; lock rimosso" "rc=$RC guardia_b4=[$GR] t28=[$TR] server_t3=[$SR] t26=[$PR]" "$RC"
