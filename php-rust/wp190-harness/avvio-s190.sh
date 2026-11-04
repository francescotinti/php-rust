#!/bin/bash
# avvio-s190.sh — S-190: attende il DRENAGGIO della CI (runner morto ×2 a 60 s, niente cargo/rustc), verifica pin phpr
# canonico == s183 e bracci Z (wp186) + B3 (wp190) presenti, poi daemonizza s190-lancio-guardia.sh (→ s186-guardia.sh
# TAG=b3, che eredita il lock TOKEN s190 scritto dal lanciatore). Derivato da ../wp189-harness/avvio-s189.sh (manifest
# avvio-s190-copia.diff, aggiunto il 04/11 — rilievo 7 revisione S-190): tolti i controlli del server (SRV_ATTESO, stash
# s188, PIN_ATTESO/PIN_OLD), aggiunti pin phpr == s183, bracci Z/B3 presenti, lanciatore presente, verdetto b3 assente;
# log in catena-out/ invece di pair-out/; attesa di drenaggio compattata su una riga.
# Il nome NON contiene «harness/s1NN-» né «phpr» di proposito. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp190-harness"
DZ="/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl"
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
LOG="$H/catena-out/avvio.log"; mkdir -p "$H/catena-out"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
ci_viva(){ pgrep -f 'ci/ci-runner.sh' > /dev/null 2>&1 || pgrep -qx cargo || pgrep -qx rustc; }
l "attesa drenaggio CI; coda ora: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._')"
n=0
while [ "$n" -lt 2 ]; do if ci_viva; then n=0; else n=$((n+1)); fi; sleep 60; done
l "CI ferma; coda residua: $(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._')"
PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
[ "$PC" = "9a6ae11bf7e900ef" ] || { l "phpr canonico $PC ≠ pin s183 — guardia NON lanciata"; exit 9; }
[ -s "$P/wp186-harness/ab-out/s186-leva/phpr-Z" ] || { l "braccio Z assente — guardia NON lanciata"; exit 7; }
[ -s "$H/ab-out/s190-leva/phpr-B3" ] || { l "braccio B3 assente (s190-leva-build.sh) — guardia NON lanciata"; exit 7; }
[ -x "$H/s190-lancio-guardia.sh" ] || { l "s190-lancio-guardia.sh assente (gen-lancio-guardia-s190.py) — guardia NON lanciata"; exit 2; }
[ -e "$P/wp186-harness/s186-guardia-verdetto-b3.out" ] && { l "verdetto b3 ESISTE già: spostarlo prima"; exit 7; }
l "pin ok, bracci ok — lancio s190-lancio-guardia.sh"
perl "$DZ" "$H/catena-out/lancio-dz.log" /bin/bash "$H/s190-lancio-guardia.sh"
sleep 5
l "lanciato (daemonize): $(pgrep -fl 's190-lancio' | grep -v pgrep | awk '{print $1}' | tr '\n' ' ')"
exit 0
