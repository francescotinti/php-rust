#!/bin/bash
# gate-feature-matrix.sh (A-AH2, Council WP-78) — KS-AH-78-1 / KS-AH-78-2.
#
# Cargo features UNION with default = ["cli-server"], so the REAL build matrix
# of php-server is a QUINTET (S-80.0.1, Council WP-81 A-AH16 — the fifth
# configuration existed in feature-space but was compiled by NOBODY):
#   1. default                                        → cli-server only
#   2. --no-default-features --features axum-server   → axum only
#   3. --features census-instrumentation              → census (union+counters)
#   4. --no-default-features
#        --features census-instrumentation            → census-axum-only
#      (weak dep php-cli?/census-instrumentation off — PINNED here and in CI
#       so it cannot bit-rot in silence; the measure driver never uses it)
#   5. --features axum-server                         → union (deployed binary)
#
# Each configuration builds with -D warnings scoped to the php-server crate
# (cargo rustc: dependency warnings stay warnings; a single php-server warning
# is a red gate — KS-AH-78-2). S-80.0.1 (A-AH15): the census bracket code now
# lives in php-runtime and php-cli too — BOTH get their own -D warnings pass
# under census-instrumentation, or that code never sees -D at all.
# The default binary must contain ZERO axum symbols (nm), with the union build
# as positive control (WP-72 lesson: a detector that can never fire proves
# nothing).
#
# Output: hash + feature set of every binary, logged to out/feature-matrix.log
# AND archived per-run to matrix-archive/ (tracked — A-SK11/KS-SK-81-2: the
# live log is overwritten every run, so a figure whose hash lives only there
# is NULL once the next build runs).
# KS-AH-78-1: any measurement without this log = NULL (unidentified binary).
#
# S-80.0.1 identity (A-AH14/KS-AH-81-1, Council WP-81): the log certifies
# `git=$GIT_REV` as the compiled source — that is a LIE on a dirty tree. The
# gate refuses to run unless `git status --porcelain` is EMPTY over the WHOLE
# repo. A-AH25 (Council WP-82) doc coherence: campaign raws ARE expected to
# be untracked mid-campaign (the measure driver tolerates them, checking only
# crates/Cargo.*) — therefore a matrix RE-RUN mid-campaign is REFUSED by
# design (fail-closed): commit the campaign raws first (KG-81-1 wants them
# committed anyway), then re-run the matrix. A matrix log produced on a
# dirty tree is NULL and every measurement citing it is VOID.
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
ARCHIVE="$HERE/matrix-archive"
mkdir -p "$OUT" "$ARCHIVE"
LOG="$OUT/feature-matrix.log"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"

# --- S-80.0.1: clean-tree assert BEFORE any row is written (A-AH14) ---------
DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
if [ -n "$DIRTY" ]; then
  echo "FAIL [tree]: working tree is DIRTY — a matrix log on a dirty tree"
  echo "  certifies a rev that is not the compiled source (KS-AH-81-1: log"
  echo "  NULL, downstream measurements VOID). Commit or stash first:"
  echo "$DIRTY" | head -20
  # Poison the live log so a stale PASS log cannot be mistaken for current.
  echo "NULL: matrix run refused on dirty tree at git=$GIT_REV" > "$LOG"
  exit 1
fi
: > "$LOG"
echo "git=$GIT_REV" | tee -a "$LOG"
# A-AH41 (Council WP-87, Hejlsberg): two same-rev archives built by
# different toolchains are otherwise indistinguishable — every hash motion
# of class b55e2f78/15fb6b46 needs its toolchain attributed in-band.
# A-AH47 (Council WP-89, Hejlsberg): sample the toolchain IN-REPO (the repo
# pins rust-toolchain.toml — a fuori-repo sample records the rustup default,
# while cargo below compiles with the pin: two wrong-but-equal samples) and
# with -Vv (host triple included; arch drift at equal version is invisible
# to -V). Single line, newlines joined with ';' — KS-AH-89-2 demands
# EXACTLY one rustc= row per archive.
echo "rustc=$( (cd "$REPO" && rustc -Vv 2>/dev/null | tr '\n' ';') || echo unknown)" | tee -a "$LOG"
echo "cargo=$(cargo -V 2>/dev/null || echo unknown)" | tee -a "$LOG"
echo "tree=clean (git status --porcelain empty at gate start, A-AH14)" | tee -a "$LOG"
FAILS=0

