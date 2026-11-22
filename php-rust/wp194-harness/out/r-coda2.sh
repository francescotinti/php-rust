#!/bin/bash
# r-coda2.sh <commit> — S-194 instruction ledger, second reading (nested loops charged to the caller): wait for the CI job of <commit> to START, run the ledger on both workloads, then the K calibration on the same binary, machine quiet.
W="$(cd "$(dirname "$0")/.." && pwd)"; rm -f "$W/out/r-coda2.done"
until grep -q "^START $(echo "$1" | cut -c1-12)" "/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; do sleep 20; done
rm -f "$W/s194-registro-op.out"
python3 "$W/s194-registro-op.py" "$1" orm wp
python3 "$W/s194-registro-op-taratura.py"
cat "$W/out/registro-op.rc" > "$W/out/r-coda2.done"
