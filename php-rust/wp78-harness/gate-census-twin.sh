#!/bin/bash
# gate-census-twin.sh — S-79.0.4 (A-AH11, Council WP-80, KS-AH-80-1).
#
# The census build is the measurement TWIN: nothing guarantees a
# `cfg(census-instrumentation)` block only COUNTS instead of altering a path
# — counters from a functionally-different program are counters of the wrong
# program. Two teeth:
#   1. Marker census (the KS-PP-4 discipline): the number of
#      `census-instrumentation` cfg sites per crate is PINNED — a new site
#      appears only with a same-commit bump here, so every new block gets
#      reviewed as instrumentation, not smuggled in.
#   2. Functional equivalence: the FULL run-gate battery (hello, stateful,
#      include, statics, fatal contract — every body vs the PHP oracle) runs
#      against the CENSUS binary (FEATURES=census-instrumentation), writing
#      its own verdict file.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
FAILS=0

# --- 1. Marker census (bump-in-commit like ALLOW_CENSUS/NSITES) -------------
# Counts are cfg-attribute OCCURRENCES in .rs files (not Cargo.toml).
# macOS bash 3.2: plain "dir:count" pairs, no associative arrays.
check_sites() { # check_sites <dir> <expected>
  local dir="$1" want="$2" n
  n=$(grep -RI --include='*.rs' --exclude='._*' -c 'census-instrumentation' \
        "$REPO/$dir" 2>/dev/null | awk -F: '{s+=$2} END{print s+0}')
  if [ "$n" -ne "$want" ]; then
    echo "FAIL: $dir has $n census-instrumentation sites, pinned $want"
    echo "      (A-AH11: a new cfg(census) block needs a same-commit bump here"
    echo "       and must only snapshot/count/eprintln — never alter a path)"
    FAILS=$((FAILS+1))
  else
    echo "OK  $dir: $want census sites (pinned)"
  fi
}
check_sites crates/php-server/src 18
check_sites crates/php-runtime/src 11

# Positive control (the WP-72 lesson): the scan must be able to COUNT — a
# decoy directory with one marker must read exactly 1.
DECOY="$(mktemp -d)"
trap 'rm -rf "$DECOY"' EXIT
printf '#[cfg(feature = "census-instrumentation")]\nfn x() {}\n' > "$DECOY/d.rs"
n=$(grep -RI --include='*.rs' -c 'census-instrumentation' "$DECOY" | awk -F: '{s+=$2} END{print s+0}')
if [ "$n" -ne 1 ]; then
  echo "FAIL: marker-census self-test counted $n != 1 on decoy — scanner broken"
  FAILS=$((FAILS+1))
else
  echo "OK  marker-census self-test (decoy counted 1)"
fi

if [ "$FAILS" -ne 0 ]; then
  echo "== CENSUS-TWIN FAIL($FAILS) at marker census [git $GIT_REV] =="
  exit 1
fi

# --- 2. Functional equivalence: full run-gate on the census binary ----------
echo "== run-gate on the CENSUS binary (FEATURES=census-instrumentation) =="
if FEATURES=census-instrumentation bash "$HERE/gate-axum/run-gate.sh"; then
  echo "== CENSUS-TWIN PASS (marker census + full-body battery) [git $GIT_REV] =="
  exit 0
else
  echo "== CENSUS-TWIN FAIL: run-gate battery failed on census binary [git $GIT_REV] =="
  exit 1
fi
