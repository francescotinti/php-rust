#!/bin/bash
# S-113: watcher sequenziale — attende il done della gamba ON, poi lancia la
# gamba OFF (run pesanti SEQUENZIALI e DETACHED, REGOLE §10). Nessuna misura
# propria: solo orchestrazione.
set -u
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp113-harness"
for _ in $(seq 1 720); do   # cap 2h
  [ -f "$H/pair-out-on/pair113.done" ] && break
  sleep 10
done
if [ ! -f "$H/pair-out-on/pair113.done" ]; then
  echo "watcher: TIMEOUT gamba ON" > "$H/watcher.log"; exit 1
fi
echo "watcher: ON done ($(cat "$H/pair-out-on/pair113.done")), lancio OFF $(date +%T)" > "$H/watcher.log"
mkdir -p "$H/pair-out-off"
"$H/pair113.sh" off >> "$H/watcher.log" 2>&1
echo "watcher: OFF done ($(cat "$H/pair-out-off/pair113.done" 2>/dev/null)) $(date +%T)" >> "$H/watcher.log"
