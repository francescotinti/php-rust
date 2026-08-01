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

# --- 1. Marker census, PER-FILE on the cfg FORM (S-80.0.5, Council WP-81 ----
# A-AH18/A-SK10/KS-SK-81-3): the old pin was a per-crate SUM of raw string
# matches — a comment mentioning the feature entered the count, and an
# intra-crate substitution (−1 site here, +1 site there) was invisible.
# Now: (a) each file pins its count of the ATTRIBUTE form
# `cfg(feature = "census-instrumentation")` / `cfg(not(feature = ...))`;
# (b) in a pinned file, raw-string mentions must EQUAL the form count (a
# comment mention is a loud FAIL: it becomes a real site or it goes);
# (c) any UNPINNED .rs file mentioning the feature at all is a FAIL.
# Bump rule (KS-AH-81-3): a bump lands ONLY with the site list named in the
# same commit — `grep -nE "$CFG_RE" <file>` is the enumerable diff.
# DECLARED RESIDUAL (A-SK15, Council WP-82): the per-file pin is a COUNT —
# an intra-file substitution at equal count (one site removed, another added
# in the same file) is invisible to tooth 1. The backstop is tooth 2: the
# full-body battery against the census binary judges the substituted site's
# BEHAVIOR. Accepted, not closed — a reviewer of a census diff must eyeball
# `grep -nE "$CFG_RE"` on any file whose count did not move.
# S-82.0 p5 (A-DL15 allocator, named forms — KS-AH-81-3 class): main.rs
# gained two COMBINED cfg shapes around the allocator choice:
#   cfg(not(any(feature = "census-instrumentation", feature = "mem-census")))
#   cfg(all(feature = "mem-census", not(feature = "census-instrumentation")))
# Both are admitted as CANONICAL FORMS here (decoys below); any other
# combinator spelling stays a raw!=form FAIL.
CFG_RE='cfg\((not\()?feature = "census-instrumentation"|cfg\(not\(any\(feature = "census-instrumentation"|cfg\(all\(feature = "mem-census", not\(feature = "census-instrumentation"'
# main.rs 5->7 (S-82.0 p5 A-DL15 allocator, NAMED lines — A-AH28): :41
# not(any(census,mem)) plain-alloc; :52+:56 all(mem,not(census)) x2
# (MemCountingMi attr + mod); :87/:91/:196/:375 pre-existing census sites.
PINS="
crates/php-server/src/main.rs:7
crates/php-server/src/worker_pool.rs:23
crates/php-runtime/src/lib.rs:1
crates/php-runtime/src/lower/mod.rs:2
crates/php-runtime/src/compile/mod.rs:6
crates/php-runtime/src/vm/mod.rs:2
crates/php-cli/src/server.rs:3
"
for pin in $PINS; do
  f="${pin%%:*}"; want="${pin##*:}"
  n=$(grep -EIc "$CFG_RE" "$REPO/$f" 2>/dev/null || true)
  raw=$(grep -Ic 'census-instrumentation' "$REPO/$f" 2>/dev/null || true)
  if [ "$n" -ne "$want" ]; then
    echo "FAIL: $f has $n cfg(census) sites, pinned $want (A-AH18)"
    echo "      sites: $(grep -nE "$CFG_RE" "$REPO/$f" | awk -F: '{printf "%s ", $1}')"
    FAILS=$((FAILS+1))
  elif [ "$raw" -ne "$n" ]; then
    echo "FAIL: $f has $raw raw mentions vs $n cfg sites — a non-attribute"
    echo "      mention (comment/string) is not a site; make it real or drop it"
    FAILS=$((FAILS+1))
  else
    echo "OK  $f: $want cfg(census) sites (pinned, raw==form)"
  fi
done

# (c) No unpinned .rs file may mention the feature at all. (while-read: the
# repo path contains a space — an unquoted for-loop word-splits it.)
STRAY=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  rel="${f#"$REPO"/}"
  case "$PINS" in
    *"$rel:"*) : ;;
    *) echo "FAIL: UNPINNED file mentions census-instrumentation: $rel (KS-SK-81-3)"
       STRAY=1; FAILS=$((FAILS+1)) ;;
  esac
done < <(grep -RIl --include='*.rs' --exclude='._*' 'census-instrumentation' "$REPO/crates" 2>/dev/null)
[ "$STRAY" = 0 ] && echo "OK  no unpinned .rs file mentions the feature"

# Positive controls (the WP-72 lesson: a detector never seen firing proves
# nothing): a decoy cfg site must count 1 under the FORM match; a decoy
# comment-only mention must count 0 under the form match (and 1 raw — the
# exact divergence check (b) exists to catch).
DECOY="$(mktemp -d)"
trap 'rm -rf "$DECOY"' EXIT
printf '#[cfg(feature = "census-instrumentation")]\nfn x() {}\n' > "$DECOY/d.rs"
printf '// census-instrumentation mentioned in a comment only\nfn y() {}\n' > "$DECOY/c.rs"
# S-82.0: the two combined forms must ALSO count 1 each (named canonical)
printf '#[cfg(not(any(feature = "census-instrumentation", feature = "mem-census")))]\nfn z() {}\n' > "$DECOY/e.rs"
printf '#[cfg(all(feature = "mem-census", not(feature = "census-instrumentation")))]\nfn w() {}\n' > "$DECOY/f.rs"
ne=$(grep -EIc "$CFG_RE" "$DECOY/e.rs")
nf=$(grep -EIc "$CFG_RE" "$DECOY/f.rs")
if [ "$ne" -ne 1 ] || [ "$nf" -ne 1 ]; then
  echo "FAIL: combined-form decoys not counted (not-any=$ne all-not=$nf, want 1/1)"
  FAILS=$((FAILS+1))
fi
n=$(grep -EIc "$CFG_RE" "$DECOY/d.rs")
m=$(grep -EIc "$CFG_RE" "$DECOY/c.rs")
mraw=$(grep -Ic 'census-instrumentation' "$DECOY/c.rs")
if [ "$n" -ne 1 ] || [ "$m" -ne 0 ] || [ "$mraw" -ne 1 ]; then
  echo "FAIL: marker-census self-test (form=$n want 1; comment-form=$m want 0; comment-raw=$mraw want 1)"
  FAILS=$((FAILS+1))
else
  echo "OK  marker-census self-test (cfg form counted, comment mention excluded from form)"
fi

# --- 1b. A-AH33 (Council WP-84, KS-AH-84-2): alloc_id source pin -------------
# Cross-campaign retained comparisons are legal ONLY between raws carrying
# the same alloc_id; the const is SEMANTIC (bump = same-commit, named).
# Pin the current identity here — a silent change of the counting surface
# without a bump FAILS this gate.
MEMCENSUS_RS="$REPO/crates/php-types/src/memcensus.rs"
WANT_ALLOC_ID='memcount-v2-s82'
if grep -qE "pub const ALLOC_ID: &str = \"$WANT_ALLOC_ID\";" "$MEMCENSUS_RS"; then
  echo "OK  alloc_id source pin: $WANT_ALLOC_ID (A-AH33)"
else
  echo "FAIL: ALLOC_ID != '$WANT_ALLOC_ID' in memcensus.rs — counting surface changed without a NAMED bump (A-AH33/KS-AH-84-2: bump the const AND this pin same-commit)"
  FAILS=$((FAILS+1))
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
