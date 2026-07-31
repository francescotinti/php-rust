#!/bin/bash
# gate-stdout-tandem.sh — A-PP8/KS-PP-5 (Council WP-79 S-78.1.4; hardened
# S-79.0.5 per Council WP-80 A-SK2/A-PP3).
#
# The HTTP body is `vm.rendered` ALONE (S-78.0: appending stdout too doubled
# every body). That contract silently decays if any php-runtime site ever
# writes `.stdout` WITHOUT the tandem `.rendered` write: those bytes would
# reach the CLI stream but vanish from every HTTP response.
#
# Rule (KS-PP-5): every `.stdout` WRITE in crates/php-runtime/src must have a
# `.rendered` WRITE within ±3 lines. S-79.0.5 closes the three WP-80 holes:
#   (a) verb set extended — direct assignment (`.stdout = x`),
#       `write!(...stdout...)` and `mem::swap/take(...stdout...)` count as
#       writes (the old list was method-calls only);
#   (b) the tandem side must itself be a WRITE — a mere read
#       (`.rendered.len()`) within ±3 lines no longer blesses a site;
#   (c) NSITES is PINNED (==4 today, bump-in-commit) — ≠0 only proved the
#       pattern had not rotted, not that no fifth site appeared with an
#       unlisted verb.
# Positive controls: tandem-less snippet flagged; read-only-tandem snippet
# flagged; assignment-verb snippet flagged; true write-tandem passes.
export PATH=/usr/bin:/bin:/usr/sbin:$PATH
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"

# Pinned census of stdout-write sites (bump in the same commit that adds one).
# 6 under the S-79.0.5 extended verb set: the 4 extend_from_slice mirrors
# (vm/mod.rs write_output + direct-out, host.rs tail, ini.rs startup diag)
# plus the two lifecycle pairs the old method-list missed — mem::take in the
# VmOutcome capture (vm/mod.rs ~1764) and .clear() in request_end (~3445),
# both with their rendered twin adjacent.
NSITES_PIN=6

# awk fragments shared by detection and counting: what counts as a WRITE.
# Passed via ENVIRON (awk -v mangles backslash escapes).
export STDOUT_W='\.stdout\.(extend_from_slice|extend|push|append|write|clear)|\.stdout[[:space:]]*=[^=]|write!\([^)]*\.stdout|mem::(swap|take)\([^)]*\.stdout'
export RENDERED_W='\.rendered\.(extend_from_slice|extend|push|append|write|clear)|\.rendered[[:space:]]*=[^=]|write!\([^)]*\.rendered|mem::(swap|take)\([^)]*\.rendered'

detect() { # detect <file> -> violations on stdout, rc 1 if any
  awk '
    BEGIN { sw = ENVIRON["STDOUT_W"]; rw = ENVIRON["RENDERED_W"] }
    { lines[NR] = $0 }
    END {
      for (i = 1; i <= NR; i++) {
        if (lines[i] ~ /^[[:space:]]*\/\//) continue
        if (lines[i] ~ sw) {
          ok = 0
          for (j = i - 3; j <= i + 3; j++)
            if (j >= 1 && j <= NR && j != i && lines[j] ~ rw) ok = 1
          # Same-line tandem (e.g. a swap writing both) also satisfies.
          if (lines[i] ~ rw) ok = 1
          if (!ok) {
            printf "%s:%d: stdout write without tandem RENDERED WRITE (KS-PP-5): %s\n", \
                   FILENAME, i, lines[i]
            v = 1
          }
        }
      }
      exit v
    }
  ' "$1"
}

count_sites() { # count_sites <file> -> number of stdout-write lines
  awk '
    BEGIN { sw = ENVIRON["STDOUT_W"] }
    /^[[:space:]]*\/\// { next }
    $0 ~ sw { n++ }
    END { print n + 0 }
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
cat > "$TMPD/bad_readonly_tandem.rs" <<'EOF'
fn bad(&mut self) {
    self.stdout.extend_from_slice(bytes);
    let n = self.rendered.len();
}
EOF
cat > "$TMPD/bad_assign.rs" <<'EOF'
fn bad(&mut self) {
    self.stdout = Vec::new();
}
EOF
cat > "$TMPD/bad_swap.rs" <<'EOF'
fn bad(&mut self) {
    std::mem::swap(&mut self.stdout, &mut other);
}
EOF
cat > "$TMPD/good.rs" <<'EOF'
fn good(&mut self) {
    self.stdout.extend_from_slice(bytes);
    self.rendered.extend_from_slice(bytes);
}
EOF
for bad in bad bad_readonly_tandem bad_assign bad_swap; do
  if detect "$TMPD/$bad.rs" > /dev/null 2>&1; then
    echo "SELF-TEST BROKEN: $bad snippet not flagged [git $GIT_REV]"; exit 2
  fi
done
if ! detect "$TMPD/good.rs" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: write-tandem snippet was flagged [git $GIT_REV]"; exit 2
fi
echo "self-test PASS: tandem-less, read-only-tandem, assignment and swap flagged; write-tandem clean"

# --- real scan --------------------------------------------------------------
FAIL=0
NSITES=0
while IFS= read -r -d '' f; do
  out=$(detect "$f")
  rc=$?
  [ -n "$out" ] && echo "$out"
  [ $rc -ne 0 ] && FAIL=1
  n=$(count_sites "$f")
  NSITES=$((NSITES + n))
done < <(find "$REPO/crates/php-runtime/src" -name '*.rs' ! -name '._*' -print0)

# A-SK2/KS-SK-80-3: the census is EXACT — a new stdout-write site (any verb)
# appears only with a same-commit bump of NSITES_PIN, so it gets reviewed
# against the tandem contract instead of drifting in.
if [ "$NSITES" -ne "$NSITES_PIN" ]; then
  echo "FAIL: $NSITES stdout-write sites found, census pins $NSITES_PIN (KS-SK-80-3) [git $GIT_REV]"
  exit 1
fi

if [ "$FAIL" = 0 ]; then
  echo "== A-PP8 stdout⊆rendered PASS ($NSITES/$NSITES_PIN write sites, all write-tandem) [git $GIT_REV] =="
  exit 0
else
  echo "== A-PP8 stdout⊆rendered FAIL → body=rendered contract decays (KS-PP-5) [git $GIT_REV] =="
  exit 1
fi