build() { # $1 label, rest: cargo flags
  local label="$1"; shift
  echo "== build [$label] $* ==" | tee -a "$LOG"
  if ! ( cd "$REPO" && cargo rustc --release -p php-server "$@" -- -D warnings ) \
      >> "$OUT/feature-matrix-build.log" 2>&1; then
    echo "FAIL [$label]: build with -D warnings failed (KS-AH-78-2)" | tee -a "$LOG"
    FAILS=$((FAILS+1)); return 1
  fi
  # A-AH12 (Council WP-80): cargo rustc's extra -D warnings flag changes the
  # crate hash and therefore the BITS — measured empirically (census row:
  # 07e6fd9d via rustc vs 69760159 via build, same source). The warnings
  # check above stays; the LOGGED hash is of the binary as operators (and
  # run-gate, and the measure driver) actually build it: plain cargo build.
  if ! ( cd "$REPO" && cargo build --release -p php-server "$@" ) \
      >> "$OUT/feature-matrix-build.log" 2>&1; then
    echo "FAIL [$label]: plain cargo build failed after rustc pass (A-AH12)" | tee -a "$LOG"
    FAILS=$((FAILS+1)); return 1
  fi
  local h
  h=$(shasum -a 256 "$BIN" | cut -c1-16)
  echo "bin[$label] sha256[0:16]=$h" | tee -a "$LOG"
}

