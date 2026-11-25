#!/bin/bash
# p3-motivi.sh <commit> — S-194 P3: why a property write misses the site cache, on the whole ORM leg (instrumentation build of
# <commit> through s194-registro-op.py). The registry outputs of 22/11 are tracked: this run's are moved to -P3 names and the
# tracked ones restored. Outcome: out/p3-motivi.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; C="$1"; rm -f "$W/out/p3-motivi.done"
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if grep -q "DONE $C" "$FEED" && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
python3 "$W/s194-registro-op.py" "$C" orm > "$W/out/p3-motivi.log" 2>&1; rc=$?
cp "$W/s194-registro-op-orm.txt" "$W/s194-registro-op-P3-orm.txt"; cp "$W/s194-registro-op.out" "$W/s194-registro-op-P3.out"
cd "$W/.." && git checkout -- wp194-harness/s194-registro-op-orm.txt wp194-harness/s194-registro-op.out
echo "$rc" > "$W/out/p3-motivi.done"
