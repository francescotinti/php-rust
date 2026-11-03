#!/bin/bash
# avvio-s188.sh — S-188 COPPIA SERVER WP (criterio s188-criterio-server-pair.md p.6): derivato dal pattern di
# ../wp184-harness/avvio-coppia-s184.sh (nuovo script, non copia: niente stash né ORM). Attende il DRENAGGIO della CI
# (runner morto ×2 a 60 s, niente cargo/rustc), verifica che php-server canonico == pin s183 674f6b9730da41ff e
# daemonizza s188-lancio.sh (→ s188-wp-server-pair.sh, che eredita il lock TOKEN s188 scritto dal lanciatore).
# Il nome NON contiene «harness/s1NN-» né «phpr» di proposito. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp188-harness"
DZ="/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl"
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
SRV_ATTESO="${SRV_ATTESO:-97b58fd225ca7289}"  # pin server s188
LOG="$H/pair-out/avvio.log"; mkdir -p "$H/pair-out"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
ci_viva(){ pgrep -f 'ci/ci-runner.sh' > /dev/null 2>&1 || pgrep -qx cargo || pgrep -qx rustc; }
l "attesa drenaggio CI (runner morto ×2 a 60 s, niente cargo/rustc); coda ora: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._')"
n=0
while [ "$n" -lt 2 ]; do
  if ci_viva; then n=0; else n=$((n+1)); fi
  sleep 60
done
l "CI ferma; coda residua: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._') (se >0: CI stallata, dichiarare nel verbale)"
SC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
[ "$SC" = "$SRV_ATTESO" ] || { l "php-server canonico $SC ≠ pin s183 $SRV_ATTESO — coppia NON lanciata"; exit 9; }
[ -x "$H/s188-lancio.sh" ] || { l "s188-lancio.sh assente (generarlo con gen-lancio-s188.py) — coppia NON lanciata"; exit 2; }
l "pin server=$SC == atteso — lancio s188-lancio.sh (PIN_ATTESO esportato per la catena)"
export PIN_ATTESO="$SRV_ATTESO"
perl "$DZ" "$H/pair-out/lancio-dz.log" /bin/bash "$H/s188-lancio.sh"
sleep 5
l "lanciato (daemonize): $(pgrep -fl 's188-lancio' | grep -v pgrep | awk '{print $1}' | tr '\n' ' ')"
exit 0