# S-80.0.1 (A-AH15): -D warnings pass on a NON-php-server crate that carries
# census code. No hash row: these are library crates — the check is that the
# census-instrumentation source in them is warning-free, which the
# php-server-scoped passes above never prove.
lint_crate() { # $1 crate, rest: cargo flags
  local crate="$1"; shift
  echo "== lint [$crate] $* -- -D warnings ==" | tee -a "$LOG"
  if ! ( cd "$REPO" && cargo rustc --release -p "$crate" "$@" -- -D warnings ) \
      >> "$OUT/feature-matrix-build.log" 2>&1; then
    echo "FAIL [lint-$crate]: -D warnings failed (A-AH15: census code outside php-server was never under -D)" | tee -a "$LOG"
    FAILS=$((FAILS+1)); return 1
  fi
  echo "OK  [lint-$crate]: warning-free under census-instrumentation (A-AH15)" | tee -a "$LOG"
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

# 3. census (union + counters) — S-79.0.4 (A-AH10, Council WP-80): the fourth
#    configuration was OUTSIDE the matrix — it could bit-rot in silence and
#    its hash was absent from this log, so census figures were auto-exempt
#    from KS-AH-78-1. Now it builds under -D warnings and its hash is the
#    bin[census] row the measure driver ENFORCES for census-mode runs.
build "census" --features census-instrumentation

# 4. census-axum-only — S-80.0.1 (A-AH16/KS-AH-81-4, Council WP-81): the
#    weak-dep configuration (axum+census WITHOUT cli) exists precisely so
#    php-cli can stay out of a census build; compiled by nobody it bit-rots
#    in silence. PINNED here (and in CI): compile + hash row. The measure
#    driver never selects this row — its presence is a build guarantee, not
#    a measurement identity.
build "census-axum-only" --no-default-features --features census-instrumentation

# 4b. mem-census — S-81.0 (KS-AH-82-4, Council WP-82): the SIXTH
#     configuration — the retained-size instrument (module/Program walker,
#     tag=unitcache dump). A retained figure cited from a binary without
#     this row is NULL; the CI lane is the same-commit twin of this row.
build "mem-census" --features mem-census

# 5. Cross-crate census lint (A-AH15): php-runtime and php-cli carry the
#    bracket/emitter code since S-79.0.3/6. S-81.0 (KS-AH-82-4): same
#    discipline for the mem-census perimeter (8 dead doc-comments paid).
lint_crate php-runtime --features census-instrumentation
lint_crate php-cli --lib --features census-instrumentation
lint_crate php-runtime --features mem-census
lint_crate php-cli --lib --features mem-census

# 6. union (deployed dual-mode binary) — built LAST so the binary on disk is
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

# 7. A-AH6: warnings fatal on TEST targets (release profile — the debug/ dir
#    must never reappear on the local disk). cargo scopes the summary line per
#    crate, so grepping for the php-server summary keeps dependency warnings
#    out of scope, same as the cargo-rustc -D pass above.
#    A-AH29 (Council WP-83): the step used to compile the test targets ONLY
#    for axum-server — warnings in census/mem-census test builds were
#    invisible locally (the asymmetry Hejlsberg named). All three now.
echo "== test-target warnings (A-AH6/A-AH29) ==" | tee -a "$LOG"
for TFEAT in axum-server census-instrumentation mem-census; do
  TESTLOG="$OUT/feature-matrix-testbuild.$TFEAT.log"
  if ! ( cd "$REPO" && cargo test --release -p php-server --features "$TFEAT" --no-run ) \
      > "$TESTLOG" 2>&1; then
    echo "FAIL [test-targets/$TFEAT]: test build failed (see $TESTLOG)" | tee -a "$LOG"
    FAILS=$((FAILS+1))
  elif grep -E 'warning: `php-server`.*generated [0-9]+ warning' "$TESTLOG"; then
    echo "FAIL [test-targets/$TFEAT]: php-server test build generated warnings (A-AH6/A-AH29)" | tee -a "$LOG"
    FAILS=$((FAILS+1))
  else
    echo "OK  [test-targets/$TFEAT]: php-server test build warning-free" | tee -a "$LOG"
  fi
done

# 8. A-AH12 (Council WP-80): the test build above runs AFTER the union build —
#    assert the binary on disk still carries the union hash, so the bin[union]
#    row keeps describing the file the measure driver will hash. (cargo build
#    vs cargo rustc identity is otherwise "reproducible luck", not an assert.)
UNION_HASH="$(awk -F= '/^bin\[union\] /{print $2}' "$LOG" | tail -1)"
FINAL_HASH="$(shasum -a 256 "$BIN" | cut -c1-16)"
if [ -n "$UNION_HASH" ] && [ "$FINAL_HASH" != "$UNION_HASH" ]; then
  echo "FAIL [identity]: binary on disk changed after the union build ($FINAL_HASH != $UNION_HASH, A-AH12)" | tee -a "$LOG"
  FAILS=$((FAILS+1))
else
  echo "OK  [identity]: on-disk binary == bin[union] after all steps (A-AH12)" | tee -a "$LOG"
fi

# --- S-80.0.1: per-run archive (A-SK11/KS-SK-81-2) --------------------------
# The live log is overwritten by the next run; the archive copy is the
# durable identity record measurements may cite. Tracked in git — commit it
# with the campaign raws (KG-81-1).
STAMP="$(date +%Y%m%d-%H%M%S)"
ARCHIVED="$ARCHIVE/feature-matrix.$GIT_REV.$STAMP.log"
cp "$LOG" "$ARCHIVED"
echo "archive: $ARCHIVED"

if [ "$FAILS" = 0 ]; then
  echo "== FEATURE-MATRIX PASS (log: $LOG) [git $GIT_REV] =="; exit 0
else
  echo "== FEATURE-MATRIX FAIL($FAILS) [git $GIT_REV] =="; exit 1
fi
