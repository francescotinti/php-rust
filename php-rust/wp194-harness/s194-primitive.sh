#!/bin/bash
# S-194 primitive probe launcher: s194-primitive.sh <tag> [probe args: --bin <phpr> --base <file.out> --only a,b]
# Waits for a quiet CI, records hygiene, runs the probe. Outcome: s194-primitive-<tag>.{out,rc,done,igiene}. Generated with AI assistance (Claude Fable 5.2).
H="$(cd "$(dirname "$0")" && pwd)"; TAG="$1"; shift
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; B="$H/s194-primitive-$TAG"
rm -f "$B.rc" "$B.done"
for i in $(seq 1 240); do
  last=$(tail -1 "$FEED" | cut -d' ' -f1)
  if [ "$last" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
echo "igiene $(date '+%F %T') attesa_cicli=$i feed=$(tail -1 "$FEED" | cut -c1-40) cargo=$(pgrep -x cargo | wc -l | tr -d ' ') load=$(sysctl -n vm.loadavg) data_free=$(df -g /System/Volumes/Data | tail -1 | awk '{print $4}')G" > "$B.igiene"
python3 "$H/s194-primitive.py" --out "s194-primitive-$TAG" "$@" > "$B.log" 2>&1
echo $? > "$B.rc"
date '+%F %T' > "$B.done"
