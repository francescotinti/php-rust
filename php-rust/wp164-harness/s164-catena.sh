#!/bin/bash
# s164-catena.sh — orchestratore S-164 p.1+p.3.0: (1) attende il drain della
# run CI 4754d04ab2cf in volo all'apertura (il lock S-164 tiene ferma la
# coda); (2) guardia disco Data >=8G poi RIESECUZIONE census AU1 (cura
# incidente S-163; rc SOLO da census-out/census.done, scritto DALLO script);
# un census fallito NON blocca la coppia DOVUTA: rc a verbale, istruttoria
# in sessione; (3) exec s164-lancio-pair.sh (anti-flare + quiet-decl + pair).
# L'ORM parte dalla catena parallela s164-lancio-orm.sh su pair164-t14.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp164-harness"
LOG="$H/catena.log"; mkdir -p "$H/pair-out" "$H/census-out"
echo "$(date '+%F %T') catena S-164: attesa drain CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
FREE=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
if [ "$FREE" -ge 8 ]; then
  echo "$(date '+%F %T') CI drenata (Data ${FREE}G) — census rerun" >> "$LOG"
  /bin/bash "$H/s164-census-au1-rerun.sh" >> "$LOG" 2>&1
  echo "$(date '+%F %T') census rerun terminato: $(cat "$H/census-out/census.done" 2>/dev/null || echo done-ASSENTE)" >> "$LOG"
else
  echo "$(date '+%F %T') Data ${FREE}G <8G — census NON parte (dichiarato), istruttoria in sessione" >> "$LOG"
fi
echo "$(date '+%F %T') passo alla catena pair t14" >> "$LOG"
exec /bin/bash "$H/s164-lancio-pair.sh"
