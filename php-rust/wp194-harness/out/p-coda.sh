#!/bin/bash
# p-coda.sh <commit> — S-194 promotion, steps before the launch: wait for the CI job of <commit> to START (the chain then waits for it to end),
# acceptance chain on HEAD, then the dry run of the 30 fixture gates with the fresh candidate.
W="$(cd "$(dirname "$0")/.." && pwd)"; rm -f "$W/out/p-coda.done"
until grep -q "^START $(echo "$1" | cut -c1-12)" "/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; do sleep 20; done
bash "$W/s194-catena-P.sh"; c=$(cat "$W/out/catena-P.done" 2>/dev/null || echo 99)
d=99; [ "$c" = 0 ] && { bash "$W/out/p-dryrun.sh"; d=$(cat "$W/out/p-dryrun.rc"); }
echo "catena=$c dryrun=$d" > "$W/out/p-coda.done"
