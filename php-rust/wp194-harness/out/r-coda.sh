#!/bin/bash
# r-coda.sh <commit> — S-194 instruction ledger: wait until the CI job of <commit> has STARTED (the ledger script then waits for it to end), then run the ledger on both workloads.
W="$(cd "$(dirname "$0")/.." && pwd)"; rm -f "$W/out/r-coda.done"
until grep -q "^START $(echo "$1" | cut -c1-12)" "/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; do sleep 20; done
rm -f "$W/s194-registro-op.out"
python3 "$W/s194-registro-op.py" "$1" orm wp
cat "$W/out/registro-op.rc" > "$W/out/r-coda.done"
