#!/bin/bash
# avvio-coppia-s184.sh — S-184 COPPIA t25 @ pin s182 (criterio s184-criterio-coppia-t25.md p.3): derivato da
# ../wp182-harness/avvio-istruttoria-s182.sh (soli nomi: stash s182, harness wp184, lanciatori t25). Attende il DRENAGGIO della CI
# (runner morto ×2 a 60 s, niente cargo/rustc), verifica stash phpr-s182/php-server-s182 == canonica, esporta PIN_ATTESO/SRV_ATTESO e
# daemonizza s184-lancio-pair-t25.sh (→ s184-pair.sh t25, che scrive il lock TOKEN s184) e s184-lancio-orm-t25.sh (ORM dopo pair rc=0).
# Il nome NON contiene «harness/s1NN-» né «phpr» di proposito.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp184-harness"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
DZ="/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl"
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
LOG="$H/pair-out/avvio-coppia.log"; mkdir -p "$H/pair-out" "$H/orm-out"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
ci_viva(){ pgrep -f 'ci/ci-runner.sh' > /dev/null 2>&1 || pgrep -qx cargo || pgrep -qx rustc; }
l "attesa drenaggio CI (runner morto ×2 a 60 s, niente cargo/rustc); coda ora: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._')"
n=0
while [ "$n" -lt 2 ]; do
  if ci_viva; then n=0; else n=$((n+1)); fi
  sleep 60
done
l "CI ferma; coda residua: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._') (se >0: CI stallata, dichiarare nel verbale)"
[ -s "$STASH/phpr-s182" ] && [ -s "$STASH/php-server-s182" ] || { l "stash s182 assenti — coppia NON lanciata"; exit 5; }
export PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s182" | cut -c1-16)
export SRV_ATTESO=$(shasum -a 256 "$STASH/php-server-s182" | cut -c1-16)
PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
SC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
if [ "$PC" != "$PIN_ATTESO" ] || [ "$SC" != "$SRV_ATTESO" ]; then l "binari canonici ≠ stash s182 (phpr $PC vs $PIN_ATTESO · server $SC vs $SRV_ATTESO) — coppia NON lanciata"; exit 9; fi
l "pin s182 phpr=$PIN_ATTESO server=$SRV_ATTESO == canonico — lancio pair t25 + attesa ORM"
perl "$DZ" "$H/pair-out/lancio-t25-dz.log" /bin/bash "$H/s184-lancio-pair-t25.sh"
perl "$DZ" "$H/orm-out/lancio-orm-t25-dz.log" /bin/bash "$H/s184-lancio-orm-t25.sh"
sleep 5
l "lanciati (daemonize): $(pgrep -fl 's184-lancio-(pair|orm)-t25' | grep -v pgrep | awk '{print $1}' | tr '\n' ' ')"
exit 0
