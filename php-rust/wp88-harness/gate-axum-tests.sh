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
# non-vacuity: the suite must have run REAL tests (a filtered-to-zero or
# feature-pruned run exits 0 with '0 passed').
NPASS=$(sed -n 's/^test result: ok\. \([0-9]*\) passed.*/\1/p' "$LOG" | sort -rn | head -1)
if [ -z "$NPASS" ] || [ "$NPASS" -lt 5 ]; then
  echo "FAIL: axum-server suite passed count='$NPASS' (<5) — vacuous run (A-PP41)"
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

if [ "$FAILS" = 0 ]; then
  echo "== GATE-AXUM-TESTS PASS ($NPASS tests, a_pp38 pinned) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-AXUM-TESTS FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
