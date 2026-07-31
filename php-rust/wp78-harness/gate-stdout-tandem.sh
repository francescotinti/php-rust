#!/bin/bash
# gate-stdout-tandem.sh — A-PP8/KS-PP-5 (Council WP-79 S-78.1.4).
#
# The HTTP body is `vm.rendered` ALONE (S-78.0: appending stdout too doubled
# every body). That contract silently decays if any php-runtime site ever
# writes `.stdout` WITHOUT the tandem `.rendered` write: those bytes would
# reach the CLI stream but vanish from every HTTP response.
#
# Rule (KS-PP-5): every `.stdout` write verb in crates/php-runtime/src must
# have a `.rendered` write within ±3 lines. A site that legitimately must not
# mirror (none exist today) would need a Council-sanctioned marker first.
# Positive control: a tandem-less snippet MUST be flagged; a tandem snippet
# MUST pass (WP-72: a detector that can never fire proves nothing).
export PATH=/usr/bin:/bin:/usr/sbin:$PATH
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"

detect() { # detect <file> -> violations on stdout, rc 1 if any
  awk '
    { lines[NR] = $0 }
    END {
      for (i = 1; i <= NR; i++) {
        if (lines[i] ~ /^[[:space:]]*\/\//) continue
        if (lines[i] ~ /\.stdout\.(extend_from_slice|extend|push|append|write)/) {
          ok = 0
          for (j = i - 3; j <= i + 3; j++)
            if (j >= 1 && j <= NR && j != i && lines[j] ~ /\.rendered\./) ok = 1
          if (!ok) {
            printf "%s:%d: stdout write without tandem rendered write (KS-PP-5): %s\n", \
                   FILENAME, i, lines[i]
            v = 1
          }
        }
      }
      exit v
    }
  ' "$1"
}

# --- positive controls ------------------------------------------------------
TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT
cat > "$TMPD/bad.rs" <<'EOF'
fn bad(&mut self) {
    self.stdout.extend_from_slice(bytes);
}
EOF
cat > "$TMPD/good.rs" <<'EOF'
fn good(&mut self) {
    self.stdout.extend_from_slice(bytes);
    self.rendered.extend_from_slice(bytes);
}
EOF
if detect "$TMPD/bad.rs" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: tandem-less snippet not flagged [git $GIT_REV]"; exit 2
fi
if ! detect "$TMPD/good.rs" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: tandem snippet was flagged [git $GIT_REV]"; exit 2
fi
echo "self-test PASS: tandem-less flagged, tandem clean"

# --- real scan --------------------------------------------------------------
FAIL=0
NSITES=0
while IFS= read -r -d '' f; do
  out=$(detect "$f")
  rc=$?
  [ -n "$out" ] && echo "$out"
  [ $rc -ne 0 ] && FAIL=1
  n=$(grep -cE '\.stdout\.(extend_from_slice|extend|push|append|write)' "$f")
  NSITES=$((NSITES + n))
done < <(find "$REPO/crates/php-runtime/src" -name '*.rs' ! -name '._*' -print0)

# Positive control on the REAL tree: zero stdout-write sites would mean the
# pattern rotted (there are 4 today) — an all-zero counter is a failed check.
if [ "$NSITES" -eq 0 ]; then
  echo "FAIL: 0 stdout-write sites found in php-runtime — detector pattern rotted [git $GIT_REV]"
  exit 2
fi

if [ "$FAIL" = 0 ]; then
  echo "== A-PP8 stdout⊆rendered PASS ($NSITES write sites, all tandem) [git $GIT_REV] =="
  exit 0
else
  echo "== A-PP8 stdout⊆rendered FAIL → body=rendered contract decays (KS-PP-5) [git $GIT_REV] =="
  exit 1
fi
