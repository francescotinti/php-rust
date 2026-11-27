#!/bin/bash
# s194d-lancio-guardia.sh — CODA d della notte b di S-194 (criterio wp194-harness/s194-criterio-notte-b.md p.15): ripete TUTTI i
# gate di finestra, riprende il lock TOKEN s194 e ripete due volte s186-guardia.sh INVARIATO (A = stash s183, Z gemello S-186):
# b7b con B = CANONICO s190, b7c con B = copia dello stash s189; poi s194d-guardia-confronto.py.
# COPIA DICHIARATA di s194c-lancio-coda.sh (manifest s194d-lancio-guardia-copia.diff, generatore gen-notte-s194d.py) coi SOLI
# adattamenti: cartella catena-out-d e file catena-s194d.done, etichette, blocco finale coi due passi di guardia. Testo sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness"
LOG="$H/catena-out-d/lancio-catena.log"; mkdir -p "$H/catena-out-d"
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
echo "s194 notte-b coda d guardia b7b+b7c $(date '+%F %T') pid=$$" > "$MLOCK"
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
( WD="$H/catena-out-d/watchdog-catena.txt"; PD="$H/catena-out-d/catena-s194d.done"; t0=$(date +%s); tdone=0
  while :; do
    f=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); a=""; [ "${f:-0}" -lt 8 ] && a=" ALLARME<8G"
    echo "$(date '+%F %T') Data=${f}G swap=$(sysctl -n vm.swapusage | awk '{print $6}')$a" >> "$WD"
    now=$(date +%s); [ -e "$PD" ] && [ "$tdone" = 0 ] && tdone=$now
    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break
    [ $((now-t0)) -ge 25200 ] && break
    sleep 60
  done ) > /dev/null 2>&1 &
# dichiarazione finestra QUIETA (criterio p.6c): uptime + top
Q="$H/catena-out-d/quiet-decl-catena.txt"
{ echo "== dichiarazione finestra quieta CODA d S-194 (b7b, b7c) $(date '+%F %T') =="
  uptime
  top -l 1 -n 8 -o cpu -stats pid,cpu,command 2>/dev/null | tail -12
} > "$Q" 2>&1
echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — (1) guardia b7b, (2) guardia b7c" >> "$LOG"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DONE="$H/catena-out-d/catena-s194d.done"; rm -f "$DONE"
fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
BIN=/private/tmp/phpr-s194c-bin
CANON="$HOME/Claude/php-rust-output/release/phpr"
[ "$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16)" = "9a6ae11bf7e900ef" ] || fine "stash s183 diverso" "rc=9 stash183" 9
[ "$(shasum -a 256 "$CANON" | cut -c1-16)" = "97e000aa00fe802e" ] || fine "canonico != s190" "rc=9 pin" 9
[ "$(shasum -a 256 "$BIN/phpr-s189" 2>/dev/null | cut -c1-16)" = "2d42efd6606e8dd9" ] || fine "copia s189 assente o diversa" "rc=9 copia-s189" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
for t in b7b b7c; do [ -e "$REPO/wp186-harness/s186-guardia-verdetto-$t.out" ] && fine "verdetto $t ESISTE già: spostarlo prima" "rc=7 verdetto-$t" 7; done
echo "$(date '+%F %T') hash ok — (1) GUARDIA b7b (A=stash s183, Z, B=CANONICO s190, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b7b /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; G1="$(cat "$REPO/wp186-harness/guardia-out-b7b/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b7b fine rc=$rc1 ($G1) — (2) GUARDIA b7c (A=stash s183, Z, B=copia stash s189, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$BIN/phpr-s189" R=5 RE3=3 TAG=b7c /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc2=$?; G2="$(cat "$REPO/wp186-harness/guardia-out-b7c/guardia.done" 2>/dev/null)"
python3 "$H/s194d-guardia-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "coda d fine: b7b rc=$rc1 · b7c rc=$rc2; confronto in s194d-guardia-confronto.out; lock rimosso" "rc=$RC b7b=[$G1] b7c=[$G2]" "$RC"
