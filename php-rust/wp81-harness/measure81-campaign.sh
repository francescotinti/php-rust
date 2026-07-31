#!/bin/bash
# measure81-campaign.sh — S-81.0 step 7: the LEVER-ARM census campaign
# (design79 §11.2), same protocol as measure80-campaign (R=3 per
# (arm, fixture), both census arms, four fixtures, one long-idle per arm)
# with 81.* labels so the WP-80 baseline raws are never overwritten.
# KS-SK-82-4/A-SK18: R>=3 is mandatory HERE TOO — the WP-80 determinism is
# NOT inheritable (canonicalize/stat/source-hash are new figure sources);
# the spread is re-derived IN-campaign by the analysis script (KG-82-1).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
DRIVER="$HERE/../wp78-harness/measure78.sh"
R=0
run() { local mode="$1" label="$2" fixture="$3"
  echo "==== campaign: $mode $label $fixture ===="
  if ! bash "$DRIVER" "$mode" "$label" "$fixture"; then
    echo "==== campaign FAIL: $mode $label $fixture ===="
    R=1
  fi
}
for arm in census censuscli; do
  for fx in hello include_gate include_heavy bare; do
    for r in 1 2 3; do
      run "$arm" "81.$fx.r$r" "$fx.php"
    done
  done
done
for arm in census censuscli; do
  echo "==== campaign: $arm idle60 hello ===="
  if ! IDLE_SECS=60 bash "$DRIVER" "$arm" "81.idle60" "hello.php"; then
    echo "==== campaign FAIL: $arm idle60 ===="; R=1
  fi
done
echo "==== campaign done rc=$R ===="
exit $R
