#!/bin/bash
# n-coda.sh <commit> — S-194 night: wait until the CI job of <commit> is DONE (queue drained up to the last push), then run the night launcher.
W="$(cd "$(dirname "$0")/.." && pwd)"; F="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; k=$(echo "$1" | cut -c1-12)
until grep -q "^DONE $k" "$F"; do sleep 30; done
exec /bin/bash "$W/s194-lancio-notte.sh"
