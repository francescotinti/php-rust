#!/bin/bash
# s186-lancio-build.sh — lanciatore serale delle cinque build (s186-leva-build.sh): attende la quiete CI (cargo/rustc/phpt-runner
# assenti per 60 s, due volte — stesso predicato di s185-lancio-catena.sh) PRIMA di costruire (mai build e CI concorrenti,
# lezione S-184), poi esegue s186-leva-build.sh con i tre commit in env. Esiti: ab-out/s186-leva.done (dallo script) e
# ab-out/lancio-build.log (qui). NESSUN lock di misura (le build non misurano).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"
LOG="$H/ab-out/lancio-build.log"; mkdir -p "$H/ab-out"
ZCOMMIT="${ZCOMMIT:?}"; BCOMMIT="${BCOMMIT:?}"; CCOMMIT="${CCOMMIT:?}"
echo "$(date '+%F %T') avvio pid=$$ Z=$ZCOMMIT B=$BCOMMIT C=$CCOMMIT — attesa quiete CI" >> "$LOG"
while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
sleep 60
if pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; then
  while pgrep -qx cargo || pgrep -qx rustc || pgrep -qf phpt-runner; do sleep 60; done
  sleep 60
fi
echo "$(date '+%F %T') CI quieta — build" >> "$LOG"
ZCOMMIT="$ZCOMMIT" BCOMMIT="$BCOMMIT" CCOMMIT="$CCOMMIT" /bin/bash "$H/s186-leva-build.sh" > "$H/ab-out/s186-leva-build.log" 2>&1
rc=$?
echo "$(date '+%F %T') build fine rc=$rc ($(cat "$H/ab-out/s186-leva.done" 2>/dev/null))" >> "$LOG"
