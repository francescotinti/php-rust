#!/bin/bash
# gate-axum-tests.sh — A-PP41 (Council WP-89, Pedersen): the axum-server
# test suite EXECUTED inside the battery perimeter. Before this gate,
# a_pp38 (full-path dispatch inheritance) compiled only under
# `--features axum-server` and RAN only in CI — outside the
# stamp/.done/--same-rev chain: the inheritance claim was CI-only
# (KS-PP-89-1). This gate runs the suite, refuses vacuity (0 tests), and
# pins a_pp38 BY NAME with its A-PP45 publish observable.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT
FAILS=0

LOG="$TMPD/axum-tests.log"
( cd "$REPO" && rustc -V ) > "$LOG" 2>&1
if ! ( cd "$REPO" && cargo test --release -p php-server --features axum-server ) >> "$LOG" 2>&1; then
  echo "FAIL: cargo test -p php-server --features axum-server failed (A-PP41)"
  tail -10 "$LOG"
  echo "== GATE-AXUM-TESTS FAIL(1) [git $GIT_REV] =="
  exit 1
fi
# non-vacuity (A-PP48, Council WP-90 — KS-PP-90-4): NPASS comes from the
# UNIT-SUITE line specifically (the one following "Running unittests
# src/main.rs"), never the MAX across binaries — a lib suite pruned to 1
# test used to pass if the doctests reached the threshold. The expected
# count is PINNED: a new test bumps it here, same commit, by name.
NPASS_EXPECTED=13
NPASS=$(awk '/Running unittests src\/main.rs/{f=1; next} f && /^test result: ok\./{ if (match($0, /ok\. [0-9]+ passed/)) print substr($0, RSTART+4, RLENGTH-11); exit }' "$LOG")
if [ -z "$NPASS" ]; then
  echo "FAIL: no unit-suite result line after 'Running unittests src/main.rs' (A-PP48)"
  FAILS=$((FAILS+1))
elif [ "$NPASS" != "$NPASS_EXPECTED" ]; then
  echo "FAIL: unit-suite passed count $NPASS != pinned $NPASS_EXPECTED (A-PP48/KS-PP-90-4 — bump the pin same-commit when adding tests)"
  FAILS=$((FAILS+1))
fi
# the inheritance anchor BY NAME (with its A-PP45 second-request tooth):
if grep -q "a_pp38_link_fatal_first_dispatched_task ... ok" "$LOG"; then
  echo "OK  a_pp38 full-path test executed by name (A-PP38/A-PP45 in-perimeter, KS-PP-89-1 lifted)"
else
  echo "FAIL: a_pp38_link_fatal_first_dispatched_task not seen passing — inheritance claim stays CI-only (KS-PP-89-1)"
  FAILS=$((FAILS+1))
fi
grep "^rustc " "$LOG" || { echo "FAIL: rustc header missing from log"; FAILS=$((FAILS+1)); }

# A-PP47 (Council WP-90, Pedersen — KS-PP-90-2): the publish observable at
# COUNTER — a targeted run with PHPR_UNIT_CACHE_LOG armed (F16b pattern):
# the worker's flushed log must show put=1 (request 1 published) and
# hit=1 (request 2 served FROM the published unit). Byte-equality alone
# is path-repetition on a deterministic fatal.
# A-PP53 (Council WP-91, Pedersen) — DECLARED constraint of the clean
# lane: the ARMED run is TARGETED-only (--test-threads=1, filter a_pp38).
# A full/parallel suite with PHPR_UNIT_CACHE_LOG in the environment
# pollutes the shared log and FAILs spuriously (fail-closed by design).
UCL="$TMPD/a_pp38.uclog"
if ! ( cd "$REPO" && PHPR_UNIT_CACHE_LOG="$UCL" cargo test --release -p php-server --features axum-server a_pp38 -- --test-threads=1 ) >> "$LOG" 2>&1; then
  echo "FAIL: armed a_pp38 run failed (A-PP47)"
  tail -5 "$LOG"
  FAILS=$((FAILS+1))
elif [ ! -s "$UCL" ]; then
  echo "FAIL: armed a_pp38 run left no uclog — channel dead (A-PP47, F16b lesson)"
  FAILS=$((FAILS+1))
else
  NPUT=$(grep -c "^unitcache main_put " "$UCL" || true)
  NHIT=$(grep -c "^unitcache main_hit " "$UCL" || true)
  if [ "$NPUT" = 1 ] && [ "$NHIT" = 1 ]; then
    echo "OK  a_pp38 publish counter put=1 hit=1 from the armed worker log (A-PP47/KS-PP-90-2)"
  else
    echo "FAIL: a_pp38 armed log main_put=$NPUT main_hit=$NHIT, expected 1/1 (A-PP47/KS-PP-90-2)"
    FAILS=$((FAILS+1))
  fi
  # A-PP52 (Council WP-91, Pedersen — KS-PP-91-2): EXTERNAL counter on the
  # fatal lane — the F8c pin (put==0 AND probe_fail==2) lived only inside
  # the test source, where a comment-preserving gutting keeps the sigla
  # census green (A-PP48(b) hole). The armed UCL is the external witness:
  # exactly 2 main_probe_fail rows (the F8c probe stats the inline path
  # twice, once per request).
  NPFAIL=$(grep -c "^unitcache main_probe_fail " "$UCL" || true)
  if [ "$NPFAIL" = 2 ]; then
    echo "OK  a_pp38 fatal-lane external counter main_probe_fail==2 from the armed UCL (A-PP52/KS-PP-91-2 lifted)"
  else
    echo "FAIL: a_pp38 armed log main_probe_fail=$NPFAIL, expected 2 — fatal-lane tooth gutted or channel changed (A-PP52/KS-PP-91-2)"
    FAILS=$((FAILS+1))
  fi
fi

if [ "$FAILS" = 0 ]; then
  echo "== GATE-AXUM-TESTS PASS ($NPASS tests, a_pp38 pinned) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-AXUM-TESTS FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
