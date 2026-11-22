#!/bin/bash
# c0-coda.sh <commit> — S-194 C0a: give the CI runner two minutes to pick up the last push, then run the reasons census (the script itself waits for CI and cargo to be quiet).
W="$(cd "$(dirname "$0")/.." && pwd)"; rm -f "$W/out/c0-coda.done"; sleep 120
rm -f "$W/s194-c0-motivi.out"
python3 "$W/s194-c0-motivi.py" "$1" orm wp
cat "$W/out/c0-motivi.rc" > "$W/out/c0-coda.done"
