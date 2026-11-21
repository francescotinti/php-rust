#!/bin/bash
# S-194 primitive anatomy launcher: waits for a quiet CI, records hygiene, runs the probe. Generated with AI assistance (Claude Fable 5.2).
H="$(cd "$(dirname "$0")" && pwd)"
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
rm -f "$H/s194-primitive.rc" "$H/s194-primitive.done"
for i in $(seq 1 240); do
  last=$(tail -1 "$FEED" | cut -d' ' -f1)
  if [ "$last" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
{
  echo "igiene $(date '+%F %T') attesa_cicli=$i feed=$(tail -1 "$FEED" | cut -c1-40) cargo=$(pgrep -x cargo | wc -l | tr -d ' ') load=$(sysctl -n vm.loadavg) data_free=$(df -g /System/Volumes/Data | tail -1 | awk '{print $4}')G"
  shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16
} > "$H/s194-primitive.igiene"
python3 "$H/s194-primitive.py" > "$H/s194-primitive.log" 2>&1
echo $? > "$H/s194-primitive.rc"
date '+%F %T' > "$H/s194-primitive.done"
