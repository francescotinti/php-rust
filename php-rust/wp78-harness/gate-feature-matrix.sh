#!/bin/bash
# gate-feature-matrix.sh (A-AH2, Council WP-78) — KS-AH-78-1 / KS-AH-78-2.
#
# Cargo features UNION with default = ["cli-server"], so the REAL build matrix
# of php-server is a TRIPLE (the old A-AH1 comment described a pair that does
# not exist):
#   1. default                                       → cli-server only
#   2. --no-default-features --features axum-server  → axum only
#   3. --features axum-server                        → union (deployed binary)
#
# Each configuration builds with -D warnings scoped to the php-server crate
# (cargo rustc: dependency warnings stay warnings; a single php-server warning
# is a red gate — KS-AH-78-2). The default binary must contain ZERO axum
# symbols (nm), with the union build as positive control (WP-72 lesson: a
# detector that can never fire proves nothing).
#
# Output: hash + feature set of every binary, logged to out/feature-matrix.log.
# KS-AH-78-1: any WP-78 measurement without this log = NULL (unidentified binary).
#
# S-78.1.3 hardening (Council WP-79):
# - A-AH7: the symbol assert covers the WHOLE optional net stack
#   (axum|tokio|hyper|tower — all optional deps of the axum-server feature),
#   with the union build as positive control.
# - A-AH6: warnings are fatal on the TEST targets too — `cargo test` compiles
#   code the -D warnings rustc pass never sees (the unused-`reg` warning
#   lived exactly there); a php-server warning summary in the test build log
#   is a red gate.
# - A-SK9: every verdict line carries the git rev.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
BIN="$HOME/Claude/php-rust-output/release/php-server"
OUT="$HERE/gate-axum/out"
mkdir -p "$OUT"
LOG="$OUT/feature-matrix.log"
: > "$LOG"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
echo "git=$GIT_REV" | tee -a "$LOG"
FAILS=0

build() { # $1 label, rest: cargo flags
  local label="$1"; shift
  echo "== build [$label] $* ==" | tee -a "$LOG"
  if ! ( cd "$REPO" && cargo rustc --release -p php-server "$@" -- -D warnings ) \
      >> "$OUT/feature-matrix-build.log" 2>&1; then
    echo "FAIL [$label]: build with -D warnings failed (KS-AH-78-2)" | tee -a "$LOG"
    FAILS=$((FAILS+1)); return 1
  fi
  local h
  h=$(shasum -a 256 "$BIN" | cut -c1-16)
  echo "bin[$label] sha256[0:16]=$h" | tee -a "$LOG"
}

# A-AH7: whole optional net stack, not just axum.
net_syms() { nm "$BIN" 2>/dev/null | grep -cE 'axum|tokio|hyper|tower'; }

# 1. default (cli-server only)
build "default" && {
  n=$(net_syms)
  if [ "$n" -eq 0 ]; then
    echo "OK  [default]: 0 axum/tokio/hyper/tower symbols" | tee -a "$LOG"
  else
    echo "FAIL [default]: $n net-stack symbols leaked into cli-only binary (A-AH7)" | tee -a "$LOG"
    FAILS=$((FAILS+1))
  fi
}

# 2. axum only
build "axum-only" --no-default-features --features axum-server

# 3. union (deployed dual-mode binary) — built LAST so the binary on disk is
#    the measured one; positive control for the nm detector.
build "union" --features axum-server && {
  n=$(net_syms)
  if [ "$n" -gt 0 ]; then
    echo "OK  [union]: $n net-stack symbols (positive control for nm detector)" | tee -a "$LOG"
  else
    echo "FAIL [union]: nm detector saw 0 net-stack symbols — detector broken" | tee -a "$LOG"
    FAILS=$((FAILS+1))
  fi
}

# 4. A-AH6: warnings fatal on TEST targets (release profile — the debug/ dir
#    must never reappear on the local disk). cargo scopes the summary line per
#    crate, so grepping for the php-server summary keeps dependency warnings
#    out of scope, same as the cargo-rustc -D pass above.
echo "== test-target warnings (A-AH6) ==" | tee -a "$LOG"
TESTLOG="$OUT/feature-matrix-testbuild.log"
if ! ( cd "$REPO" && cargo test --release -p php-server --features axum-server --no-run ) \
    > "$TESTLOG" 2>&1; then
  echo "FAIL [test-targets]: test build failed (see $TESTLOG)" | tee -a "$LOG"
  FAILS=$((FAILS+1))
elif grep -E 'warning: `php-server`.*generated [0-9]+ warning' "$TESTLOG"; then
  echo "FAIL [test-targets]: php-server test build generated warnings (A-AH6)" | tee -a "$LOG"
  FAILS=$((FAILS+1))
else
  echo "OK  [test-targets]: php-server test build warning-free" | tee -a "$LOG"
fi

if [ "$FAILS" = 0 ]; then
  echo "== FEATURE-MATRIX PASS (log: $LOG) [git $GIT_REV] =="; exit 0
else
  echo "== FEATURE-MATRIX FAIL($FAILS) [git $GIT_REV] =="; exit 1
fi
