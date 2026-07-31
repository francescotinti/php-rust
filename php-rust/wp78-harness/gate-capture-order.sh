#!/bin/bash
# KS-PP-2 grep-gate (A-PP2/A-TH1, Council WP-78; hardened S-78.1.3 per Council
# WP-79 A-TH7/A-PP6/A-PP7): in the php-server crate no function may read
# `.rendered` / `.stdout` lexically AFTER calling `request_end(`.
# The permanent rule (Pedersen/Stogov, affirmed 9/9) says the reset boundary
# clears those buffers: a post-reset capture reads zeroes.
#
# Hardening (Council WP-79):
# - Pattern is `request_end\(` — OPEN on the arg list (A-TH7), so a future
#   `request_end(self)` (A-TH4) stays in scope. KH79-1: changing the
#   request_end signature without updating this pattern IN THE SAME COMMIT
#   invalidates the gate.
# - RECURSIVE scan over crates/php-server/src/**/*.rs (was: top-level glob),
#   AppleDouble `._*` excluded.
# - Same-line fix (Pedersen c.4): `request_end(); x = vm.rendered;` on ONE
#   line is now a violation (the old awk `next` skipped that line entirely).
# - Allow-marker bound by FORM and CENSUS (A-PP6, KS-PP-4): only the exact
#   marker `grep-gate-allow: post-reset-emptiness-check` is recognized; any
#   other `grep-gate-allow` spelling is a violation, and the total count of
#   sanctioned markers must equal the pinned census (drift → FAIL).
#
# Self-tests (positive controls, WP-72 lesson): known-bad next-line snippet,
# known-bad SAME-LINE snippet, malformed marker — all three MUST be flagged.
export PATH=/usr/bin:/bin:/usr/sbin:$PATH
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"

# KS-PP-4 pinned census: exactly this many sanctioned allow-markers may exist
# in the scanned tree (today: 1, the post-reset emptiness assert in
# worker_pool.rs tests). Adding one REQUIRES bumping this constant in the same
# commit — that bump is the audit trail.
ALLOW_CENSUS=1
ALLOW_FORM='grep-gate-allow: post-reset-emptiness-check'

detect() {
  # awk: reset the seen-flag at each fn definition; after a request_end( line,
  # any .rendered/.stdout read within the same fn is a violation. The
  # request_end line itself is checked for a same-line capture in its tail.
  awk '
    /grep-gate-allow: post-reset-emptiness-check/ { next }  # sanctioned, censused
    /^[[:space:]]*\/\// { next }          # comments do not arm nor violate
    /fn [A-Za-z0-9_]+ *(<[^>]*>)? *\(/ { seen = 0 }
    /request_end\(/ {
      rest = $0
      sub(/.*request_end\(/, "", rest)
      if (rest ~ /\.(rendered|stdout)/) {
        printf "%s:%d: SAME-LINE capture after request_end(: %s\n", FILENAME, FNR, $0
        v = 1
      }
      seen = 1; next
    }
    seen && /\.(rendered|stdout)/ {
      printf "%s:%d: capture after request_end(: %s\n", FILENAME, FNR, $0
      v = 1
    }
    END { exit v }
  ' "$@"
}

# --- positive controls ------------------------------------------------------
TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT

cat > "$TMPD/bad_nextline.rs" <<'EOF'
fn bad() {
    vm.request_end();
    let out = vm.rendered.clone();
}
EOF
cat > "$TMPD/bad_sameline.rs" <<'EOF'
fn bad() {
    vm.request_end(); let out = vm.rendered.clone();
}
EOF
for bad in bad_nextline bad_sameline; do
  if detect "$TMPD/$bad.rs" > /dev/null 2>&1; then
    echo "SELF-TEST BROKEN: detector did not flag $bad snippet [git $GIT_REV]"
    exit 2
  fi
done

# Malformed-marker control: a wrong spelling must NOT be treated as sanctioned.
cat > "$TMPD/bad_marker.rs" <<'EOF'
fn bad() {
    vm.request_end();
    let out = vm.rendered.clone(); // grep-gate-allow: wrong-form
}
EOF
if detect "$TMPD/bad_marker.rs" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: malformed marker was treated as sanctioned [git $GIT_REV]"
  exit 2
fi
echo "self-test PASS: next-line, same-line and malformed-marker snippets all flagged"

# --- real scan (recursive, A-TH7; space-safe: find -print0 + array) ---------
FAIL=0
FILES=()
while IFS= read -r -d '' f; do FILES+=("$f"); done \
  < <(find "$REPO/crates/php-server/src" -name '*.rs' ! -name '._*' -print0 | sort -z)
if [ "${#FILES[@]}" -eq 0 ]; then
  echo "HARNESS ERROR: no files found to scan [git $GIT_REV]"; exit 2
fi
for f in "${FILES[@]}"; do
  out=$(detect "$f")
  rc=$?
  [ -n "$out" ] && echo "$out"
  [ $rc -ne 0 ] && FAIL=1
done

# --- marker form + census (KS-PP-4) ------------------------------------------
BADFORM=$(grep -Hn 'grep-gate-allow' "${FILES[@]}" | grep -Fv "$ALLOW_FORM")
if [ -n "$BADFORM" ]; then
  echo "$BADFORM"
  echo "FAIL: grep-gate-allow marker with unsanctioned form (KS-PP-4)"
  FAIL=1
fi
NMARK=$(grep -F "$ALLOW_FORM" "${FILES[@]}" | grep -c .)
if [ "$NMARK" -ne "$ALLOW_CENSUS" ]; then
  grep -Hn -F "$ALLOW_FORM" "${FILES[@]}"
  echo "FAIL: $NMARK allow-markers found, census pins $ALLOW_CENSUS (KS-PP-4)"
  FAIL=1
fi

if [ "$FAIL" = 0 ]; then
  echo "== KS-PP-2 grep-gate PASS (recursive, markers $NMARK/$ALLOW_CENSUS) [git $GIT_REV] =="
  exit 0
else
  echo "== KS-PP-2 grep-gate FAIL: capture after request_end( → reject [git $GIT_REV] =="
  exit 1
fi
