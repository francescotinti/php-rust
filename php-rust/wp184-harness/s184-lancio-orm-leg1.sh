#!/bin/bash
# s184-lancio-orm-leg1.sh — S-184 bis: rilancio della SOLA gamba 1 ORM (s184-orm-leg1-rerun.sh) nella finestra ancora
# aperta della coppia t25 (lock TOKEN s184 scritto dal lanciatore notturno). Gate: lock presente, CI ferma (coda 0, nessun
# runner/cargo/rustc), nessun phpr vivo, pin s182 == stash. Esiti in orm-out/*-t25bis.*; rc SOLO da orm-out/rerun-leg1.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$REPO/wp184-harness"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
CIQ="/Volumes/Extreme Pro/Claude/phpr-ci/queue"
LOG="$H/orm-out/lancio-orm-leg1.log"; DONE="$H/orm-out/rerun-leg1.done"; rm -f "$DONE"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
[ -e /private/tmp/phpr-measure.lock ] || { l "lock ASSENTE — finestra chiusa, niente rerun"; echo "rc=6 lock-assente" > "$DONE"; exit 6; }
if pgrep -f 'ci/ci-runner.sh' > /dev/null 2>&1 || pgrep -qx cargo || pgrep -qx rustc; then l "CI viva — niente rerun"; echo "rc=6 ci-viva" > "$DONE"; exit 6; fi
Q=$(ls -1 "$CIQ" 2>/dev/null | grep -vc '^\._'); [ "$Q" = 0 ] || { l "coda CI $Q — niente rerun"; echo "rc=6 coda-ci" > "$DONE"; exit 6; }
if pgrep -fl 'release/phpr|php-server' | grep -v pgrep > /dev/null; then l "phpr vivo — niente rerun"; echo "rc=6 phpr-vivo" > "$DONE"; exit 6; fi
PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s182" | cut -c1-16)
PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
[ "$PC" = "$PIN_ATTESO" ] || { l "pin canonico $PC != stash $PIN_ATTESO"; echo "rc=9 pin" > "$DONE"; exit 9; }
SPD=/private/tmp/phpr-s184-orm; mkdir -p "$SPD"
l "lock presente, CI ferma, pin $PC == stash s182 — lancio rerun gamba 1 (MAPPA_SP=$SPD)"
PIN_ATTESO="$PIN_ATTESO" MAPPA_SP="$SPD" /bin/bash "$H/s184-orm-leg1-rerun.sh"
rc=$?
cp "$REPO/wp176-harness/s176-orm-coppia-verdetto.out" "$H/orm-out/s184-orm-coppia-verdetto-t25bis.out" 2>/dev/null
cp "$REPO/wp176-harness/orm-out/rimisura.done" "$H/orm-out/rimisura-t25bis.done" 2>/dev/null
l "rerun fine rc=$rc ($(cat "$REPO/wp176-harness/orm-out/rimisura.done" 2>/dev/null)) — verdetto in orm-out/s184-orm-coppia-verdetto-t25bis.out"
echo "rc=$rc" > "$DONE"
exit "$rc"
