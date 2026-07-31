#!/bin/bash
# measure80-campaign.sh — S-80.0.6 (Council WP-81 p6): the reference measure
# R>=3 that SUPERSEDES design79 §1 (ADVISORY per A-BG17/KG-81-1).
#
# Both census arms (census = axum worker, censuscli = cli-server), four
# fixtures each (hello, include_gate, include_heavy, bare — the last is the
# A-BB16 ex-ante floor proxy), R=3 per (arm, fixture), plus ONE long-idle
# run per arm (IDLE_SECS=60, A-BG20). Sequential by rule; every run goes
# through measure78.sh ENFORCE (identity quintet + git match + source-clean
# + rows==N + tripwire fields). Raws land in wp78-harness/measure-out and
# MUST be committed with the results (KG-81-1).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
DRIVER="$HERE/../wp78-harness/measure78.sh"
R=0
run() { # run <mode> <label> <fixture> [env...]
  local mode="$1" label="$2" fixture="$3"
  echo "==== campaign: $mode $label $fixture ===="
  if ! bash "$DRIVER" "$mode" "$label" "$fixture"; then
    echo "==== campaign FAIL: $mode $label $fixture ===="
    R=1
  fi
}
for arm in census censuscli; do
  for fx in hello include_gate include_heavy bare; do
    for r in 1 2 3; do
      run "$arm" "80.$fx.r$r" "$fx.php"
    done
  done
done
# Long-idle runs (A-BG20): one per arm, 60s window.
for arm in census censuscli; do
  echo "==== campaign: $arm idle60 hello ===="
  if ! IDLE_SECS=60 bash "$DRIVER" "$arm" "80.idle60" "hello.php"; then
    echo "==== campaign FAIL: $arm idle60 ===="; R=1
  fi
done
echo "==== campaign done rc=$R ===="
exit $R
