#!/bin/bash
# avvio-s184.sh — S-184: derivato da avvio-scomp-s183.sh. Attende il DRENAGGIO della CI (runner morto ×2 a 60 s,
# niente cargo/rustc), poi SCRIVE il lock col TOKEN s184 (finestra A/B della leva composta), daemonizza s184-leva-build.sh (COMMIT/ZCOMMIT da env) e s184-lancio-rt2.sh
# (che aspetta da sé updater/app utente, calma CPU, quiescenza). Nome senza «harness/s1NN-» né «phpr» di proposito.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp184-harness"
COMMIT="${COMMIT:?COMMIT della leva}"; ZCOMMIT="${ZCOMMIT:-f5f746cf}"
DZ="/Volumes/Extreme Pro/Claude/wp52-harness/daemonize.pl"
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
LOG="$H/ab-out/avvio-s184.log"; mkdir -p "$H/ab-out"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
ci_viva(){ pgrep -f 'ci/ci-runner.sh' > /dev/null 2>&1 || pgrep -qx cargo || pgrep -qx rustc; }
l "attesa drenaggio CI (runner morto ×2 a 60 s, niente cargo/rustc); coda ora: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._')"
n=0
while [ "$n" -lt 2 ]; do if ci_viva; then n=0; else n=$((n+1)); fi; sleep 60; done
l "CI ferma; coda residua: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._')"
pgrep -f "s184-leva-build|s184-lancio-rt2" > /dev/null && { l "build/lanciatore già vivi — non rilancio"; exit 7; }
echo "s184 leva composta L-RT1+L-RT2 $(date '+%F %T') pid=$$ commit=$COMMIT" > /private/tmp/phpr-measure.lock
l "lock scritto (TOKEN s184) — lancio s184-leva-build.sh (COMMIT=$COMMIT ZCOMMIT=$ZCOMMIT) e s184-lancio-rt2.sh"
COMMIT="$COMMIT" ZCOMMIT="$ZCOMMIT" perl "$DZ" "$H/ab-out/leva-build-dz.log" /bin/bash "$H/s184-leva-build.sh"
perl "$DZ" "$H/ab-out/lancio-rt2-dz.log" /bin/bash "$H/s184-lancio-rt2.sh"
sleep 3; l "lanciati: $(ps -Ao pid,command | grep -E '[s]184-(leva-build|lancio-rt2)' | awk '{print $1}' | tr '\n' ' ')"
exit 0
