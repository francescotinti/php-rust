#!/bin/bash
# avvio-s189.sh — S-189 FETTA 1 (criterio s189-criterio-guardia-ax.md p.7): derivato dal pattern di
# ../wp188-harness/avvio-s188.sh. Attende il DRENAGGIO della CI (runner morto ×2 a 60 s, niente cargo/rustc),
# verifica che php-server canonico == pin s189 (SRV_ATTESO) e che lo stash del pin s188 abbia l'hash atteso,
# poi daemonizza s189-lancio.sh (→ s189-catena-fetta1.sh, che eredita il lock TOKEN s189 scritto dal lanciatore).
# Il nome NON contiene «harness/s1NN-» né «phpr» di proposito. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp189-harness"
DZ="/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl"
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
SRV_ATTESO="${SRV_ATTESO:?SRV_ATTESO obbligatorio (hash server pin s189)}"
OLD_ATTESO="${OLD_ATTESO:-97b58fd225ca7289}"
SRVOLD="${SRVOLD:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/php-server-s188}"
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
[ "$SC" = "$SRV_ATTESO" ] || { l "php-server canonico $SC ≠ pin s189 $SRV_ATTESO — FETTA 1 NON lanciata"; exit 9; }
SO=$(shasum -a 256 "$SRVOLD" 2>/dev/null | cut -c1-16)
[ "$SO" = "$OLD_ATTESO" ] || { l "stash pin s188 $SO ≠ $OLD_ATTESO ($SRVOLD) — FETTA 1 NON lanciata"; exit 9; }
[ -x "$H/s189-lancio.sh" ] || { l "s189-lancio.sh assente (generarlo con gen-fetta1.py) — FETTA 1 NON lanciata"; exit 2; }
l "pin server=$SC == atteso, stash s188=$SO — lancio s189-lancio.sh (PIN_ATTESO/PIN_OLD esportati per la catena)"
export PIN_ATTESO="$SRV_ATTESO" PIN_OLD="$OLD_ATTESO" SRVOLD
perl "$DZ" "$H/pair-out/lancio-dz.log" /bin/bash "$H/s189-lancio.sh"
sleep 5
l "lanciato (daemonize): $(pgrep -fl 's189-lancio' | grep -v pgrep | awk '{print $1}' | tr '\n' ' ')"
exit 0
