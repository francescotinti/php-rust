#!/bin/bash
# s194c-lancio-coda.sh — CODA c della notte b di S-194 (criterio wp194-harness/s194-criterio-notte-b.md p.10-13): aspetta la fine
# della catena b (file catena-s194b.done), poi ripete TUTTI i gate di finestra, riprende il lock TOKEN s194 e misura in ORDINE:
# ORM t32 @ stash s186 · ORM t33 @ stash s189 · WP t30 @ stash s186 · WP t31 @ stash s189 · poi s194c-notte-confronto.py.
# COPIA DICHIARATA di s194b-lancio-notte.sh (manifest s194c-lancio-coda-copia.diff, generatore gen-notte-s194c.py) coi SOLI
# adattamenti: attesa iniziale della catena b, cartella catena-out-c e file catena-s194c.done, etichette, blocco finale coi
# quattro passi sui binari copiati dallo stash in /private/tmp/phpr-s194c-bin (la canonica s190 non si tocca). Testo precedente sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness"
LOG="$H/catena-out-c/lancio-catena.log"; mkdir -p "$H/catena-out-c"
echo "$(date '+%F %T') coda c: attesa fine catena b" >> "$LOG"
while [ ! -e "$H/catena-out-b/catena-s194b.done" ]; do sleep 60; done
echo "$(date '+%F %T') catena b finita ($(cat "$H/catena-out-b/catena-s194b.done")) — attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
# EMENDA t24 (i): LOCK col TOKEN s183 — apre la finestra multi-gamba (la CI si mette in mutex sul lock); rimosso a chiusura di sessione
MLOCK=/private/tmp/phpr-measure.lock
echo "s194 notte-b coda c t32+t33+t30+t31 @ stash s186/s189 $(date '+%F %T') pid=$$" > "$MLOCK"
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
( WD="$H/catena-out-c/watchdog-catena.txt"; PD="$H/catena-out-c/catena-s194c.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 25200 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/catena-out-c/quiet-decl-catena.txt"
{ echo "== dichiarazione finestra quieta CODA c S-194 (t32, t33, t30, t31) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — (1) ORM t32, (2) ORM t33, (3) WP t30, (4) WP t31" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DONE="$H/catena-out-c/catena-s194c.done"; rm -f "$DONE"
fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
BIN=/private/tmp/phpr-s194c-bin; mkdir -p "$BIN"
for pair in s186:9977cc4b1cf19991 s189:2d42efd6606e8dd9; do
  n=${pair%%:*}; h=${pair##*:}
  [ "$(shasum -a 256 "$STASH/phpr-$n" | cut -c1-16)" = "$h" ] || fine "stash $n != $h" "rc=9 stash-$n" 9
  cp "$STASH/phpr-$n" "$BIN/phpr-$n" && chmod 755 "$BIN/phpr-$n"
  [ "$(shasum -a 256 "$BIN/phpr-$n" | cut -c1-16)" = "$h" ] || fine "copia $n != $h" "rc=9 copia-$n" 9
done
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
for v in s194c-orm-coppia-verdetto-t32.out s194c-orm-coppia-verdetto-t33.out s194-pair-verdetto-t30.out s194-pair-verdetto-t31.out; do
  [ -e "$H/$v" ] && fine "verdetto $v ESISTE già: spostarlo prima" "rc=7 $v" 7
done
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
echo "$(date '+%F %T') binari copiati e verificati in $BIN — (1) ORM t32 @ s186" >> "$LOG"
SPD=/private/tmp/phpr-s194c-orm-s186; mkdir -p "$SPD"
PIN_ATTESO=9977cc4b1cf19991 MAPPA_SP="$SPD" /bin/bash "$H/s194c-orm-coppia-t32.sh"
rc1=$?; R1="$(cat "$H/orm-out-c-s186/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t32 fine rc=$rc1 ($R1) — (2) ORM t33 @ s189" >> "$LOG"
SPD=/private/tmp/phpr-s194c-orm-s189; mkdir -p "$SPD"
PIN_ATTESO=2d42efd6606e8dd9 MAPPA_SP="$SPD" /bin/bash "$H/s194c-orm-coppia-t33.sh"
rc2=$?; R2="$(cat "$H/orm-out-c-s189/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t33 fine rc=$rc2 ($R2) — (3) WP t30 @ s186" >> "$LOG"
PIN_ATTESO=9977cc4b1cf19991 SRV_ATTESO="$SRVC" /bin/bash "$H/s194c-pair-s186.sh" t30
rc3=$?; R3="$(cat "$H/pair-out/pair194-t30.done" 2>/dev/null)"
echo "$(date '+%F %T') t30 fine rc=$rc3 ($R3) — (4) WP t31 @ s189" >> "$LOG"
PIN_ATTESO=2d42efd6606e8dd9 SRV_ATTESO="$SRVC" /bin/bash "$H/s194c-pair-s189.sh" t31
rc4=$?; R4="$(cat "$H/pair-out/pair194-t31.done" 2>/dev/null)"
python3 "$H/s194c-notte-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2" "$rc3" "$rc4"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "coda c fine: t32 rc=$rc1 · t33 rc=$rc2 · t30 rc=$rc3 · t31 rc=$rc4; confronto in s194c-notte-confronto.out; lock rimosso" "rc=$RC t32=[$R1] t33=[$R2] t30=[$R3] t31=[$R4]" "$RC"
